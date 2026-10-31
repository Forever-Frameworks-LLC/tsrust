// Independent standard Rust nested functions and closures; checked integer arithmetic.
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
fn length(s: &str) -> i64 {
    i64::try_from(s.chars().count()).unwrap()
}

fn next<'a>(total: &'a mut i64, hits: &'a mut i64) -> impl FnMut(Option<i64>) -> i64 + 'a {
    move |step| {
        let step = step.unwrap_or_else(|| {
            *hits = add(*hits, 1);
            2
        });
        *total = add(*total, step);
        *total
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let mut hits = 0;
        let mut total = value;
        let (first, second, third) = {
            let mut action = next(&mut total, &mut hits);
            (action(None), action(Some(3)), action(None))
        };
        let maybe = |delta: Option<i64>| add(value, delta.unwrap_or(4));
        let saved = std::string::String::from("owner");
        let once = move |text: Option<std::string::String>| text.unwrap_or(saved);
        let text = once(None);
        output.push(NumericArray(vec![
            first,
            second,
            third,
            hits,
            maybe(None),
            maybe(Some(1)),
            length(&text),
            length(value2),
        ]));
    }
    Array(output)
}
fn main() {}
