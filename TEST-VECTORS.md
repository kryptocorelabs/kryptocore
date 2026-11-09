# Public test vectors and interoperability fixtures

These fixtures contain synthetic data. The links pin the source revision so
reviewers can reproduce the same inputs even as tests on `main` evolve.

| Fixture | Pinned source | Coverage |
| --- | --- | --- |
| Core cryptographic vectors | [Core unit vectors](https://github.com/kryptocorelabs/kryptocore/blob/89abb9bdf8cf4751f718343920abf70c0af8d2eb/rust/lib.rs) | XChaCha20-Poly1305, Argon2id and domain-separated HKDF-SHA-256. |
| Core finite stream v1 | [Independent libsodium vector](https://github.com/kryptocorelabs/kryptocore/blob/89abb9bdf8cf4751f718343920abf70c0af8d2eb/rust/stream.rs) | Authenticated final record, frame order and truncation. |
| Rust/Python interoperability v1 | [Rust driver](https://github.com/kryptocorelabs/kryptocore/blob/89abb9bdf8cf4751f718343920abf70c0af8d2eb/tests/interop.rs) and [Python peer](https://github.com/kryptocorelabs/kryptocore/blob/89abb9bdf8cf4751f718343920abf70c0af8d2eb/tests/interop_peer.py) | Cross-language exchange and tampering with public AAD. |
| Vault/Rescue | [Vault CI fixture driver](https://github.com/kryptocorelabs/vault-source/blob/c91e12e36078bb894025f889cd965645a9deaffd/apps/vault/tests/vault-rescue-interop.py) and [its reader pin](https://github.com/kryptocorelabs/vault-source/blob/c91e12e36078bb894025f889cd965645a9deaffd/.github/workflows/vault-ci.yml) | Synthetic vaults opened in a separate reader, recovery and revision checks. |
| Container signature v1 | [Independent PyNaCl fixture](https://github.com/kryptocorelabs/apps-source/blob/a95e8bf168449e19e5741cc65e93666ab87cfdfd/apps/container/tests/fixtures/native-v1.txt) | Exact framing and signature verification. |
| Rescue legacy Vault fixture | [KCV1 bytes](https://github.com/kryptocorelabs/apps-source/blob/a95e8bf168449e19e5741cc65e93666ab87cfdfd/apps/rescue/tests/fixtures/vault-v1.hex) | Legacy recognition and authenticated recovery. |

Keep existing fixture revisions when adding a new format. Record the format
version, expected outcome and the exact writer and reader revisions when
reporting a mismatch. See [Contributing](CONTRIBUTING.md#run-the-tests) for
commands that run the fixtures.
