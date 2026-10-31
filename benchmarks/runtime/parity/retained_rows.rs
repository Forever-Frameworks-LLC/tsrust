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

fn function0(value0: i64, value1: i64) -> Array<NumericArray<i64>> {
    let mut rows = Vec::new();
    for i in 0..value0 {
        let width = add(i, value1) % 65;
        let mut row = Vec::new();
        for j in 0..width {
            row.push(add(add(mul(i, 3), value1), j) % 1009);
        }
        rows.push(NumericArray(row));
    }
    Array(rows)
}
fn main() {}
