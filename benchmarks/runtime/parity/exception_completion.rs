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

fn attempt(value: i64, trace: &mut Vec<i64>) -> i64 {
    let completion: Result<i64, i64> = if value % 3 == 0 {
        Err(value)
    } else {
        Ok(add(value, 1))
    };
    let result = match completion {
        Ok(v) => v,
        Err(error) => add(error, 2),
    };
    trace.push(value);
    result
}
fn replaced(value: i64) -> i64 {
    add(value, 4)
}
fn retained(value: std::string::String, _early: bool, trace: &mut Vec<i64>) -> std::string::String {
    let pending = value;
    trace.push(7);
    pending
}
struct RetainedOwner {
    text: std::string::String,
    stamp: i64,
}
struct CallbackOwner<F> {
    text: std::string::String,
    run: F,
}
fn retained_field(value: RetainedOwner, _early: bool, trace: &mut Vec<i64>) -> std::string::String {
    let stamp = vec![value.stamp];
    let row = CallbackOwner {
        text: value.text,
        run: || stamp[0],
    };
    let pending = row.text;
    trace.push((row.run)());
    pending
}
fn retained_map(value: RetainedOwner, early: bool, trace: &mut Vec<i64>) -> std::string::String {
    let old = vec![3];
    let current = vec![7];
    let mut handlers: std::collections::BTreeMap<&str, Box<dyn Fn() -> i64 + '_>> =
        std::collections::BTreeMap::new();
    handlers.insert("old", Box::new(|| old[0]));
    if early {
        let pending = value.text;
        if let Some(item) = handlers.get("old") {
            trace.push(add(item(), value.stamp));
        }
        return pending;
    }
    handlers.clear();
    handlers.insert("new", Box::new(|| current[0]));
    let completion: Result<(), i64> = Err(32);
    if let Err(error) = completion {
        trace.push(error);
    }
    if let Some(item) = handlers.get("new") {
        trace.push(add(item(), value.stamp));
    }
    value.text
}
fn retained_loop(value: RetainedOwner, mode: i64, trace: &mut Vec<i64>) -> std::string::String {
    for i in 0..2 {
        if mode == 0 {
            let pending = value.text;
            trace.push(add(20, i));
            return pending;
        }
        let completion: Result<(), i64> = if mode == 1 || mode == 2 {
            Err(mode)
        } else {
            Ok(())
        };
        let control = match completion {
            Err(1) => 1,
            Err(2) => 2,
            _ => 0,
        };
        trace.push(add(20, i));
        if control == 1 {
            break;
        }
        if control == 2 {
            continue;
        }
    }
    value.text
}
fn retained_catch(value: RetainedOwner, mode: i64, trace: &mut Vec<i64>) -> std::string::String {
    let run = || {
        match mode {
            0 => (),
            1 => trace.push(30),
            _ => trace.push(29),
        }
        let pending = value.text;
        trace.push(31);
        pending
    };
    run()
}
fn conditional_cleanup(
    value: std::string::String,
    early: bool,
    replace: bool,
) -> std::string::String {
    let alternate = "alternate".to_owned();
    let pending = if early { Some(value) } else { None };
    if replace {
        alternate
    } else {
        pending.unwrap_or(alternate)
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let value = add(mul(i, 13), value1) % 97;
        let mut trace = Vec::new();
        let result = attempt(value, &mut trace);
        let owned = retained(
            value2.to_string() + &value.to_string(),
            value % 2 == 0,
            &mut trace,
        );
        let field = retained_field(
            RetainedOwner {
                text: owned,
                stamp: 11,
            },
            value % 2 == 0,
            &mut trace,
        );
        let mapped = retained_map(
            RetainedOwner {
                text: field,
                stamp: 1,
            },
            value % 2 == 0,
            &mut trace,
        );
        let looped = retained_loop(
            RetainedOwner {
                text: mapped,
                stamp: 0,
            },
            value % 4,
            &mut trace,
        );
        let caught = retained_catch(
            RetainedOwner {
                text: looped,
                stamp: 0,
            },
            value % 3,
            &mut trace,
        );
        let selected = conditional_cleanup(caught, value % 2 == 0, value % 3 == 0);
        output.push(Array(
            vec![
                result.to_string(),
                replaced(value).to_string(),
                trace
                    .iter()
                    .map(i64::to_string)
                    .collect::<Vec<_>>()
                    .join(","),
                selected,
            ]
            .into_iter()
            .map(String::Owned)
            .collect(),
        ));
    }
    output
}
fn main() {}
