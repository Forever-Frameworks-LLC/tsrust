//! String phases: generated text, splitting, a word-count map and formatting.
use std::collections::HashMap;

const VOCABULARY: [&str; 26] = [
    "alpha", "bravo", "charlie", "delta", "echo", "foxtrot", "golf", "hotel", "india", "juliet",
    "kilo", "lima", "mike", "november", "oscar", "papa", "quebec", "romeo", "sierra", "tango",
    "uniform", "victor", "whiskey", "xray", "yankee", "zulu",
];

/// One step of a 31-bit linear congruential generator, shared with the Tsrust version.
pub fn next_random(state: i64) -> i64 {
    (state * 1_103_515_245 + 12_345) % 2_147_483_648
}

pub fn make_text(words: usize) -> String {
    let mut parts = Vec::with_capacity(words);
    let mut state = 42;
    for _ in 0..words {
        state = next_random(state);
        let word = VOCABULARY[(state % VOCABULARY.len() as i64) as usize];
        parts.push(if state % 7 == 0 {format!("{word}{}", state % 100)} else {word.to_owned()});
    }
    parts.join(" ")
}

pub fn word_count(text: &str) -> i64 {
    let mut counts: HashMap<&str, i64> = HashMap::new();
    for word in text.split(' ') {
        *counts.entry(word).or_insert(0) += 1;
    }
    let mut ranked: Vec<(&str, i64)> = counts.iter().map(|(word, count)| (*word, *count)).collect();
    ranked.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    let mut checksum = counts.len() as i64;
    for (word, count) in &ranked[..10] {
        checksum = (checksum * 31 + count + word.len() as i64) % 1_000_000_007;
    }
    checksum
}

pub fn formatting(count: usize) -> i64 {
    let mut total = 0i64;
    for index in 0..count {
        let line = format!("item-{index} score={} name={}", (index * 7) % 1000, VOCABULARY[index % VOCABULARY.len()].to_uppercase());
        let replaced = line.replace('-', "_");
        let at = replaced.find("score=").unwrap();
        total = (total + replaced.len() as i64 + at as i64 + replaced[at + 6..(at + 9).min(replaced.len())].len() as i64) % 1_000_000_007;
    }
    total
}
