// Independent standard Rust string operations; retained dynamic outputs are independently owned.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
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
fn index(i: i64, stride: i64, seed: i64, len: i64) -> usize {
    usize::try_from(add(mul(i, stride), seed) % len).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {
    let mut predicates = 0;
    let mut left = 0;
    let mut right = 0;
    let mut output = Vec::new();
    for i in 0..value0 {
        predicates = add(predicates, 1);
        let selected = if add(i, value1) % 2 == 0 {
            left = add(left, 1);
            "L".to_owned() + value2.as_ref()
        } else {
            right = add(right, 1);
            "R".to_owned() + &value2.repeat(2)
        };
        output.push(String::Owned(selected));
    }
    output.push(String::Owned(
        "P".repeat(usize::try_from(predicates).unwrap()),
    ));
    output.push(String::Owned("L".repeat(usize::try_from(left).unwrap())));
    output.push(String::Owned("R".repeat(usize::try_from(right).unwrap())));
    Array(output)
}
fn main() {}
