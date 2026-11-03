# Security

KryptoCore is a development preview. Independent review is still pending.

The library authenticates each ciphertext and its associated data. The caller
owns key storage, salt generation, format versioning, resource limits, and
protection against replay or rollback. The Rust library processes complete
messages in memory and overwrites its returned key and plaintext buffers on
drop. Callers own input buffers and copies. Python byte strings in the reference
implementation can leave copies of secrets in memory.

Enable GitHub private vulnerability reporting when publishing this repository.
Use **Security → Report a vulnerability** to share a suspected issue privately.
Include the affected revision, impact, and a reproduction with synthetic data.
