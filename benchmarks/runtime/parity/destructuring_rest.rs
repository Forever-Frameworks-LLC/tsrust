// Independent standard Rust reference; retain pattern/default/control operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> {
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
#[derive(Clone, Copy, PartialEq)]
enum Key {
    Before,
    Active,
    Name,
    Count,
    After,
}
impl Key {
    fn name(self) -> &'static str {
        match self {
            Self::Before => "before",
            Self::Active => "active",
            Self::Name => "name",
            Self::Count => "count",
            Self::After => "after",
        }
    }
}
struct Row {
    name: Text,
    count: i64,
    active: bool,
    order: [Key; 3],
}
struct Fields {
    count: i64,
    active: bool,
    order: Vec<Key>,
}
struct Merged {
    before: i64,
    name: Text,
    count: i64,
    active: bool,
    after: i64,
    order: Vec<Key>,
}
fn merge(row: &Row, before: i64, after: i64) -> Merged {
    let mut order = vec![Key::Before];
    order.extend(row.order);
    order.push(Key::After);
    Merged {
        before,
        name: row.name.clone(),
        count: row.count,
        active: row.active,
        after,
        order,
    }
}
fn keys(order: &[Key]) -> Text {
    order
        .iter()
        .map(|key| key.name())
        .collect::<Vec<_>>()
        .join(",")
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let words = vec![value2.repeat(1), base.to_string(), value2.repeat(2)];
        let (first, rest) = words.split_first().unwrap();
        let mut rest = rest.to_vec();
        rest.push("tail".to_owned());
        let pair = (base, value2.repeat(1));
        let (_, tail) = pair;
        let tuple_tail = vec![tail];
        let row = Row {
            active: base % 2 == 0,
            name: value2.repeat(1),
            count: base,
            order: [Key::Active, Key::Name, Key::Count],
        };
        let mut merged = merge(&row, base, value1);
        merged.count = add(base, 1);
        let Row {
            name: label,
            count,
            active,
            order,
        } = row;
        let fields = Fields {
            count,
            active,
            order: order.into_iter().filter(|key| *key != Key::Name).collect(),
        };
        output.push(Array(vec![
            owned(first.clone()),
            owned(rest.join("|")),
            owned(tuple_tail[0].clone()),
            owned(label),
            owned(fields.count.to_string()),
            owned(keys(&fields.order)),
            owned(merged.name.clone()),
            owned(merged.count.to_string()),
            owned(keys(&merged.order)),
            owned(words.len().to_string()),
            owned(rest.len().to_string()),
        ]));
    }
    output
}
fn main() {}
