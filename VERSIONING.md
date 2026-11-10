# API and encrypted data versions

## Library API

The Rust crate and Python reference each declare a package version. While they
are at `0.x`, incompatible public API changes require a new minor version and
release notes with migration steps. Patch releases keep supported calls and
their results compatible. From `1.0`, incompatible API changes require a major
version. Test fixtures and fuzz targets are internal tooling.

The original `kryptocore::Error` and `kryptocore_signatures::Error` enums are
exhaustive in the `0.1` line. Callers can match the variants they handle and use
an `_` arm for the rest. Adding `#[non_exhaustive]` to either enum requires a
new minor version because downstream exhaustive matches would stop compiling.

The Python reference emits a `FutureWarning` when `encrypt` or `decrypt`
uses the deprecated empty AAD fallback after omission or `aad=None`. Pass the
existing format's exact AAD bytes explicitly, including `aad=b""` for formats
that use empty AAD. This preserves ciphertext compatibility when a breaking API
release removes the fallback.

The existing Argon2id profile is fixed for this API. New profiles must have an
explicit identifier and preserve access to data written with older profiles
for as long as those profiles are supported. A dependency update must still
pass the known-answer and cross-language tests.

The [Rust encryption profiles](PROFILES.md) specify their AAD and framing bytes
independently of the package version. Preserve support for written records when
adopting new profiles. Existing applications select their existing decoders.

## Application formats

Each application owns its container format and format version. Library versions
are independent of the on-disk version. A container should identify its format,
format version and cryptographic profile before decryption. Bind those choices
and any security-relevant public metadata to the ciphertext as authenticated
associated data. Specify the exact byte encoding of the header, nonce, salt,
authentication tag and any wrapped keys. Apply size and resource limits before
expensive work. Reject unknown or unsupported versions and profiles explicitly.

An incompatible encoding or change to the meaning of authenticated fields gets
a new format version. Readers dispatch only to known decoders. Writers use a
documented current version. Migration reads and authenticates the old data,
writes a new container, and verifies the result before replacing the old file.
Keep fixtures for every supported version, with a negative case for modified
headers and an explicit test for unknown versions. The cross-language tests in
this repository cover the primitives. Each application tests its own format.

[Vault](https://github.com/kryptocorelabs/vault) maintains its own container
versions and migration paths for existing preview files.
