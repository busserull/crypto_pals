mod base64;
mod crypto;
mod gliding_slice;
mod linux_random;
mod result_keeper;
mod score;

use lazy_static::lazy_static;
use std::convert;
use std::fmt;
use std::fs;

use gliding_slice::GlidingSlice;
use result_keeper::ResultKeeper;

#[derive(Debug, Clone)]
struct Buffer {
    bytes: Vec<u8>,
}

impl Buffer {
    fn new(bytes: &[u8]) -> Self {
        Self {
            bytes: bytes.to_vec(),
        }
    }

    fn from_hex(hex: &str) -> Self {
        Self {
            bytes: hex::decode(hex).expect("invalid hex input"),
        }
    }

    fn from_base64(base64: &str) -> Self {
        Self {
            bytes: base64::decode(base64),
        }
    }

    fn pad(&mut self, buffer_size: usize) {
        if buffer_size > self.bytes.len() {
            let pad_size = buffer_size - self.bytes.len();
            self.bytes.extend(vec![pad_size as u8; pad_size]);
        }
    }

    fn unpad(&mut self) {
        let last = self.bytes.last().copied().unwrap_or_default();

        let count = self
            .bytes
            .iter()
            .rev()
            .take_while(|byte| **byte == last)
            .count();

        if count == last as usize {
            self.bytes.truncate(self.bytes.len() - count);
        }
    }

    fn len(&self) -> usize {
        self.bytes.len()
    }

    fn xor<T: AsRef<[u8]>>(&self, key: T) -> Self {
        Self {
            bytes: self
                .bytes
                .iter()
                .zip(key.as_ref().iter().cycle())
                .map(|(a, b)| a ^ b)
                .collect(),
        }
    }

    fn xor_repeating_key_search(&self, size: usize) -> Option<f64> {
        if size > self.bytes.len() / 2 {
            return None;
        }

        let (chunks, distances) = GlidingSlice::new(&self.bytes, size).into_iter().fold(
            (1, 0.0),
            |(chunks, acc), (one, two)| {
                let distance = hamming_distance(one, two) as f64 / size as f64;
                (chunks + 1, acc + distance)
            },
        );

        Some(distances / chunks as f64)
    }

    fn transpose(&self, size: usize) -> Vec<Self> {
        let size = std::cmp::min(size, self.bytes.len());

        let mut rows = vec![
            Self {
                bytes: Vec::with_capacity(self.bytes.len() / size)
            };
            size
        ];

        for (row, byte) in self
            .bytes
            .iter()
            .enumerate()
            .map(|(i, byte)| (i % size, byte))
        {
            rows[row].bytes.push(*byte);
        }

        rows
    }

    fn count_identical_runs(&self, run_length: usize) -> usize {
        if run_length > self.bytes.len() {
            return 0;
        }

        let mut runs = 0;

        for start in 0..self.bytes.len() - run_length - 2 {
            for compare_start in start + 1..self.bytes.len() - run_length - 1 {
                let template = &self.bytes[start..start + run_length];
                let compare = &self.bytes[compare_start..compare_start + run_length];

                if template == compare {
                    runs += 1;
                }
            }
        }

        runs
    }

    fn as_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    fn as_base64(&self) -> String {
        base64::encode(&self.bytes)
    }

    fn aes_128_ecb_encrypt(&self, key: &[u8]) -> Self {
        Self {
            bytes: crypto::aes_128_ecb_encrypt(key, &self.bytes),
        }
    }

    fn aes_128_ecb_decrypt(&self, key: &[u8]) -> Self {
        Self {
            bytes: crypto::aes_128_ecb_decrypt(key, &self.bytes),
        }
    }

    fn aes_128_cbc_encrypt(&self, key: &[u8], iv: &[u8]) -> Self {
        let plaintext_blocks = self
            .bytes
            .chunks(16)
            .map(|bytes| {
                let mut block = Self::new(bytes);
                block.pad(16);
                block
            })
            .collect::<Vec<_>>();

        let ecb = openssl::symm::Cipher::aes_128_ecb();

        let mut last_cipher_block = Self::new(iv);
        let mut ciphertext = Vec::with_capacity(16 * plaintext_blocks.len());

        for block in plaintext_blocks.into_iter() {
            let combined = block.xor(last_cipher_block);
            let mut encrypted = openssl::symm::encrypt(ecb, key, None, combined.as_ref())
                .expect("failed to encrypt");

            encrypted.truncate(16);

            ciphertext.extend(&encrypted);
            last_cipher_block = Self { bytes: encrypted };
        }

        Self { bytes: ciphertext }
    }

    fn aes_128_cbc_decrypt(&self, key: &[u8], iv: &[u8]) -> Self {
        assert!(self.bytes.len() % 16 == 0, "padding error");

        let ciphertext_blocks = self
            .bytes
            .chunks_exact(16)
            .map(Self::new)
            .collect::<Vec<_>>();

        let mut crypter = openssl::symm::Crypter::new(
            openssl::symm::Cipher::aes_128_ecb(),
            openssl::symm::Mode::Decrypt,
            key,
            None,
        )
        .unwrap();

        crypter.pad(false);

        let mut last_cipher_block = Self::new(iv);
        let mut buffer = [0; 32];
        let mut cleartext = Vec::with_capacity(self.bytes.len());

        for block in ciphertext_blocks.into_iter() {
            crypter
                .update(block.as_ref(), &mut buffer)
                .expect("failed to decrypt");

            let combined = Self::new(&buffer[0..16]);
            let decrypted = combined.xor(last_cipher_block);

            cleartext.extend(decrypted.as_ref());
            last_cipher_block = block;
        }

        Self { bytes: cleartext }
    }
}

impl convert::AsRef<[u8]> for Buffer {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes))
    }
}

fn best_one_byte_xor(buffer: &Buffer) -> (u8, f64) {
    let mut best_key = 0u8;
    let mut best_penalty = score::english_text_frequency(buffer.xor([0]).as_ref());

    for k in 1..u8::MAX {
        let text = buffer.xor([k]);
        let penalty = score::english_text_frequency(text.as_ref());

        if penalty < best_penalty {
            best_penalty = penalty;
            best_key = k;
        }
    }

    (best_key, best_penalty)
}

fn hamming_distance(one: &[u8], two: &[u8]) -> usize {
    one.iter()
        .zip(two.iter())
        .map(|(a, b)| (a ^ b).count_ones() as usize)
        .sum()
}

fn encryption_oracle(plaintext: &[u8]) -> Buffer {
    lazy_static! {
        static ref SECRET: Buffer = Buffer::from_base64(
            "Um9sbGluJyBpbiBteSA1LjAKV2l0aCBteSByYWctdG9wIGRvd24gc28gbXkg\
            aGFpciBjYW4gYmxvdwpUaGUgZ2lybGllcyBvbiBzdGFuZGJ5IHdhdmluZyBq\
            dXN0IHRvIHNheSBoaQpEaWQgeW91IHN0b3A/IE5vLCBJIGp1c3QgZHJvdmUg\
            YnkK",
        );
    }

    lazy_static! {
        static ref KEY: Vec<u8> = linux_random::random(16);
    }

    Buffer {
        bytes: openssl::symm::encrypt(
            openssl::symm::Cipher::aes_128_ecb(),
            &KEY,
            None,
            &plaintext
                .iter()
                .copied()
                .chain(SECRET.bytes.iter().copied())
                .collect::<Vec<u8>>(),
        )
        .unwrap(),
    }
}

fn main() {
    // Find block size of cipher

    let base_length = encryption_oracle(&[]).len();
    let mut cipher_block_size = 0;

    for pre_pad in 1.. {
        let length = encryption_oracle(&vec![b'a'; pre_pad]).len();

        if length != base_length {
            cipher_block_size = length - base_length;
            break;
        }
    }

    // Detect ECB

    let uses_ecb = encryption_oracle(&vec![b'a'; 4 * cipher_block_size])
        .count_identical_runs(cipher_block_size)
        > 3;

    println!("Uses ECB: {}\n", uses_ecb);

    // Decrypt first block one byte at a time

    let mut stage = vec![b'a'; 2 * base_length];
    let range = (base_length - cipher_block_size)..base_length;

    for start in 0..base_length {
        let oracle = encryption_oracle(&stage[start..base_length - 1]);

        for byte in 0..u8::MAX {
            stage[start + base_length - 1] = byte;
            let test = encryption_oracle(&stage[start..start + base_length]);

            if oracle.bytes[range.clone()] == test.bytes[range.clone()] {
                break;
            }
        }
    }

    let buffer = Buffer::new(&stage[base_length - 1..2 * base_length - 1]);

    println!("{}", buffer);
}
