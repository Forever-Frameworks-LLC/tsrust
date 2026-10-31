// Independent Rust module state: the driver owns one counter across all calls.
fn increment(completed: &mut i64) {
    *completed = completed.checked_add(1).unwrap();
}
fn function0(value0: i64, value1: i64, value2: &mut i64) -> i64 {
    let before = *value2;
    let mut checksum = value1;
    for _ in 0..value0 {
        increment(value2);
        checksum = checksum.checked_add(3).unwrap();
    }
    assert_eq!(
        value2.checked_sub(before).unwrap(),
        value0,
        "Module state was not retained"
    );
    checksum
}
fn main() {}
