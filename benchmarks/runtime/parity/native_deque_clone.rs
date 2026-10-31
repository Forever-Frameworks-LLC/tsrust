// Independent standard Rust native storage ownership and collection operations.
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
fn length<T>(values: &[T]) -> i64 {
    i64::try_from(values.len()).unwrap()
}
use std::collections::VecDeque;
#[derive(Clone)]
struct Item {
    values: Vec<i64>,
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(i, value1) % 256;
        let mut original = VecDeque::new();
        original.push_back(Item {
            values: vec![value, 255],
        });
        let observed = original.front().unwrap().values[0];
        let mut copied = original.clone();
        let mut changed = copied.pop_front().unwrap();
        changed.values.push(add(value, 1));
        let retained = original.pop_front().unwrap();
        output.push(NumericArray(vec![
            length(&retained.values),
            retained.values[0],
            length(&changed.values),
            changed.values[2],
            i64::try_from(original.len()).unwrap(),
            i64::try_from(copied.len()).unwrap(),
            observed,
        ]));
    }
    Array(output)
}
fn main() {}
