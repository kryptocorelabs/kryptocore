#![no_main]
#![forbid(unsafe_code)]

use std::sync::OnceLock;

use kryptocore::{decrypt_checked, encrypt, DecryptError, Key};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    static KEY: OnceLock<Key> = OnceLock::new();
    static OTHER_KEY: OnceLock<Key> = OnceLock::new();
    let key = KEY.get_or_init(|| Key::generate().expect("OS randomness"));
    let other_key = OTHER_KEY.get_or_init(|| Key::generate().expect("OS randomness"));

    let (header, data) = data.split_at(data.len().min(4));
    let mut selector = [0; 4];
    selector[..header.len()].copy_from_slice(header);
    let aad_length = usize::from(u16::from_le_bytes([selector[0], selector[1]]));
    let mutation = usize::from(u16::from_le_bytes([selector[2], selector[3]]));
    let (aad, plaintext) = data.split_at(aad_length.min(data.len()));
    let (nonce, mut ciphertext) = encrypt(key, plaintext, aad).expect("bounded encryption input");
    assert_eq!(
        &decrypt_checked(key, &nonce, &ciphertext, aad).unwrap()[..],
        plaintext
    );
    assert!(matches!(
        decrypt_checked(other_key, &nonce, &ciphertext, aad),
        Err(DecryptError::AuthenticationFailed)
    ));
    let mut changed_nonce = nonce;
    changed_nonce[mutation % nonce.len()] ^= 1;
    assert!(matches!(
        decrypt_checked(key, &changed_nonce, &ciphertext, aad),
        Err(DecryptError::AuthenticationFailed)
    ));
    let changed_aad = [aad, &[0]].concat();
    assert!(matches!(
        decrypt_checked(key, &nonce, &ciphertext, &changed_aad),
        Err(DecryptError::AuthenticationFailed)
    ));
    let cut = mutation % ciphertext.len();
    assert!(matches!(
        decrypt_checked(key, &nonce, &ciphertext[..cut], aad),
        Err(DecryptError::AuthenticationFailed)
    ));
    ciphertext[cut] ^= 1;
    assert!(matches!(
        decrypt_checked(key, &nonce, &ciphertext, aad),
        Err(DecryptError::AuthenticationFailed)
    ));

    let raw_length = usize::from(selector[2] % 32).min(data.len());
    let (raw_nonce, raw_ciphertext) = data.split_at(raw_length);
    let result = decrypt_checked(key, raw_nonce, raw_ciphertext, aad);
    if raw_nonce.len() != nonce.len() {
        assert!(matches!(result, Err(DecryptError::InvalidNonceLength)));
    }
});
