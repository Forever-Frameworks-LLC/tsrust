// Independent standard Rust nested functions and closures; checked integer arithmetic.
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
fn length(s: &str) -> i64 {
    i64::try_from(s.len()).unwrap()
}

fn double(v: i64) -> i64 {
    add(v, v)
}
fn factorial(v: i64) -> i64 {
    if v == 0 {
        1
    } else {
        mul(v, factorial(sub(v, 1)))
    }
}
fn even(v: i64) -> bool {
    v == 0 || odd(sub(v, 1))
}
fn odd(v: i64) -> bool {
    v != 0 && even(sub(v, 1))
}
fn row(value: i64) -> Vec<i64> {
    let first = double(value);
    let shadowed = {
        fn double(v: i64) -> i64 {
            mul(v, 3)
        }
        double(value)
    };
    let scoped = {
        let value = add(first, 3);
        let increment = |value| add(value, 1);
        increment(value)
    };
    let mapped: Vec<_> = [value, add(value, 1)].into_iter().map(double).collect();
    vec![
        first,
        factorial(5),
        i64::from(even(value)),
        shadowed,
        mapped[0],
        mapped[1],
        add(scoped, value),
    ]
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        output.push(NumericArray(row(value)));
    }
    Array(output)
}
fn main() {}
