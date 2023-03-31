mod base64;
mod score;

use std::convert;
use std::fmt;

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

fn main() {
    let input = "1b37373331363f78151b7f2b783431333d78397828372d363c78373e783a393b3736";
    let buffer = Buffer::from_hex(input);

    let mut best_key = 0u8;
    let mut best_penalty = score::english_text_frequency(buffer.xor([0]).as_ref());

    for i in 1..u8::MAX {
        let text = buffer.xor([i]);
        let penalty = score::english_text_frequency(text.as_ref());

        if penalty < best_penalty {
            best_penalty = penalty;
            best_key = i;
        }
    }

    let clear = buffer.xor([best_key]);

    println!("{}", clear.as_hex());
    println!("{}", clear);
}
