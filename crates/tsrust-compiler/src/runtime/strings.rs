// Generated strings use only Rust's standard valid UTF-8 storage.
pub type String = std::borrow::Cow<'static, str>;
// The body stays raw. Only ASCII double quotes in an attribute are escaped.
pub fn tsr_string_html(
    value: &String,
    tag: &'static str,
    attribute: Option<(&'static str, &String)>,
) -> String {
    let output = if let Some((name, attribute)) = attribute {
        let escaped = attribute.replace('"', "&quot;");
        format!("<{tag} {name}=\"{escaped}\">{value}</{tag}>")
    } else {
        format!("<{tag}>{value}</{tag}>")
    };
    String::Owned(output)
}

pub fn tsr_string_identity(value: &String) -> String {
    value.clone()
}
pub fn tsr_boolean_string(value: bool) -> String {
    String::Borrowed(if value { "true" } else { "false" })
}

pub fn tsr_int_to_string(value: i64, radix: i64, failure: &'static str) -> String {
    assert!((2..=36).contains(&radix), "{failure}");
    if radix == 10 {
        return String::Owned(value.to_string());
    }
    // The largest unsigned magnitude needs at most 64 radix-2 digits.
    // A fixed character buffer avoids an intermediate allocated string.
    let mut digits = ['0'; 64];
    let mut cursor = digits.len();
    let mut magnitude = value.unsigned_abs();
    loop {
        cursor -= 1;
        digits[cursor] = char::from_digit((magnitude % radix as u64) as u32, radix as u32)
            .expect("validated radix digit");
        magnitude /= radix as u64;
        if magnitude == 0 {
            break;
        }
    }
    let mut output =
        std::string::String::with_capacity(digits.len() - cursor + usize::from(value < 0));
    if value < 0 {
        output.push('-');
    }
    for digit in &digits[cursor..] {
        output.push(*digit);
    }
    String::Owned(output)
}

// An integer prefix in radix 2 through 36. A missing or zero radix means 10,
// or 16 after a 0x prefix. Decimal digits use standard f64 parsing for
// correct rounding; other radices accumulate in f64.
// UTF-16 code units to text. A surrogate pair is one character; a surrogate
// with no partner becomes U+FFFD because text always holds valid Unicode.
#[allow(dead_code)]
pub fn tsr_string_from_code_units(units: impl Iterator<Item = u16>) -> String {
    String::Owned(
        char::decode_utf16(units)
            .map(|unit| unit.unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect(),
    )
}
#[allow(dead_code)]
pub fn tsr_string_from_code_points(codes: impl Iterator<Item = i64>, failure: &'static str) -> String {
    tsr_string_from_code_units(codes.flat_map(|code| {
        assert!((0..=0x10FFFF).contains(&code), "{failure}");
        let code = code as u32;
        if code < 0x10000 {
            [code as u16, 0].into_iter().take(1)
        } else {
            let offset = code - 0x10000;
            [0xD800 | (offset >> 10) as u16, 0xDC00 | (offset & 0x3FF) as u16].into_iter().take(2)
        }
    }))
}
pub fn tsr_number_parse_int(text: &str, radix: Option<i64>) -> f64 {
    let text = text.trim_start_matches(|character| {
        matches!(character,
            '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}'
            | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
            | '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    });
    let (negative, mut digits) = match text.as_bytes().first() {
        Some(b'-') => (true, &text[1..]),
        Some(b'+') => (false, &text[1..]),
        _ => (false, text),
    };
    let mut radix = radix.unwrap_or(0);
    let mut prefix = true;
    if radix == 0 {
        radix = 10;
    } else if !(2..=36).contains(&radix) {
        return f64::NAN;
    } else if radix != 16 {
        prefix = false;
    }
    if prefix && (digits.starts_with("0x") || digits.starts_with("0X")) {
        digits = &digits[2..];
        radix = 16;
    }
    let radix = radix as u32;
    let end = digits
        .bytes()
        .position(|byte| !byte.is_ascii() || (byte as char).to_digit(radix).is_none())
        .unwrap_or(digits.len());
    if end == 0 {
        return f64::NAN;
    }
    let value = if radix == 10 {
        digits[..end].parse::<f64>().unwrap_or(f64::NAN)
    } else {
        digits[..end].bytes().fold(0.0f64, |value, byte| {
            value * f64::from(radix) + f64::from((byte as char).to_digit(radix).unwrap_or(0))
        })
    };
    if negative { -value } else { value }
}

// Only the decimal prefix is selected here; standard f64 parsing supplies all
// rounding, overflow, underflow, and signed-zero behavior without input copies.
pub fn tsr_number_parse_float(text: &str) -> f64 {
    let text = text.trim_start_matches(|character| {
        matches!(character,
            '\u{0009}'..='\u{000d}' | '\u{0020}' | '\u{00a0}' | '\u{1680}'
            | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}'
            | '\u{205f}' | '\u{3000}' | '\u{feff}'
        )
    });
    let bytes = text.as_bytes();
    let mut cursor = usize::from(matches!(bytes.first(), Some(b'+' | b'-')));
    if text[cursor..].starts_with("Infinity") {
        return if bytes.first() == Some(&b'-') {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }
    let first_digit = cursor;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    let mut has_digit = cursor > first_digit;
    if bytes.get(cursor) == Some(&b'.') {
        cursor += 1;
        let fraction_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        has_digit |= cursor > fraction_start;
    }
    if !has_digit {
        return f64::NAN;
    }
    let mantissa_end = cursor;
    if matches!(bytes.get(cursor), Some(b'e' | b'E')) {
        cursor += 1;
        if matches!(bytes.get(cursor), Some(b'+' | b'-')) {
            cursor += 1;
        }
        let exponent_start = cursor;
        while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
            cursor += 1;
        }
        if cursor == exponent_start {
            cursor = mantissa_end;
        }
    }
    text[..cursor].parse::<f64>().unwrap_or(f64::NAN)
}

// Standard f64 formatters supply the digits and use Rust's ties-even rounding.
// These adapters change only decimal layout and JavaScript's surface spelling.
pub fn number_format_special(value: f64) -> Option<String> {
    if value.is_nan() {
        Some(String::Borrowed("NaN"))
    } else if value == f64::INFINITY {
        Some(String::Borrowed("Infinity"))
    } else if value == f64::NEG_INFINITY {
        Some(String::Borrowed("-Infinity"))
    } else {
        None
    }
}

pub fn number_decimal_layout(mut scientific: std::string::String, upper: i32) -> std::string::String {
    let exponent_position = scientific.rfind('e').expect("standard scientific format");
    let exponent: i32 = scientific[exponent_position + 1..]
        .parse()
        .expect("standard scientific exponent");
    if exponent < -6 || exponent >= upper {
        if exponent >= 0 {
            scientific.insert(exponent_position + 1, '+');
        }
        return scientific;
    }
    let sign = usize::from(scientific.starts_with('-'));
    scientific.truncate(exponent_position);
    if let Some(point) = scientific.find('.') {
        scientific.remove(point);
    }
    let digits = scientific.len() - sign;
    let point = exponent + 1;
    if point <= 0 {
        scientific.insert_str(sign, "0.");
        for _ in 0..-point {
            scientific.insert(sign + 2, '0');
        }
    } else if (point as usize) < digits {
        scientific.insert(sign + point as usize, '.');
    } else {
        for _ in digits..point as usize {
            scientific.push('0');
        }
    }
    scientific
}

pub fn number_scientific(value: f64, digits: Option<usize>, capacity: usize) -> std::string::String {
    use std::fmt::Write as _;
    let mut output = std::string::String::with_capacity(capacity);
    match digits {
        Some(digits) => write!(&mut output, "{:.*e}", digits, value),
        None => write!(&mut output, "{value:e}"),
    }
    .expect("standard String formatting");
    if digits.is_none() {
        tsr_number_tie(value, &mut output, 0);
    }
    output
}

pub fn number_shortest(value: f64) -> String {
    if value == 0.0 || (1e-6..1e21).contains(&value.abs()) {
        use std::fmt::Write as _;
        let mut output = std::string::String::with_capacity(32);
        write!(&mut output, "{value}").expect("standard String formatting");
        tsr_number_tie(value, &mut output, 0);
        String::Owned(output)
    } else {
        String::Owned(number_decimal_layout(
            number_scientific(value, None, 32),
            21,
        ))
    }
}

// Radix 10 is the shortest round-trip text. Other radices emit the represented
// integer digits and the fraction to the shortest text that reads back as the
// same value, rounding the last digit to nearest with carry.
pub fn tsr_number_to_string(value: f64, radix: i64, failure: &'static str) -> String {
    assert!((2..=36).contains(&radix), "{failure}");
    if let Some(special) = number_format_special(value) {
        return special;
    }
    if value == 0.0 {
        return String::Borrowed("0");
    }
    if radix == 10 {
        return number_shortest(value);
    }
    let digit_radix = radix as u32;
    let base = radix as f64;
    let negative = value < 0.0;
    let value = value.abs();
    let mut integer = value.floor();
    let mut fraction = value - integer;
    let mut delta = (0.5 * (f64::from_bits(value.to_bits() + 1) - value)).max(f64::from_bits(1));
    let mut fraction_digits = std::string::String::new();
    if fraction >= delta {
        loop {
            fraction *= base;
            delta *= base;
            let digit = fraction as u32;
            fraction_digits.push(char::from_digit(digit, digit_radix).expect("fraction digit"));
            fraction -= f64::from(digit);
            if (fraction > 0.5 || (fraction == 0.5 && digit % 2 == 1)) && fraction + delta > 1.0 {
                loop {
                    match fraction_digits.pop() {
                        None => {
                            integer += 1.0;
                            break;
                        }
                        Some(last) => {
                            let next = last.to_digit(digit_radix).expect("fraction digit") + 1;
                            if next < digit_radix {
                                fraction_digits.push(char::from_digit(next, digit_radix).expect("fraction digit"));
                                break;
                            }
                        }
                    }
                }
                break;
            }
            if fraction < delta {
                break;
            }
        }
    }
    // Digits are produced least significant first.
    let mut integer_digits = Vec::new();
    // Low digits that the value cannot represent print as zeros: divide while
    // the quotient is still at least 2^53.
    while integer / base >= 9007199254740992.0 {
        integer /= base;
        integer_digits.push('0');
    }
    loop {
        let remainder = integer % base;
        integer_digits.push(char::from_digit(remainder as u32, digit_radix).expect("integer digit"));
        integer = (integer - remainder) / base;
        if integer <= 0.0 {
            break;
        }
    }
    let mut output = std::string::String::with_capacity(
        integer_digits.len() + fraction_digits.len() + 2,
    );
    if negative {
        output.push('-');
    }
    output.extend(integer_digits.iter().rev());
    if !fraction_digits.is_empty() {
        output.push('.');
        output.push_str(&fraction_digits);
    }
    String::Owned(output)
}

pub fn tsr_number_fixed(value: f64, digits: Option<i64>, failure: &'static str) -> String {
    let digits = digits.unwrap_or(0);
    assert!((0..=100).contains(&digits), "{failure}");
    if let Some(special) = number_format_special(value) {
        return special;
    }
    let value = if value == 0.0 { 0.0 } else { value };
    if value.abs() >= 1e21 {
        return number_shortest(value);
    }
    use std::fmt::Write as _;
    let mut output = std::string::String::with_capacity(digits as usize + 24);
    write!(&mut output, "{:.*}", digits as usize, value).expect("standard String formatting");
    String::Owned(output)
}

pub fn tsr_number_exponential(value: f64, digits: Option<i64>, failure: &'static str) -> String {
    if let Some(special) = number_format_special(value) {
        return special;
    }
    if let Some(digits) = digits {
        assert!((0..=100).contains(&digits), "{failure}");
    }
    let value = if value == 0.0 { 0.0 } else { value };
    let capacity = digits.map_or(32, |digits| digits as usize + 8);
    let scientific = number_scientific(value, digits.map(|digits| digits as usize), capacity);
    String::Owned(number_decimal_layout(scientific, -6))
}

pub fn tsr_number_precision(value: f64, precision: Option<i64>, failure: &'static str) -> String {
    if let Some(special) = number_format_special(value) {
        return special;
    }
    if let Some(precision) = precision {
        assert!((1..=100).contains(&precision), "{failure}");
    }
    let value = if value == 0.0 { 0.0 } else { value };
    match precision {
        None => number_shortest(value),
        Some(precision) => String::Owned(number_decimal_layout(
            number_scientific(value, Some(precision as usize - 1), precision as usize + 8),
            precision as i32,
        )),
    }
}

pub fn string_owned_capacity(capacity: usize, failure: &'static str) -> std::string::String {
    assert!(capacity <= isize::MAX as usize, "{failure}");
    let mut output = std::string::String::new();
    output.try_reserve_exact(capacity).expect(failure);
    output
}

pub fn tsr_string_concat(value: &String, arguments: &[&String], failure: &'static str) -> String {
    tsr_string_concat_iter(value, arguments.iter().copied(), failure)
}
pub fn tsr_string_concat_iter<'a, I>(value: &String, arguments: I, failure: &'static str) -> String
where
    I: Iterator<Item = &'a String> + Clone,
{
    if arguments.clone().all(|argument| argument.is_empty()) {
        return value.clone();
    }
    let capacity = arguments.clone().fold(value.len(), |capacity, argument| {
        capacity.checked_add(argument.len()).expect(failure)
    });
    let mut output = string_owned_capacity(capacity, failure);
    output.push_str(value);
    for argument in arguments {
        output.push_str(argument);
    }
    String::Owned(output)
}

pub fn tsr_string_pad(
    value: &String,
    target: i64,
    filler: Option<&String>,
    start: bool,
    failure: &'static str,
    boundary_failure: &'static str,
) -> String {
    let filler = filler.map_or(" ", |filler| filler.as_ref());
    // Cow's standard clone retains static borrows and copies owned buffers.
    // Check no-ops before converting/reserving an extreme target or tail access.
    if target <= 0 || target as u128 <= value.len() as u128 || filler.is_empty() {
        return value.clone();
    }
    let capacity = usize::try_from(target).expect(failure);
    let needed = capacity - value.len();
    let repetitions = needed / filler.len();
    let tail = filler.get(..needed % filler.len()).expect(boundary_failure);
    let mut output = string_owned_capacity(capacity, failure);
    if !start {
        output.push_str(value);
    }
    for _ in 0..repetitions {
        output.push_str(filler);
    }
    output.push_str(tail);
    if start {
        output.push_str(value);
    }
    String::Owned(output)
}

/// Appends decimal digits without a temporary string.
pub fn tsr_push_int(output: &mut std::string::String, value: i64) {
    let mut digits = [0u8; 20];
    let mut index = digits.len();
    let mut rest = value.unsigned_abs();
    loop {
        index -= 1;
        digits[index] = b'0' + (rest % 10) as u8;
        rest /= 10;
        if rest == 0 {
            break;
        }
    }
    if value < 0 {
        output.push('-');
    }
    output.push_str(std::str::from_utf8(&digits[index..]).expect("ASCII digits"));
}

/// Whole numbers below 2^53 print as their integer digits. Returns false for
/// other values, which the caller formats with its own number spelling.
pub fn tsr_push_whole_number(output: &mut std::string::String, value: f64) -> bool {
    if value.fract() != 0.0 || value.abs() >= 9007199254740992.0 {
        return false;
    }
    tsr_push_int(output, value as i64);
    true
}

/// `text += value` in place: a borrowed text becomes one exactly sized owned
/// string, and an owned text appends into its buffer with amortized growth.
pub fn tsr_string_append(text: &mut String, value: &String, failure: &'static str) {
    if value.is_empty() {
        return;
    }
    let capacity = text.len().checked_add(value.len()).expect(failure);
    match text {
        String::Owned(owned) => owned.push_str(value),
        String::Borrowed(borrowed) => {
            let mut owned = std::string::String::with_capacity(capacity);
            owned.push_str(borrowed);
            owned.push_str(value);
            *text = String::Owned(owned);
        }
    }
}

/// Make room for `additional` bytes at the end of `text` and return its owned
/// buffer: a borrowed text becomes one exactly sized owned string, an owned
/// text reserves with amortized growth.
pub fn tsr_string_reserve<'a>(text: &'a mut String, additional: usize, failure: &'static str) -> &'a mut std::string::String {
    let capacity = text.len().checked_add(additional).expect(failure);
    if let String::Borrowed(borrowed) = text {
        let mut owned = std::string::String::with_capacity(capacity);
        owned.push_str(borrowed);
        *text = String::Owned(owned);
    }
    match text {
        String::Owned(owned) => {
            owned.reserve(additional);
            owned
        }
        String::Borrowed(_) => unreachable!("borrowed text was made owned"),
    }
}

pub fn tsr_string_add(left: &String, right: &String, failure: &'static str) -> String {
    let capacity = left.len().checked_add(right.len()).expect(failure);
    let mut output = std::string::String::with_capacity(capacity);
    output.push_str(left);
    output.push_str(right);
    String::Owned(output)
}

pub fn tsr_length(value: &str) -> i64 {
    i64::try_from(value.len()).expect("string length exceeds int range")
}

pub fn string_offset(len: usize, index: i64) -> usize {
    if index < 0 {
        (len as i128 + index as i128).max(0) as usize
    } else {
        (index as u128).min(len as u128) as usize
    }
}

/// The range `slice(start, end)` selects, borrowed from `value`.
pub fn tsr_slice_str(value: &String, start: i64, end: Option<i64>) -> &str {
    let len = value.len();
    let start = string_offset(len, start);
    let end = end.map_or(len, |end| string_offset(len, end));
    if end <= start {
        return "";
    }
    value.get(start..end).expect("string slice is not on UTF-8 boundaries")
}

pub fn tsr_slice(value: &String, start: i64, end: Option<i64>) -> String {
    let len = value.len();
    let start = string_offset(len, start);
    let end = end.map_or(len, |end| string_offset(len, end));
    if end <= start {
        return String::Borrowed("");
    }
    if start == 0 && end == len {
        return value.clone();
    }
    match value {
        String::Borrowed(text) => String::Borrowed(
            text.get(start..end)
                .expect("string slice is not on UTF-8 boundaries"),
        ),
        String::Owned(text) => String::Owned(
            text.get(start..end)
                .expect("string slice is not on UTF-8 boundaries")
                .to_owned(),
        ),
    }
}

pub fn tsr_substring(value: &String, start: i64, end: Option<i64>, failure: &str) -> String {
    let len = value.len();
    let clamp = |index: i64| (index.max(0) as u128).min(len as u128) as usize;
    let first = clamp(start);
    let last = end.map_or(len, clamp);
    let start = first.min(last);
    let end = first.max(last);
    if start == end {
        return String::Borrowed("");
    }
    if start == 0 && end == len {
        return value.clone();
    }
    match value {
        String::Borrowed(text) => String::Borrowed(text.get(start..end).expect(failure)),
        String::Owned(text) => String::Owned(text.get(start..end).expect(failure).to_owned()),
    }
}

// Substr counts a length from a relative byte start. Cap before addition so
// every signed extreme is safe and an empty interior range needs no boundary.
pub fn tsr_substr(value: &String, start: i64, length: Option<i64>, failure: &str) -> String {
    let len = value.len();
    let start = string_offset(len, start);
    let remaining = len - start;
    let length = length.map_or(remaining, |length| {
        (length.max(0) as u128).min(remaining as u128) as usize
    });
    let end = start + length;
    if start == end {
        return String::Borrowed("");
    }
    if start == 0 && end == len {
        return value.clone();
    }
    match value {
        String::Borrowed(text) => String::Borrowed(text.get(start..end).expect(failure)),
        String::Owned(text) => String::Owned(text.get(start..end).expect(failure).to_owned()),
    }
}

/// Every ASCII character once, so a one-character ASCII string borrows its
/// text instead of allocating (JavaScript engines cache these the same way).
const TSR_ASCII: &str = "\u{0}\u{1}\u{2}\u{3}\u{4}\u{5}\u{6}\u{7}\u{8}\u{9}\u{a}\u{b}\u{c}\u{d}\u{e}\u{f}\u{10}\u{11}\u{12}\u{13}\u{14}\u{15}\u{16}\u{17}\u{18}\u{19}\u{1a}\u{1b}\u{1c}\u{1d}\u{1e}\u{1f}\u{20}\u{21}\u{22}\u{23}\u{24}\u{25}\u{26}\u{27}\u{28}\u{29}\u{2a}\u{2b}\u{2c}\u{2d}\u{2e}\u{2f}\u{30}\u{31}\u{32}\u{33}\u{34}\u{35}\u{36}\u{37}\u{38}\u{39}\u{3a}\u{3b}\u{3c}\u{3d}\u{3e}\u{3f}\u{40}\u{41}\u{42}\u{43}\u{44}\u{45}\u{46}\u{47}\u{48}\u{49}\u{4a}\u{4b}\u{4c}\u{4d}\u{4e}\u{4f}\u{50}\u{51}\u{52}\u{53}\u{54}\u{55}\u{56}\u{57}\u{58}\u{59}\u{5a}\u{5b}\u{5c}\u{5d}\u{5e}\u{5f}\u{60}\u{61}\u{62}\u{63}\u{64}\u{65}\u{66}\u{67}\u{68}\u{69}\u{6a}\u{6b}\u{6c}\u{6d}\u{6e}\u{6f}\u{70}\u{71}\u{72}\u{73}\u{74}\u{75}\u{76}\u{77}\u{78}\u{79}\u{7a}\u{7b}\u{7c}\u{7d}\u{7e}\u{7f}";

/// One character as text: ASCII borrows from a static table, other
/// characters own their UTF-8 bytes.
#[inline]
pub fn tsr_char_text(character: char) -> String {
    let code = character as usize;
    if code < 128 {
        String::Borrowed(&TSR_ASCII[code..code + 1])
    } else {
        String::Owned(character.to_string())
    }
}

/// `text.at(index)`: the character at `index`, counted from the end when
/// negative; `None` outside the text.
pub fn tsr_string_at(value: &String, index: i64) -> Option<String> {
    let character = if index >= 0 {
        usize::try_from(index).ok().and_then(|index| value.chars().nth(index))
    } else {
        usize::try_from(index.unsigned_abs() - 1).ok().and_then(|index| value.chars().rev().nth(index))
    };
    character.map(tsr_char_text)
}

pub fn tsr_char_at(value: &String, index: i64) -> String {
    usize::try_from(index)
        .ok()
        .and_then(|index| value.chars().nth(index))
        .map_or(String::Borrowed(""), tsr_char_text)
}

pub fn tsr_trim(value: &String) -> String {
    match value {
        String::Borrowed(text) => String::Borrowed(text.trim()),
        String::Owned(text) => String::Owned(text.trim().to_owned()),
    }
}

pub fn tsr_trim_start(value: &String) -> String {
    match value {
        String::Borrowed(text) => String::Borrowed(text.trim_start()),
        String::Owned(text) => String::Owned(text.trim_start().to_owned()),
    }
}

pub fn tsr_trim_end(value: &String) -> String {
    match value {
        String::Borrowed(text) => String::Borrowed(text.trim_end()),
        String::Owned(text) => String::Owned(text.trim_end().to_owned()),
    }
}

pub fn tsr_lower(value: &String) -> String {
    String::Owned(value.to_lowercase())
}

pub fn tsr_normalize(value: &String) -> String {
    String::Owned(value.trim().to_lowercase())
}

pub fn tsr_upper(value: &String) -> String {
    String::Owned(value.to_uppercase())
}

// Search positions are nonnegative clamped UTF-8 byte coordinates.
pub fn string_search_position(len: usize, position: Option<i64>, default: usize) -> usize {
    position.map_or(default, |position| {
        if position < 0 {
            0
        } else {
            usize::try_from(position).unwrap_or(usize::MAX).min(len)
        }
    })
}

pub fn tsr_string_starts_with(value: &str, needle: &str, position: Option<i64>) -> bool {
    let start = string_search_position(value.len(), position, 0);
    value
        .get(start..)
        .expect("string search position is not on a UTF-8 boundary")
        .starts_with(needle)
}

pub fn tsr_string_ends_with(value: &str, needle: &str, position: Option<i64>) -> bool {
    let end = string_search_position(value.len(), position, value.len());
    value
        .get(..end)
        .expect("string search position is not on a UTF-8 boundary")
        .ends_with(needle)
}

pub fn tsr_string_includes(value: &str, needle: &str, position: Option<i64>) -> bool {
    let start = string_search_position(value.len(), position, 0);
    let tail = value
        .get(start..)
        .expect("string search position is not on a UTF-8 boundary");
    tsr_find(tail, needle).is_some()
}

// Short haystacks: a direct byte-window scan avoids building a two-way searcher
// on every call. A valid UTF-8 needle cannot start with a continuation byte, so
// every byte match begins on a character boundary.
const SHORT_SEARCH: usize = 64;

fn tsr_find(haystack: &str, needle: &str) -> Option<usize> {
    let (bytes, wanted) = (haystack.as_bytes(), needle.as_bytes());
    let Some(&first) = wanted.first() else { return Some(0) };
    if bytes.len() > SHORT_SEARCH {
        return haystack.find(needle);
    }
    let last = bytes.len().checked_sub(wanted.len())?;
    (0..=last).find(|&start| bytes[start] == first && &bytes[start..start + wanted.len()] == wanted)
}

fn tsr_rfind(haystack: &str, needle: &str) -> Option<usize> {
    let (bytes, wanted) = (haystack.as_bytes(), needle.as_bytes());
    let Some(&first) = wanted.first() else { return Some(bytes.len()) };
    if bytes.len() > SHORT_SEARCH {
        return haystack.rfind(needle);
    }
    let last = bytes.len().checked_sub(wanted.len())?;
    (0..=last).rev().find(|&start| bytes[start] == first && &bytes[start..start + wanted.len()] == wanted)
}

pub fn tsr_string_index_of(value: &str, needle: &str, position: Option<i64>) -> i64 {
    let start = string_search_position(value.len(), position, 0);
    let tail = value
        .get(start..)
        .expect("string search position is not on a UTF-8 boundary");
    tsr_find(tail, needle)
        .map_or(-1, |offset| {
            i64::try_from(start + offset).expect("string search index exceeds int range")
        })
}

pub fn tsr_string_last_index_of(value: &str, needle: &str, position: Option<i64>) -> i64 {
    let position = string_search_position(value.len(), position, value.len());
    assert!(
        value.is_char_boundary(position),
        "string search position is not on a UTF-8 boundary"
    );
    if needle.is_empty() {
        return i64::try_from(position).expect("string search index exceeds int range");
    }
    // Bound match starts, retaining an entire match that begins at position.
    // Only the internal endpoint is rounded; the selected boundary is checked.
    let end = value.floor_char_boundary(position.saturating_add(needle.len()).min(value.len()));
    tsr_rfind(&value[..end], needle).map_or(-1, |offset| {
        i64::try_from(offset).expect("string search index exceeds int range")
    })
}

pub fn tsr_string_repeat(value: &String, count: i64, failure: &'static str) -> String {
    assert!(count >= 0, "string repeat count must be nonnegative");
    let count = usize::try_from(count).expect(failure);
    let capacity = value.len().checked_mul(count).expect(failure);
    assert!(capacity <= isize::MAX as usize, "{failure}");
    String::Owned(value.repeat(count))
}

// String patterns have no captures. Only the four ordinary substitution tokens
// expand; prefix/suffix always refer to the original source, not built output.
pub fn string_replacement_template(
    mut emit: impl FnMut(&str),
    source: &str,
    position: usize,
    matched: &str,
    template: &str,
) {
    let mut remainder = template;
    while let Some(dollar) = remainder.find('$') {
        emit(&remainder[..dollar]);
        remainder = &remainder[dollar + 1..];
        match remainder.as_bytes().first() {
            Some(b'$') => emit("$"),
            Some(b'&') => emit(matched),
            Some(b'`') => emit(&source[..position]),
            Some(b'\'') => emit(&source[position + matched.len()..]),
            _ => {
                emit("$");
                continue;
            }
        }
        remainder = &remainder[1..];
    }
    emit(remainder);
}

pub fn tsr_string_replace(value: &String, pattern: &String, replacement: &String, all: bool) -> String {
    if !replacement.contains('$') {
        return String::Owned(if all {
            value.replace(pattern.as_ref(), replacement.as_ref())
        } else {
            value.replacen(pattern.as_ref(), replacement.as_ref(), 1)
        });
    }
    let source = value.as_ref();
    let matches = source.match_indices(pattern.as_ref()).take(if all { usize::MAX } else { 1 });
    let mut capacity = source.len();
    if !all {
        for (position, matched) in matches.clone() {
            capacity -= matched.len();
            string_replacement_template(
                |piece| capacity = capacity.checked_add(piece.len()).expect("capacity overflow"),
                source, position, matched, replacement.as_ref(),
            );
        }
    }
    let mut output = std::string::String::with_capacity(capacity);
    let mut end = 0;
    for (position, matched) in matches {
        output.push_str(&source[end..position]);
        string_replacement_template(|piece| output.push_str(piece), source, position, matched, replacement.as_ref());
        end = position + matched.len();
    }
    output.push_str(&source[end..]);
    String::Owned(output)
}

// Deterministic decimal display: ASCII grouping, up to three fractional digits,
// shortest decimal input and ties away from zero. No locale database is needed.
pub fn tsr_locale_group(text: &str) -> String {
    let negative = text.starts_with('-');
    let text = text.strip_prefix('-').unwrap_or(text);
    let (integer, fraction) = text.split_once('.').unwrap_or((text, ""));
    let mut output = std::string::String::with_capacity(text.len() + integer.len() / 3 + 1);
    if negative { output.push('-'); }
    for (index, digit) in integer.chars().enumerate() {
        if index != 0 && (integer.len() - index) % 3 == 0 { output.push(','); }
        output.push(digit);
    }
    if !fraction.is_empty() { output.push('.'); output.push_str(fraction); }
    String::Owned(output)
}

pub fn tsr_int_locale_string(value: i64) -> String {
    tsr_locale_group(&value.to_string())
}

pub fn tsr_number_locale_string(value: f64) -> String {
    if value.is_nan() { return String::Borrowed("NaN"); }
    if value == f64::INFINITY { return String::Borrowed("∞"); }
    if value == f64::NEG_INFINITY { return String::Borrowed("-∞"); }
    let negative = value.is_sign_negative();
    if value.abs() < 0.0005 {
        return String::Borrowed(if negative { "-0" } else { "0" });
    }
    // Expand even large exponents before inserting separators. Scientific text
    // supplies the same shortest decimal digits as ordinary String conversion.
    let text = number_decimal_layout(number_scientific(value.abs(), None, 32), 1000);
    let mut digits = text.into_bytes();
    if let Some(point) = digits.iter().position(|&digit| digit == b'.') {
        if digits.len() > point + 4 {
            let round_up = digits[point + 4] >= b'5';
            digits.truncate(point + 4);
            if round_up {
                let mut cursor = digits.len();
                loop {
                    if cursor == 0 { digits.insert(0, b'1'); break; }
                    cursor -= 1;
                    if digits[cursor] == b'.' { continue; }
                    if digits[cursor] != b'9' { digits[cursor] += 1; break; }
                    digits[cursor] = b'0';
                }
            }
        }
        while digits.last() == Some(&b'0') { digits.pop(); }
        if digits.last() == Some(&b'.') { digits.pop(); }
    }
    if negative { digits.insert(0, b'-'); }
    tsr_locale_group(std::str::from_utf8(&digits).expect("decimal ASCII digits"))
}
