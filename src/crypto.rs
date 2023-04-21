pub fn fixed_xor<T: AsRef<[u8]>, U: AsRef<[u8]>>(a: T, b: T) -> Vec<u8> {
    assert!(
        a.as_ref().len() == b.as_ref().len(),
        "input streams do not have equal length"
    );

    a.as_ref()
        .iter()
        .zip(b.as_ref().iter())
        .map(|(a, b)| a ^ b)
        .collect()
}

pub fn aes_128_ecb_encrypt(key: &[u8], input: &[u8]) -> Vec<u8> {
    let cipher = openssl::symm::Cipher::aes_128_ecb();
    let mut output = openssl::symm::encrypt(cipher, key, None, input).expect("encryption failed");

    if input.len() % 16 == 0 {
        output.truncate(output.len() - 16);
    }

    output
}

pub fn aes_128_ecb_decrypt(key: &[u8], input: &[u8]) -> Vec<u8> {
    assert!(input.len() % 16 == 0, "malformed input");

    let mut crypter = openssl::symm::Crypter::new(
        openssl::symm::Cipher::aes_128_ecb(),
        openssl::symm::Mode::Decrypt,
        key,
        None,
    )
    .expect("malformed key");

    crypter.pad(false);

    let mut output = Vec::with_capacity(input.len());
    let mut buffer = [0; 32];

    for block in input.chunks_exact(16) {
        let length = crypter
            .update(block, &mut buffer)
            .expect("decryption failed");

        output.extend(&buffer[0..length]);
    }

    output
}
