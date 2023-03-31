mod base64;
mod score;

use std::convert;
use std::fmt;
use std::fs;

#[derive(Debug, Clone)]
struct Buffer {
    bytes: Vec<u8>,
}

impl Buffer {
    fn from_hex(hex: &str) -> Self {
        Self {
            bytes: hex::decode(hex).expect("invalid hex input"),
        }
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

    fn as_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    fn as_base64(&self) -> String {
        base64::encode(&self.bytes)
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

fn main() {
    let file_content = fs::read_to_string("4.txt").unwrap();

    let mut best_line = "";
    let mut best_key = 0;
    let mut best_penalty = 1000.0;

    for line in file_content.lines() {
        let buffer = Buffer::from_hex(line);

        let (key, penalty) = best_one_byte_xor(&buffer);

        if penalty < best_penalty {
            best_penalty = penalty;
            best_key = key;
            best_line = line;
        }
    }

    let clear = Buffer::from_hex(best_line).xor([best_key]);

    println!("{}", clear.as_hex());
    println!("{}", clear);
}
