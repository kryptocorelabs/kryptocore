import secrets
from typing import Tuple

from argon2.low_level import Type, hash_secret_raw
from nacl import bindings
from nacl.exceptions import CryptoError

_KEY_BYTES = bindings.crypto_aead_xchacha20poly1305_ietf_KEYBYTES
_NONCE_BYTES = bindings.crypto_aead_xchacha20poly1305_ietf_NPUBBYTES
_TAG_BYTES = bindings.crypto_aead_xchacha20poly1305_ietf_ABYTES


class AuthenticationError(ValueError):
    """The ciphertext could not be authenticated."""


def derive_key(password: bytes, salt: bytes) -> bytes:
    """Derive a 32-byte key using the fixed Argon2id profile for this API."""
    if not isinstance(password, bytes) or not password:
        raise ValueError("password must be non-empty bytes")
    if not isinstance(salt, bytes) or len(salt) != 16:
        raise ValueError("salt must be exactly 16 bytes")
    return hash_secret_raw(
        secret=password,
        salt=salt,
        time_cost=3,
        memory_cost=256 * 1024,
        parallelism=1,
        hash_len=_KEY_BYTES,
        type=Type.ID,
        version=19,
    )


def encrypt(key: bytes, plaintext: bytes, *, aad: bytes = b"") -> Tuple[bytes, bytes]:
    """Return a fresh nonce and authenticated ciphertext, in that order."""
    nonce = secrets.token_bytes(_NONCE_BYTES)
    ciphertext = bindings.crypto_aead_xchacha20poly1305_ietf_encrypt(
        plaintext, aad, nonce, key
    )
    return nonce, ciphertext


def decrypt(key: bytes, nonce: bytes, ciphertext: bytes, *, aad: bytes = b"") -> bytes:
    """Authenticate ciphertext and associated data before returning plaintext."""
    if isinstance(ciphertext, bytes) and len(ciphertext) < _TAG_BYTES:
        raise AuthenticationError("ciphertext is shorter than the authentication tag")
    try:
        return bindings.crypto_aead_xchacha20poly1305_ietf_decrypt(
            ciphertext, aad, nonce, key
        )
    except CryptoError as exc:
        raise AuthenticationError("ciphertext authentication failed") from exc
