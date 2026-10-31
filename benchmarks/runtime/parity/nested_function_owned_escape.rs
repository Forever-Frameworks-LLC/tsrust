// Independent standard Rust nested functions and closures; checked integer arithmetic.
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
fn length(s: &str) -> i64 {
    i64::try_from(s.chars().count()).unwrap()
}

struct Item {
    value: i64,
}
fn independent() -> i64 {
    7
}
fn make(mut value: i64) -> impl FnMut() -> i64 {
    move || {
        value = add(value, 1);
        value
    }
}
fn recursive(base: Vec<i64>) -> impl Fn(i64) -> i64 {
    fn even(base: &[i64], n: i64) -> i64 {
        if n == 0 {
            base[0]
        } else {
            odd(base, sub(n, 1))
        }
    }
    fn odd(base: &[i64], n: i64) -> i64 {
        if n == 0 {
            add(base[0], 1)
        } else {
            even(base, sub(n, 1))
        }
    }
    move |n| even(&base, n)
}
fn owner(value: i64) -> impl FnOnce() -> Item {
    let item = Item { value };
    move || item
}
fn failure(value: i64) -> impl Fn() -> Result<i64, i64> {
    move || Err(value)
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    assert_eq!(independent(), 7);
    let _later = vec![1, 2];
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let mut actions = vec![make(value), make(add(value, 10))];
        let first = actions[0]();
        let second = actions[1]();
        let again = actions[0]();
        let recurse = recursive(vec![value]);
        let take = owner(add(value, 20));
        let item = take();
        let fail = failure(value);
        let caught = i64::from(fail().is_err());
        output.push(NumericArray(vec![
            first,
            second,
            again,
            recurse(4),
            recurse(3),
            item.value,
            caught,
            length(value2),
        ]));
    }
    Array(output)
}
fn main() {}
