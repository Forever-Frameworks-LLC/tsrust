// Independent standard Rust higher-order ownership and retained results.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn change(values: &mut Vec<i64>) -> Vec<i64> {
    values[0] = add(values[0], 1);
    values.clone()
}
fn drive<F: FnMut(&mut Vec<i64>) -> Vec<i64>>(f: &mut F, values: &mut Vec<i64>) -> Vec<i64> {
    f(values)
}
fn function0(value0: i64, value1: i64) -> Vec<Vec<i64>> {
    let consumer = drive;
    let mut output = Vec::new();
    for i in 0..value0 {
        let mut values = vec![add(value1, i), 2, 3];
        let result = consumer(&mut change, &mut values);
        output.push(result);
        output.push(values.clone());
    }
    output
}
fn main() {}
