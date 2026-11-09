# Third-party notices

KryptoCore's own source is covered by [the MIT license](LICENSE).
Resolved Core dependencies use permissive license options; preserve their
applicable upstream license and copyright notices when redistributing them.
The workspace license workflow checks the complete locked Cargo graphs.

The Rust subkey API uses RustCrypto's `hkdf` and `sha2` crates. Their exact
resolved versions and the transitive `hmac` dependency are in `Cargo.lock`.

The optional [Signatures companion](modules/signatures/THIRD_PARTY_NOTICES.md)
has its own exact-version dependency inventory and original upstream license
texts. It does not add dependencies to the encryption Core package.

Application runtime and distribution notices belong to their separate
release repositories. A library inventory does not cover a bundled application.
