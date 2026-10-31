// Independent standard Rust nested functions and closures; checked integer arithmetic.
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
fn length(s: &str) -> i64 {
    i64::try_from(s.len()).unwrap()
}

fn depth(base: &[i64], level: i64) -> i64 {
    if level == 0 {
        base[1]
    } else {
        add(depth(base, sub(level, 1)), 1)
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
    let mut total = 0;
    let mut history = Vec::new();
    let mut output = Vec::with_capacity(usize::try_from(value0).unwrap());
    for i in 0..value0 {
        let value = add(add(i % 5, value1 % 5), 1);
        let base = vec![value, add(value, 1)];
        {
            let mut add_one = |amount| {
                total = add(total, amount);
                history.push(amount);
            };
            let mut add_twice = |amount| {
                add_one(amount);
                add_one(amount);
            };
            add_twice(value);
        }
        let shifted = |item| add(item, base[0]);
        let mapped: Vec<_> = [1, 2].into_iter().map(shifted).collect();
        let later = add(value, 3);
        let read_later = || mul(later, 2);
        let mut counts = vec![0];
        {
            let mut tick = || {
                counts[0] = add(counts[0], value);
                counts[0]
            };
            {
                let run = &mut tick;
                run();
            }
            tick();
        }
        output.push(NumericArray(vec![
            total,
            i64::try_from(history.len()).unwrap(),
            mapped[0],
            mapped[1],
            depth(&base, 3),
            read_later(),
            counts[0],
        ]));
    }
    Array(output)
}
fn main() {}
