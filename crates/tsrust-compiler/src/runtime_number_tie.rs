// Shortest round-trip digits can have two equally near candidates. Standard
// formatting takes the upper one; number text takes the one with an even last
// digit. `text[start..]` holds the standard plain or scientific text of `value`.
fn tsr_number_tie(value: f64, text: &mut std::string::String, start: usize) {
    let bits = value.to_bits();
    let biased = ((bits >> 52) & 0x7ff) as i32;
    if biased == 0 || biased == 0x7ff {
        return;
    }
    let mantissa = (bits & ((1u64 << 52) - 1)) | (1u64 << 52);
    let zeros = mantissa.trailing_zeros() as i32;
    let odd = mantissa >> zeros;
    let exponent = biased - 1075 + zeros;
    // A tie needs an exact decimal expansion one digit longer than the text,
    // which is at most 18 digits. Most values leave here.
    let exact: u128 = if exponent < 0 {
        if exponent < -27 {
            return;
        }
        odd as u128 * 5u128.pow((-exponent) as u32)
    } else {
        if exponent > 22 {
            return;
        }
        let power = 5u64.pow(exponent as u32);
        if odd % power != 0 {
            return;
        }
        (odd / power) as u128
    };
    let lower = exact / 10;
    if exact % 10 != 5 || lower % 2 != 0 {
        return;
    }
    let bytes = text.as_bytes();
    let end = bytes[start..]
        .iter()
        .position(|&byte| byte == b'e')
        .map_or(bytes.len(), |position| start + position);
    let significant = |index: &usize| bytes[*index].is_ascii_digit() && bytes[*index] != b'0';
    let (Some(first), Some(last)) = ((start..end).find(significant), (start..end).rev().find(significant)) else {
        return;
    };
    let count = (first..=last).filter(|&index| bytes[index].is_ascii_digit()).count();
    let mut exact_count = 0;
    let mut rest = exact;
    while rest > 0 {
        exact_count += 1;
        rest /= 10;
    }
    let digit = bytes[last];
    if exact_count != count + 1 || (digit - b'0') % 2 == 0 {
        return;
    }
    let mut buffer = [0u8; 4];
    text.replace_range(last..=last, ((digit - 1) as char).encode_utf8(&mut buffer));
    // The lower candidate must still read back as the same value.
    if text[start..].parse::<f64>() != Ok(value) {
        text.replace_range(last..=last, (digit as char).encode_utf8(&mut buffer));
    }
}
