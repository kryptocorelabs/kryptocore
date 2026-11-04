use std::{sync::Barrier, thread};

use kryptocore::{decrypt, derive_key, encrypt, Error};

const PASSWORD: &[u8] = b"synthetic password for memory tests";
const SALT: [u8; 16] = [42; 16];

#[test]
#[ignore = "uses 512 MiB for two concurrent Argon2id work buffers"]
fn argon2id_derives_keys_concurrently() {
    let start = Barrier::new(2);
    let (first, second) = thread::scope(|scope| {
        let worker = scope.spawn(|| {
            start.wait();
            derive_key(PASSWORD, &SALT).unwrap()
        });
        start.wait();
        let second = derive_key(PASSWORD, &SALT).unwrap();
        (worker.join().unwrap(), second)
    });

    let (nonce, ciphertext) = encrypt(&first, b"memory check", b"test/v1").unwrap();
    let plaintext = decrypt(&second, &nonce, &ciphertext, b"test/v1").unwrap();
    assert_eq!(&plaintext[..], b"memory check");
}

#[test]
#[ignore = "run the compiled test with a process memory limit below 256 MiB"]
fn argon2id_reports_reservation_failure() {
    assert!(matches!(
        derive_key(PASSWORD, &SALT),
        Err(Error::DerivationFailed)
    ));
}
