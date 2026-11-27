# Core security contract

One Core. Many Apps. One Community.

This contract defines how applications use the small KryptoCore library and
which security decisions remain with them. It describes the inspected Rust
API at review revision `28a4c1d91728516cbd84e5de42f212a5b16f37ce` and the
existing ecosystem integrations assessed on 2026-11-27. It is a maintainer
integration contract, not an independent security audit.

## Reading this contract

**Required** statements are rules for new integrations and changes to supported
integrations. Existing encrypted profiles retain their documented algorithms,
key derivations, AAD and reader behavior. A rule does not authorize rewriting
old data or changing an established protocol. A necessary incompatibility needs
the migration and versioning process in [VERSIONING.md](VERSIONING.md).

**Current mechanism** descriptions report what the inspected code implements.
Evidence labels use IMPLEMENTED, TESTED, VERIFIED, PROPOSED, BLOCKED and
NOT VERIFIED. A mechanism can be implemented while a particular deployment or
platform remains unverified. Successful automated tests establish their tested
properties, not universal security.

## Responsibility boundary

| Core owns | Application owns |
| --- | --- |
| Local cryptographic operations, opaque Rust key values, fixed supported derivation profiles, nonce/salt helpers, canonical context encoding and defined errors. | File formats, key provenance and persistence, credential policy, unlock/lock/recovery, I/O permissions, process authentication, user consent, logs, update trust and resource budgets. |
| Overwriting the selected Rust buffers it owns when those wrappers are dropped. | Password input and all caller copies, GUI/runtime/OS buffers, process lifetime and any plaintext exported to another destination. |

The encryption Core starts no service, discovers no other app, opens no vault
file and issues no access grant. An in-process library call creates no new
process-authentication boundary. Using the same library does not provide the
keys or permissions needed to open another application's data.

## API inputs, outputs and ownership

The APIs below are implemented in [rust/lib.rs](rust/lib.rs),
[context.rs](rust/context.rs), [wrapping.rs](rust/wrapping.rs) and the optional
[stream.rs](rust/stream.rs). All input slices are borrowed for the operation.
The caller retains ownership and clearing responsibility for those inputs.

| API | Input and result | Failure and ownership contract |
| --- | --- | --- |
| `Key::generate()` | Generates 32 secret bytes with the OS random source and returns opaque `Key`. | Random-source failure returns `Error::RandomnessUnavailable`. The key owns a `Zeroizing<[u8; 32]>` and has no `Debug` or `Clone` implementation. |
| `Key::from_bytes(&[u8; 32])` | Copies existing secret bytes into a new opaque key. | Fixed shape is enforced by the type. Entropy, origin and prior authentication are the caller's responsibility. The original bytes remain caller-owned. |
| `derive_key(password, &Salt)` | Password bytes and a 16-byte salt produce a 32-byte opaque key. | Empty input returns `EmptyPassword`. Other detailed derivation failures map to `DerivationFailed` for compatibility. |
| `derive_key_detailed(password, &Salt)` | Same derivation and resulting bytes as `derive_key`. | Distinguishes empty/unsupported-length password, rejected work-memory reservation and derivation failure. Owned key/work buffers use `Zeroizing`. |
| `Key::derive_subkey(&Context)` | Deterministic HKDF-SHA-256 subkey from an existing high-entropy key. | Returns a new owned key. The fixed 32-byte expansion has no caller-dependent failure result. It is not a password-strengthening operation. |
| `Context::new(app, version, purpose, metadata)` | Returns canonical public context bytes after label/version/size checks. | Returns `ContextError` for invalid labels, zero version or metadata over 64 KiB. Context contains public data. |
| `generate_salt()` | Returns 16 OS-random public bytes. | Random-source failure returns an error. A new derivation context needs a fresh salt. Opening stored data reuses its stored salt. |
| `generate_nonce()` | Returns 24 OS-random public bytes. | Random-source failure returns an error. Ordinary `encrypt` generates its own nonce. |
| `encrypt(&Key, plaintext, aad)` | Generates a fresh nonce and returns `(Nonce, Vec<u8>)` with ciphertext and its 16-byte tag. Empty plaintext is supported. | Random-source or encryption failure returns `Error`. The ciphertext result is ordinary caller-owned storage. Core does not serialize a file. |
| `decrypt(&Key, &Nonce, ciphertext, aad)` | Authenticates the complete single-message ciphertext/AAD and returns `Zeroizing<Vec<u8>>` plaintext on success. | Authentication failure returns `Error::AuthenticationFailed`. Callers must not treat failed authentication as empty plaintext or retry another format silently. |
| `decrypt_checked(&Key, nonce_bytes, ciphertext, aad)` | Checks raw nonce length and otherwise decrypts as above. | Wrong-length nonce returns `DecryptError::InvalidNonceLength`. Well-sized wrong nonce, wrong key/AAD and damaged ciphertext use the same authentication error. |

The fixed password profile is Argon2id v19, 256 MiB, three passes, one lane
and a 32-byte result. Core accepts non-empty password bytes rather than scoring
human passwords. Applications define text encoding and password/recovery policy.
The same password, salt and supported profile must derive the same key when
opening existing data.

Memory reservation rejection is handled explicitly by `derive_key_detailed`.
Other allocations and an OS accepting a reservation but later exhausting memory
do not establish universal graceful OOM handling. Bound concurrent derivations,
queued requests, input sizes and plaintext sizes before expensive work. A
single lane does not limit the number of simultaneous API calls. See
[Argon2 memory and concurrency](ARCHITECTURE.md#argon2id-memory-and-concurrency).

## Key lifecycle and isolation

Required integration behavior follows these rules.

1. Generate independent random roots for separate apps and cryptographic roles,
   unless a documented app-local design intentionally derives role keys from
   one high-entropy root. There is no universal ecosystem master key.
2. Import bytes only when their source and intended role are established.
   Authenticate wrapped key material before import. Do not pass passwords or
   low-entropy identifiers to `Key::from_bytes` or use them as HKDF roots.
3. Use distinct expected contexts for subkeys. Compromise of a shared parent
   reveals the subkeys derived from it. HKDF does not isolate an app from an
   attacker who already has its root.
4. Keep owned secret buffers short-lived and clear caller copies where the
   language permits. Drop an unneeded Rust key rather than keeping a global
   unlocked copy. Avoid secret-bearing logs, debug values and command lines.
5. A real application lock invalidates decrypting operations and pending grants,
   ends relevant workers and releases regular access to session keys. A UI
   hiding its window is insufficient. Previously delivered plaintext cannot
   be revoked by locking.
6. Define recovery and rotation at the app format boundary. Rewrapping a key
   does not rotate it. Re-keying new data does not erase old snapshots or revoke
   their original credentials.

## Contexts, AAD and compatibility

Required new context use has an app-owned lower-case namespace, nonzero format
version, stable purpose and unambiguous metadata. Current `Context` labels are
1–255 bytes from `[a-z0-9._-]`. Metadata is public and limited to 64 KiB.
The exact v1 encoding and HKDF salt are specified in [PROFILES.md](PROFILES.md).

Metadata occupies the entire tail of `Context::as_bytes()`. Encode several
metadata fields using fixed widths or explicit lengths. If a containing format
places data after the whole context, length-frame that context as well. Raw
concatenation does not acquire a boundary merely because one field is a context.

The reader chooses the expected app, version and purpose from its supported
protocol. A sender-controlled field is not permission to select an arbitrary
key, operation or profile. Bind security-relevant header fields to AEAD and
validate their meaning. AAD is authenticated but public, and is not a place to
hide a password or signing seed.

Existing Vault, KCC, MSG and signing profiles use their own established header
and domain bytes. Keep those exact values when reading and updating those
profiles. Do not replace a legacy constant with a new `Context` encoding under
the same file marker. Explicit `aad=b""` remains valid only for formats that
deliberately specify empty AAD.

## Encryption and error behavior

Current single-message encryption uses XChaCha20-Poly1305 and chooses a fresh
random 24-byte nonce. Store the nonce, ciphertext/tag, salt and required public
profile metadata according to the app format. Never deliberately reuse a
nonce with the same key. Random sampling is not an enforced global uniqueness
registry.

Only expose a single-message decryption result after success. Wrong credentials
and authentication failures must not publish a file, replace the source or
become a success response. Bind the selected entry/format and any authorization
to the same operation that consumes the authenticated data.

Core error messages contain no input values. Applications may give actionable
messages for rejected input, unavailable resources and denied operations, but
must not append plaintext, passwords, tokens, raw requests or backend dumps.
Detailed typed errors do not identify which secret was correct. The original
Core and signature `Error` enums remain exhaustive in the 0.1 line. Match the
variants an app handles and retain a fallback arm for the rest.

## Key wrapping

The optional wrapping API returns public `WrappedKey` fields containing a
24-byte nonce and a 48-byte ciphertext/tag. `PasswordWrappedKey` also contains
the 16-byte salt. Its current functions are `wrap_key`, `unwrap_key`,
`wrap_key_with_password` and `unwrap_key_with_password`.

Wrapping authenticates independent expected context. The key-wrapping and
password-wrapping profiles have distinct fixed domain labels. Password wrapping
generates a fresh salt, derives the wrapping key with the fixed Argon2id profile
and returns an opaque key only after successful unwrapping. Wrapping a key with
itself is rejected using a fixed-size constant-time comparison. `WrapError`
preserves typed cryptographic/derivation failures without exposing inputs.

The app specifies record serialization, key-slot identities, supported versions
and recovery routes. The wrapper structs do not replace an app file format.
Previously copied wrapped keys remain accessible with their original secrets.

## Authenticated streaming

The opt-in `stream` feature exposes `encrypt_stream` and `decrypt_stream` over
caller-owned `Read`/`Write` values. Successful operations consume input through
EOF. Input, I/O or authentication failures can return earlier. Stream v1 uses a random
19-byte nonce prefix and RustCrypto STREAM-BE32 with 64 KiB plaintext records,
an authenticated record counter/final flag and a mandatory final record.
Every record authenticates the stream header and expected context.

Required streaming publication rules apply to both success and failure.

- Use a key dedicated to that stream role and enforce app limits on total
  plaintext, records, filesystem entries and elapsed work.
- Keep staged output private. A valid plaintext prefix can be written before
  a later tag, final-record or I/O failure. No incomplete result becomes the
  final destination merely because some records authenticated.
- Publish only after the whole function succeeds and output flush/sync and any
  required policy checks succeed. Preserve source data and existing destinations
  on failure. Ownership of flushing, persistence and cleanup stays with the app.
- On cancellation or error, remove owned staging where possible and report
  failure. Crashes, failed cleanup, journals and backups may retain plaintext.
  On Windows a selected staging parent can confer inherited access permissions.
- Never allow the stream counter to wrap. The header and framing remain those
  of the selected profile. A successful stream authentication supplies no
  independent freshness/rollback guarantee.

`StreamError` reports a bounded error kind rather than copying an underlying
reader/writer's secret-bearing error string. The transactional file example in
[PROFILES.md](PROFILES.md#finite-streams) demonstrates caller-owned staging.

## Memory and process boundaries

Current Rust `Key`, selected plaintext/key buffers and Argon2 work memory use
zeroizing wrappers. This does not cover every caller copy, dependency-internal
temporary, allocator copy, OS buffer, swap page or crash dump. It does not
establish hardware key isolation or a fully constant-time application.

The independent Python reference returns immutable bytes. Its supported
compatibility fallback for omitted AAD emits `FutureWarning`. New calls pass
the exact AAD explicitly. Python, Qt and C# can retain strings, JSON and widget
copies that cannot be reliably erased with these APIs.

Workers use explicit bounded local inputs and defined session lifetimes.
Parent-created redirected pipes avoid putting secrets in argv or environment
variables. The parent, worker binary and local OS remain trusted. A shared
library call needs no extra encryption envelope. A separate process or browser
channel needs its own authentication, authorization and replay policy. A
self-reported app name or PID does not establish that identity.

## Optional signatures

`kryptocore-signatures` is a separate companion, not an encryption Core
dependency. Its `SigningKey` has independent OS-random generation or explicit
import of a dedicated zeroizing Ed25519 seed. It has no `Debug` or `Clone`.
Explicit seed export returns a zeroizing owned copy for authenticated storage.

`sign` and `verify` use typed `Purpose` values, the exact versioned domain and
length framing, and a bounded message of at most 1 MiB. Verification expects a
caller-trusted public key, validated canonical prime-order key representation
and strict Ed25519 verification. Failures have typed generic errors.

Required callers keep signing seeds separate from passwords, Core encryption
keys and MSG X25519 identities. They establish expected signer trust, validity,
revocation and recovery. Successful verification authenticates bytes under that
key. It does not establish file safety, confidentiality or freshness. Protocol
details and API ownership are in the [companion guide](modules/signatures/README.md).

## Existing app integrations

This matrix assesses supported source mechanisms rather than certifying every
installed artifact. Vault and Apps currently pin Core
`89abb9bdf8cf4751f718343920abf70c0af8d2eb`. Apps pins Vault format
`778b7e22c959b5e209a5b97c35f797c6c47faa9f`. Updating a Core document or test
does not update those consumers. A pin update is a separate reviewed change.

| Application | Actual Core use | Contract alignment | Exceptions and remaining boundary | Existing evidence |
| --- | --- | --- | --- | --- |
| Vault | Argon2id slots, Core AEAD and opaque payload keys. KCV4 imports its authenticated random seed directly. | IMPLEMENTED for current formats and staged persistence. | KCV2/KCV3 retain their original Argon2-derived payload key. Existing AAD stays profile-specific. Python/Qt copies and local revision-history trust remain bounded assumptions. | KCV1–KCV4 tampering/re-key/recovery tests, `desktop_security`, real-worker Vault/Rescue interop and Qt workflow. |
| Firefox companion | Consumes selected Vault results. Its native/WASM encrypted transport uses maintained Snow, not the local encryption API. | IMPLEMENTED for channel/session limits and explicit per-origin fill approval. | Browser permissions, pairing and process identity belong to Vault/Firefox. Same-user process compromise is outside full isolation. Real signed-install coverage is tracked separately. | Native-channel malformed-frame/replay tests, `firefox-security.test.cjs`, desktop approval tests and artifact-specific real-browser harness. |
| Container | Core password derivation and AEAD key wrapping, plus optional signature companion. | IMPLEMENTED for KCC key wrapping and authenticated staged extraction. | KCC v1 uses its established RustCrypto STREAM-BE32 framing rather than changing to Core stream v1. Parent-directory trust, executable provenance and scan limits remain app responsibilities. | KCC final-record/path/no-clobber tests, large-file test and independent signature framing vector. |
| MSG | Core protects KCMI identities, invitations, KCMB stores and KCMO onion keys. KCMB v2 and KCMO use distinct expected HKDF contexts. | IMPLEMENTED for independent roots, role binding and session-scoped owned buffers. | KCMB v1 retains its authenticated legacy derivation. Snow handles app networking. Same-user memory, path-replacement and rollback limits remain. | Wrong-identity/store-role, sidecar/rotation/migration tests, ready-confirmation/replay tests and WPF lock/profile-switch workflow. |
| Rescue | Reuses the pinned canonical Vault reader and Container extractor. | IMPLEMENTED for credential-required read-only recovery and confirmed new plaintext outputs. | Accepts authentic old backups intentionally. Inspection is not authentication. It obtains no universal decrypting key from using Core. | Credential/format/truncation/no-overwrite tests and real-worker Vault/Rescue interop. |
| Verify | Core encrypts a dedicated signing seed. Optional companion verifies `ReleaseManifest`. | IMPLEMENTED for key separation and expected-signer verification. | Apps currently carries a local companion source copy. Publisher trust, clock/sequence policy, executable authenticity and post-check replacement remain distinct. | Manifest/expiry/rename/digest/signing-store tests and native-platform Verify CI. |
| Python reference | Independent Argon2id and XChaCha20-Poly1305 reference through argon2-cffi/PyNaCl. | PARTIAL as a memory-ownership substitute, IMPLEMENTED as a primitive interoperability reference. | No Rust opaque key, HKDF/wrapping/stream API or reliable immutable-byte clearing. AAD omission remains a deprecated compatibility path. | Reference tests and bidirectional Rust/Python AEAD plus optional signature interop. |

No inspected integration demonstrates a need for a new common envelope.
KCV, KCC, KCMI/KCMB/KCMO, the signature message frame and channel protocols
already identify their supported profiles and authenticate the required bytes.
A second envelope would add migration and parser responsibilities without
solving app authorization or process identity. Keep cross-app handoffs explicit
and use the existing format when the receiver already supports it. A future
incompatible format decision requires a concrete unmet requirement, fixtures
and a recovery/migration plan.

## Reproducible checks and adoption

Run these existing checks from the indicated repository using the reviewed
lockfile and supported toolchain. The independent Python interop requires the
installed reference and PyNaCl, with `PYTHON`/`PYTHONPATH` pointing to that
controlled environment when needed.

```sh
# kryptocore
cargo test --workspace --locked --release --all-features
python -m pytest -q -W error::FutureWarning
cargo test --workspace --locked --release --all-features --test interop -- --ignored
cargo test --locked --release --test argon2_memory -- --ignored --exact argon2id_derives_keys_concurrently
```

The allocation-failure test needs an enforced process limit. Windows uses
`tests/check_argon2_memory_windows.py` with the compiled `argon2_memory` test
binary. Linux CI runs it under a 192 MiB virtual-memory limit. These exercise
rejected reservation, not every possible OS OOM condition.

```sh
# vault-source
cargo test --workspace --locked --release -- --test-threads=1
node --test apps/vault/tests/firefox-security.test.cjs

# apps-source
cargo test --workspace --locked --release -- --test-threads=1
cargo test --locked --release -p kryptocore-container --lib -- --ignored --exact tests::streams_a_file_larger_than_the_argon2_work_buffer
```

Source tests cover Key non-Debug/non-Clone, independent cryptographic vectors,
wrong keys/AAD, changed contexts, invalid nonce lengths, final-record truncation,
short writers/read failures, allocation rejection, store-role/identity binding,
recovery and source-preserving publication. Exact execution results and platform
limits belong to the dated ecosystem review and CI records. A command listed
here is not evidence that a particular deployed artifact passed it.

Adoption is a review of an app's existing responsibilities against this
contract. Preserve compatible profile exceptions, attach tests to concrete
deviations and record deferred architecture work. No app gets permissions,
another app's key or a new data format merely by declaring conformance.
