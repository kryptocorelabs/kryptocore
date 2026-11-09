# Build and artifact verification

GitHub CI runs the Rust and Python suites and bidirectional interoperability
checks on Windows, Linux, and macOS. Linux also runs cargo-audit and a bounded
instrumented fuzz target. A passing dependency scan covers known advisories at
the time of the run. Independent security review is still pending.

## Source artifact

From the desired Git checkout on a machine with Git, GNU gzip and Bash, run:

```sh
bash scripts/source-archive.sh
cd dist/source
sha256sum -c SHA256SUMS.txt
```

The script archives a commit from Git with automatic line ending conversion
disabled. It uses stable permissions and suppresses gzip filename and timestamp
metadata. It generates the archive twice and
compares the bytes. `SOURCE.txt` records the full commit and tool versions.
Use the same Git and gzip versions when comparing builds. CI stores the archive,
checksum and source record together on the workflow run. The source artifact
contains tracked Core files only.

## Rebuild and verify the Linux Core library

On x86-64 Ubuntu 24.04, use a clean checkout of the commit recorded in
`SOURCE.txt`. Match the Git and gzip versions in that file and install Rust
1.99.1 with Rustup. Run:

```sh
rustup toolchain install 1.99.1 --profile minimal
bash scripts/build-core.sh
(cd dist/source && sha256sum -c SHA256SUMS.txt)
```

The build uses the checked-in Cargo lockfile and remaps checkout and registry
paths. Two independent CI jobs build the commit. The comparison job checks
each checksum file and compares the source archives and
`libkryptocore.rlib` byte for byte. Download the `core-verified-<commit>`
artifact from the same workflow run. It includes the checked outputs,
`SOURCE.txt`, `RUSTC.txt` and `SHA256SUMS.txt`.

The successful main-branch push and manual workflows attach signed GitHub
build provenance to each of those two outputs. Check it with the GitHub CLI:

```sh
gh attestation verify libkryptocore.rlib --repo kryptocorelabs/kryptocore
gh attestation verify kryptocore-COMMIT.tar.gz --repo kryptocorelabs/kryptocore
```

Replace `COMMIT` with the full commit from `SOURCE.txt`.
This build recipe covers the Core source archive and the
`x86_64-unknown-linux-gnu` Rust library. Windows and macOS run functional
tests. Python wheels and desktop distributions have separate build
environments. Publisher-signed release manifests and platform code signing
are separate release steps. See [Verify downloads](VERIFY-DOWNLOADS.md) and
[GitHub artifact attestations](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/use-artifact-attestations).

## Windows application builds

Desktop applications use their own packaging recipes. See [Vault's Windows
build](https://github.com/kryptocorelabs/vault-source/blob/main/apps/vault/WINDOWS-ONEFILE.md)
and the [shared application builds](https://github.com/kryptocorelabs/apps-source/blob/main/apps/desktop/README.md).
The reproducibility result above covers the Linux Core library and Core source
archive. For a Windows preview, use its release-specific evidence. Rescue
preview.3, for example, records a local Windows build and a public source commit
with ZIP and EXE hashes. Independent Windows binary reproduction is pending.

To establish reproducibility for a future Windows release, build its package
twice from a clean public commit with pinned inputs and compare the exact bytes.
Publish the commit, toolchain, packaging recipe and artifact hashes with the
release. Workflow provenance, publisher-signed manifests and Authenticode each
add a separate check. Publisher signing requires a provisioned signing key and
a published trusted public key.
