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
fn counter(start: i64, depth: i64) -> impl FnMut() -> i64 {
    if depth > 0 {
        return counter(start, sub(depth, 1));
    }
    let mut value = start;
    move || {
        value = add(value, 1);
        value
    }
}
fn chain(seed: i64, depth: i64) -> impl Fn() -> i64 {
    move || {
        if depth == 0 {
            seed
        } else {
            let next = chain(seed, sub(depth, 1));
            next()
        }
    }
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = 0;
    for i in 0..value0 {
        let mut action = counter(i, 8);
        sum = add(add(sum, action()), action());
        let lazy = chain(value1, 4);
        sum = add(sum, lazy());
    }
    sum
}
fn main() {}
