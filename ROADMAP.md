# Roadmap

| Product | Current priority | Out of scope for this phase |
| --- | --- | --- |
| Core | Review the existing cryptographic APIs, public test vectors, interoperability and reproducible builds. | Application file formats, accounts, networking and new cryptographic algorithms. |
| Vault and Firefox | Deliver tested recovery re-keying, settle on KCV4 for new vaults and require clear browser approval. | Cloud sync, unattended browser fills and additional write profiles. |
| Container | Keep data-only transfer, extraction and sender verification reliable. | Promoting self-extracting executables as the normal sharing format. |
| Messenger | Stabilize direct sessions, peer verification, attachment approval and local encrypted stores. | New Noise patterns, message or sidecar formats and new transports during the experiment. |
| Rescue | Preserve read-only access to supported formats and test recovery against pinned Vault fixtures. | Password bypass, ciphertext repair and in-place changes to a damaged vault. |
| Verify | Tie downloadable files to reviewed source, provenance and publisher keys. | Malware detection or a guarantee about program behavior. |

Public previews prioritize compatibility, repeatable tests, and independent
review. Changes to formats or protocols need a documented user requirement,
an interoperability plan, and tests against old data before release.

See the [project charter](PROJECT_CHARTER.md), [test vector index](TEST-VECTORS.md)
and [security reporting policy](SECURITY.md).
