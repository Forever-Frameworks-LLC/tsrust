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
struct User {
    value: i64,
}
fn make(value: i64) -> Result<User, i64> {
    if value % 3 == 0 {
        Err(add(value, 5))
    } else {
        Ok(User { value })
    }
}
fn inspect(result: &Result<User, i64>) -> i64 {
    match result {
        Ok(user) => mul(user.value, 2),
        Err(error) => mul(*error, 3),
    }
}
fn update(result: &mut Result<User, Vec<i64>>) {
    match result {
        Ok(user) => user.value = add(user.value, 2),
        Err(values) => values.push(7),
    }
}
fn counter(start: i64) -> Result<impl FnMut() -> i64, i64> {
    if start % 4 == 0 {
        return Err(start);
    }
    let mut value = start;
    Ok(move || {
        value = add(value, 1);
        value
    })
}
fn failure_scenario(value: i64) -> i64 {
    let mut held: Result<Box<dyn Fn() -> Result<i64, std::string::String>>, i64> = Err(value);
    if value % 5 != 0 {
        let text = "x".repeat(usize::try_from(add(value % 4, 1)).unwrap());
        held = Ok(Box::new(move || Err(text.clone())));
    }
    let observed = match held {
        Ok(action) => match action() {
            Ok(_) => 0,
            Err(text) => i64::try_from(text.len()).unwrap(),
        },
        Err(error) => error,
    };
    add(observed, 1)
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(add(i % 17, value1 % 19), 1);
        let result = make(value);
        let first = inspect(&result);
        let second = inspect(&result);
        let owned = match result {
            Ok(user) => add(user.value, 7),
            Err(error) => add(error, 11),
        };
        let mut editable: Result<User, Vec<i64>> = if value % 2 == 0 {
            Ok(User { value })
        } else {
            Err(vec![value])
        };
        update(&mut editable);
        update(&mut editable);
        let mutated = match editable {
            Ok(user) => user.value,
            Err(values) => add(i64::try_from(values.len()).unwrap(), values[2]),
        };
        let (call_a, call_b) = match counter(value) {
            Ok(mut action) => (action(), action()),
            Err(error) => (error, add(error, 1)),
        };
        let failed = failure_scenario(value);
        output.push(NumericArray(vec![
            first, second, owned, call_a, call_b, mutated, failed,
        ]));
    }
    Array(output)
}
fn main() {}
