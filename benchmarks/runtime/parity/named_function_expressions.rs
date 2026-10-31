// Independent standard Rust optional callbacks and function expressions.
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
fn length(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
fn factorial(count: i64) -> i64 {
    if count <= 1 {
        1
    } else {
        mul(count, factorial(sub(count, 1)))
    }
}
fn down(depth: i64, base: &[i64], total: &mut i64) {
    if depth == 0 {
        return;
    }
    *total = add(*total, add(depth, base[0]));
    down(sub(depth, 1), base, total)
}
fn recursive_add(offset: i64, count: i64) -> i64 {
    if count <= 0 {
        offset
    } else {
        add(1, recursive_add(offset, sub(count, 1)))
    }
}
fn make(offset: i64) -> impl Fn(i64) -> i64 {
    move |count| recursive_add(offset, count)
}
fn choose<T>(left: T, right: T, first: bool) -> T {
    if first { left } else { right }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let base = vec![value, add(value, 1)];
        let mut total = 0;
        {
            let mut walk = |depth| down(depth, &base, &mut total);
            walk(3);
        }
        let mapped: Vec<_> = [1, 2].into_iter().map(|item| add(item, base[1])).collect();
        output.push(NumericArray(vec![
            factorial(value),
            total,
            mapped[0],
            mapped[1],
            make(value)(2),
            choose(value, 0, true),
            length(&choose("ab".to_owned(), "c".to_owned(), false)),
        ]));
    }
    Array(output)
}
fn main() {}
