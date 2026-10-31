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
enum Scalar {
    Integer(i64),
    Text(Text),
    Boolean(bool),
    Null,
}
impl Scalar {
    fn render(&self) -> Text {
        match self {
            Self::Integer(v) => v.to_string(),
            Self::Text(v) => v.clone(),
            Self::Boolean(v) => v.to_string(),
            Self::Null => "null".to_owned(),
        }
    }
    fn kind(&self) -> &'static str {
        match self {
            Self::Integer(_) => "number",
            Self::Text(_) => "string",
            Self::Boolean(_) => "boolean",
            Self::Null => "object",
        }
    }
}
#[derive(Clone)]
enum Owner {
    Text(Text),
    Integers(Vec<i64>),
}
impl Owner {
    fn render(&self) -> Text {
        match self {
            Self::Text(v) => v.clone(),
            Self::Integers(v) => v
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(","),
        }
    }
}
#[derive(Clone)]
struct Row {
    value: Scalar,
    owner: Owner,
}
fn describe(value: &Scalar) -> Text {
    match value {
        Scalar::Text(v) => v.to_uppercase(),
        Scalar::Null => "missing".to_owned(),
        other => other.render(),
    }
}
fn copied_text(value: &mut Owner) -> Text {
    match value.clone() {
        Owner::Text(text) => text,
        copy => copy.render(),
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let mode = add(i, value1) % 4;
        let mut source = Row {
            owner: if mode % 2 == 0 {
                Owner::Text(value2.as_ref().to_owned())
            } else {
                Owner::Integers(vec![base, add(base, 1)])
            },
            value: match mode {
                0 => Scalar::Integer(base),
                1 => Scalar::Text(value2.repeat(usize::try_from(base % 3 + 1).unwrap())),
                2 => Scalar::Boolean(false),
                _ => Scalar::Null,
            },
        };
        let mut copied = source.clone();
        copied.owner = Owner::Integers(vec![add(base, 5)]);
        output.push(Array(vec![
            owned(source.value.render()),
            String::Borrowed(source.value.kind()),
            owned(source.owner.render()),
            owned(copied.owner.render()),
            owned(describe(&source.value)),
            owned(copied_text(&mut source.owner)),
        ]));
    }
    output
}
fn main() {}
