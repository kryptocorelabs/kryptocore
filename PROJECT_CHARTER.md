# Why KryptoCore Exists

**One open foundation. A growing ecosystem. Security built together.**

KryptoCoreLabs develops privacy-focused software that helps people retain
control of their data and keys. Our direction is a small, reusable security
foundation and applications that people can understand and use without
installing development tools or learning cryptography.

## Open across the ecosystem

The Core, optional shared modules, Vault, Firefox extension, Container, MSG
and Rescue are open source. First-party code is available under the MIT
license: people may use, inspect, modify, redistribute and build on it,
including commercially, while retaining its notices. Dependencies keep
their own licenses and attribution requirements. Platform requirements are
documented separately.

Source, build instructions, tests and release limitations must be accessible.
Each release should identify its source revision and artifact checksums.
A successful local build is not evidence of byte-for-byte reproducibility;
we state the verification scope actually achieved.

## A small Core with explicit boundaries

The Core provides local authenticated encryption and password-based key
derivation, using established implementations of XChaCha20-Poly1305 and
Argon2id. Rust is the primary implementation; a Python reference supports
interoperability testing. We do not invent cryptographic algorithms.

Application formats, browser integration, networking, hardware-key support,
account management and payment logic belong outside this minimal Core.
Optional modules, including purpose-bound signatures, have their own APIs,
tests and documented assumptions. Applications may use established protocols
such as Noise in addition to the Core; documentation must explain each
component's role rather than imply the Core protects every operation.

Changes to keys, protocols, formats or public APIs need a documented reason,
compatibility analysis and tests for failure as well as success. Additional
complexity must address a concrete requirement.

## Security through evidence and transparency

Open source enables scrutiny; it does not guarantee security. We document
threats, trust boundaries, unsupported cases and known limitations. Tests,
dependency checks, fuzzing and independent review serve different purposes;
none is presented as proof of absolute safety.

Applications must preserve original data on failure, reject invalid protected
inputs and make plaintext exports, recovery credentials and destructive
operations understandable. A fully compromised endpoint can expose data
while it is in use. Losing every valid credential can make recovery
impossible. Backups and tested recovery paths remain essential.

Experiments stay separate from supported protection paths until their
assumptions, results and review are documented. Preview status remains
explicit. Mozilla signing and Windows signing concern distribution; they
do not replace an independent security audit.

## Applications that work for people

- **Vault and Firefox:** local credential storage and deliberate, origin-bound
  browser integration.
- **Container:** encrypted file transfer, with explicit sender-verification
  and optional malware-check boundaries.
- **MSG:** experimental conversations and file transfer over Tor, with local
  identities and documented connectivity and session-history limits.
- **Rescue:** independent access to supported encrypted formats using existing
  valid credentials; recognition alone is not authentication or recovery.

The usability goal is download, launch, guided setup and everyday use without
development tools. Security-relevant choices must be explained when needed.
Optional advanced settings must not obstruct the normal workflow.
An end-user release needs tests of the exact distributed artifacts on clean
systems and through the real application/browser/network workflows.

## Community and stewardship

We welcome contributions to code, tests, documentation, accessibility,
translations and security review. Constructive criticism and independently
challenging our assumptions are part of development.

Maintainers remain accountable for merge and release decisions. Security-
critical work needs a clear rationale and review; missing independent review
must be disclosed rather than implied. Major design decisions are discussed
publicly. Vulnerabilities, private data and credentials use private reporting
and coordinated disclosure, not public issue attachments.

Project documentation and maintainer-facing GitHub content are written in
English. Application languages and translations are separate choices.

## Funding without locking people out

Donations are the primary intended funding source. Optional paid support,
maintenance agreements or hosted services may fund continuing work, with
their scope and pricing stated clearly. The local applications and their
source stay freely available. Opening, exporting or recovering one's own
data, and receiving security fixes for supported versions, must not depend
on an account or subscription.

We will distinguish actual funding and supported commitments from future
plans. Community funding does not buy influence over security findings or
justify concealing limitations.

## Explore and contribute

- [Core and optional signature module](https://github.com/kryptocorelabs/kryptocore)
- [MSG, Container and Rescue source](https://github.com/kryptocorelabs/apps-source)
- [Vault and Firefox source](https://github.com/kryptocorelabs/vault-source)
- [Vault](https://github.com/kryptocorelabs/vault), [Container](https://github.com/kryptocorelabs/container),
  [MSG](https://github.com/kryptocorelabs/msg) and [Rescue](https://github.com/kryptocorelabs/rescue) downloads and user guides

Testing to date has been carried out by the KryptoCoreLabs maintainers, locally
and through automated CI. No independent security audit has been completed.
We welcome community testing, code reviews and scrutiny of our security
assumptions. Please use synthetic data and report suspected vulnerabilities
privately through the project's security-reporting channel.

The current applications remain previews. Public source is an invitation
to inspect and improve the work, not a claim that an independent audit or
production release has already been completed.
