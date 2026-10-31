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
#[derive(Clone)]
enum Item {
    Integer(i64),
    Text(Text),
    Null,
}
impl Item {
    fn render(&self) -> Text {
        match self {
            Self::Integer(v) => v.to_string(),
            Self::Text(v) => v.clone(),
            Self::Null => "null".to_owned(),
        }
    }
}
fn joined(values: &[Item]) -> Text {
    use std::fmt::Write;
    let mut text = Text::new();
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            text.push('|');
        }
        match value {
            Item::Integer(v) => write!(&mut text, "{v}").unwrap(),
            Item::Text(v) => text.push_str(v),
            Item::Null => {}
        }
    }
    text
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let mode = add(i, value1) % 3;
        let mut values = vec![
            Item::Integer(base),
            Item::Text(value2.repeat(usize::try_from(mode).unwrap())),
            Item::Null,
        ];
        for extra in 1..=16 {
            values.push(Item::Integer(add(base, extra)));
        }
        if mode == 0 {
            values.push(Item::Text("tail".to_owned()));
        } else {
            values.push(Item::Integer(add(base, 1)));
        }
        let mut sum = 0;
        let mut text = Text::new();
        for value in &values {
            match value {
                Item::Integer(v) => sum = add(sum, *v),
                Item::Text(v) => text.push_str(v),
                Item::Null => {}
            }
        }
        let includes = values.iter().any(|v| matches!(v, Item::Null));
        let position = values
            .iter()
            .position(|v| matches!(v,Item::Integer(n)if *n==add(base,1)))
            .map(|v| i64::try_from(v).unwrap())
            .unwrap_or(-1);
        let last = values.last().unwrap().render();
        let filtered: Vec<&Item> = values
            .iter()
            .filter(|v| matches!(v, Item::Text(_)))
            .collect();
        output.push(Array(vec![
            owned(sum.to_string()),
            owned(text),
            owned(joined(&values)),
            owned(includes.to_string()),
            owned(position.to_string()),
            owned(last),
            owned(filtered.len().to_string()),
        ]));
    }
    output
}
fn main() {}
