// Independent Rust environment reads: missing and present-empty remain distinct.
type String = std::borrow::Cow<'static, str>;
struct Array<T>(Vec<T>);
impl<T> Array<T> {
    fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}
fn read(name: &str) -> Result<Option<std::string::String>, std::env::VarError> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error),
    }
}
fn function0(value0: i64, value1: i64) -> Array<String> {
    let names = [
        "TSRUST_BENCH_ENV_TEXT",
        "TSRUST_BENCH_ENV_EMPTY",
        "TSRUST_BENCH_ENV_MISSING",
    ];
    let mut values = Vec::new();
    for i in 0..value0 {
        let name = names[usize::try_from(i.checked_add(value1).unwrap() % 3).unwrap()];
        values.push(match read(name).expect("Invalid environment text") {
            Some(value) => String::Owned(value),
            None => String::Borrowed("<missing>"),
        });
    }
    Array(values)
}
fn main() {}
