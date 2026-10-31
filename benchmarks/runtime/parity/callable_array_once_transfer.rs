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
fn action(base: i64, offset: i64) -> impl Fn() -> i64 {
    move || add(base, offset)
}
fn consume<F: FnOnce() -> i64>(action: F) -> i64 {
    action()
}

fn function0(value0: i64, value1: i64) -> i64 {
    let mut checksum = value1;
    for i in 0..value0 {
        let base = add(i, value1);
        let mut actions = vec![action(base, 0)];
        actions.push(action(base, 1));
        actions.insert(0, action(base, 2));
        actions[1] = action(base, 3);
        let first = actions.remove(0);
        let mut removed: Vec<_> = actions.splice(0..1, [action(base, 4)]).collect();
        let second = removed.remove(0);
        let third = actions.remove(0);
        let fourth = actions.remove(0);
        checksum = add(
            add(
                add(add(checksum, consume(first)), consume(second)),
                consume(third),
            ),
            consume(fourth),
        );
        checksum = add(checksum, i64::try_from(actions.len()).unwrap());
    }
    checksum
}
fn main() {}
