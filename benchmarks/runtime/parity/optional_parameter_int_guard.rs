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
struct Trace {
    calls: i64,
    defaults: i64,
}
fn apply(trace: &mut Trace, value: Option<i64>) -> i64 {
    trace.calls = add(trace.calls, 1);
    value.map_or(1000, |v| add(v, 3))
}
fn function0(value0: i64, value1: i64, value2: &String) -> NumericArray<i64> {
    let mut output = Vec::new();
    let mut trace = Trace {
        calls: 0,
        defaults: 0,
    };
    for i in 0..value0 {
        let mode = add(i, value1) % 4;
        let value = add(mul(i, 13), value1) % 97;
        let before = trace.calls;
        let _before_defaults = trace.defaults;
        let result = if mode == 0 {
            apply(&mut trace, None)
        } else if mode == 1 {
            apply(&mut trace, None)
        } else {
            apply(&mut trace, Some(value))
        };
        output.push(result);
        output.push(trace.calls.checked_sub(before).unwrap());
    }
    NumericArray(output)
}
fn main() {}
