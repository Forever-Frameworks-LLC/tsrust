// Independently typed representation of the checked, compiled JSON import.
// Module data is prepared outside measured bodies; the loop reads every field/value.
struct Settings {
    base: i64,
    values: [i64; 3],
}
const SETTINGS: Settings = Settings {
    base: 5,
    values: [3, 11, 23],
};
fn function0(value0: i64, value1: i64) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        checksum = checksum.checked_add(SETTINGS.base).unwrap();
        for &value in &SETTINGS.values {
            checksum = checksum.checked_add(value).unwrap();
        }
    }
    checksum
}
fn main() {}
