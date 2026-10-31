// Independent standard Rust owned results and observable control-flow completion.
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
fn mul(a: i64, b: i64) -> i64 {
    a.checked_mul(b).unwrap()
}
fn fallback(trace: &mut [i64], value: String) -> String {
    trace[0] = add(trace[0], 1);
    value
}
fn selected(mode: i64) -> Option<String> {
    match mode {
        0 => None,
        1 => Some("".into()),
        2 => Some("é".into()),
        _ => Some("😀".into()),
    }
}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let mut output = Vec::new();
    let mut trace = vec![0];
    for i in 0..value0 {
        let mode = add(i, value1) % 4;
        let left = selected(mode);
        let disjunction = left
            .filter(|text| !text.is_empty())
            .unwrap_or_else(|| fallback(&mut trace, "Guest".into()));
        output.push(disjunction);
        output.push(String::Owned(trace[0].to_string()));
        let next = selected(mode);
        let conjunction = match next {
            None => "undefined".into(),
            Some(text) if text.is_empty() => text,
            Some(_) => fallback(&mut trace, "YES".into()),
        };
        output.push(conjunction);
        output.push(String::Owned(trace[0].to_string()));
    }
    Array(output)
}
fn main() {}
