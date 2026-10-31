// Independent standard Rust optional callbacks and function expressions.
type String = std::borrow::Cow<'static, str>;
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
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.chars().count()).unwrap()
}
#[derive(Default)]
struct Trace {
    calls: i64,
    defaults: i64,
}
fn apply<F: FnOnce(&mut Trace) -> std::string::String>(
    trace: &mut Trace,
    action: Option<F>,
) -> Vec<std::string::String> {
    action.map(|f| f(trace)).into_iter().collect()
}
fn consume_text(text: std::string::String) -> std::string::String {
    text
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut trace = Trace::default();
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(mul(i, 13), value1) % 97;
        let mode = add(i, value1) % 4;
        let before = (trace.calls, trace.defaults);
        let action = if mode < 2 {
            None
        } else {
            let text = value2.repeat(usize::try_from(value % 3 + 1).unwrap())
                + ":present:"
                + &value.to_string();
            Some(move |trace: &mut Trace| {
                trace.calls = add(trace.calls, 1);
                consume_text(text)
            })
        };
        output.push(Array(
            apply(&mut trace, action)
                .into_iter()
                .map(String::Owned)
                .collect(),
        ));
        output.push(Array(vec![
            String::Owned(sub(trace.calls, before.0).to_string()),
            String::Owned(sub(trace.defaults, before.1).to_string()),
        ]));
    }
    output
}
fn main() {}
