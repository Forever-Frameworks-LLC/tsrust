// Independent standard Rust scalar control and operators, with checked arithmetic.
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn digest(state: i64, value: i64) -> i64 {
    add(mul(state, 131), value) % 1_000_000_007
}

fn function0(value0: i64, value1: i64) -> i64 {
    let mut sum = value1;
    for i in 0..value0 {
        let bound = add(i, value1) % 10;
        let stop = add(mul(i, 3), value1) % 7;
        let mut j = 0;
        loop {
            let current = j;
            j = add(j, 1);
            sum = digest(sum, add(current, value1) % 97);
            if current == stop {
                break;
            }
            if add(current, value1) % 3 != 0 {
                sum = digest(sum, add(mul(current, 3), 1) % 97);
            }
            if j >= bound {
                break;
            }
        }
        sum = digest(sum, j);
    }
    sum
}
fn main() {}
