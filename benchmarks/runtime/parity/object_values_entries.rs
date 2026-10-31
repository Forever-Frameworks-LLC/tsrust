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
#[derive(Clone, Copy)]
enum Key {
    Alpha,
    Beta,
    Gamma,
}
impl Key {
    fn name(self) -> &'static str {
        match self {
            Self::Alpha => "alpha",
            Self::Beta => "beta",
            Self::Gamma => "gamma",
        }
    }
}
#[derive(Clone)]
struct Row {
    alpha: i64,
    beta: Text,
    gamma: bool,
    order: [Key; 3],
}
impl Row {
    fn new(base: i64, input: &str, even: bool) -> Self {
        Self {
            alpha: base,
            beta: input.to_owned(),
            gamma: even,
            order: if even {
                [Key::Gamma, Key::Alpha, Key::Beta]
            } else {
                [Key::Beta, Key::Gamma, Key::Alpha]
            },
        }
    }
    fn keys(&self) -> Vec<&'static str> {
        self.order.iter().map(|key| key.name()).collect()
    }
}
#[derive(Clone, Copy)]
enum Field<'a> {
    Integer(i64),
    Text(&'a str),
    Boolean(bool),
}
impl Field<'_> {
    fn render(self) -> Text {
        match self {
            Self::Integer(v) => v.to_string(),
            Self::Text(v) => v.to_owned(),
            Self::Boolean(v) => v.to_string(),
        }
    }
}
impl Row {
    fn value(&self, key: Key) -> Field<'_> {
        match key {
            Key::Alpha => Field::Integer(self.alpha),
            Key::Beta => Field::Text(&self.beta),
            Key::Gamma => Field::Boolean(self.gamma),
        }
    }
    fn values(&self) -> Vec<Field<'_>> {
        self.order.iter().map(|&key| self.value(key)).collect()
    }
    fn entries(&self) -> Vec<(&'static str, Field<'_>)> {
        self.order
            .iter()
            .map(|&key| (key.name(), self.value(key)))
            .collect()
    }
}
struct Sides {
    w: i64,
    h: i64,
}
impl Sides {
    fn values(&self) -> Vec<i64> {
        vec![self.h, self.w]
    }
    fn entries(&self) -> Vec<(&'static str, i64)> {
        vec![("h", self.h), ("w", self.w)]
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let row = Row::new(base, value2, add(i, value1) % 2 == 0);
        let values = row.values();
        let mut text = Text::new();
        let mut sum = 0;
        for &value in &values {
            match value {
                Field::Text(v) => text.push_str(v),
                Field::Integer(v) => sum = add(sum, v),
                Field::Boolean(v) => text.push_str(if v { "T" } else { "F" }),
            }
        }
        let sides = Sides { h: base, w: i % 5 };
        let lengths = sides.values();
        let mut names = Text::new();
        for (key, value) in row.entries() {
            names.push_str(key);
            names.push('=');
            names.push_str(&value.render());
            names.push(';');
        }
        let joined = values
            .iter()
            .copied()
            .map(Field::render)
            .collect::<Vec<_>>()
            .join("|");
        output.push(Array(vec![
            owned(text),
            owned(sum.to_string()),
            owned(joined),
            owned(lengths[0].checked_sub(lengths[1]).unwrap().to_string()),
            owned(names),
            owned(sides.entries().len().to_string()),
        ]));
    }
    output
}
fn main() {}
