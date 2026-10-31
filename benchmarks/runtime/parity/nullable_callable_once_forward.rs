// Independent standard Rust callable ownership. Null and undefined both mean absent here.
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

#[derive(Default, Clone, Copy)]
struct Trace {
    calls: i64,
    arguments: i64,
    fallbacks: i64,
    factories: i64,
}
impl Trace {
    fn delta(self, b: Self) -> Vec<i64> {
        vec![
            sub(self.calls, b.calls),
            sub(self.arguments, b.arguments),
            sub(self.fallbacks, b.fallbacks),
            sub(self.factories, b.factories),
        ]
    }
}
fn argument(trace: &mut Trace, value: i64) -> i64 {
    trace.arguments = add(trace.arguments, 1);
    value
}
fn step(value: i64, trace: &mut Trace) -> i64 {
    trace.calls = add(trace.calls, 1);
    add(mul(value, 17), 3) % 1009
}
fn fallback_step(value: i64, trace: &mut Trace) -> i64 {
    trace.calls = add(trace.calls, 1);
    add(mul(value, 17), 103) % 1009
}
type Action = fn(i64, &mut Trace) -> i64;
fn declared(tag: i64) -> Option<Action> {
    if tag == 0 { Some(step) } else { None }
}

fn forward_once<F: FnOnce(i64, &mut Trace) -> i64>(
    action: Option<F>,
    value: i64,
    trace: &mut Trace,
) -> Option<i64> {
    action.map(|f| {
        let value = argument(trace, value);
        f(value, trace)
    })
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut trace = Trace::default();
    let mut output = Vec::with_capacity(usize::try_from(mul(value0, 2)).unwrap());
    for i in 0..value0 {
        let tag = add(i, value1) % 3;
        let value = add(mul(i, 13), value1) % 97;
        let before = trace;
        let mut observed = Vec::new();
        let offset = add(i, value1) % 11;
        let action = if tag == 0 {
            Some(move |v, trace: &mut Trace| {
                trace.calls = add(trace.calls, 1);
                add(add(v, offset), 5) % 1009
            })
        } else {
            None
        };
        observed.extend(forward_once(action, value, &mut trace));
        output.push(NumericArray(observed));
        output.push(NumericArray(trace.delta(before)));
    }
    Array(output)
}
fn main() {}
