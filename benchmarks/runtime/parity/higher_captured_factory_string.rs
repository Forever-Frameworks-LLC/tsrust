// Independent standard Rust higher-order ownership and retained results.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
// Literal x replacement with the observable dollar-token expansion contract.
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
fn carry<F: Fn() -> R, R: Fn(&str) -> std::string::String>(f: F) -> impl Fn() -> R {
    move || f()
}
fn make(prefix: std::string::String) -> impl Fn() -> Box<dyn Fn(&str) -> std::string::String> {
    move || {
        let copied = prefix.clone();
        Box::new(move |text| replace_x(text, &copied))
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<String> {
    let consumer = carry;
    let mut output = Vec::new();
    for _ in 0..value0 {
        let factory = make(value2.to_string());
        let carried = consumer(factory);
        let callback = carried();
        output.push(String::Owned(callback("xé")));
    }
    output
}
fn main() {}
