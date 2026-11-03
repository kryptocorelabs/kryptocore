"""Python side of the Rust interoperability test. Only synthetic data is used."""

import sys

from kryptocore import AuthenticationError, decrypt, derive_key, encrypt

AAD = b"kryptocore/interop/v1"
MESSAGES = (b"", b"\0\xff\n\x80", b"a message from the same test fixture")


def reject(key, nonce, ciphertext, aad, expected_error):
    try:
        decrypt(key, nonce, ciphertext, aad=aad)
    except expected_error:
        return
    raise AssertionError("Invalid input was accepted")


def main():
    key = derive_key(b"synthetic interop password", bytes([42]) * 16)
    wrong_key = bytes([key[0] ^ 1]) + key[1:]
    for expected in MESSAGES:
        nonce, ciphertext = map(bytes.fromhex, sys.stdin.readline().split())
        assert decrypt(key, nonce, ciphertext, aad=AAD) == expected
        changed_nonce = bytes([nonce[0] ^ 1]) + nonce[1:]
        modified = bytes([ciphertext[0] ^ 1]) + ciphertext[1:]
        for bad_key, bad_nonce, bad_ciphertext, bad_aad in (
            (wrong_key, nonce, ciphertext, AAD),
            (key, changed_nonce, ciphertext, AAD),
            (key, nonce, modified, AAD),
            (key, nonce, ciphertext[:15], AAD),
            (key, nonce, ciphertext, b"other context"),
        ):
            reject(bad_key, bad_nonce, bad_ciphertext, bad_aad, AuthenticationError)
        for invalid_nonce in (nonce[:-1], nonce + b"\0"):
            reject(key, invalid_nonce, ciphertext, AAD, (ValueError, TypeError))
        nonce, ciphertext = encrypt(key, expected, aad=AAD)
        print(nonce.hex(), ciphertext.hex())
    assert not sys.stdin.read(), "Unexpected extra input"


if __name__ == "__main__":
    main()
