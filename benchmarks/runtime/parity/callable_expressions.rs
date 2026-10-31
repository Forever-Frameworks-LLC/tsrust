// Independent standard Rust optional callbacks and function expressions.
type String = std::borrow::Cow<'static, str>;
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
fn sub(a: i64, b: i64) -> i64 {
    a.checked_sub(b).unwrap()
}
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn length(text: &str) -> i64 {
    i64::try_from(text.chars().count()).unwrap()
}
fn make(base: i64) -> impl Fn(i64) -> i64 {
    move |value| add(base, value)
}
fn chain(a: i64) -> impl Fn(i64) -> Box<dyn Fn(i64) -> i64> {
    move |b| Box::new(move |c| add(add(a, b), c))
}
fn factory(trace: &mut Vec<i64>, base: i64) -> impl Fn(i64) -> i64 + use<> {
    trace.push(1);
    move |value| add(value, base)
}
fn mark(trace: &mut Vec<i64>, value: i64) -> i64 {
    trace.push(2);
    value
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(mul(i, 13), value1) % 97;
        let direct = make(value)(3);
        let nested = chain(value)(2)(3);
        let inline = (|v| mul(v, 2))(value);
        let named = (|v| add(v, 4))(value);
        let args = vec![value, 2];
        let spread = (|a, b| add(a, b))(args[0], args[1]);
        let omitted = (|value: Option<i64>| value.unwrap_or(7))(None);
        let mut trace = Vec::new();
        let action = factory(&mut trace, value);
        let argument = mark(&mut trace, 5);
        let ordered = action(argument);
        let mut row: Vec<_> = [direct, nested, inline, named, spread, omitted, ordered]
            .into_iter()
            .map(|v| String::Owned(v.to_string()))
            .collect();
        row.push(String::Owned(
            trace
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(","),
        ));
        row.push(String::Owned(value2.to_string() + &value.to_string()));
        output.push(Array(row));
    }
    output
}
fn main() {}
