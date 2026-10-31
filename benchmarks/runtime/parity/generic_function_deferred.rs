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

fn identity<T>(value: T) -> T {
    value
}
fn first<T>(mut values: Vec<T>) -> T {
    values.remove(0)
}
fn size<T>(values: &[T]) -> i64 {
    i64::try_from(values.len()).unwrap()
}
fn offset<T>(value: T, _index: Option<i64>) -> T {
    value
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let base = add(add(i % 5, value1 % 5), 1);
        let number_result = identity(base);
        let string_length = length(&identity("owner".to_owned()));
        let result = identity(vec![add(base, 1), add(base, 2)]);
        let array_first = result[0];
        let head_result = first(vec![add(base, 2), add(base, 3)]);
        let selected = offset(add(base, 4), None);
        let values = vec![base, add(base, 1)];
        let remaining = size(&values);
        output.push(NumericArray(vec![
            number_result,
            string_length,
            array_first,
            head_result,
            selected,
            remaining,
            length(value2),
        ]));
    }
    Array(output)
}
fn main() {}
