#![no_main]
#![forbid(unsafe_code)]

use std::sync::OnceLock;

use kryptocore::{decrypt, encrypt, Error, Key, Nonce};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    static KEY: OnceLock<Key> = OnceLock::new();
    static OTHER_KEY: OnceLock<Key> = OnceLock::new();
    let key = KEY.get_or_init(|| Key::generate().expect("OS randomness"));
    let other_key = OTHER_KEY.get_or_init(|| Key::generate().expect("OS randomness"));

    let (selector, data) = data.split_first().unwrap_or((&0, &[]));
    let (aad, plaintext) = data.split_at(usize::from(*selector).min(data.len()));
    let (nonce, mut ciphertext) = encrypt(key, plaintext, aad).expect("bounded encryption input");
    assert_eq!(
        &decrypt(key, &nonce, &ciphertext, aad).unwrap()[..],
        plaintext
    );
    assert!(matches!(
        decrypt(other_key, &nonce, &ciphertext, aad),
        Err(Error::AuthenticationFailed)
    ));
    let mut changed_nonce = nonce;
    changed_nonce[usize::from(*selector) % nonce.len()] ^= 1;
    assert!(matches!(
        decrypt(key, &changed_nonce, &ciphertext, aad),
        Err(Error::AuthenticationFailed)
    ));
    let changed_aad = [aad, &[0]].concat();
    assert!(matches!(
        decrypt(key, &nonce, &ciphertext, &changed_aad),
        Err(Error::AuthenticationFailed)
    ));
    let cut = usize::from(*selector) % ciphertext.len();
    assert!(matches!(
        decrypt(key, &nonce, &ciphertext[..cut], aad),
        Err(Error::AuthenticationFailed)
    ));
    ciphertext[cut] ^= 1;
    assert!(matches!(
        decrypt(key, &nonce, &ciphertext, aad),
        Err(Error::AuthenticationFailed)
    ));

    if let Some((nonce_bytes, ciphertext)) = data.split_at_checked(24) {
        let nonce = Nonce::try_from(nonce_bytes).unwrap();
        let _ = decrypt(key, &nonce, ciphertext, aad);
    }
});
