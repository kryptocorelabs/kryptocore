use std::{sync::Barrier, thread};

use kryptocore::{decrypt, derive_key, derive_key_detailed, encrypt, Error, KeyDerivationError};

const PASSWORD: &[u8] = b"synthetic password for memory tests";
const SALT: [u8; 16] = [42; 16];

#[test]
#[ignore = "uses 512 MiB for two concurrent Argon2id work buffers"]
fn argon2id_derives_keys_concurrently() {
    let other_salt = [43; 16];
    let expected_first = derive_key(PASSWORD, &SALT).unwrap();
    let expected_second = derive_key(PASSWORD, &other_salt).unwrap();
    let start = Barrier::new(2);
    let (first, second) = thread::scope(|scope| {
        let worker = scope.spawn(|| {
            start.wait();
            derive_key(PASSWORD, &SALT).unwrap()
        });
        start.wait();
        let second = derive_key_detailed(PASSWORD, &other_salt).unwrap();
        (worker.join().unwrap(), second)
    });

    let context = b"test/v1";
    let message = b"memory check";
    for (actual, expected, wrong) in [
        (&first, &expected_first, &expected_second),
        (&second, &expected_second, &expected_first),
    ] {
        let (nonce, ciphertext) = encrypt(actual, message, context).unwrap();
        let plaintext = decrypt(expected, &nonce, &ciphertext, context).unwrap();
        assert_eq!(&plaintext[..], message);
        assert!(matches!(
            decrypt(wrong, &nonce, &ciphertext, context),
            Err(Error::AuthenticationFailed)
        ));
    }
}

#[test]
#[ignore = "run the compiled test with a process memory limit below 256 MiB"]
fn argon2id_reports_reservation_failure() {
    assert!(matches!(
        derive_key_detailed(PASSWORD, &SALT),
        Err(KeyDerivationError::MemoryUnavailable)
    ));
    assert!(matches!(
        derive_key(PASSWORD, &SALT),
        Err(Error::DerivationFailed)
    ));
}
