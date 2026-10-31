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

struct Counter {
    calls: i64,
}
impl Counter {
    fn visit<T>(&mut self, _value: T) -> i64 {
        self.calls = add(self.calls, 1);
        self.calls
    }
}
struct Selector {
    calls: i64,
}
impl Selector {
    fn select<T>(&mut self, value: T) -> T {
        self.calls = add(self.calls, 1);
        value
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let base = add(add(i % 5, value1 % 5), 1);
        let mut first = Counter { calls: base };
        let mut second = Counter {
            calls: add(base, 10),
        };
        let first_count = first.visit(base);
        let second_count = second.visit("value".to_owned());
        let shared_count = first.visit(vec![base, add(base, 1)]);
        let mut local = Selector { calls: 0 };
        let number_result = local.select(base);
        let string_length = length(&local.select("value".to_owned()));
        let result = local.select(vec![add(base, 3), add(base, 4)]);
        let array_first = result[0];
        let final_count = second.visit(base);
        output.push(NumericArray(vec![
            first_count,
            second_count,
            shared_count,
            number_result,
            string_length,
            array_first,
            local.calls,
            final_count,
            length(value2),
        ]));
    }
    Array(output)
}
fn main() {}
