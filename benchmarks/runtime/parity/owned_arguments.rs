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
fn read_twice<F: FnMut(&User) -> i64>(mut action: F, user: &User) -> i64 {
    add(action(user), action(user))
}
fn apply<F: FnMut(&mut User)>(mut action: F, user: &mut User) {
    action(user)
}
fn take_once<F: FnOnce(User) -> User>(action: F, user: User) -> User {
    action(user)
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = value1;
    for i in 0..value0 {
        let mut user = User {
            name: " Pat ".into(),
            value: i,
        };
        let read = |item: &User| item.value;
        sum = add(sum, read_twice(read, &user));
        let mutate = |item: &mut User| item.value = add(item.value, value1);
        apply(mutate, &mut user);
        sum = add(sum, user.value);
        let take = |item: User| item;
        let saved = take_once(take, user);
        sum = add(sum, saved.value);
    }
    sum
}
fn main() {}
