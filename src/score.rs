use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref ENGLISH_TEXT_FREQUENCY: HashMap<char, f64> = [
        (' ', 0.2),
        ('a', 0.06532628287475481),
        ('b', 0.011949929794162463),
        ('c', 0.022306535615769934),
        ('d', 0.03425646540993239),
        ('e', 0.10356605821607469),
        ('f', 0.017526563698104944),
        ('g', 0.015933239725549952),
        ('h', 0.04859638116292735),
        ('i', 0.055766339039424836),
        ('j', 0.0011949929794162464),
        ('k', 0.0061342972943367316),
        ('l', 0.031866479451099904),
        ('m', 0.019119887670659943),
        ('n', 0.05337635308059234),
        ('o', 0.059749648970812315),
        ('p', 0.015136577739272454),
        ('q', 0.0007568288869636226),
        ('r', 0.04779971917664985),
        ('s', 0.05018970513548235),
        ('t', 0.07249624075125227),
        ('u', 0.022306535615769934),
        ('v', 0.007807287465519476),
        ('w', 0.019119887670659943),
        ('x', 0.0011949929794162464),
        ('y', 0.015933239725549952),
        ('z', 0.0005895298698453482),
    ]
    .into_iter()
    .collect();
}

pub fn english_text_frequency(text: &[u8]) -> f64 {
    let characters = text
        .iter()
        .filter_map(|byte| char::from_u32(*byte as u32))
        .map(|ch| ch.to_ascii_lowercase());

    let mut counts: HashMap<char, usize> = HashMap::new();

    for ch in characters {
        counts.entry(ch).and_modify(|e| *e += 1).or_insert(1);
    }

    counts
        .into_iter()
        .map(|(ch, count)| (ch, count as f64 / text.len() as f64))
        .map(|(ch, frequency)| {
            frequency - ENGLISH_TEXT_FREQUENCY.get(&ch).copied().unwrap_or_default()
        })
        .map(|diff| diff * diff)
        .sum()
}
