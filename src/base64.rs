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

fn encode_single(index: u8) -> char {
    match index {
        0..=25 => char::from_u32('A' as u32 + index as u32).unwrap(),
        26..=51 => char::from_u32('a' as u32 + index as u32 - 26).unwrap(),
        52..=61 => char::from_u32('0' as u32 + index as u32 - 52).unwrap(),
        62 => '+',
        63 => '/',
        _ => panic!(
            "cannot encode `{}` as a base64 character; must be in range [0, 63]",
            index
        ),
    }
}

struct SextetIter<'a> {
    bytes: std::slice::Iter<'a, u8>,
    step: SextetStep,
    last: u8,
    end: bool,
}

impl<'a> SextetIter<'a> {
    fn from(bytes: &'a [u8]) -> Self {
        Self {
            bytes: bytes.iter(),
            step: SextetStep::First,
            last: 0,
            end: false,
        }
    }

    fn grab_next_byte(&mut self) -> Option<(u8, u8)> {
        match self.step {
            SextetStep::First => self
                .bytes
                .next()
                .map(|byte| (byte >> 2, byte & 0b0000_0011)),

            SextetStep::Second => self
                .bytes
                .next()
                .map(|byte| (byte >> 4, byte & 0b0000_1111)),

            SextetStep::Third => self
                .bytes
                .next()
                .map(|byte| (byte >> 6, byte & 0b0011_1111)),

            SextetStep::Fourth => unreachable!(),
        }
    }
}

impl<'a> std::iter::Iterator for SextetIter<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end {
            return None;
        }

        let sextet = match self.step {
            SextetStep::First => match self.grab_next_byte() {
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

            SextetStep::Second => match self.grab_next_byte() {
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

            SextetStep::Third => match self.grab_next_byte() {
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

            SextetStep::Fourth => Some(self.last),
        };

        self.step = self.step.next();

        sextet
    }
}

#[derive(Clone, Copy)]
enum SextetStep {
    First,
    Second,
    Third,
    Fourth,
}

impl SextetStep {
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
    fn base64_three_letters() {
        assert_eq!(encode(b"Man"), String::from("TWFu"));
    }

    #[test]
    fn base64_two_letters() {
        assert_eq!(encode(b"Ma"), String::from("TWE="));
    }

    #[test]
    fn base64_one_letter() {
        assert_eq!(encode(b"M"), String::from("TQ=="));
    }

    #[test]
    fn base64_zero_letters() {
        assert_eq!(encode(b""), String::from(""));
    }

    #[test]
    fn base64_sentence() {
        assert_eq!(
            encode(b"Many hands make light work."),
            String::from("TWFueSBoYW5kcyBtYWtlIGxpZ2h0IHdvcmsu")
        );
    }
}
