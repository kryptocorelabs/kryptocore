# Security

KryptoCore is a development preview. Independent review is still pending.

Testing to date has been carried out by the KryptoCoreLabs maintainers, locally
and through automated CI. No independent security audit has been completed.
We welcome community testing, code reviews and scrutiny of our security
assumptions. Please use synthetic data and report suspected vulnerabilities
privately through the project's security-reporting channel.

The library authenticates each ciphertext and its associated data. The caller
owns key storage, salt generation, format versioning, resource limits, and
protection against replay or rollback. The Rust library processes complete
messages in memory and overwrites its returned key and plaintext buffers and
its Argon2 work buffer on drop. Callers own input buffers and copies. Python
byte strings in the reference implementation can leave copies of secrets in
memory.

Use **Security → Report a vulnerability** to share a suspected issue privately.
Include the affected revision, impact, and a reproduction with synthetic data.
If the private reporting option is unavailable, open an issue requesting a
private contact and reserve the technical details for that conversation.
