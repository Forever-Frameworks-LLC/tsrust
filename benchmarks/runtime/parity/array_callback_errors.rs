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
fn mapped(values: &[i64], stop: i64, input: &str) -> Result<Vec<i64>, std::string::String> {
    values
        .iter()
        .map(|value| {
            if *value == stop {
                Err(input.to_owned())
            } else {
                Ok(*value)
            }
        })
        .collect()
}
fn sorted(values: &[i64]) -> Result<Vec<i64>, &'static str> {
    if values.iter().any(|value| *value < 0) {
        return Err("negative");
    }
    let mut sorted = values.to_vec();
    sorted.sort();
    Ok(sorted)
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(i, value1);
        let values = vec![add(base, 2), base, add(base, 1)];
        let caught = mapped(&values, base, value2).err().unwrap_or_default();
        let finalized = 1;
        let sorted = sorted(&values).unwrap();
        output.push(Array(
            vec![
                caught,
                finalized.to_string(),
                values
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                sorted
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
            ]
            .into_iter()
            .map(String::Owned)
            .collect(),
        ));
    }
    output
}
fn main() {}
