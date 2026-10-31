// Independent standard Rust language reference; preserve values and owned data operations.
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
fn owned(text: impl Into<Text>) -> String {
    String::Owned(text.into())
}
struct Named {
    name: Text,
}
struct Player {
    name: Text,
    x: i64,
    y: Option<i64>,
    score: i64,
}
fn describe(player: &Player) -> Text {
    format!(
        "{}:{}:{}",
        player.name,
        add(player.x, player.y.unwrap_or(0)),
        player.score
    )
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let k = add(i, value1);
        let plain = Player {
            name: value2.as_ref().to_owned(),
            x: k,
            y: None,
            score: k % 5,
        };
        let placed = Player {
            name: "p".to_owned(),
            x: k,
            y: Some(k % 3),
            score: 1,
        };
        let named = Named {
            name: k.to_string(),
        };
        output.push(Array(vec![
            owned(describe(&plain)),
            owned(describe(&placed)),
            owned(named.name),
            owned(add(plain.score, placed.score).to_string()),
            owned(placed.y.unwrap_or(-1).to_string()),
        ]));
    }
    output
}
fn main() {}
