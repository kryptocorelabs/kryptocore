# Check a download before opening it

A checksum detects a mismatch. A **signed release manifest** also binds those
bytes to a publisher key you already trust. It does not prove that the program
is bug-free, malware-free or the latest release.

**Current status.** Windows app previews provide SHA-256 checksums. Signed
downloads also provide a `.release.json` and `.release.sig` pair alongside the
archive. Firefox validates the Mozilla signature on the published Firefox XPI.

## KryptoCoreLabs publisher key

The [public release key](PUBLIC_RELEASE_KEY.txt) is anchored in this repository's
GitHub-verified commit history. The SHA-256 fingerprint of the 32 public key bytes
encoded after `KCP1:` is
`9c77898f06d12f234101e633c713f095d488d45a212661ac3ac06c4271dc00d8`.
Confirm the key through a trusted route before using it to verify downloads.
The Verify manifest includes an expiry. Check the validity period for the
download you intend to use.

## Signed downloads

| Product | Release archives with detached signatures |
| --- | --- |
| Messenger | [alpha.1](https://github.com/kryptocorelabs/msg/releases/tag/v0.1.0-alpha.1), [alpha.2 preview.4](https://github.com/kryptocorelabs/msg/releases/tag/v0.1.0-alpha.2-preview.4) |
| Container | [alpha.1](https://github.com/kryptocorelabs/container/releases/tag/v0.1.0-alpha.1), [alpha.2 preview.1](https://github.com/kryptocorelabs/container/releases/tag/v0.1.0-alpha.2-preview.1) |
| Rescue | [preview.2](https://github.com/kryptocorelabs/rescue/releases/tag/v0.1.0-preview.2), [preview.3](https://github.com/kryptocorelabs/rescue/releases/tag/v0.1.0-preview.3) |
| Vault for Firefox | [0.1.16 XPI](https://github.com/kryptocorelabs/vault/releases/tag/firefox-v0.1.16) |

The current manifest pairs expire on 2027-02-07 at 09:56:11 UTC. The historical
Windows Vault ZIPs provide checksums and still need a verifiable build/source
mapping for publisher signing.

## Check an available Windows preview

Download the ZIP and `SHA256SUMS.txt` from the same release. In PowerShell,
run `Get-FileHash -Algorithm SHA256 -LiteralPath .\your-download.zip` and
compare the result to the line for that exact filename. Matching hashes confirm
that the download matches the published digest. A trusted publisher signature
requires an independently trusted key.

When a release provides `RELEASE_MANIFEST.json`, check its source commit and
build context as well. For example, [Rescue preview.3](https://github.com/kryptocorelabs/rescue/releases/tag/v0.1.0-preview.3)
records its public source revision and hashes for the ZIP and EXE. The [build
guide](BUILD.md#windows-application-builds) explains which artifacts have
reproducibility and provenance evidence.

## Check a publisher-signed release

1. Obtain [KryptoCore Verify](https://github.com/kryptocorelabs/apps-source/tree/main/apps/verify)
   through a trusted route, or build it from a reviewed source revision.
2. On the chosen GitHub release, download the archive and its matching
   **`<archive-name>.release.json`** and **`<archive-name>.release.sig`** files.
   Keep the download's original filename. If either signature file is missing,
   this verification route is not available for that asset.
3. Confirm the publisher's full `KCP1:` public key through an independently
   trusted route. A key bundled with an untrusted download does not establish
   whose key it is. Never supply a password or private signing key to check a file.
4. Run the check below using the product, version and target you intended to
   download. Continue only if Verify exits successfully and prints `VERIFIED`.
   A rejected check means stop, preserve the files for diagnosis and recheck the
   source; do not disable the check to make the download open.

### Windows PowerShell example

From the folder containing Verify and the three downloaded files:

```powershell
$KryptoReleaseArchive = 'KryptoCore-Rescue-0.1.0-preview.3-windows-x64.zip'
$KryptoPublisherKey = (Get-Content -Raw -LiteralPath .\PUBLIC_RELEASE_KEY.txt).Trim()
.\kryptocore-verify.exe check $KryptoReleaseArchive "$KryptoReleaseArchive.release.json" "$KryptoReleaseArchive.release.sig" --key $KryptoPublisherKey --product rescue --version 0.1.0-preview.3 --target windows-x86_64
```

Place your independently verified `PUBLIC_RELEASE_KEY.txt` in the same folder.
For another download, use its exact three filenames and expected metadata.
Choose the expected version from the release you intended to download.

On Linux/macOS, use `./kryptocore-verify` with the same arguments and ordinary
relative paths. Target names are `linux-x86_64`, `macos-x86_64`,
`macos-aarch64` or `windows-x86_64`; Firefox XPI manifests use `firefox`.
Verify can check an archive for another OS without running it.

## What the result means

Verify checks the purpose-bound signature, expected product/version/platform,
filename, exact size and SHA-256, plus the signed validity period. Keep the
system clock correct. `--min-sequence` can enforce a minimum learned through a
trusted route; it does not automatically discover the latest release.

`VERIFIED` means the checked bytes match the selected key and metadata. It does
not replace Windows Authenticode, SmartScreen, Apple notarization or an audit.
Files can still be replaced after checking. A signature added to a historical
preview does not patch it or remove its documented security limits.

[Detailed protocol](https://github.com/kryptocorelabs/apps-source/blob/main/apps/verify/PROTOCOL.md)
· [Security in 30 seconds](https://github.com/kryptocorelabs/kryptocore/blob/main/SECURITY-IN-BRIEF.md)
