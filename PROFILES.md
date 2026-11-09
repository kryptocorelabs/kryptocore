# Encryption profiles for application developers

Use these Rust APIs when designing a new format. Keep an existing format's
decoder, key derivation profile and AAD bytes for files already written. The
Python reference covers the original primitives. The new profiles have Rust
APIs and byte-level test vectors produced with libsodium.

## Associated data and randomness

Build AAD with `Context::new(application, version, purpose, metadata)`. Choose
an application namespace you control, a format version, and a stable purpose
such as `payload` or `password-slot`. Include security-relevant public metadata
such as an expected record identifier. The reader selects the expected context.
Copying an untrusted purpose or record identifier from the input without checking
it gives the caller no assurance about where that record belongs. AAD is public.
Use independent keys for different roles as well as distinct contexts.

`generate_salt()` returns 16 random bytes for a new Argon2id derivation. Store
and reuse that salt to derive the same key when opening. `encrypt()` generates
and returns its own 24-byte nonce. Store it with the ciphertext and keep it
unchanged. `generate_nonce()` is available when implementing a format that needs
its own XChaCha20-Poly1305 nonce handling. Never deliberately reuse a nonce with
the same key. These helpers use OS randomness and return an error if it fails.

Context encoding is `b"KryptoCore/AAD\0\x01" || u8(app_len) || app ||
u32_be(version) || u8(purpose_len) || purpose || metadata`, where `||` means
byte concatenation. Both labels are 1 to 255 ASCII bytes from `[a-z0-9._-]`, the
version is nonzero, and the metadata is at most 64 KiB. If metadata has several
fields, the application must encode them unambiguously.

## Key wrapping

Generate a random data key and use an independent wrapping key or a strong
password to protect it. A wrapped key can be stored next to its encrypted data.
`unwrap_key` and `unwrap_key_with_password` return an opaque key after authentication.

```rust
use kryptocore::{wrapping::{unwrap_key_with_password, wrap_key_with_password}, Context, Key};

fn example(password: &[u8]) -> Result<Key, Box<dyn std::error::Error>> {
    let data_key = Key::generate()?;
    let slot = Context::new("org.example.archive", 1, "password-slot", b"slot-1")?;
    let stored = wrap_key_with_password(password, &data_key, &slot)?;
    unwrap_key_with_password(password, &stored, &slot).map_err(Into::into)
}
```

Each record contains a 24-byte nonce and 48 bytes of ciphertext including the
16-byte tag. Password records also contain the 16-byte salt. The application
defines how to serialize and identify the wrapping profile and its version.

- **Key profile v1.** XChaCha20-Poly1305 over the 32-byte data key with AAD
  `b"KryptoCore/wrap-key\0\x01" || context`.
- **Password profile v1.** Derive a wrapping key with the fixed Argon2id v19
  profile (256 MiB, three passes, one lane, 32-byte output), then use the same
  AEAD with AAD `b"KryptoCore/wrap-password\0\x01" || salt || context`.

An attacker who gets a password-wrapped key can test guesses offline. Bound the
number of concurrent [Argon2id operations](ARCHITECTURE.md#argon2id-memory-and-concurrency).
Rewrapping leaves the data key intact. If it may have leaked, generate a new
data key and re-encrypt the payload. Previously copied data and wrapped keys
can still be opened with their original credentials.

## Finite streams

Enable `features = ["stream"]` for the Core dependency. `stream::encrypt_stream`
and `stream::decrypt_stream` accept `Read` and `Write` handles. Both read through
EOF. They keep bounded buffers regardless of file size, and the writer owns
flushing and persistence. Supply a stream-specific key and the expected `Context`.

For file output, stage the encrypted or decrypted result in a new private
temporary file. Publish it only after the function and output flush succeed.
Discard staging on error. On decryption, an authentic prefix may already have
been written when a later record fails. Preserve the source and any existing
destination until the full stream is verified. The application controls
temporary file permissions and atomic replacement.

The stream v1 header is `b"KCSSTRM\x01" || random_19_byte_nonce_prefix`.
Records use RustCrypto STREAM-BE32 with XChaCha20-Poly1305. Each nonce is the
prefix followed by a zero-based `u32_be` record counter and a final flag byte
(`0` or `1`). Every record authenticates `header || context`. Full records carry
65,536 bytes plus the 16-byte tag. The mandatory final record carries fewer
than 65,536 bytes plus the tag, including an empty final record at exact block
boundaries. The counter must never wrap. The reader checks the final record at
EOF, which detects missing records, reordering and appended data. Stream length
remains visible. Applications should enforce their own total size limits and
decide whether they require freshness checks for an older, authentic stream.

## Hardware key providers

TPM, Secure Enclave and security-key integration is a companion-module design.
A provider should protect its own non-exportable root and use a platform-backed
operation to unlock a software data key. Record the provider, profile version
and expected context. Keep provider cancellation and an unavailable device
distinct from authentication failure. Use a separately configured recovery
slot as the explicit recovery path. Require the user to choose that path when
the provider fails or the device is unavailable.

Prototype each provider against its real API and device. Test enrollment,
unlock, removal, replacement, recovery and failure during a write before
enabling it for real data. A software data key exists in the process while the
data is unlocked. See [Windows TPM and CNG](https://learn.microsoft.com/en-us/windows/security/hardware-security/tpm/how-windows-uses-the-tpm),
[Apple Secure Enclave](https://developer.apple.com/documentation/cryptokit/secureenclave),
and [Yubico's PRF guidance](https://developers.yubico.com/WebAuthn/Concepts/PRF_Extension/Developers_Guide_to_PRF.html).
