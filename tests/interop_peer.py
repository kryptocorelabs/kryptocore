"""Python side of the Rust interoperability test. Only synthetic data is used."""

import sys

from kryptocore import AuthenticationError, decrypt, derive_key, encrypt

AAD = b"kryptocore/interop/v1"
LARGE_AAD = bytes(range(256)) * 512
CASES = (
    (b"", b""),
    (b"", AAD),
    (b"\0\xff\n\x80", b""),
    (b"a message from the same test fixture", AAD),
    (b"", LARGE_AAD),
    (b"message with large AAD", LARGE_AAD),
)


def reject(key, nonce, ciphertext, aad, expected_error):
    try:
        decrypt(key, nonce, ciphertext, aad=aad)
    except expected_error:
        return
    raise AssertionError("Invalid input was accepted")


def main():
    key = derive_key(b"synthetic interop password", bytes([42]) * 16)
    wrong_key = bytes([key[0] ^ 1]) + key[1:]
    for expected, aad in CASES:
        nonce, ciphertext = map(bytes.fromhex, sys.stdin.readline().split())
        assert decrypt(key, nonce, ciphertext, aad=aad) == expected
        changed_nonce = bytes([nonce[0] ^ 1]) + nonce[1:]
        modified = ciphertext[:-1] + bytes([ciphertext[-1] ^ 1])
        changed_aad = aad[:-1] + bytes([aad[-1] ^ 1]) if aad else b"\0"
        for bad_key, bad_nonce, bad_ciphertext, bad_aad in (
            (wrong_key, nonce, ciphertext, aad),
            (key, changed_nonce, ciphertext, aad),
            (key, nonce, modified, aad),
            (key, nonce, b"", aad),
            (key, nonce, ciphertext[:15], aad),
            (key, nonce, ciphertext[:-1], aad),
            (key, nonce, ciphertext + b"\0", aad),
            (key, nonce, ciphertext, changed_aad),
        ):
            reject(bad_key, bad_nonce, bad_ciphertext, bad_aad, AuthenticationError)
        for invalid_nonce in (b"", nonce[:-1], nonce + b"\0"):
            reject(key, invalid_nonce, ciphertext, aad, (ValueError, TypeError))
        nonce, ciphertext = encrypt(key, expected, aad=aad)
        print(nonce.hex(), ciphertext.hex())
    assert not sys.stdin.read(), "Unexpected extra input"


if __name__ == "__main__":
    main()
