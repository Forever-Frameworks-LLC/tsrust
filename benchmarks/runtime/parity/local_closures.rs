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

fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = value1;
    for i in 0..value0 {
        let mut state = i;
        let mut outer = || {
            let mut inner = || {
                state = add(state, 2);
                state
            };
            add(inner(), value1)
        };
        sum = add(sum, outer());
    }
    sum
}
fn main() {}
