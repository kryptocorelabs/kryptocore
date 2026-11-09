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

## Scope

Reports are in scope when they affect first-party code, supported data
formats, build workflows, or release integrity. Use the source repository
for the affected product. A dependency defect belongs with its maintainer.
Report its impact here as well if a KryptoCore build ships or exposes it.
The product threat model lists expected limitations. A way to cross a stated
trust boundary is in scope.

- [Core and Signatures](https://github.com/kryptocorelabs/kryptocore/security/advisories/new)
- [Vault and Firefox](https://github.com/kryptocorelabs/vault-source/security/advisories/new)
- [Messenger, Container, Rescue and Verify](https://github.com/kryptocorelabs/apps-source/security/advisories/new)

## Response and disclosure

Maintainers aim to acknowledge reports within 7 calendar days and send an
update at least every 14 days while a report is active. After validation,
the target is coordinated disclosure within 90 days. We will discuss a
different date with the reporter when the fix, an active attack, or the
impact on users warrants it. These are volunteer response targets.

Keep reproductions private while the report is being assessed. After
coordination, the public advisory records impact, affected versions, fixes,
and credit if the reporter wants it. Use public issues for review notes
that contain neither undisclosed vulnerabilities nor sensitive data.

For a suspected publisher-key incident, use the same private channel.
See the [release key procedure](RELEASE-KEY-PROCESS.md).
