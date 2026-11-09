use std::{
    io::Write,
    process::{Command, Stdio},
};

use kryptocore::{decrypt, decrypt_checked, derive_key, encrypt, DecryptError, Error, Key, Nonce};

const PASSWORD: &[u8] = b"synthetic interop password";
const SALT: [u8; 16] = [42; 16];
const AAD: &[u8] = b"kryptocore/interop/v1";

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
    let large_aad: Vec<_> = (0..=255u8).cycle().take(128 * 1024).collect();
    let cases: [(&[u8], &[u8]); 6] = [
        (b"", b""),
        (b"", AAD),
        (b"\0\xff\n\x80", b""),
        (b"a message from the same test fixture", AAD),
        (b"", &large_aad),
        (b"message with large AAD", &large_aad),
    ];
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
    for (plaintext, aad) in cases {
        let (nonce, ciphertext) = encrypt(&key, plaintext, aad).unwrap();
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
    assert_eq!(replies.len(), cases.len());
    for (reply, (expected, aad)) in replies.into_iter().zip(cases) {
        let (nonce, ciphertext) = reply.split_once(' ').unwrap();
        let nonce_bytes = unhex(nonce);
        let nonce = Nonce::try_from(nonce_bytes.as_slice()).unwrap();
        let ciphertext = unhex(ciphertext);
        assert_eq!(
            &decrypt_checked(&key, &nonce, &ciphertext, aad).unwrap()[..],
            expected
        );
        assert_authentication_failure(decrypt(&wrong_key, &nonce, &ciphertext, aad));
        let mut changed_aad = aad.to_vec();
        if let Some(last) = changed_aad.last_mut() {
            *last ^= 1;
        } else {
            changed_aad.push(0);
        }
        assert_authentication_failure(decrypt(&key, &nonce, &ciphertext, &changed_aad));
        let mut changed_nonce = nonce;
        changed_nonce[0] ^= 1;
        assert!(matches!(
            decrypt_checked(&key, &changed_nonce, &ciphertext, aad),
            Err(DecryptError::AuthenticationFailed)
        ));
        let long_nonce = [nonce_bytes.as_slice(), &[0]].concat();
        for invalid in [&[][..], &nonce_bytes[..23], long_nonce.as_slice()] {
            assert!(matches!(
                decrypt_checked(&key, invalid, &ciphertext, aad),
                Err(DecryptError::InvalidNonceLength)
            ));
        }
        for length in [0, 15, ciphertext.len() - 1] {
            assert_authentication_failure(decrypt(&key, &nonce, &ciphertext[..length], aad));
        }
        assert_authentication_failure(decrypt(
            &key,
            &nonce,
            &[ciphertext.as_slice(), &[0]].concat(),
            aad,
        ));
        let mut modified = ciphertext;
        *modified.last_mut().unwrap() ^= 1;
        assert_authentication_failure(decrypt(&key, &nonce, &modified, aad));
    }
}
