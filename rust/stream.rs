//! Authenticated, bounded-memory encryption for finite streams.

use crate::{random_bytes, Context, Key};
use chacha20poly1305::{
    aead::{
        stream::{DecryptorBE32, EncryptorBE32},
        Payload,
    },
    XChaCha20Poly1305,
};
use std::{
    fmt,
    io::{self, Read, Write},
};
use zeroize::Zeroizing;

const MAGIC: &[u8; 8] = b"KCSSTRM\x01";
const CHUNK_BYTES: usize = 64 * 1024;
const TAG_BYTES: usize = 16;
const RECORD_BYTES: usize = CHUNK_BYTES + TAG_BYTES;

/// Failure of a finite stream operation. Output may have been partially written.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum StreamError {
    Io(io::ErrorKind),
    InvalidHeader,
    RandomnessUnavailable,
    EncryptionFailed,
    AuthenticationFailed,
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(kind) => write!(f, "Stream I/O failed ({kind})."),
            Self::InvalidHeader => f.write_str("Stream header is incomplete or unsupported."),
            Self::RandomnessUnavailable => {
                f.write_str("Operating system randomness is unavailable.")
            }
            Self::EncryptionFailed => f.write_str("Stream encryption failed."),
            Self::AuthenticationFailed => f.write_str("Stream authentication failed."),
        }
    }
}

impl std::error::Error for StreamError {}

impl From<io::Error> for StreamError {
    fn from(error: io::Error) -> Self {
        Self::Io(error.kind())
    }
}

/// Encrypt through EOF with a fresh nonce prefix. Use a key dedicated to streaming.
/// Stage the output and publish it only after success and a successful flush.
pub fn encrypt_stream(
    key: &Key,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &Context,
) -> Result<(), StreamError> {
    let nonce = random_bytes::<19>().map_err(|_| StreamError::RandomnessUnavailable)?;
    let aad = [MAGIC.as_slice(), &nonce, context.as_bytes()].concat();
    let mut encryptor = EncryptorBE32::<XChaCha20Poly1305>::new((&*key.0).into(), (&nonce).into());
    let mut chunk = Zeroizing::new(vec![0; CHUNK_BYTES]);
    output.write_all(MAGIC)?;
    output.write_all(&nonce)?;
    loop {
        let len = read_chunk(input, &mut chunk)?;
        let payload = Payload {
            msg: &chunk[..len],
            aad: &aad,
        };
        if len < CHUNK_BYTES {
            let last = encryptor
                .encrypt_last(payload)
                .map_err(|_| StreamError::EncryptionFailed)?;
            output.write_all(&last)?;
            return Ok(());
        }
        let record = encryptor
            .encrypt_next(payload)
            .map_err(|_| StreamError::EncryptionFailed)?;
        output.write_all(&record)?;
    }
}

/// Authenticate records through EOF, including a mandatory final record.
/// A valid plaintext prefix can be written before a later failure. Use private
/// staging and make plaintext available only after success and a successful flush.
pub fn decrypt_stream(
    key: &Key,
    input: &mut impl Read,
    output: &mut impl Write,
    context: &Context,
) -> Result<(), StreamError> {
    let mut header = [0; MAGIC.len() + 19];
    if read_chunk(input, &mut header)? != header.len() || !header.starts_with(MAGIC) {
        return Err(StreamError::InvalidHeader);
    }
    let nonce: &[u8; 19] = header[MAGIC.len()..].try_into().expect("fixed header size");
    let aad = [header.as_slice(), context.as_bytes()].concat();
    let mut decryptor = DecryptorBE32::<XChaCha20Poly1305>::new((&*key.0).into(), nonce.into());
    let mut record = vec![0; RECORD_BYTES];
    loop {
        let len = read_chunk(input, &mut record)?;
        if len < TAG_BYTES {
            return Err(StreamError::AuthenticationFailed);
        }
        let payload = Payload {
            msg: &record[..len],
            aad: &aad,
        };
        if len < RECORD_BYTES {
            let last = decryptor
                .decrypt_last(payload)
                .map_err(|_| StreamError::AuthenticationFailed)?;
            output.write_all(&Zeroizing::new(last))?;
            return Ok(());
        }
        let chunk = decryptor
            .decrypt_next(payload)
            .map_err(|_| StreamError::AuthenticationFailed)?;
        output.write_all(&Zeroizing::new(chunk))?;
    }
}

fn read_chunk(input: &mut impl Read, mut buffer: &mut [u8]) -> io::Result<usize> {
    let capacity = buffer.len();
    while !buffer.is_empty() {
        match input.read(buffer) {
            Ok(0) => break,
            Ok(len) => buffer = &mut buffer[len..],
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(capacity - buffer.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    const HEADER_BYTES: usize = MAGIC.len() + 19;

    fn context() -> Context {
        Context::new("example", 1, "test", b"").unwrap()
    }

    struct ShortReads<'a>(&'a [u8]);

    impl Read for ShortReads<'_> {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            let len = buffer.len().min(101);
            self.0.read(&mut buffer[..len])
        }
    }

    #[test]
    fn round_trip_across_chunk_boundaries_and_short_reads() {
        let key = Key::generate().unwrap();
        let context = context();
        for len in [
            0,
            1,
            CHUNK_BYTES - 1,
            CHUNK_BYTES,
            CHUNK_BYTES + 1,
            2 * CHUNK_BYTES,
        ] {
            let message: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let mut encrypted = Vec::new();
            encrypt_stream(&key, &mut ShortReads(&message), &mut encrypted, &context).unwrap();
            assert_eq!(
                encrypted.len(),
                HEADER_BYTES + len + TAG_BYTES * (len / CHUNK_BYTES + 1)
            );
            let mut output = Vec::new();
            decrypt_stream(&key, &mut ShortReads(&encrypted), &mut output, &context).unwrap();
            assert_eq!(output, message);
        }
    }

    #[test]
    fn rejects_wrong_keys_and_contexts_and_tampered_streams() {
        let key = Key::generate().unwrap();
        let other_key = Key::generate().unwrap();
        let context = context();
        let other_context = Context::new("example", 2, "test", b"").unwrap();
        let mut message = vec![0x12; CHUNK_BYTES];
        message.resize(2 * CHUNK_BYTES, 0x34);
        let mut encrypted = Vec::new();
        encrypt_stream(&key, &mut message.as_slice(), &mut encrypted, &context).unwrap();
        let open =
            |key, ctx, mut input: &[u8]| decrypt_stream(key, &mut input, &mut io::sink(), ctx);
        let auth_failed = Err(StreamError::AuthenticationFailed);
        assert_eq!(open(&other_key, &context, &encrypted), auth_failed);
        assert_eq!(open(&key, &other_context, &encrypted), auth_failed);
        for offset in [MAGIC.len(), HEADER_BYTES, encrypted.len() - 1] {
            encrypted[offset] ^= 1;
            assert_eq!(open(&key, &context, &encrypted), auth_failed);
            encrypted[offset] ^= 1;
        }
        encrypted[0] ^= 1;
        assert_eq!(
            open(&key, &context, &encrypted),
            Err(StreamError::InvalidHeader)
        );
        encrypted[0] ^= 1;
        assert_eq!(
            open(&key, &context, &encrypted[..HEADER_BYTES - 1]),
            Err(StreamError::InvalidHeader)
        );
        for cut in [
            HEADER_BYTES,
            HEADER_BYTES + 20,
            HEADER_BYTES + RECORD_BYTES,
            encrypted.len() - 1,
        ] {
            assert_eq!(open(&key, &context, &encrypted[..cut]), auth_failed);
        }
        let mut staged = Vec::new();
        assert_eq!(
            decrypt_stream(
                &key,
                &mut &encrypted[..encrypted.len() - TAG_BYTES],
                &mut staged,
                &context
            ),
            auth_failed
        );
        assert_eq!(staged, message);

        let first = HEADER_BYTES;
        let second = first + RECORD_BYTES;
        let last = second + RECORD_BYTES;
        let swapped = [
            &encrypted[..first],
            &encrypted[second..last],
            &encrypted[first..second],
            &encrypted[last..],
        ]
        .concat();
        assert_eq!(open(&key, &context, &swapped), auth_failed);
        encrypted.push(0);
        assert_eq!(open(&key, &context, &encrypted), auth_failed);
    }

    #[test]
    fn opens_independent_libsodium_final_record_vector() {
        let key = Key(Zeroizing::new(std::array::from_fn(|i| i as u8)));
        let encoded = hex!("4b43535354524d01000102030405060708090a0b0c0d0e0f101112a755231937d75a32ee91ed7c425af76c96f7efc854b1ac5c98c048cce5");
        let mut output = Vec::new();
        decrypt_stream(&key, &mut encoded.as_slice(), &mut output, &context()).unwrap();
        assert_eq!(output, b"stream vector");
    }
}
