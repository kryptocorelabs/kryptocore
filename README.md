# KryptoCore

[Security in 30 seconds](SECURITY-IN-BRIEF.md) · [Recover without the main app](RESCUE-FIRST.md) · [Check a download](VERIFY-DOWNLOADS.md)

<img src="branding/kryptocore.png" alt="KryptoCore emblem" width="112">

[![Tests on main](https://github.com/kryptocorelabs/kryptocore/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kryptocorelabs/kryptocore/actions/workflows/ci.yml)
[![Rust dependency licenses](https://github.com/kryptocorelabs/kryptocore/actions/workflows/licenses.yml/badge.svg?branch=main)](https://github.com/kryptocorelabs/kryptocore/actions/workflows/licenses.yml)
[MIT License](LICENSE) · [Security](SECURITY.md) · [Contribute](CONTRIBUTING.md) · [Discussions](https://github.com/kryptocorelabs/kryptocore/discussions)

KryptoCore is a small, community-developed Rust library for local authenticated
encryption and password-based key derivation. Anyone can use, study, and
improve it under the MIT license, or build an application on top of it.

- Encrypt and authenticate data with XChaCha20-Poly1305.
- Derive encryption keys from passwords with Argon2id.
- Check Rust and Python interoperability with the included reference implementation.

KryptoCore is a development preview. Independent security review is pending.
Use synthetic data for evaluation and read the [threat model](THREAT_MODEL.md)
before integrating it into an application.

[KryptoCore Vault](https://github.com/kryptocorelabs/vault) is a local password
manager built on the library. Its desktop downloads and Firefox integration
updates are managed separately.

Testing to date has been carried out by the KryptoCoreLabs maintainers, locally
and through automated CI. No independent security audit has been completed.
We welcome community testing, code reviews and scrutiny of our security
assumptions. Please use synthetic data and report suspected vulnerabilities
privately through the project's security-reporting channel.

## Optional signatures

[KryptoCore Signatures](modules/signatures/README.md) is an optional Rust
companion for purpose-bound Ed25519 signatures. It has separate signing
keys and explicit verification inputs. The encryption Core has no signature
dependency. [Container](https://github.com/kryptocorelabs/container) and
[Rescue](https://github.com/kryptocorelabs/rescue) use this preview component
for optional detached KCC sender checks. An independent audit remains pending.
See [verification boundaries](VERIFICATION.md).

For offline file inspection and the recovery roadmap, see
[Rescue](https://github.com/kryptocorelabs/rescue). For conversations and
files over Tor, see [MSG](https://github.com/kryptocorelabs/msg).

## Why KryptoCore?

We want people to retain control of their data and keys through software they
can inspect, use and improve. The Core stays small; applications and optional
modules have explicit responsibilities and security boundaries. Established
cryptographic libraries, reproducible tests and independent scrutiny guide
the work. Open source alone does not guarantee security.

The whole ecosystem is open source under MIT for first-party code: Core,
Vault, Firefox, Container, MSG and Rescue. See the
[project charter](PROJECT_CHARTER.md),
[application source](https://github.com/kryptocorelabs/apps-source) and
[Vault/Firefox source](https://github.com/kryptocorelabs/vault-source).
Contributions to code, tests, documentation and independent review are welcome.

## Quick start

Install Git and the stable [Rust toolchain](https://rustup.rs/). On Windows
with the MSVC toolchain, install the C++ build tools and Windows SDK when
prompted by Rustup.

```sh
git clone https://github.com/kryptocorelabs/kryptocore.git
cd kryptocore
cargo build --locked --release
cargo test --workspace --locked --release
```

### Rust

The preview is distributed through GitHub. Add it as a Git dependency in
your application's `Cargo.toml` and pin a full commit you have reviewed.
Crates.io publication is disabled during the preview (`publish = false`).

```toml
[dependencies]
kryptocore = { git = "https://github.com/kryptocorelabs/kryptocore.git", rev = "REPLACE_WITH_FULL_COMMIT_SHA" }
```

```rust
use kryptocore::{decrypt, encrypt, Key};

fn main() -> Result<(), kryptocore::Error> {
    let key = Key::generate()?;
    let context = b"example/v1";
    let (nonce, ciphertext) = encrypt(&key, b"hello", context)?;
    let plaintext = decrypt(&key, &nonce, &ciphertext, context)?;
    assert_eq!(&plaintext[..], b"hello");
    Ok(())
}
```

The Rust API generates opaque keys and uses the operating system for randomness.
Keep ownership of passwords and salts in the application. The library overwrites
its returned key and plaintext buffers and the Argon2 work buffer on drop.
Applications should handle copies, input buffers, and process memory separately.

Use `derive_key_detailed` to distinguish rejected input from failure to
reserve the Argon2 work buffer. Use `decrypt_checked` when a parser supplies
the nonce as a byte slice. A nonce with the wrong length is an input error.
A wrong key, a changed nonce of the correct length, modified AAD and invalid
ciphertext all return the same authentication error. The original Rust entry
points still return `Error`. Error messages contain no input values.

### Python reference

Use Python 3.9 or later. See the [contribution guide](CONTRIBUTING.md#run-the-tests)
for virtual environment commands on Windows and Unix.

```sh
python -m pip install -e .
```

```python
from secrets import token_bytes

from kryptocore import decrypt, encrypt

key = token_bytes(32)
nonce, ciphertext = encrypt(key, b"hello", aad=b"example/v1")
assert decrypt(key, nonce, ciphertext, aad=b"example/v1") == b"hello"
```

Keep the key secret. Store the nonce alongside the ciphertext. Associated data
is authenticated and remains public. Supply exactly the same bytes when decrypting.
If authentication fails, `decrypt` raises `AuthenticationError` for well-formed
inputs. The ciphertext includes the authentication tag.

To derive a key from a password, call `derive_key(password_bytes, salt)` with a
fresh random 16-byte salt for each new derivation context. Store the salt and
profile with the encrypted data. The development profile uses Argon2id v19,
256 MiB, three passes, one lane, and a 32-byte output. Applications define the
password encoding. See [Argon2id memory and concurrency](ARCHITECTURE.md#argon2id-memory-and-concurrency)
for the memory cost and how to bound concurrent derivations.

The new API starts independently of the archived v0.5 container format.

## Contribute

Code review, interoperability tests, and clear documentation are welcome.
Read the [contribution guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).
Pick a [good first issue](https://github.com/kryptocorelabs/kryptocore/labels/good%20first%20issue)
or ask a question in [Discussions](https://github.com/kryptocorelabs/kryptocore/discussions).

See [API and data format versioning](VERSIONING.md) for compatibility rules.
See the [architecture](ARCHITECTURE.md), [threat model](THREAT_MODEL.md), and
[build and artifact verification](BUILD.md) for design and release details.

### Security checks

With the Python reference installed, run the cross-language tests using
`cargo test --locked --release -p kryptocore --test interop -- --ignored`.
Set `PYTHON` to the interpreter path if the command is `python3` on your system.
CI also runs the Rust and Python suites on Windows, Linux and macOS.
The interop matrix includes empty and binary messages, empty and 128 KiB AAD,
invalid nonce lengths and modified or truncated tags. The Linux CI also runs
concurrent Argon2 derivations and exercises both Rust error APIs under a
process memory limit.

With `cargo-audit` installed, run `cargo audit --deny warnings` and
`cargo audit --file fuzz/Cargo.lock --deny warnings`. With nightly Rust and
`cargo-fuzz` installed, run
`cargo +nightly fuzz run aead -- -max_total_time=45 -max_len=65536 -rss_limit_mb=1024`.
These checks cover known dependency advisories and exercise authenticated
encryption and decryption. See the [security scope](SECURITY.md) for review status.

Install the test dependencies with `python -m pip install -e ".[dev]"` and run
`python -m pytest`. Describe changes to the security assumptions or public API
in the pull request. Use the private reporting channel in [SECURITY.md](SECURITY.md)
for vulnerabilities.

## Support KryptoCore

Sponsorship helps fund Core maintenance, cross-platform testing, documentation,
and independent security review. KryptoCore is free to use and build on under
the [MIT license](LICENSE).
