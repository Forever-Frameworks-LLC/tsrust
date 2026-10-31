// Independent Rust records/unions reference; preserve storage, order and observed operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn owned(value: impl Into<Text>) -> String {
    String::Owned(value.into())
}
#[derive(Clone, Copy)]
enum Key {
    Alpha,
    Beta,
    Gamma,
}
impl Key {
    fn name(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Gamma => "gamma",
        }
    }
}
#[derive(Clone)]
struct Row {
    alpha: i64,
    beta: Text,
    gamma: bool,
    order: [Key; 3],
}
impl Row {
    fn new(base: i64, input: &str, even: bool) -> Self {
        Self {
            alpha: base,
            beta: input.to_owned(),
            gamma: even,
            order: if even {
                [Key::Gamma, Key::Alpha, Key::Beta]
            } else {
                [Key::Beta, Key::Gamma, Key::Alpha]
            },
        }
    }
    fn keys(&self) -> Vec<&'static str> {
        self.order.iter().map(|key| key.name()).collect()
    }
}
fn text_keys(text: &str) -> Vec<Text> {
    (0..text.encode_utf16().count())
        .map(|index| index.to_string())
        .collect()
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let source = Row::new(base, value2, add(i, value1) % 2 == 0);
        let mut names = source.keys();
        let copied = source.clone();
        let copied_names = copied.keys();
        let text_names = text_keys(value2);
        let original = names.join("|");
        names[0] = "changed";
        output.push(Array(vec![
            owned(original),
            owned(copied_names.join("|")),
            owned(names.join("|")),
            owned(text_names.join("|")),
            owned(source.alpha.to_string()),
        ]));
    }
    output
}
fn main() {}
