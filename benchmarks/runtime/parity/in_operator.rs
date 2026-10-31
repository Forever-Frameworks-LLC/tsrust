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
struct Sample {
    name: Text,
    amount: i64,
}
impl Sample {
    fn keys(&self) -> [&'static str; 2] {
        ["name", "amount"]
    }
    fn contains(&self, key: &str) -> bool {
        self.keys().contains(&key)
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let sample = Sample {
            name: format!("{value2}{base}"),
            amount: base,
        };
        let values = vec![base, add(base, 1), add(base, 2)];
        let wanted = if base % 2 == 0 { "name" } else { "missing" };
        let present = sample.contains(wanted);
        let at = base % 5;
        let positioned = has_position(values.len(), &Property::Integer(at));
        let fractional = has_position(values.len(), &Property::Number(at as f64 + 0.5));
        let length = has_position(values.len(), &Property::Text("length"));
        let mut names = Text::new();
        for key in sample.keys() {
            names.push_str(key);
            names.push(',');
        }
        let mut indices = Text::new();
        for key in 0..values.len() {
            indices.push_str(&key.to_string());
        }
        output.push(Array(vec![
            owned(present.to_string()),
            owned(positioned.to_string()),
            owned(fractional.to_string()),
            owned(length.to_string()),
            owned(names),
            owned(indices),
            owned(sample.name),
            owned(sample.amount.to_string()),
        ]));
    }
    output
}
fn main() {}
