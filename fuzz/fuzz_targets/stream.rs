#![no_main]
#![forbid(unsafe_code)]

use std::{io, sync::OnceLock};

use kryptocore::{
    stream::{decrypt_stream, encrypt_stream},
    Context, Key,
};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    static KEY: OnceLock<Key> = OnceLock::new();
    let key = KEY.get_or_init(|| Key::generate().expect("OS randomness"));
    let context = Context::new("example", 1, "fuzz-stream", b"").unwrap();
    let _ = decrypt_stream(
        key,
        &mut &data[..data.len().min(2 * 65_536)],
        &mut io::sink(),
        &context,
    );

    let Some((selector, plaintext)) = data.split_at_checked(4) else {
        return;
    };
    let plaintext = &plaintext[..plaintext.len().min(2 * 65_536)];
    let mut input = plaintext;
    let mut encrypted = Vec::new();
    encrypt_stream(key, &mut input, &mut encrypted, &context).unwrap();
    let mut decoded = Vec::new();
    decrypt_stream(key, &mut encrypted.as_slice(), &mut decoded, &context).unwrap();
    assert_eq!(decoded, plaintext);

    let offset = u32::from_le_bytes(selector.try_into().unwrap()) as usize % encrypted.len();
    assert!(decrypt_stream(key, &mut &encrypted[..offset], &mut io::sink(), &context).is_err());
    encrypted[offset] ^= 1;
    assert!(decrypt_stream(key, &mut encrypted.as_slice(), &mut io::sink(), &context).is_err());
});
