#![no_main]
#![forbid(unsafe_code)]

use std::sync::OnceLock;

use kryptocore::{decrypt, encrypt, Error, Key, Nonce};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    static KEY: OnceLock<Key> = OnceLock::new();
    let key = KEY.get_or_init(|| Key::generate().expect("OS randomness"));

    let (selector, data) = data.split_first().unwrap_or((&0, &[]));
    let (aad, plaintext) = data.split_at(usize::from(*selector).min(data.len()));
    let (nonce, mut ciphertext) = encrypt(key, plaintext, aad).expect("bounded encryption input");
    assert_eq!(
        &decrypt(key, &nonce, &ciphertext, aad).unwrap()[..],
        plaintext
    );
    ciphertext
        .last_mut()
        .map(|tag_byte| *tag_byte ^= 1)
        .expect("authentication tag");
    assert!(matches!(
        decrypt(key, &nonce, &ciphertext, aad),
        Err(Error::AuthenticationFailed)
    ));

    if let Some((nonce_bytes, ciphertext)) = data.split_at_checked(24) {
        let nonce = Nonce::try_from(nonce_bytes).unwrap();
        let _ = decrypt(key, &nonce, ciphertext, aad);
    }
});
