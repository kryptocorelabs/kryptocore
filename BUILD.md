# Build and artifact verification

GitHub CI runs the Rust and Python suites and bidirectional interoperability
checks on Windows, Linux, and Intel macOS. Linux also runs cargo-audit and a bounded
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

## Binary release preparation

Before publishing a binary or wheel, record the commit, target, complete
toolchain versions, runner or image identity, lockfile digests, build command
and SHA-256 for each asset. Build twice in independent clean environments and
compare the binary digests before describing the binaries as reproducible.
Rust uses `cargo build --locked --release`. Python packaging needs a separately
fixed build environment. The source archive check does not establish binary
reproducibility. Vault release provenance is tracked separately from the Core.
