#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"
commit=$(git rev-parse --verify HEAD)
out=dist/source
name="kryptocore-${commit}.tar.gz"
mkdir -p "$out"
second=$(mktemp "$out/rebuild.XXXXXX")
trap 'rm -f -- "$second"' EXIT

archive() {
    git -c tar.umask=0022 archive --format=tar --prefix=kryptocore/ "$commit" | gzip -n -9
}

archive > "$out/$name"
archive > "$second"
cmp "$out/$name" "$second"
git_version=$(git --version)
gzip_version=$(gzip --version)
printf 'commit=%s\ngit=%s\ngzip=%s\n' "$commit" "$git_version" "${gzip_version%%$'\n'*}" > "$out/SOURCE.txt"
(cd "$out" && sha256sum "$name" > SHA256SUMS.txt && sha256sum -c SHA256SUMS.txt)
