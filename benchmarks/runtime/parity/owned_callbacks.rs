// Independent standard Rust callback contracts and owned callable storage.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn counter(start: i64) -> impl FnMut() -> i64 {
    let mut value = start;
    move || {
        value = add(value, 1);
        value
    }
}
struct User {
    name: String,
    value: i64,
}
fn identity<F: FnMut() -> i64>(action: F) -> F {
    let moved = action;
    moved
}
fn retain<F: FnMut() -> i64>(mut action: F) -> impl FnMut() -> i64 {
    move || action()
}
fn consume<F: FnOnce(User) -> User>(action: F, user: User) -> User {
    action(user)
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = value1;
    for i in 0..value0 {
        let first = counter(i);
        let moved = identity(first);
        let mut wrapped = retain(moved);
        sum = add(add(sum, wrapped()), wrapped());
        let take = |user: User| user;
        let user = consume(
            take,
            User {
                name: " Pat ".into(),
                value: value1,
            },
        );
        sum = add(sum, user.value);
    }
    sum
}
fn main() {}
