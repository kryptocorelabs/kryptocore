# Verification boundaries

This document distinguishes existing application checks from the optional
signature companion. Existing application file formats remain unchanged.

## Current responsibilities

| Question | Existing mechanism | Owner |
| --- | --- | --- |
| Does ciphertext authenticate with this key, nonce and associated data? | XChaCha20-Poly1305 authentication before returning plaintext; failure returns `AuthenticationFailed`. | Core `decrypt` in `rust/lib.rs`. |
| Are decrypted records, paths and file versions valid? | Format-specific parsers, size limits and schema/path validation. | Vault, Container and Rescue's shared readers for KCV1/KCV2 and KCC v1. |
| Does a chat peer possess the selected peer key? | Noise KK handshake and encrypted ready confirmations in both directions. | MSG protocol, above Core. |
| Does a container match an explicitly trusted signer's signature? | Existing detached OpenSSH signature verification over the exact SHA-256 digest, with a container-specific namespace and allowed signers. | Container `signature` module. |
| Does a KCC file match the selected shared signing key? | Detached KCCS v1 manifest, Ed25519 via the companion, full-file SHA-256/length/kind binding and same-stream extraction checks. | Container and Rescue. |
| Did the requested malware scan finish without detection? | Explicit local ClamAV selection; detection, errors and bounds block publication. This is not a virus-free certificate. | Container and Rescue, separately from signatures. |
| Does a known format marker exist? | Bounded header inspection, explicitly unauthenticated. | Rescue inspection; separate authenticated export requires the existing credential. MSG recovery remains unimplemented. |

Symmetric authentication does not identify an individual sender: other
holders of the same secret can also create authenticated ciphertext.
Likewise, a peer key check does not establish a person's real-world identity
without a suitable trust decision or independent comparison.

## Optional module: signatures (preview)

Password derivation and authenticated encryption remain in the minimal Core.
The separate [signature companion](modules/signatures/README.md) implements
purpose-bound signing and strict verification. Core does not depend on it;
applications must explicitly opt in. Container and Rescue now opt in for
detached sender checks; Vault, MSG and the Firefox channel retain their
existing authentication protocols.

It wraps `ed25519-dalek = 2.2.0`, using pure Ed25519 with explicit versioned
framing, separate purpose labels and bounded messages. Published
[RFC 8032](https://www.rfc-editor.org/rfc/rfc8032) vectors and a two-direction
PyNaCl reference test cover the primitive and the wrapper respectively.
See its [validation record](modules/signatures/VALIDATION.md) for scope and
remaining limits. This component has not received an independent audit.

Maintained boundaries (application responsibilities still apply):

- Separate typed signing, verification and encryption keys. Do not reuse
  or reinterpret the current MSG X25519 identity key as a signing key.
- Verification receives an expected public key from the application's trust
  store or an explicit verification decision. A key bundled with a signature
  must not automatically become trusted.
- Bind signatures to an explicit protocol version and application purpose.
  A container signature must not become a release or contact authorization.
- Define exact bytes and framing, with length limits. Do not depend on
  arbitrary JSON formatting or an unspecified hash/signature construction.
- Report signature validity separately from whether the signer is trusted,
  the content is confidential, or the data is safe to execute.
- Preserve existing OpenSSH signatures and their verification behavior.
  A new signature envelope requires an explicit version and compatibility
  plan; do not reinterpret old containers silently.
- Require published vectors, cross-implementation fixtures, wrong-key,
  wrong-purpose, malformed-key/signature and modified-data tests.

Trust stores, contact names, key approval, rotation, revocation and recovery
policy remain application responsibilities. They are not implicit effects
of a successful signature check.

## Software origin and integrity

Authenticating encrypted data is separate from authenticating an application
or the Core binary itself. Signed artifacts/manifests, trusted verification
keys, source-to-artifact traceability and platform code signing belong to
the build and distribution chain. A checksum only proves a match to the
expected checksum when that expected value is obtained through a trusted
route.

A self-test may detect accidental failures against known vectors. A check
inside a replaceable program is not an independent trust anchor for that
program: an attacker able to replace both code and its verifier can replace
the result. Do not advertise self-verification as a guarantee against a
compromised operating system.

## UI meaning

Keep distinct results such as `not checked`, `ciphertext authenticated`,
`peer key confirmed` and `trusted signature verified`. Do not collapse
unrelated properties into a universal `is_secure` flag.

The MSG green marker represents the established authenticated chat channel.
Rescue's format recognition remains explicitly unauthenticated. Its separate
Vault/Container export marks success only after format-reader authentication.
A future sender-signature marker must be distinct from both.
