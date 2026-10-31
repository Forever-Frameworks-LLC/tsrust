// Independent Rust records/unions reference; preserve storage, order and observed operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn owned(value: impl Into<Text>) -> String {
    String::Owned(value.into())
}
enum Property<'a> {
    Text(&'a str),
    Integer(i64),
    Number(f64),
}
fn index(property: &Property<'_>) -> Option<usize> {
    match property {
        Property::Integer(value) => usize::try_from(*value).ok(),
        Property::Number(value) => {
            if value.is_finite()
                && *value >= 0.0
                && value.fract() == 0.0
                && *value < (usize::MAX as f64)
            {
                Some(*value as usize)
            } else {
                None
            }
        }
        Property::Text(value) => {
            let parsed = value.parse::<usize>().ok()?;
            if parsed.to_string() == *value {
                Some(parsed)
            } else {
                None
            }
        }
    }
}
fn has_position(length: usize, property: &Property<'_>) -> bool {
    matches!(property, Property::Text("length"))
        || index(property).is_some_and(|index| index < length)
}
struct Row {
    label: Option<Text>,
    amount: i64,
}
impl Row {
    fn has_own(&self, key: &str) -> bool {
        matches!(key, "label" | "amount")
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let row = Row {
            label: None,
            amount: add(i, value1),
        };
        let values = vec![i, value1];
        let position = Property::Integer(add(i, value1) % 4);
        output.push(Array(vec![
            owned(row.has_own("label").to_string()),
            owned(row.has_own("missing").to_string()),
            owned(row.has_own("toString").to_string()),
            owned(has_position(values.len(), &position).to_string()),
            owned(has_position(values.len(), &Property::Text("length")).to_string()),
            owned(has_position(values.len(), &Property::Text("01")).to_string()),
            owned(has_position(value2.encode_utf16().count(), &position).to_string()),
            owned(
                has_position(value2.encode_utf16().count(), &Property::Text("length")).to_string(),
            ),
            owned(has_position(values.len(), &Property::Number(-0.0)).to_string()),
        ]));
    }
    output
}
fn main() {}
