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
#[derive(Clone, Copy)]
enum Amount {
    Undefined,
    Null,
    Value(i64),
}
impl Amount {
    fn default(self, value: i64) -> Self {
        match self {
            Self::Undefined => Self::Value(value),
            other => other,
        }
    }
    fn render(self) -> Text {
        match self {
            Self::Undefined => "undefined".to_owned(),
            Self::Null => "null".to_owned(),
            Self::Value(v) => v.to_string(),
        }
    }
}
struct Row {
    label: Option<Text>,
    amount: Amount,
}
fn fallback(trace: &mut Vec<i64>, value: Text) -> Text {
    trace.push(1);
    value
}
fn describe(row: &Row) -> Text {
    format!(
        "{}:{}",
        row.label.as_deref().unwrap_or("none"),
        row.amount.default(7).render()
    )
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    let mut trace = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let mode = add(i, value1) % 3;
        let row = Row {
            label: if mode == 0 {
                None
            } else {
                Some(value2.repeat(usize::try_from(base % 3 + 1).unwrap()))
            },
            amount: match mode {
                1 => Amount::Null,
                2 => Amount::Undefined,
                _ => Amount::Value(base),
            },
        };
        let values = vec![if add(i, value1) % 2 == 0 {
            None
        } else {
            Some(value2.repeat(1))
        }];
        let text: std::borrow::Cow<'_, str> = match values.first().and_then(Option::as_ref) {
            Some(value) => std::borrow::Cow::Borrowed(value),
            None => std::borrow::Cow::Owned(fallback(&mut trace, value2.repeat(1))),
        };
        let missing = values.get(1).and_then(Option::as_deref).unwrap_or("tail");
        let amount = row.amount.default(8);
        output.push(Array(vec![
            owned(text.as_ref().to_owned()),
            owned(missing.to_owned()),
            owned(describe(&row)),
            owned(amount.render()),
            owned(trace.len().to_string()),
        ]));
    }
    output
}
fn main() {}
