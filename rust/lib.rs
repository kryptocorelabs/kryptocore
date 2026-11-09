#![forbid(unsafe_code)]
//! Local authenticated encryption and password-based key derivation.

use argon2::{Algorithm, Argon2, Block, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305,
};
use std::fmt;
use zeroize::Zeroizing;

mod context;
#[cfg(feature = "stream")]
pub mod stream;
pub mod wrapping;
pub use context::{Context, ContextError};

/// A secret key whose backing bytes are overwritten on drop.
pub struct Key(Zeroizing<[u8; 32]>);

impl Key {
    /// Import 32 bytes of existing secret key material into an opaque key.
    /// The caller retains responsibility for the source buffer. Use
    /// [`derive_key`] for passwords and authenticate any wrapped input first.
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let mut key = Zeroizing::new([0; 32]);
        key.copy_from_slice(bytes);
        Self(key)
    }

    /// Generate a new key using the operating system's random source.
    pub fn generate() -> Result<Self, Error> {
        let mut key = Zeroizing::new([0u8; 32]);
        getrandom::fill(&mut key[..]).map_err(|_| Error::RandomnessUnavailable)?;
        Ok(Self(key))
    }
}
/// A public, random salt for the password derivation profile.
pub type Salt = [u8; 16];
/// A public XChaCha20-Poly1305 nonce.
pub type Nonce = [u8; 24];

/// Generate a fresh 16-byte salt for a new password derivation context.
/// Store it with the encrypted data and reuse that stored salt when opening.
pub fn generate_salt() -> Result<Salt, Error> {
    random_bytes()
}

/// Generate a fresh 24-byte XChaCha20-Poly1305 nonce using OS randomness.
/// [`encrypt`] does this internally and returns the nonce to store.
pub fn generate_nonce() -> Result<Nonce, Error> {
    random_bytes()
}

fn random_bytes<const N: usize>() -> Result<[u8; N], Error> {
    let mut bytes = [0; N];
    getrandom::fill(&mut bytes).map_err(|_| Error::RandomnessUnavailable)?;
    Ok(bytes)
}

/// Failure of a cryptographic operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    EmptyPassword,
    DerivationFailed,
    RandomnessUnavailable,
    EncryptionFailed,
    AuthenticationFailed,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyPassword => "Password is empty.",
            Self::DerivationFailed => "Key derivation failed.",
            Self::RandomnessUnavailable => "Operating system randomness is unavailable.",
            Self::EncryptionFailed => "Encryption failed.",
            Self::AuthenticationFailed => "Ciphertext authentication failed.",
        })
    }
}

impl std::error::Error for Error {}

/// Actionable failures for callers that want to handle key derivation errors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum KeyDerivationError {
    EmptyPassword,
    PasswordTooLong,
    MemoryUnavailable,
    DerivationFailed,
}

impl fmt::Display for KeyDerivationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::EmptyPassword => "Password is empty.",
            Self::PasswordTooLong => "Password exceeds the supported length.",
            Self::MemoryUnavailable => "Key derivation could not reserve its work memory.",
            Self::DerivationFailed => "Key derivation failed.",
        })
    }
}

impl std::error::Error for KeyDerivationError {}

/// Derive a key using Argon2id v19 with 256 MiB, three passes and one lane.
/// The caller supplies a fresh random salt for each new derivation context
/// and bounds the number of concurrent derivations.
/// Returns [`Error::DerivationFailed`] if the allocator rejects the work buffer reservation.
pub fn derive_key(password: &[u8], salt: &Salt) -> Result<Key, Error> {
    derive_key_detailed(password, salt).map_err(|error| match error {
        KeyDerivationError::EmptyPassword => Error::EmptyPassword,
        _ => Error::DerivationFailed,
    })
}

/// Derive the same key as [`derive_key`], with separate input and memory errors.
pub fn derive_key_detailed(password: &[u8], salt: &Salt) -> Result<Key, KeyDerivationError> {
    if password.is_empty() {
        return Err(KeyDerivationError::EmptyPassword);
    }
    if password.len() > argon2::MAX_PWD_LEN {
        return Err(KeyDerivationError::PasswordTooLong);
    }
    let params = Params::new(256 * 1024, 3, 1, Some(32))
        .map_err(|_| KeyDerivationError::DerivationFailed)?;
    let mut memory = Zeroizing::new(Vec::new());
    memory
        .try_reserve_exact(params.block_count())
        .map_err(|_| KeyDerivationError::MemoryUnavailable)?;
    memory.resize(params.block_count(), Block::default());
    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::new(Algorithm::Argon2id, Version::V0x13, params)
        .hash_password_into_with_memory(password, salt, &mut key[..], &mut memory[..])
        .map_err(|_| KeyDerivationError::DerivationFailed)?;
    Ok(Key(key))
}

/// Encrypt a message and return a fresh nonce and the ciphertext with its tag.
/// Associated data is authenticated and remains public.
pub fn encrypt(key: &Key, plaintext: &[u8], aad: &[u8]) -> Result<(Nonce, Vec<u8>), Error> {
    let nonce = generate_nonce()?;
    let ciphertext = XChaCha20Poly1305::new((&*key.0).into())
        .encrypt(
            (&nonce).into(),
            Payload {
                msg: plaintext,
                aad,
            },
        )
        .map_err(|_| Error::EncryptionFailed)?;
    Ok((nonce, ciphertext))
}

/// Authenticate the ciphertext and associated data before returning plaintext.
pub fn decrypt(
    key: &Key,
    nonce: &Nonce,
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Error> {
    XChaCha20Poly1305::new((&*key.0).into())
        .decrypt(
            nonce.into(),
            Payload {
                msg: ciphertext,
                aad,
            },
        )
        .map(Zeroizing::new)
        .map_err(|_| Error::AuthenticationFailed)
}

/// Input shape or authentication errors for a nonce received as bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum DecryptError {
    InvalidNonceLength,
    AuthenticationFailed,
}

impl fmt::Display for DecryptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidNonceLength => f.write_str("Nonce must be exactly 24 bytes."),
            Self::AuthenticationFailed => fmt::Display::fmt(&Error::AuthenticationFailed, f),
        }
    }
}

impl std::error::Error for DecryptError {}

/// Check a raw nonce's length, then authenticate exactly as [`decrypt`] does.
/// Wrong keys, wrong values in a well-sized nonce, wrong AAD and damaged
/// ciphertext all return the same authentication error.
pub fn decrypt_checked(
    key: &Key,
    nonce: &[u8],
    ciphertext: &[u8],
    aad: &[u8],
) -> Result<Zeroizing<Vec<u8>>, DecryptError> {
    let nonce = <&Nonce>::try_from(nonce).map_err(|_| DecryptError::InvalidNonceLength)?;
    decrypt(key, nonce, ciphertext, aad).map_err(|_| DecryptError::AuthenticationFailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn decrypts_published_xchacha20_poly1305_vector() {
        // draft-irtf-cfrg-xchacha-03, Appendix A.1
        let key = Key::from_bytes(&hex!(
            "808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f"
        ));
        let nonce = hex!("404142434445464748494a4b4c4d4e4f5051525354555657");
        let aad = hex!("50515253c0c1c2c3c4c5c6c7");
        let ciphertext = hex!(
            "bd6d179d3e83d43b9576579493c0e939572a1700252bfaccbed2902c21396cbb
             731c7f1b0b4aa6440bf3a82f4eda7e39ae64c6708c54c216cb96b72e1213b452
             2f8c9ba40db5d945b11b69b982c1bb9e3f3fac2bc369488f76b2383565d3fff9
             21f9664c97637da9768812f615c68b13b52ec0875924c1c7987947deafd8780acf49"
        );
        let plaintext = decrypt(&key, &nonce, &ciphertext, &aad).unwrap();
        assert_eq!(
            &plaintext[..],
            b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it."
        );
    }

    #[test]
    fn rejects_wrong_key_nonce_aad_and_modified_or_truncated_ciphertext() {
        let key = Key(Zeroizing::new([1; 32]));
        let wrong_key = Key(Zeroizing::new([2; 32]));
        let (nonce, ciphertext) = encrypt(&key, b"secret", b"vault/v1").unwrap();
        assert_eq!(
            &decrypt(&key, &nonce, &ciphertext, b"vault/v1").unwrap()[..],
            b"secret"
        );

        let mut wrong_nonce = nonce;
        wrong_nonce[0] ^= 1;
        let mut modified = ciphertext.clone();
        modified[0] ^= 1;
        for (key, nonce, data, aad) in [
            (&wrong_key, &nonce, ciphertext.as_slice(), &b"vault/v1"[..]),
            (&key, &wrong_nonce, &ciphertext, &b"vault/v1"[..]),
            (&key, &nonce, &ciphertext, &b"vault/v2"[..]),
            (&key, &nonce, &modified, &b"vault/v1"[..]),
            (&key, &nonce, &ciphertext[..15], &b"vault/v1"[..]),
        ] {
            assert!(matches!(
                decrypt(key, nonce, data, aad),
                Err(Error::AuthenticationFailed)
            ));
            assert!(matches!(
                decrypt_checked(key, nonce, data, aad),
                Err(DecryptError::AuthenticationFailed)
            ));
        }
        assert!(matches!(
            decrypt_checked(&key, &nonce[..23], &ciphertext, b"vault/v1"),
            Err(DecryptError::InvalidNonceLength)
        ));
    }

    #[test]
    fn generated_key_round_trips_an_empty_message() {
        let key = Key::generate().unwrap();
        let (nonce, ciphertext) = encrypt(&key, b"", b"").unwrap();
        assert_eq!(ciphertext.len(), 16);
        assert!(decrypt(&key, &nonce, &ciphertext, b"").unwrap().is_empty());
    }

    #[test]
    fn argon2id_matches_python_and_libsodium_reference() {
        let salt = hex!("000102030405060708090a0b0c0d0e0f");
        let key = derive_key(b"password used only in tests", &salt).unwrap();
        assert_eq!(
            *key.0,
            hex!("d2250c8b3ccc488a296fc8d6875c6a9a2cf6e838c79c14b714e3945ba46f8e3c")
        );
        assert!(matches!(derive_key(b"", &salt), Err(Error::EmptyPassword)));
        assert!(matches!(
            derive_key_detailed(b"", &salt),
            Err(KeyDerivationError::EmptyPassword)
        ));
    }
}
