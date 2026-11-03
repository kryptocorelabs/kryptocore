# Core architecture

The Rust crate is a local library. Its API exposes `Key::generate`, `derive_key`,
`encrypt`, and `decrypt`. An opaque `Key` owns a 32-byte buffer. Its backing
buffer and Rust's returned plaintext buffer use `zeroize` on drop. Applications
own input buffers and any copies they make.

- `getrandom` obtains keys and 24-byte nonces from the OS. Encryption chooses a
  fresh nonce. The caller stores it with the ciphertext and its 16-byte tag.
- `argon2` implements Argon2id v19. This profile derives 32 bytes with a
  16-byte caller-generated salt, 256 MiB, three passes, and one lane.
- `chacha20poly1305` implements XChaCha20-Poly1305. AAD is public and is
  authenticated as exact bytes. A failed authentication returns an error.
- The Python reference uses `argon2-cffi` and PyNaCl/libsodium. Published
  vectors and bidirectional tests check interoperability independently of
  application serialization.

The Core has no file parser or network service. Vault and other callers own
versioning, key wrapping, recovery, persistence, and UI. This keeps the public
API small and leaves format changes in the application that owns the format.
See [versioning](VERSIONING.md) and the [threat model](THREAT_MODEL.md).
