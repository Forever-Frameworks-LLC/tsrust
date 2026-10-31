// Independent standard Rust declared functions, callbacks and owned values.
type String = std::borrow::Cow<'static, str>;
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn digest(state: i64, value: i64) -> i64 {
    add(mul(state, 131), value) % 1_000_000_007
}
fn count(i: i64, seed: i64) -> usize {
    [0, 1, 4, 5][usize::try_from(add(seed, i) % 4).unwrap()]
}
// Observation fields represent code, text and children.
struct Record0 {
    field0: i64,
    field1: String,
    field2: Vec<Record0>,
}
fn take(root: Record0) -> Record0 {
    root
}

fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Record0> {
    let mut output = Vec::new();
    let action = take;
    for i in 0..value0 {
        let root = Record0 {
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
                    field1: "e\u{301}😀".into(),
                    field2: vec![Record0 {
                        field0: value1,
                        field1: "leaf".into(),
                        field2: vec![],
                    }],
                },
            ],
        };
        let moved = action(root);
        output.push(moved);
    }
    output
}
fn main() {}
