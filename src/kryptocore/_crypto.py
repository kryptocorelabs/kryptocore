import secrets
import warnings
from typing import Optional, Tuple

from argon2.low_level import Type, hash_secret_raw
from nacl import bindings
from nacl.exceptions import CryptoError

_KEY_BYTES = bindings.crypto_aead_xchacha20poly1305_ietf_KEYBYTES
_NONCE_BYTES = bindings.crypto_aead_xchacha20poly1305_ietf_NPUBBYTES
_TAG_BYTES = bindings.crypto_aead_xchacha20poly1305_ietf_ABYTES
_SALT_BYTES = 16


class AuthenticationError(ValueError):
    """The ciphertext could not be authenticated."""


def generate_salt() -> bytes:
    """Generate a fresh public salt for a new Argon2id derivation."""
    return secrets.token_bytes(_SALT_BYTES)


def derive_key(password: bytes, salt: bytes) -> bytes:
    """Derive a 32-byte key using the fixed Argon2id profile for this API."""
    if not isinstance(password, bytes) or not password:
        raise ValueError("password must be non-empty bytes")
    if not isinstance(salt, bytes) or len(salt) != _SALT_BYTES:
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


def _explicit_aad(aad: Optional[bytes]) -> bytes:
    if aad is not None:
        return aad
    warnings.warn(
        "Pass aad explicitly. Use aad=b'' when the format specifies empty AAD.",
        FutureWarning,
        stacklevel=3,
    )
    return b""


def encrypt(
    key: bytes, plaintext: bytes, *, aad: Optional[bytes] = None
) -> Tuple[bytes, bytes]:
    """Return a fresh nonce and ciphertext. Pass the format's public AAD explicitly."""
    aad = _explicit_aad(aad)
    nonce = secrets.token_bytes(_NONCE_BYTES)
    ciphertext = bindings.crypto_aead_xchacha20poly1305_ietf_encrypt(
        plaintext, aad, nonce, key
    )
    return nonce, ciphertext


def decrypt(
    key: bytes, nonce: bytes, ciphertext: bytes, *, aad: Optional[bytes] = None
) -> bytes:
    """Authenticate ciphertext and the supplied public AAD before returning plaintext."""
    aad = _explicit_aad(aad)
    if isinstance(ciphertext, bytes) and len(ciphertext) < _TAG_BYTES:
        raise AuthenticationError("ciphertext is shorter than the authentication tag")
    try:
        return bindings.crypto_aead_xchacha20poly1305_ietf_decrypt(
            ciphertext, aad, nonce, key
        )
    except CryptoError as exc:
        raise AuthenticationError("ciphertext authentication failed") from exc
