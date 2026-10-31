// Independent standard Rust higher-order ownership and retained results.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
// Nominal observation fields correspond to code, text and items/children.
#[derive(Clone)]
struct Record0 {
    field0: i64,
    field1: String,
    field2: Vec<i64>,
}
fn copy(row: &Record0) -> Record0 {
    row.clone()
}
fn drive<F: FnMut(&Record0) -> Record0>(f: &mut F, row: &Record0) -> Record0 {
    f(row)
}

fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Record0> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let row = Record0 {
            field0: add(value1, i),
            field1: String::Owned(value2.to_string()),
            field2: vec![i, value1],
        };
        output.push(drive(&mut copy, &row));
    }
    output
}
fn main() {}
