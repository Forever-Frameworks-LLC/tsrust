// Independent standard Rust declared functions, callbacks and owned values.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn digest(state: i64, value: i64) -> i64 {
    add(mul(state, 131), value) % 1_000_000_007
}
fn count(i: i64, seed: i64) -> usize {
    [0, 1, 4, 5][usize::try_from(add(seed, i) % 4).unwrap()]
}
fn replace_x(text: &str, replacement: &str) -> std::string::String {
    let mut output = std::string::String::new();
    let mut consumed = 0;
    for (at, _) in text.match_indices('x') {
        output.push_str(&text[consumed..at]);
        let mut chars = replacement.chars().peekable();
        while let Some(ch) = chars.next() {
            if ch != '$' {
                output.push(ch);
                continue;
            }
            match chars.peek().copied() {
                Some('$') => {
                    chars.next();
                    output.push('$');
                }
                Some('&') => {
                    chars.next();
                    output.push('x');
                }
                Some('`') => {
                    chars.next();
                    output.push_str(&text[..at]);
                }
                Some('\'') => {
                    chars.next();
                    output.push_str(&text[at + 1..]);
                }
                _ => output.push('$'),
            }
        }
        consumed = at + 1;
    }
    output.push_str(&text[consumed..]);
    output
}
fn make(prefix: std::string::String) -> impl Fn(&str) -> std::string::String {
    move |text| replace_x(text, &prefix)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<String> {
    let prefixes = [
        value2.to_string(),
        "a\0b".to_owned(),
        "😀".to_owned(),
        "e\u{301}".to_owned(),
    ];
    let mut output = Vec::new();
    let factory = make;
    for i in 0..value0 {
        let prefix = prefixes[usize::try_from(add(value1, i) % 4).unwrap()].clone();
        let action = factory(prefix);
        output.push(String::Owned(action(value2)));
    }
    output
}
fn main() {}
