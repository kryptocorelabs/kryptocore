# KryptoCore

<img src="branding/kryptocore.png" alt="KryptoCore emblem" width="112">

[![Tests on main](https://github.com/kryptocorelabs/kryptocore/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/kryptocorelabs/kryptocore/actions/workflows/ci.yml)
[![Rust dependency licenses](https://github.com/kryptocorelabs/kryptocore/actions/workflows/licenses.yml/badge.svg?branch=main)](https://github.com/kryptocorelabs/kryptocore/actions/workflows/licenses.yml)
[MIT License](LICENSE) · [Security](SECURITY.md) · [Contribute](CONTRIBUTING.md)

KryptoCore is a small, community-developed Rust library for local authenticated
encryption and password-based key derivation. Anyone can use, study, and
improve it under the MIT license, or build an application on top of it.

- Encrypt and authenticate data with XChaCha20-Poly1305.
- Derive encryption keys from passwords with Argon2id.
- Check Rust and Python interoperability with the included reference implementation.

The API is in development. Use synthetic data when evaluating it and read the
[threat model](THREAT_MODEL.md) before integrating it into an application.

[KryptoCore Vault](https://github.com/kryptocorelabs/vault) is a Windows password
manager built on the library. Its desktop downloads and Firefox integration
updates are managed separately.

## Quick start

```sh
git clone https://github.com/kryptocorelabs/kryptocore.git
cd kryptocore
cargo test --workspace --locked --release
```

### Rust

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

### Python reference

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
password encoding and need to account for the cost of concurrent derivations.

The new API starts independently of the archived v0.5 container format.

## Contribute

Code review, interoperability tests, and clear documentation are welcome.
Read the [contribution guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md).

See [API and data format versioning](VERSIONING.md) for compatibility rules.
See the [architecture](ARCHITECTURE.md), [threat model](THREAT_MODEL.md), and
[build and artifact verification](BUILD.md) for design and release details.

### Security checks

With the Python reference installed, run the cross-language tests using
`cargo test --locked --release -p kryptocore --test interop -- --ignored`.
Set `PYTHON` to the interpreter path if the command is `python3` on your system.
CI also runs the Rust and Python suites on Windows, Linux and macOS.

With `cargo-audit` installed, run `cargo audit --deny warnings` and
`cargo audit --file fuzz/Cargo.lock --deny warnings`. With nightly Rust and
`cargo-fuzz` installed, run
`cargo +nightly fuzz run aead -- -max_total_time=45 -max_len=4096`.
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
