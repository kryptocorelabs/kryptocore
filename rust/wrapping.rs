//! Protect a software key with an independent key or a password.

use crate::{
    decrypt, derive_key_detailed, encrypt, generate_salt, Context, Error, Key, KeyDerivationError,
    Nonce, Salt,
};
use std::fmt;
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

const KEY_LABEL: &[u8] = b"KryptoCore/wrap-key\0\x01";
const PASSWORD_LABEL: &[u8] = b"KryptoCore/wrap-password\0\x01";

/// Public fields of a wrapped 32-byte key. The ciphertext includes the tag.
pub struct WrappedKey {
    pub nonce: Nonce,
    pub ciphertext: [u8; 48],
}

/// Public fields of a password-wrapped key using the current Argon2id profile.
pub struct PasswordWrappedKey {
    pub salt: Salt,
    pub wrapped: WrappedKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum WrapError {
    SameKey,
    Crypto(Error),
    Derivation(KeyDerivationError),
}

impl fmt::Display for WrapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SameKey => f.write_str("A key must be wrapped with an independent key."),
            Self::Crypto(error) => fmt::Display::fmt(error, f),
            Self::Derivation(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl std::error::Error for WrapError {}

/// Wrap a key with a fresh nonce and the expected application context.
pub fn wrap_key(wrapping_key: &Key, key: &Key, context: &Context) -> Result<WrappedKey, WrapError> {
    seal(wrapping_key, key, &[KEY_LABEL, context.as_bytes()].concat())
}

/// Authenticate and recover an opaque key using the same context.
pub fn unwrap_key(
    wrapping_key: &Key,
    wrapped: &WrappedKey,
    context: &Context,
) -> Result<Key, WrapError> {
    open(
        wrapping_key,
        wrapped,
        &[KEY_LABEL, context.as_bytes()].concat(),
    )
}

/// Generate a new salt and protect a key with the fixed Argon2id profile.
pub fn wrap_key_with_password(
    password: &[u8],
    key: &Key,
    context: &Context,
) -> Result<PasswordWrappedKey, WrapError> {
    let salt = generate_salt().map_err(WrapError::Crypto)?;
    let wrapping_key = derive_key_detailed(password, &salt).map_err(WrapError::Derivation)?;
    let aad = [PASSWORD_LABEL, &salt, context.as_bytes()].concat();
    Ok(PasswordWrappedKey {
        salt,
        wrapped: seal(&wrapping_key, key, &aad)?,
    })
}

/// Derive the wrapping key from the stored salt and authenticate the wrapped key.
pub fn unwrap_key_with_password(
    password: &[u8],
    wrapped: &PasswordWrappedKey,
    context: &Context,
) -> Result<Key, WrapError> {
    let wrapping_key =
        derive_key_detailed(password, &wrapped.salt).map_err(WrapError::Derivation)?;
    let aad = [PASSWORD_LABEL, &wrapped.salt, context.as_bytes()].concat();
    open(&wrapping_key, &wrapped.wrapped, &aad)
}

fn seal(wrapping_key: &Key, key: &Key, aad: &[u8]) -> Result<WrappedKey, WrapError> {
    if bool::from(wrapping_key.0[..].ct_eq(&key.0[..])) {
        return Err(WrapError::SameKey);
    }
    let (nonce, ciphertext) = encrypt(wrapping_key, &key.0[..], aad).map_err(WrapError::Crypto)?;
    let ciphertext = ciphertext
        .try_into()
        .map_err(|_| WrapError::Crypto(Error::EncryptionFailed))?;
    Ok(WrappedKey { nonce, ciphertext })
}

fn open(wrapping_key: &Key, wrapped: &WrappedKey, aad: &[u8]) -> Result<Key, WrapError> {
    let plaintext = decrypt(wrapping_key, &wrapped.nonce, &wrapped.ciphertext, aad)
        .map_err(WrapError::Crypto)?;
    let mut key = Zeroizing::new([0; 32]);
    key.copy_from_slice(&plaintext);
    Ok(Key(key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const AUTH_FAILED: WrapError = WrapError::Crypto(Error::AuthenticationFailed);

    #[test]
    fn key_wrapping_round_trip_and_rejects_wrong_inputs() {
        let context = Context::new("example", 1, "slot", b"one").unwrap();
        let key = Key::generate().unwrap();
        let kek = Key::generate().unwrap();
        let mut wrapped = wrap_key(&kek, &key, &context).unwrap();
        let opened = unwrap_key(&kek, &wrapped, &context).unwrap();
        let (nonce, ciphertext) = encrypt(&key, b"data", b"").unwrap();
        assert_eq!(
            &decrypt(&opened, &nonce, &ciphertext, b"").unwrap()[..],
            b"data"
        );

        assert_eq!(
            unwrap_key(&key, &wrapped, &context).err(),
            Some(AUTH_FAILED)
        );
        for different in [
            Context::new("other", 1, "slot", b"one").unwrap(),
            Context::new("example", 2, "slot", b"one").unwrap(),
            Context::new("example", 1, "payload", b"one").unwrap(),
            Context::new("example", 1, "slot", b"two").unwrap(),
        ] {
            assert_eq!(
                unwrap_key(&kek, &wrapped, &different).err(),
                Some(AUTH_FAILED)
            );
        }
        for offset in [0, 47] {
            wrapped.ciphertext[offset] ^= 1;
            assert_eq!(
                unwrap_key(&kek, &wrapped, &context).err(),
                Some(AUTH_FAILED)
            );
            wrapped.ciphertext[offset] ^= 1;
        }
        wrapped.nonce[0] ^= 1;
        assert_eq!(
            unwrap_key(&kek, &wrapped, &context).err(),
            Some(AUTH_FAILED)
        );
        let copy = Key(Zeroizing::new(*key.0));
        assert_eq!(
            wrap_key(&copy, &key, &context).err(),
            Some(WrapError::SameKey)
        );
    }

    #[test]
    fn password_wrapping_authenticates_salt_and_has_its_own_domain() {
        let context = Context::new("example", 1, "slot", b"").unwrap();
        let key = Key::generate().unwrap();
        let password = b"synthetic test passphrase";
        let mut wrapped = wrap_key_with_password(password, &key, &context).unwrap();
        let opened = unwrap_key_with_password(password, &wrapped, &context).unwrap();
        assert!(bool::from(opened.0[..].ct_eq(&key.0[..])));
        assert_eq!(
            unwrap_key_with_password(b"wrong", &wrapped, &context).err(),
            Some(AUTH_FAILED)
        );
        let kek = derive_key_detailed(password, &wrapped.salt).unwrap();
        assert_eq!(
            unwrap_key(&kek, &wrapped.wrapped, &context).err(),
            Some(AUTH_FAILED)
        );
        wrapped.salt[0] ^= 1;
        assert_eq!(
            unwrap_key_with_password(password, &wrapped, &context).err(),
            Some(AUTH_FAILED)
        );
        assert_eq!(
            wrap_key_with_password(b"", &key, &context).err(),
            Some(WrapError::Derivation(KeyDerivationError::EmptyPassword))
        );
    }

    #[test]
    fn opens_independent_libsodium_vector() {
        let key = Key(Zeroizing::new([7; 32]));
        let context = Context::new("example", 1, "test", b"").unwrap();
        let wrapped = WrappedKey {
            nonce: std::array::from_fn(|i| i as u8),
            ciphertext: hex!("a01fa38df21453ad2dcf894feeeaa7a457de301f1cff3f79fb15bf8992c49c9fd638318dad150b61d75a2f87ffe55d49"),
        };
        assert!(bool::from(
            unwrap_key(&key, &wrapped, &context).unwrap().0[..].ct_eq(&[8; 32])
        ));
    }
}
