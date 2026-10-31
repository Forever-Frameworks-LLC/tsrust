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
struct State {
    value: i64,
}
fn state_action(start: i64, step: i64) -> impl FnMut() -> i64 {
    let mut state = State { value: start };
    move || {
        state.value = add(state.value, step);
        state.value
    }
}

fn function0(value0: i64, value1: i64) -> i64 {
    let mut checksum = value1;
    for i in 0..value0 {
        let base = add(i, value1);
        let mut actions = vec![state_action(base, 1), state_action(base, 2)];
        let forward = actions.iter_mut().fold(0, |sum, action| add(sum, action()));
        let reverse = actions
            .iter_mut()
            .rev()
            .fold(0, |sum, action| add(mul(sum, 10), action()));
        checksum = add(add(checksum, forward), reverse);
        checksum = add(add(checksum, actions[0]()), actions[1]());
    }
    checksum
}
fn main() {}
