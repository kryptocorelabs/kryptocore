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

## Dependencies and alternatives

| Dependency | Why we use it | Possible alternative and trade-off |
| --- | --- | --- |
| RustCrypto `argon2` and `chacha20poly1305` | Implement the documented password derivation and authenticated encryption profiles. The optional stream profile also uses RustCrypto's STREAM implementation. | A compatible provider could use libargon2 and libsodium primitives. A switch would add a native dependency and require byte-for-byte tests of the fixed parameters, AAD, ciphertexts and stream framing. |
| Operating system randomness through `getrandom` | Supplies entropy for keys, salts and nonces. Randomness failure stops the operation. | Call a platform CSPRNG through a reviewed backend. This changes the adapter while retaining the platform's entropy and availability assumptions. |
| `subtle` and `zeroize` | Provide a constant-time key equality check and clear selected Rust-owned buffers. | Equivalent reviewed helpers from a replacement provider would need to preserve those properties. The application still owns other copies and OS-managed memory. |
| GitHub Actions and artifact attestations | Run CI and bind the published Core build artifacts to a repository and workflow identity. | Run the locked build recipe independently. A different CI or attestation service would need its own documented trust root and verification path. |

Application-specific dependencies are documented at their own boundaries.
See [Messenger transport](https://github.com/kryptocorelabs/apps-source/blob/main/apps/msg/README.md#security-boundaries)
and the [Vault and Firefox threat model](https://github.com/kryptocorelabs/vault-source/blob/main/apps/vault/THREAT_MODEL.md).
