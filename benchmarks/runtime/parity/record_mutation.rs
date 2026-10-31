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
struct Counter {
    value: i64,
    active: bool,
}
fn increment(counter: &mut Counter) {
    counter.value = add(counter.value, 1);
    counter.active = counter.value % 2 == 0;
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut counter = Counter {
        value: value1,
        active: true,
    };
    let mut sum = 0;
    for _ in 0..value0 {
        increment(&mut counter);
        if counter.active {
            sum = add(sum, counter.value);
        }
    }
    sum
}
fn main() {}
