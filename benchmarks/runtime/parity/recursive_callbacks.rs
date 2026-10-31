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
fn visit<F: FnMut(i64) -> i64>(action: &mut F, depth: i64) -> i64 {
    if depth == 0 {
        action(depth)
    } else {
        add(action(depth), visit(action, sub(depth, 1)))
    }
}
fn countdown<F: FnMut() -> i64>(mut action: F, depth: i64) -> i64 {
    if depth == 0 {
        action()
    } else {
        let value = action();
        let moved = action;
        add(value, countdown(moved, sub(depth, 1)))
    }
}
fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = 0;
    for i in 0..value0 {
        let mut action = |value| add(value, value1);
        sum = add(add(sum, visit(&mut action, 8)), countdown(counter(i), 4));
    }
    sum
}
fn main() {}
