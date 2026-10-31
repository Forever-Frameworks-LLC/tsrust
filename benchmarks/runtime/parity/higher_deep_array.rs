// Independent standard Rust higher-order ownership and retained results.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn duplicate(values: &Vec<Vec<i64>>) -> Vec<Vec<i64>> {
    values.clone()
}
fn copying<F: FnMut(&Vec<Vec<i64>>) -> Vec<Vec<i64>>>(
    f: &mut F,
    values: &Vec<Vec<i64>>,
) -> Vec<Vec<i64>> {
    f(values)
}
fn function0(value0: i64, value1: i64) -> Vec<Vec<Vec<i64>>> {
    let consumer = copying;
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = vec![vec![add(value1, i), 1], vec![], vec![i, 2, 3]];
        output.push(consumer(&mut duplicate, &value));
    }
    output
}
fn main() {}
