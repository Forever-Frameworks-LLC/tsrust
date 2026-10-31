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
    factories: i64,
    inner_factories: i64,
    arguments: i64,
    outer_calls: i64,
    inner_calls: i64,
    fallbacks: i64,
}
impl Trace {
    fn delta(self, b: Self) -> Vec<i64> {
        vec![
            sub(self.factories, b.factories),
            sub(self.inner_factories, b.inner_factories),
            sub(self.arguments, b.arguments),
            sub(self.outer_calls, b.outer_calls),
            sub(self.inner_calls, b.inner_calls),
            sub(self.fallbacks, b.fallbacks),
        ]
    }
}
fn argument(trace: &mut Trace, value: i64) -> i64 {
    trace.arguments = add(trace.arguments, 1);
    value
}
fn make_inner(trace: &mut Trace, offset: i64) -> impl Fn(i64, &mut Trace) -> i64 + use<> {
    trace.inner_factories = add(trace.inner_factories, 1);
    move |value, trace: &mut Trace| {
        trace.inner_calls = add(trace.inner_calls, 1);
        add(add(mul(value, 3), offset), 7) % 1009
    }
}
struct Higher {
    bias: i64,
}
impl Higher {
    fn call<F: Fn(i64, &mut Trace) -> i64>(&self, inner: &F, value: i64, trace: &mut Trace) -> i64 {
        trace.outer_calls = add(trace.outer_calls, 1);
        add(inner(value, trace), self.bias) % 1009
    }
}
fn make_higher(tag: i64, bias: i64, trace: &mut Trace) -> Option<Higher> {
    trace.factories = add(trace.factories, 1);
    if tag == 0 {
        Some(Higher { bias })
    } else {
        None
    }
}
fn empty(_tag: i64, trace: &mut Trace) -> Option<Higher> {
    trace.factories = add(trace.factories, 1);
    None
}
fn forward_empty(tag: i64, trace: &mut Trace) -> Option<Higher> {
    empty(tag, trace)
}
fn make_present(bias: i64, trace: &mut Trace) -> Higher {
    trace.fallbacks = add(trace.fallbacks, 1);
    Higher { bias }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut trace = Trace::default();
    let mut output = Vec::with_capacity(usize::try_from(mul(value0, 2)).unwrap());
    for i in 0..value0 {
        let tag = add(i, value1) % 3;
        let value = add(add(mul(i, 13), value1), length(value2)) % 97;
        let before = trace;
        let mut observed = Vec::new();
        if let Some(action) = make_higher(tag, i % 7, &mut trace) {
            let inner = make_inner(&mut trace, value1 % 11);
            let arg = argument(&mut trace, value);
            observed.push(action.call(&inner, arg, &mut trace));
        }
        output.push(NumericArray(observed));
        output.push(NumericArray(trace.delta(before)));
    }
    Array(output)
}
fn main() {}
