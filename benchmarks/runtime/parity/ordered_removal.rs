// Independent standard Rust record ownership, recursion and retained arrays.
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
fn size(n: i64) -> usize {
    usize::try_from(n).unwrap()
}
struct Entry {
    value: i64,
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = 0;
    for i in 0..value0 {
        let mut values: Vec<_> = (0..32).map(|j| add(add(value1, i), j)).collect();
        sum = add(sum, values.remove(3));
        let end = values.len() - 1;
        sum = add(sum, values.remove(end));
        sum = add(sum, values.remove(0));
        let mut position = 1;
        for value in values {
            sum = add(sum, mul(value, position));
            position = add(position, 1);
        }
        let mut entries = vec![Entry { value: i }, Entry { value: value1 }];
        let entry = entries.remove(0);
        sum = add(sum, entry.value);
        let mut rows = vec![vec![i, value1], vec![value1]];
        let row = rows.remove(0);
        sum = add(add(sum, row[0]), row[1]);
    }
    sum
}
fn main() {}
