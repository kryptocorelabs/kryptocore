#!/usr/bin/env bash
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"
if [[ -n $(git status --porcelain) ]]; then
    echo 'Build from a clean checkout of the intended commit.' >&2
    exit 1
fi

bash scripts/source-archive.sh
toolchain=1.99.1
target=x86_64-unknown-linux-gnu
export CARGO_TARGET_DIR="$root/dist/core-build"
cargo_home=${CARGO_HOME:-"$HOME/.cargo"}
# The unit separator preserves a remap argument even when a path has spaces.
printf -v CARGO_ENCODED_RUSTFLAGS '%s\x1f%s' \
    "--remap-path-prefix=$root=/kryptocore" "--remap-path-prefix=$cargo_home=/cargo"
export CARGO_ENCODED_RUSTFLAGS
cargo +"$toolchain" build --locked --release --lib --package kryptocore --target "$target"
cp "$CARGO_TARGET_DIR/$target/release/libkryptocore.rlib" dist/source/
rustc +"$toolchain" --version --verbose > dist/source/RUSTC.txt
(
    cd dist/source
    sha256sum --binary libkryptocore.rlib >> SHA256SUMS.txt
    sha256sum --check SHA256SUMS.txt
)
