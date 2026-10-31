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
    i64::try_from(text.len()).unwrap()
}
#[derive(Default)]
struct Trace {
    calls: i64,
    defaults: i64,
}
fn apply<F: FnMut(i64, &mut Trace) -> i64>(
    trace: &mut Trace,
    value: i64,
    action: Option<&mut F>,
) -> Vec<i64> {
    action.map(|f| f(value, trace)).into_iter().collect()
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut trace = Trace::default();
    let mut state = value1 % 11;
    let mut callback = |value, trace: &mut Trace| {
        state = add(state, 1);
        trace.calls = add(trace.calls, 1);
        add(value, state)
    };
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(mul(i, 13), value1) % 97;
        let mode = add(i, value1) % 4;
        let before = (trace.calls, trace.defaults);
        let observed = apply(
            &mut trace,
            value,
            if mode < 2 { None } else { Some(&mut callback) },
        );
        output.push(NumericArray(observed));
        output.push(NumericArray(vec![
            sub(trace.calls, before.0),
            sub(trace.defaults, before.1),
        ]));
    }
    Array(output)
}
fn main() {}
