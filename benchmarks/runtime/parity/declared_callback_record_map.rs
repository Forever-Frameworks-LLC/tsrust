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
// Observation fields represent code, text and items.
#[derive(Clone)]
struct Record0 {
    field0: i64,
    field1: String,
    field2: Vec<i64>,
}
fn change(row: &Record0, index: usize) -> Record0 {
    Record0 {
        field0: add(add(row.field0, i64::try_from(index).unwrap()), 1),
        field1: row.field1.clone(),
        field2: row.field2.clone(),
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<Record0>> {
    let base = [
        Record0 {
            field0: value1,
            field1: String::Owned(value2.to_string()),
            field2: vec![value1, 1],
        },
        Record0 {
            field0: 2,
            field1: "x😀".into(),
            field2: vec![],
        },
        Record0 {
            field0: 3,
            field1: "a\0b".into(),
            field2: vec![2, 3],
        },
        Record0 {
            field0: 4,
            field1: "e\u{301}".into(),
            field2: vec![4],
        },
        Record0 {
            field0: 5,
            field1: "xxé".into(),
            field2: vec![5, 6],
        },
    ];
    let mut output = Vec::new();
    for i in 0..value0 {
        let values = &base[..count(i, value1)];
        output.push(
            values
                .iter()
                .enumerate()
                .map(|(index, row)| change(row, index))
                .collect(),
        );
    }
    output
}
fn main() {}
