use std::iter;

pub fn encode(bytes: &[u8]) -> String {
    let padding = match bytes.len() % 3 {
        1 => vec!['=', '='],
        2 => vec!['='],
        _ => vec![],
    };

    let sextets = SextetIter::from(bytes);

    sextets
        .into_iter()
        .map(encode_single)
        .chain(padding.into_iter())
        .collect()
}

pub fn decode(base64: &str) -> Vec<u8> {
    base64
        .chars()
        .filter_map(decode_single)
        .fold(
            OctetBuilder::with_sextet_capacity(base64.len()),
            |acc, sextet| acc.add(sextet),
        )
        .bytes()
}

fn encode_single(index: u8) -> char {
    match index {
        0..=25 => char::from_u32('A' as u32 + index as u32).unwrap(),
        26..=51 => char::from_u32('a' as u32 + index as u32 - 26).unwrap(),
        52..=61 => char::from_u32('0' as u32 + index as u32 - 52).unwrap(),
        62 => '+',
        63 => '/',
        _ => panic!(
            "cannot encode {:?} as a base64 character; must be in range [0, 63]",
            index
        ),
    }
}

fn decode_single(point: char) -> Option<u8> {
    match point {
        'A'..='Z' => Some((point as u32 - 'A' as u32) as u8),
        'a'..='z' => Some((point as u32 - 'a' as u32) as u8 + 26),
        '0'..='9' => Some((point as u32 - '0' as u32) as u8 + 52),
        '+' => Some(62),
        '/' => Some(63),
        '=' => None,
        _ => panic!("cannot decode {:?} as base64", point),
    }
}

struct SextetIter<'a> {
    bytes: std::slice::Iter<'a, u8>,
    step: EncodeDecodeStep,
    last: u8,
    end: bool,
}

impl<'a> SextetIter<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes.iter(),
            step: EncodeDecodeStep::First,
            last: 0,
            end: false,
        }
    }

    fn grab_next_byte(&mut self) -> Option<(u8, u8)> {
        match self.step {
            EncodeDecodeStep::First => self
                .bytes
                .next()
                .map(|byte| (byte >> 2, byte & 0b0000_0011)),

            EncodeDecodeStep::Second => self
                .bytes
                .next()
                .map(|byte| (byte >> 4, byte & 0b0000_1111)),

            EncodeDecodeStep::Third => self
                .bytes
                .next()
                .map(|byte| (byte >> 6, byte & 0b0011_1111)),

            EncodeDecodeStep::Fourth => unreachable!(),
        }
    }
}

impl<'a> iter::Iterator for SextetIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        let sextet = match self.step {
            EncodeDecodeStep::First => match self.grab_next_byte() {
                Some((head, tail)) => {
                    let sextet = head;
                    self.last = tail;

                    Some(sextet)
                }

                None => {
                    self.end = true;
                    None
                }
            },

            EncodeDecodeStep::Second => match self.grab_next_byte() {
                Some((head, tail)) => {
                    let sextet = self.last << 4 | head;
                    self.last = tail;

                    Some(sextet)
                }

                None => {
                    self.end = true;
                    Some(self.last << 4)
                }
            },

            EncodeDecodeStep::Third => match self.grab_next_byte() {
                Some((head, tail)) => {
                    let sextet = self.last << 2 | head;
                    self.last = tail;

                    Some(sextet)
                }

                None => {
                    self.end = true;
                    Some(self.last << 2)
                }
            },

            EncodeDecodeStep::Fourth => Some(self.last),
        };

        self.step = self.step.next();

        sextet
    }
}

struct OctetBuilder {
    bytes: Vec<u8>,
    step: EncodeDecodeStep,
    last: u8,
}

impl OctetBuilder {
    fn with_sextet_capacity(sextets: usize) -> Self {
        Self {
            bytes: Vec::with_capacity(sextets * 3 / 4 + 1),
            step: EncodeDecodeStep::First,
            last: 0,
        }
    }

    fn add(mut self, sextet: u8) -> Self {
        match self.step {
            EncodeDecodeStep::First => {
                self.last = sextet << 2;
            }

            EncodeDecodeStep::Second => {
                self.bytes.push(self.last | sextet >> 4);
                self.last = (sextet & 0b0000_1111) << 4;
            }

            EncodeDecodeStep::Third => {
                self.bytes.push(self.last | sextet >> 2);
                self.last = (sextet & 0b0000_0011) << 6;
            }

            EncodeDecodeStep::Fourth => {
                self.bytes.push(self.last | sextet);
            }
        }

        self.step = self.step.next();

        self
    }

    fn bytes(mut self) -> Vec<u8> {
        self.bytes
    }
}

#[derive(Clone, Copy)]
enum EncodeDecodeStep {
    First,
    Second,
    Third,
    Fourth,
}

impl EncodeDecodeStep {
    fn next(self) -> Self {
        match self {
            Self::First => Self::Second,
            Self::Second => Self::Third,
            Self::Third => Self::Fourth,
            Self::Fourth => Self::First,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_alphabet_matches_rfc4648() {
        let rfc4648 = [
            'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q',
            'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
            'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y',
            'z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '+', '/',
        ];

        let alphabet: Vec<char> = (0..=63).into_iter().map(encode_single).collect();

        assert_eq!(rfc4648.len(), alphabet.len());
        assert!(rfc4648.iter().zip(alphabet.iter()).all(|(a, b)| a == b));
    }

    const BYTES: [u8; 3] = [0b0100_1101, 0b0110_0001, 0b0110_1110];

    #[test]
    fn sextet_iterator_three_bytes() {
        let sextets: Vec<u8> = SextetIter::from(&BYTES).into_iter().collect();
        let expected: [u8; 4] = [0b01_0011, 0b01_0110, 0b00_0101, 0b10_1110];

        assert_eq!(sextets.len(), expected.len());
        assert!(sextets.iter().zip(expected.iter()).all(|(a, b)| a == b));
    }

    #[test]
    fn sextet_iterator_two_bytes() {
        let sextets: Vec<u8> = SextetIter::from(&BYTES[0..2]).into_iter().collect();
        let expected: [u8; 3] = [0b01_0011, 0b01_0110, 0b00_0100];

        assert_eq!(sextets.len(), expected.len());
        assert!(sextets.iter().zip(expected.iter()).all(|(a, b)| a == b));
    }

    #[test]
    fn sextet_iterator_one_byte() {
        let sextets: Vec<u8> = SextetIter::from(&BYTES[0..1]).into_iter().collect();
        let expected: [u8; 2] = [0b01_0011, 0b01_0000];

        assert_eq!(sextets.len(), expected.len());
        assert!(sextets.iter().zip(expected.iter()).all(|(a, b)| a == b));
    }

    #[test]
    fn sextet_iterator_zero_bytes() {
        let sextets: Vec<u8> = SextetIter::from(&BYTES[0..0]).into_iter().collect();
        let expected: [u8; 0] = [];

        assert_eq!(sextets.len(), expected.len());
    }

    #[test]
    fn base64_encode_three_letters() {
        assert_eq!(encode(b"Man"), String::from("TWFu"));
    }

    #[test]
    fn base64_encode_two_letters() {
        assert_eq!(encode(b"Ma"), String::from("TWE="));
    }

    #[test]
    fn base64_encode_one_letter() {
        assert_eq!(encode(b"M"), String::from("TQ=="));
    }

    #[test]
    fn base64_encode_zero_letters() {
        assert_eq!(encode(b""), String::from(""));
    }

    #[test]
    fn base64_encode_sentence() {
        assert_eq!(
            encode(b"Many hands make light work."),
            String::from("TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu")
        );
    }

    #[test]
    fn base64_decode_three_letters() {
        assert_eq!(decode("TWFu"), b"Man");
    }

    #[test]
    fn base64_decode_two_letters() {
        assert_eq!(decode("TWE="), b"Ma");
    }

    #[test]
    fn base64_decode_one_letter() {
        assert_eq!(decode("TQ=="), b"M");
    }

    #[test]
    fn base64_decode_zero_letters() {
        assert_eq!(decode(""), b"");
    }

    #[test]
    fn base64_decode_sentence() {
        assert_eq!(
            decode("TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu"),
            b"Many hands make light work."
        );
    }
}
