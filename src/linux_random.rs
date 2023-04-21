use std::fs::File;
use std::io::Read;

pub fn random(byte_count: usize) -> Vec<u8> {
    let mut random = File::open("/dev/urandom").expect("cannot open /dev/urandom");
    let mut buffer = vec![0; byte_count];

    random
        .read_exact(&mut buffer)
        .expect("cannot read /dev/urandom");

    buffer
}
