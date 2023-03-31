mod base64;

use std::fmt;
use std::ops;

#[derive(Debug)]
struct Buffer {
    bytes: Vec<u8>,
}

impl Buffer {
    fn from_hex(hex: &str) -> Self {
        Self {
            bytes: hex::decode(hex).expect("invalid hex input"),
        }
    }

    fn as_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    fn as_base64(&self) -> String {
        base64::encode(&self.bytes)
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes))
    }
}

impl ops::BitXor for Buffer {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self {
            bytes: self
                .bytes
                .into_iter()
                .zip(rhs.bytes.into_iter())
                .map(|(a, b)| a ^ b)
                .collect(),
        }
    }
}

fn main() {
    let b1 = Buffer::from_hex("1c0111001f010100061a024b53535009181c");
    let b2 = Buffer::from_hex("686974207468652062756c6c277320657965");

    println!("{}", b1);
    println!("{}", b2);

    let xor = b1 ^ b2;

    println!("{}", xor.as_hex());
    println!("{}", xor);
}
