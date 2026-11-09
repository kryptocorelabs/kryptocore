import pytest
from nacl.pwhash import argon2id

from kryptocore import AuthenticationError, decrypt, derive_key, encrypt, generate_salt


def test_xchacha20_poly1305_known_answer(monkeypatch):
    # https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha-03#appendix-A.1
    key = bytes(range(0x80, 0xA0))
    nonce = bytes(range(0x40, 0x58))
    aad = bytes.fromhex("50515253c0c1c2c3c4c5c6c7")
    plaintext = (
        b"Ladies and Gentlemen of the class of '99: If I could offer you only "
        b"one tip for the future, sunscreen would be it."
    )
    ciphertext = bytes.fromhex(
        "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb"
        "731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b452"
        "2f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9"
        "21f9664c97637da9768812f615c68b13b52ec0875924c1c7987947deafd8780acf49"
    )
    monkeypatch.setattr("kryptocore._crypto.secrets.token_bytes", lambda size: nonce)
    assert encrypt(key, plaintext, aad=aad) == (nonce, ciphertext)
    assert decrypt(key, nonce, ciphertext, aad=aad) == plaintext


@pytest.mark.parametrize("field", ["key", "nonce", "ciphertext", "aad"])
def test_tampering_is_rejected(field):
    key, aad = bytes(32), b"test/v1"
    nonce, ciphertext = encrypt(key, b"secret", aad=aad)
    args = dict(key=key, nonce=nonce, ciphertext=ciphertext, aad=aad)
    value = args[field]
    args[field] = bytes([value[0] ^ 1]) + value[1:]
    with pytest.raises(AuthenticationError):
        decrypt(**args)


def test_empty_message_and_fresh_nonce():
    key = bytes(32)
    nonce, ciphertext = encrypt(key, b"", aad=b"")
    assert len(nonce) == 24
    assert nonce != encrypt(key, b"", aad=b"")[0]
    assert decrypt(key, nonce, ciphertext, aad=b"") == b""
    with pytest.raises(AuthenticationError):
        decrypt(key, nonce, ciphertext[:-1], aad=b"")


def test_implicit_aad_warns_and_preserves_empty_aad_compatibility():
    key = bytes(32)
    with pytest.warns(DeprecationWarning, match="Pass aad explicitly"):
        nonce, ciphertext = encrypt(key, b"old format")
    assert decrypt(key, nonce, ciphertext, aad=b"") == b"old format"
    with pytest.warns(DeprecationWarning, match="Pass aad explicitly"):
        assert decrypt(key, nonce, ciphertext) == b"old format"


def test_fresh_salt():
    salt = generate_salt()
    assert len(salt) == 16
    assert salt != generate_salt()


def test_argon2id_matches_libsodium():
    password = b"password used only in tests"
    salt = bytes(range(16))
    expected = argon2id.kdf(32, password, salt, opslimit=3, memlimit=256 * 1024**2)
    assert derive_key(password, salt) == expected


@pytest.mark.parametrize("password,salt", [(b"", bytes(16)), (b"password", b"short")])
def test_invalid_derivation_inputs(password, salt):
    with pytest.raises(ValueError):
        derive_key(password, salt)


@pytest.mark.parametrize("nonce", [b"", bytes(23), bytes(25), "not bytes"])
def test_rejects_malformed_nonce(nonce):
    key = bytes(32)
    _, ciphertext = encrypt(key, b"binary\0\xff", aad=b"context\0")
    with pytest.raises((ValueError, TypeError)):
        decrypt(key, nonce, ciphertext, aad=b"context\0")


def test_binary_inputs_and_appended_ciphertext():
    key = bytes(range(32))
    aad = bytes(range(256))
    nonce, ciphertext = encrypt(key, aad, aad=aad)
    assert decrypt(key, nonce, ciphertext, aad=aad) == aad
    with pytest.raises(AuthenticationError):
        decrypt(key, nonce, ciphertext + b"\0", aad=aad)
