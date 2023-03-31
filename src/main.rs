mod base64;

use std::fmt;

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

    fn as_base64(&self) -> String {
        base64::encode(&self.bytes)
    }
}

impl fmt::Display for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", String::from_utf8_lossy(&self.bytes))
    }
}

fn main() {
    let input = "49276d206b696c6c696e6720796f757220627261696e206c696b65206120706f69736f6e6f7573206d757368726f6f6d";
    let buffer = Buffer::from_hex(input);

    println!("{}", input);
    println!("{}", buffer.as_base64());
    println!("{}", buffer);
}
