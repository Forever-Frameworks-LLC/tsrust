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
    i64::try_from(s.len()).unwrap()
}

struct Context {
    calls: i64,
}
impl Context {
    fn select<T>(&mut self, value: T) -> T {
        self.calls = add(self.calls, 1);
        value
    }
    fn head<T>(&mut self, values: &mut Vec<T>) -> T {
        self.calls = add(self.calls, 1);
        values.remove(0)
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let base = add(add(i % 5, value1 % 5), 1);
        let mut context = Context { calls: 0 };
        let number_result = context.select(base);
        let string_length = length(&context.select("owner".to_owned()));
        let result = context.select(vec![add(base, 1), add(base, 2)]);
        let array_first = result[0];
        let mut values = vec![add(base, 3), add(base, 4)];
        let head_result = context.head(&mut values);
        let remaining = i64::try_from(values.len()).unwrap();
        output.push(NumericArray(vec![
            number_result,
            string_length,
            array_first,
            head_result,
            remaining,
            context.calls,
            length(value2),
        ]));
    }
    Array(output)
}
fn main() {}
