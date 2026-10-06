// Whole-string scalar numeric conversion. Borrowed slices; no parsing buffer.
fn tsr_numeric_whitespace(character: char) -> bool {
    matches!(character,
        '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}'
        | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
        | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

fn tsr_numeric_radix(digits: &[u8], width: u32) -> f64 {
    if digits.is_empty() { return f64::NAN; }
    let mut mantissa = 0u64;
    let mut bits = 0usize;
    let mut guard = false;
    let mut sticky = false;
    for &byte in digits {
        let Some(digit) = (byte as char).to_digit(1 << width) else { return f64::NAN; };
        for position in (0..width).rev() {
            let bit = (digit >> position) & 1;
            if bits == 0 && bit == 0 { continue; }
            bits += 1;
            if bits <= 53 { mantissa = (mantissa << 1) | u64::from(bit); }
            else if bits == 54 { guard = bit != 0; }
            else { sticky |= bit != 0; }
        }
    }
    if bits <= 53 { return mantissa as f64; }
    if bits > 1024 { return f64::INFINITY; }
    if guard && (sticky || mantissa & 1 != 0) { mantissa += 1; }
    (mantissa as f64) * 2.0f64.powi((bits - 53) as i32)
}

fn tsr_number_from_text(text: &str) -> f64 {
    let text = text.trim_matches(tsr_numeric_whitespace);
    if text.is_empty() { return 0.0; }
    let bytes = text.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'0' {
        let width = match bytes[1] { b'b' | b'B' => Some(1), b'o' | b'O' => Some(3), b'x' | b'X' => Some(4), _ => None };
        if let Some(width) = width { return tsr_numeric_radix(&bytes[2..], width); }
    }
    let mut at = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    if &text[at..] == "Infinity" { return if bytes[0] == b'-' { f64::NEG_INFINITY } else { f64::INFINITY }; }
    let start = at;
    while bytes.get(at).is_some_and(u8::is_ascii_digit) { at += 1; }
    let mut digits = at - start;
    if bytes.get(at) == Some(&b'.') {
        at += 1;
        let start = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) { at += 1; }
        digits += at - start;
    }
    if digits == 0 { return f64::NAN; }
    if matches!(bytes.get(at), Some(b'e' | b'E')) {
        at += 1;
        if matches!(bytes.get(at), Some(b'+' | b'-')) { at += 1; }
        let start = at;
        while bytes.get(at).is_some_and(u8::is_ascii_digit) { at += 1; }
        if at == start { return f64::NAN; }
    }
    if at != bytes.len() { return f64::NAN; }
    text.parse::<f64>().unwrap_or(f64::NAN)
}
