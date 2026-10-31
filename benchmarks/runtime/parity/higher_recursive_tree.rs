// Independent standard Rust higher-order ownership and retained results.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
// Nominal observation fields correspond to code, text and children.
#[derive(Clone)]
struct Record0 {
    field0: i64,
    field1: String,
    field2: Vec<Record0>,
}
fn duplicate(value: &Record0) -> Record0 {
    value.clone()
}
fn copying<F: FnMut(&Record0) -> Record0>(f: &mut F, value: &Record0) -> Record0 {
    f(value)
}

fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Record0> {
    let consumer = copying;
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = Record0 {
            field0: add(value1, i),
            field1: String::Owned(value2.to_string()),
            field2: vec![
                Record0 {
                    field0: i,
                    field1: "a\0b".into(),
                    field2: vec![],
                },
                Record0 {
                    field0: add(i, 1),
                    field1: "😀".into(),
                    field2: vec![Record0 {
                        field0: value1,
                        field1: "leaf".into(),
                        field2: vec![],
                    }],
                },
            ],
        };
        output.push(consumer(&mut duplicate, &value));
    }
    output
}
fn main() {}
