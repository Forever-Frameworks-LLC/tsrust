// Independent Rust reference using rand's established thread-local generator.
// Algorithms differ across engines; this workload observes range/difference checks.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> { fn iter(&self) -> std::slice::Iter<'_, T> { self.0.iter() } }
struct NumericArray<T>(Vec<T>);
impl<T> NumericArray<T> { fn iter(&self) -> std::slice::Iter<'_, T> { self.0.iter() } }
fn draw() -> f64 { rand::random::<f64>() }
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<bool>> {
    let _ = value2;
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let sides = (i.checked_mul(13).unwrap().checked_add(value1).unwrap() % 97 + 1) as f64;
        let first = draw();
        let second = draw();
        let roll = (first * sides).floor();
        output.push(NumericArray(vec![first >= 0.0 && first < 1.0, second >= 0.0 && second < 1.0, first != second, roll >= 0.0 && roll < sides]));
    }
    Array(output)
}
fn main() {}
