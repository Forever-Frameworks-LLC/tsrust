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
use std::collections::BinaryHeap;
fn sum_items(values: impl IntoIterator<Item = i64>) -> i64 {
    values.into_iter().fold(0, add)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let mut values = BinaryHeap::new();
        for j in 0..9 {
            values.push(add(add(i, value1), j) % 256);
        }
        let traversed = sum_items(values.iter().copied());
        let spread = sum_items(values.iter().copied());
        let flattened: Vec<_> = values.iter().copied().collect();
        let top = *values.peek().unwrap();
        let mut copied = values.clone();
        let mut sum = 0;
        while let Some(value) = copied.pop() {
            sum = add(sum, value);
        }
        output.push(NumericArray(vec![
            i64::try_from(values.len()).unwrap(),
            top,
            sum,
            i64::try_from(copied.len()).unwrap(),
            traversed,
            spread,
            length(&flattened),
        ]));
    }
    Array(output)
}
fn main() {}
