// Independent standard Rust owned results and observable control-flow completion.
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
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
struct Choice {
    tag: i64,
    payload: Vec<i64>,
}
#[derive(Default)]
struct Trace {
    predicates: i64,
    left: i64,
    right: i64,
}
fn predicate(trace: &mut Trace, flag: bool) -> bool {
    trace.predicates = add(trace.predicates, 1);
    flag
}
fn left_value(trace: &mut Trace, at: i64, size: i64) -> Choice {
    trace.left = add(trace.left, 1);
    Choice {
        tag: 101,
        payload: (0..size).map(|j| add(mul(at, 8), j)).collect(),
    }
}
fn right_value(trace: &mut Trace, at: i64, size: i64) -> Choice {
    trace.right = add(trace.right, 1);
    Choice {
        tag: 202,
        payload: (0..size).map(|j| add(add(mul(at, 8), j), 64)).collect(),
    }
}

fn function0(value0: i64, value1: i64) -> NumericArray<i64> {
    let mut output = Vec::new();
    let mut trace = Trace::default();
    for i in 0..value0 {
        let size = add(mul(i, 7), mul(value1, 2)) % 4;
        let at = add(mul(i, 13), mul(value1, 2)) % 8;
        let selected = if predicate(&mut trace, add(i, value1) % 2 == 0) {
            left_value(&mut trace, at, size)
        } else {
            right_value(&mut trace, at, size)
        };
        output.push(selected.tag);
        output.push(i64::try_from(selected.payload.len()).unwrap());
        output.extend(selected.payload);
    }
    output.extend([trace.predicates, trace.left, trace.right]);
    NumericArray(output)
}
fn main() {}
