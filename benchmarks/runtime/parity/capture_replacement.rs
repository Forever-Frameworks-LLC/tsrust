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
struct User {
    name: String,
    value: i64,
}
fn make(user: User, step: i64) -> impl FnMut() -> i64 {
    let mut saved = user;
    move || {
        let previous = saved.value;
        saved = User {
            name: "Sam".into(),
            value: add(previous, step),
        };
        add(add(previous, saved.value), 1)
    }
}
fn once_value(user: User) -> impl FnOnce() -> i64 {
    move || {
        let moved = user;
        moved.value
    }
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = value1;
    for i in 0..value0 {
        let mut action = make(
            User {
                name: "Pat".into(),
                value: i,
            },
            value1,
        );
        sum = add(add(sum, action()), action());
        let take = once_value(User {
            name: "Pat".into(),
            value: value1,
        });
        sum = add(sum, take());
    }
    sum
}
fn main() {}
