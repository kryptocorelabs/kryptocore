use std::{
    io::Write,
    process::{Command, Stdio},
};

use kryptocore::{decrypt, derive_key, encrypt, Error, Key, Nonce};

const PASSWORD: &[u8] = b"synthetic interop password";
const SALT: [u8; 16] = [42; 16];
const AAD: &[u8] = b"kryptocore/interop/v1";
const MESSAGES: [&[u8]; 3] = [
    b"",
    b"\0\xff\n\x80",
    b"a message from the same test fixture",
];

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    (0..value.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&value[i..i + 2], 16).unwrap())
        .collect()
}

fn assert_authentication_failure(result: Result<zeroize::Zeroizing<Vec<u8>>, Error>) {
    assert!(matches!(result, Err(Error::AuthenticationFailed)));
}

#[test]
#[ignore = "requires the installed Python reference and its dependencies"]
fn python_and_rust_exchange_authenticated_ciphertexts() {
    let key = derive_key(PASSWORD, &SALT).unwrap();
    let python = std::env::var_os("PYTHON").unwrap_or_else(|| "python".into());
    let mut child = Command::new(python)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/interop_peer.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut input = child.stdin.take().unwrap();
    for plaintext in MESSAGES {
        let (nonce, ciphertext) = encrypt(&key, plaintext, AAD).unwrap();
        writeln!(input, "{} {}", hex(&nonce), hex(&ciphertext)).unwrap();
    }
    drop(input);
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "Python interoperability peer failed"
    );

    let wrong_key = Key::generate().unwrap();
    let output = String::from_utf8(output.stdout).unwrap();
    let replies: Vec<_> = output.lines().collect();
    assert_eq!(replies.len(), MESSAGES.len());
    for (reply, expected) in replies.into_iter().zip(MESSAGES) {
        let (nonce, ciphertext) = reply.split_once(' ').unwrap();
        let nonce_bytes = unhex(nonce);
        let nonce = Nonce::try_from(nonce_bytes.as_slice()).unwrap();
        let ciphertext = unhex(ciphertext);
        assert_eq!(
            &decrypt(&key, &nonce, &ciphertext, AAD).unwrap()[..],
            expected
        );
        assert_authentication_failure(decrypt(&wrong_key, &nonce, &ciphertext, AAD));
        assert_authentication_failure(decrypt(&key, &nonce, &ciphertext, b"other context"));
        let mut changed_nonce = nonce;
        changed_nonce[0] ^= 1;
        assert_authentication_failure(decrypt(&key, &changed_nonce, &ciphertext, AAD));
        // Rust's public API takes a fixed-size array. A parser must reject these lengths first.
        assert!(Nonce::try_from(&nonce_bytes[..23]).is_err());
        assert!(Nonce::try_from([nonce_bytes.as_slice(), &[0]].concat().as_slice()).is_err());
        assert_authentication_failure(decrypt(&key, &nonce, &ciphertext[..15], AAD));
        let mut modified = ciphertext;
        modified[0] ^= 1;
        assert_authentication_failure(decrypt(&key, &nonce, &modified, AAD));
    }
}
