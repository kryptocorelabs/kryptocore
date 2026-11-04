# Contributing to KryptoCore

Bug reports, tests, documentation, and code reviews are welcome. Follow the
[Code of Conduct](CODE_OF_CONDUCT.md) in project discussions.

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

From the repository root, with stable Rust and Python installed, run the
checks relevant to your change.

```sh
cargo test --workspace --locked --release
python -m pip install -e ".[dev]"
python -m pytest
cargo test --locked --release -p kryptocore --test interop -- --ignored
```

Set `PYTHON` to your interpreter path if it is not available as `python`.
See [BUILD.md](BUILD.md) for CI and artifact verification.
