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

## Argon2id memory and concurrency

Each derivation allocates a 256 MiB work buffer. Two overlapping derivations
need 512 MiB for those buffers alone. The process also needs memory for the
application and other operations. The one-lane parameter applies to each
derivation. It places no limit on how many times callers can invoke the API
at once.

Set a concurrency limit from the memory budget available to the process and
leave room for other allocations. For a desktop unlock flow, start with one
derivation at a time. For a service, bound the queue as well as the worker
count and reject excess work before starting a derivation.

The Rust API returns `Error::DerivationFailed` if the allocator rejects the
reservation. Some operating systems can accept the reservation and run out of
memory when the pages are written. Concurrency limits remain necessary in
those environments. The Linux CI runs a parallel derivation test and checks
that a process with a 192 MiB virtual memory limit returns the error.
