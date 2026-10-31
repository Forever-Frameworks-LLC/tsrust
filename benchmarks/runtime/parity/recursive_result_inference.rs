// Independent Rust recursion, including the captured mutable steps owner.
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
fn factorial(count: i64) -> i64 {
    if count == 0 {
        1
    } else {
        count
            .checked_mul(factorial(count.checked_sub(1).unwrap()))
            .unwrap()
    }
}
fn parity(count: i64) -> bool {
    count == 0 || !parity(count.checked_sub(1).unwrap())
}
fn label(count: i64) -> std::string::String {
    if count <= 0 {
        std::string::String::new()
    } else {
        format!("x{}", label(count.checked_sub(1).unwrap()))
    }
}
fn depth(count: i64, steps: &mut Vec<i64>) -> i64 {
    steps.push(count);
    if count == 0 {
        0
    } else {
        add(1, depth(count.checked_sub(1).unwrap(), steps))
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let _ = value2;
    fn fib(count: i64) -> i64 {
        if count < 2 {
            count
        } else {
            add(
                fib(count.checked_sub(1).unwrap()),
                fib(count.checked_sub(2).unwrap()),
            )
        }
    }
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let mut steps = Vec::new();
        output.push(NumericArray(vec![
            factorial(value),
            i64::from(parity(value)),
            i64::try_from(label(value).len()).unwrap(),
            depth(value, &mut steps),
            i64::try_from(steps.len()).unwrap(),
            fib(value),
        ]));
    }
    Array(output)
}
fn main() {}
