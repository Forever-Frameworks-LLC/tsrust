// Independently authored Rust scoped callback records and generic environments.
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
    i64::try_from(text.len()).unwrap()
}
#[derive(Debug)]
enum Failure {
    Text(std::string::String),
    Code(i64),
}
struct Holder<'a> {
    run: Box<dyn Fn() -> Result<i64, Failure> + 'a>,
}
fn call(holder: &Holder<'_>) -> Result<i64, Failure> {
    (holder.run)()
}
fn guarded(holder: &Holder<'_>) -> i64 {
    match (holder.run)() {
        Ok(value) => value,
        Err(Failure::Text(text)) => length(&text),
        Err(Failure::Code(error)) => add(error, 1000),
    }
}
fn checked(holder: &Holder<'_>, limit: i64) -> Result<i64, Failure> {
    let value = (holder.run)()?;
    if value > limit {
        Err(Failure::Text("big".to_owned()))
    } else {
        Ok(value)
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let base = vec![value];
        let plain = Holder {
            run: Box::new(|| Ok(base[0])),
        };
        let text = Holder {
            run: Box::new(|| Err(Failure::Text("x".repeat(usize::try_from(value).unwrap())))),
        };
        let coded = Holder {
            run: Box::new(|| Err(Failure::Code(value))),
        };
        let outer = match checked(&plain, 2) {
            Ok(_) => 1,
            Err(Failure::Text(error)) => length(&error),
            Err(_) => 0,
        };
        output.push(NumericArray(vec![
            call(&plain).unwrap(),
            guarded(&plain),
            guarded(&text),
            guarded(&coded),
            outer,
        ]));
    }
    Array(output)
}
fn main() {}
