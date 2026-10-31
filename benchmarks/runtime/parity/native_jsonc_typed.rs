// Independent standard Rust/serde JSON stream reference.
use std::io::{self, BufRead, Write};
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
#[derive(serde::Deserialize)]
struct Item {
    name: std::string::String,
    values: Vec<i64>,
    active: bool,
}
fn observe(item: Item) -> i64 {
    assert!(item.active, "Inactive item");
    add(
        add(i64::try_from(item.name.len()).unwrap(), item.values[0]),
        item.values[1],
    )
}
// Default JSONC permits comments, with trailing commas still rejected by serde.
fn strip_comments(input: &str) -> Result<Vec<u8>, &'static str> {
    let bytes = input.as_bytes();
    let mut output = bytes.to_vec();
    let mut at = 0;
    let mut quoted = false;
    while at < bytes.len() {
        if quoted {
            match bytes[at] {
                b'\\' => {
                    at += 2;
                }
                b'"' => {
                    quoted = false;
                    at += 1;
                }
                _ => {
                    at += 1;
                }
            }
            continue;
        }
        if bytes[at] == b'"' {
            quoted = true;
            at += 1;
            continue;
        }
        if bytes[at] == b'/' && bytes.get(at + 1) == Some(&b'/') {
            while at < bytes.len() && bytes[at] != b'\n' && bytes[at] != b'\r' {
                output[at] = b' ';
                at += 1;
            }
            continue;
        }
        if bytes[at] == b'/' && bytes.get(at + 1) == Some(&b'*') {
            let start = at;
            at += 2;
            while at + 1 < bytes.len() && &bytes[at..at + 2] != b"*/" {
                at += 1;
            }
            if at + 1 >= bytes.len() {
                return Err("unterminated comment");
            }
            at += 2;
            for byte in &mut output[start..at] {
                if *byte != b'\n' && *byte != b'\r' {
                    *byte = b' ';
                }
            }
            continue;
        }
        at += 1;
    }
    Ok(output)
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let bytes = strip_comments(value2).expect("Invalid JSONC");
        let item: Item = serde_json::from_slice(&bytes).expect("Invalid item");
        checksum = add(checksum, observe(item));
    }
    checksum
}
fn main() {}
