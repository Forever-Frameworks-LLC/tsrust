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
use std::collections::BTreeMap;
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let mut values = BTreeMap::new();
        for j in 0..9 {
            let key = add(add(i, value1), j) % 256;
            values.insert(key, add(mul(key, 3), 1));
        }
        values.insert(add(i, value1) % 256, 999);
        let mut count = 0;
        let mut weighted = 0;
        let mut sum = 0;
        let mut first = -1;
        let mut last = -1;
        for (key, value) in &values {
            if count == 0 {
                first = *key;
            }
            count = add(count, 1);
            weighted = add(weighted, mul(*key, count));
            sum = add(sum, *value);
            last = *key;
        }
        let copied = values.clone();
        let key = add(add(i, value1), 4) % 256;
        let selected = *values.get(&key).unwrap();
        let removed = i64::from(values.remove(&key).is_some());
        output.push(NumericArray(vec![
            count,
            first,
            last,
            weighted,
            sum,
            selected,
            removed,
            i64::try_from(values.len()).unwrap(),
            i64::try_from(copied.len()).unwrap(),
        ]));
    }
    Array(output)
}
fn main() {}
