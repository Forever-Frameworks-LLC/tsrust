// Independent standard Rust reference; retain pattern/default/control operations.
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
fn owned(value: impl Into<Text>) -> String {
    String::Owned(value.into())
}
struct Bounds {
    start: i64,
    end: i64,
}
fn bounds(base: i64) -> Bounds {
    Bounds {
        start: base % 3,
        end: add(base % 3, 3),
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let words = vec![value2.repeat(1), base.to_string()];
        let Bounds { start, end } = bounds(base);
        let mut sum = 0;
        for k in start..end {
            if k == add(base % 3, 1) {
                continue;
            }
            sum = add(sum, k);
        }
        let initial = [0_i64];
        let mut j = initial[0];
        let stop = initial.get(1).copied().unwrap_or(2);
        let mut count = 0;
        while j < stop {
            count = add(count, add(j, 1));
            j = add(j, 1);
        }
        let mut text = Text::new();
        let (first, tail) = words.split_first().unwrap();
        let mut tail = tail.to_vec();
        while tail.len() < 3 {
            text.push_str(first);
            text.push_str(&tail.len().to_string());
            tail.push("x".to_owned());
        }
        output.push(Array(vec![
            owned(sum.to_string()),
            owned(count.to_string()),
            owned(text),
            owned(words.len().to_string()),
        ]));
    }
    output
}
fn main() {}
