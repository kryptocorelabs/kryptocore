# Contributing to KryptoCore

Bug reports, tests, documentation, and code reviews are welcome. Follow the
[Code of Conduct](CODE_OF_CONDUCT.md) in project discussions.

## Make your first change

1. Choose a [good first issue](https://github.com/kryptocorelabs/kryptocore/labels/good%20first%20issue)
   or ask a question in [Discussions](https://github.com/kryptocorelabs/kryptocore/discussions).
2. Fork [KryptoCore](https://github.com/kryptocorelabs/kryptocore) on GitHub.
   Copy your fork's clone URL from **Code**, then run these commands with
   `YOUR-USERNAME` replaced by your GitHub username.

   ```sh
   git clone https://github.com/YOUR-USERNAME/kryptocore.git
   cd kryptocore
   git switch -c my-first-change
   ```

3. Edit the relevant files and run the checks below. Review `git diff`, stage
   the files you changed, and commit with a short description.
4. Run `git push -u origin my-first-change`. Open a pull request from the
   branch on your fork to `kryptocorelabs/kryptocore:main`. Link the issue
   and include the checks you ran.

## Before opening a pull request

- For bugs, include the revision, platform, and a small reproduction with
  synthetic data. Report vulnerabilities privately through [SECURITY.md](SECURITY.md).
- Open an issue before changing the public API, cryptographic profile, or
  dependency set. Explain which use case the change addresses and how it
  affects callers. See [the threat model](THREAT_MODEL.md) and
  [versioning policy](VERSIONING.md).
- Keep the change focused. In the pull request, describe what changed and
  which checks you ran.

## Run the tests

From the repository root, use stable Rust and Python 3.9 or later. On Windows
with the MSVC Rust toolchain, install the C++ build tools and Windows SDK
when prompted by Rustup.

```sh
cargo build --locked --release
cargo test --workspace --locked --release
```

On Linux or macOS, use a virtual environment for the Python reference and
point the Rust interoperability test to that interpreter.

```sh
python3 -m venv .venv
.venv/bin/python -m pip install --upgrade pip
.venv/bin/python -m pip install -e ".[dev]"
.venv/bin/python -m pytest
PYTHON="$PWD/.venv/bin/python" cargo test --locked --release -p kryptocore --test interop -- --ignored
```

On Windows, run these commands in PowerShell.

```powershell
python -m venv .venv
.\.venv\Scripts\python.exe -m pip install --upgrade pip
.\.venv\Scripts\python.exe -m pip install -e ".[dev]"
.\.venv\Scripts\python.exe -m pytest
$env:PYTHON = (Resolve-Path .\.venv\Scripts\python.exe).Path
cargo test --locked --release -p kryptocore --test interop -- --ignored
```

See [BUILD.md](BUILD.md) for CI and artifact verification.

## Optional signature companion

`cargo test --workspace --locked --release` includes the signature unit
and documentation tests. With PyNaCl installed, also run:

```sh
cargo test --locked --release -p kryptocore-signatures --test interop -- --ignored
```

See [the signature protocol](modules/signatures/README.md) before changing
framing, purpose labels, key validation or dependencies. Keep encryption
and signing keys separate. App protocols and trust decisions require
their own review.
