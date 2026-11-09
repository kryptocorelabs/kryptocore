# Core threat model

The library protects message confidentiality and integrity for a caller that
supplies a secret key, or a password and a suitable salt. It authenticates the
exact associated data supplied by the caller. Applications own the lifetime of
passwords, storage keys, and decrypted records.

| Scenario | Core behavior and boundary |
| --- | --- |
| Someone obtains a copy of the ciphertext | XChaCha20-Poly1305 protects its contents while the key stays secret. Password-derived keys remain subject to offline guessing. Applications choose strong secrets and fresh salts. |
| Someone alters the ciphertext, nonce, or associated data | One-shot decryption authenticates before returning plaintext. Stream decryption authenticates each record before writing it and requires a final record. Stage the plaintext privately until the entire stream verifies. |
| Someone replays or swaps complete records | Applications bind the appropriate object identity and context into AAD and enforce freshness where required. An old authentic record can still verify. |
| Someone sends oversized inputs or triggers parallel derivations | The application applies size and concurrency limits before calling the library. Each derivation uses 256 MiB. |
| A local process reads application memory or monitors input/output | Rust clears the returned buffers on drop. Other copies and values observed during a live session may remain accessible. Python has additional runtime-managed copies. |
| Someone compromises dependencies, a build, or the OS random source | Dependency checks, locked builds and artifact digests aid review. Trusted tooling and a functioning OS random source are assumptions. |

Trust crosses the application API, the cryptographic dependencies, and the OS
randomness interface. A format parser is a separate boundary. It must validate
lengths before constructing typed inputs and reject unknown profiles. The Core
does not provide identity, authorization, key recovery, hardware integration,
rollback protection, a defense against a compromised operating system, or
side-channel guarantees for every deployment. See [architecture](ARCHITECTURE.md)
for application boundaries, [encryption profiles](PROFILES.md) for usage, and
the [security policy](SECURITY.md) for reporting.
