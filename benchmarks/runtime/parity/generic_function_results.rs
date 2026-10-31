// Independent standard Rust generics, explicit mutable environments and owned values.
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
fn length(s: &str) -> i64 {
    i64::try_from(s.chars().count()).unwrap()
}

fn empty<T>() -> Vec<T> {
    Vec::new()
}
struct Fresh {
    calls: i64,
}
impl Fresh {
    fn fresh<T>(&mut self) -> Vec<T> {
        self.calls = add(self.calls, 1);
        Vec::new()
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let base = add(add(i % 5, value1 % 5), 1);
        let mut numbers = empty();
        let mut strings = empty();
        numbers.push(base);
        strings.push("value".to_owned());
        let mut action = Fresh { calls: 0 };
        let mut owned_numbers = action.fresh();
        let mut owned_strings = action.fresh();
        owned_numbers.push(add(base, 1));
        owned_strings.push("owner".to_owned());
        output.push(NumericArray(vec![
            numbers[0],
            length(&strings[0]),
            owned_numbers[0],
            length(&owned_strings[0]),
            action.calls,
            length(value2),
        ]));
    }
    Array(output)
}
fn main() {}
