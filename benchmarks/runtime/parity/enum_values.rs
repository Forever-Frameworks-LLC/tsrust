// Independent standard Rust language reference; preserve values and owned data operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
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
fn owned(text: impl Into<Text>) -> String {
    String::Owned(text.into())
}
#[derive(Clone, Copy)]
#[repr(i64)]
enum Phase {
    Ready,
    Busy = 4,
    Done,
}
enum Label {
    Short,
    Long,
}
impl Label {
    fn text(&self) -> &'static str {
        match self {
            Self::Short => "s",
            Self::Long => "long",
        }
    }
}
fn choose(value: i64) -> Phase {
    match value % 3 {
        0 => Phase::Ready,
        1 => Phase::Busy,
        _ => Phase::Done,
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let phase = choose(add(i, value1));
        let copy = phase;
        let label = if matches!(phase, Phase::Ready) {
            Label::Short
        } else {
            Label::Long
        };
        output.push(Array(vec![
            owned((copy as i64).to_string()),
            String::Borrowed(label.text()),
            owned(format!("{value2}{}", phase as i64)),
        ]));
    }
    output
}
fn main() {}
