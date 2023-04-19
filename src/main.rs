mod base64;
mod gliding_slice;
mod result_keeper;
mod score;

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

fn main() {
    let mut buffer = Buffer::new(b"YELLOW SUBMARINE");
    buffer.pad(20);

    println!("{:?}", buffer);
}
