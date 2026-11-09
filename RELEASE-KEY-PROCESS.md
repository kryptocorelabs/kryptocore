# Release key procedure

The current release key is published in a
[verified anchor commit](https://github.com/kryptocorelabs/kryptocore/blob/2e7bc84a45dc26b03cbc1bf3575f5f8f1af55e36/PUBLIC_RELEASE_KEY.txt).
Its public-key fingerprint is listed in [Verify downloads](VERIFY-DOWNLOADS.md).
Record the key fingerprint, anchor commit, affected products and release
sequence when reviewing a signature.

## Create and use a key

1. Generate a dedicated publisher key with Verify on a trusted offline
   machine. Protect the encrypted key file with a strong, unique password.
2. Store a protected backup of the encrypted key and keep the password
   separately. Test recovery on the trusted machine before relying on it.
   Keep both out of GitHub, build runners, chat, and project archives.
3. Publish the public key and fingerprint in a verified source commit.
   Confirm the fingerprint through an independently trusted maintainer
   channel and document the activation date and the first release sequence.
4. Review the exact artifact digest, size, source commit and build record
   before signing. Sign on the trusted machine and verify the output locally.
5. Publish the manifest and detached signature with the download. Fetch the
   public artifact again and verify it against the anchored public key.

## Suspected compromise

1. Suspend signing with the affected key. Treat signatures from the
   suspected compromise window as untrusted and preserve the records needed
   to identify affected artifacts.
2. Publish a security advisory and a verified trust notice with the old
   fingerprint, effective time, affected versions, and safe recovery steps.
   Use the independent maintainer channel from the key ceremony to confirm it.
3. Establish a replacement key on a clean machine. Anchor it separately and
   confirm its fingerprint using the independent channel. An old-key
   signature alone cannot authorize its replacement.
4. Rebuild and verify affected downloads from reviewed commits. Publish
   replacements under fresh release versions and advance each product's
   sequence. Keep the advisory and retired fingerprint publicly visible.

An offline Verify installation follows the key and minimum sequence supplied
by its operator. It needs the published trust notice and replacement key
before it can act on a revocation. The GitHub build attestation, the publisher
key, and Windows Authenticode have separate trust roots.
