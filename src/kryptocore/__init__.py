"""Local authenticated encryption and password-based key derivation."""

from ._crypto import AuthenticationError, decrypt, derive_key, encrypt, generate_salt

__all__ = ["AuthenticationError", "decrypt", "derive_key", "encrypt", "generate_salt"]
