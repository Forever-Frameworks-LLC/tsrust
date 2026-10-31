// Independent standard Rust reference; retain pattern/default/control operations.
type String = std::borrow::Cow<'static, str>;
type Text = std::string::String;
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
fn owned(value: impl Into<Text>) -> String {
    String::Owned(value.into())
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let skip = base % 3;
        let mut trace = Text::new();
        let mut updates = 0;
        let mut row = 0;
        'outer: while row < 4 {
            for col in 0..3 {
                if row == skip {
                    row = add(row, 1);
                    updates = add(updates, 1);
                    continue 'outer;
                }
                if row == 3 {
                    break 'outer;
                }
                trace.push_str(&row.to_string());
                trace.push_str(&col.to_string());
            }
            row = add(row, 1);
            updates = add(updates, 1);
        }
        let mut count = 0;
        'again: while count < add(base % 4, 1) {
            count = add(count, 1);
            for x in [1, 2] {
                if x == 1 {
                    continue 'again;
                }
                count = add(count, 10);
            }
        }
        let mut post = 0;
        let mut keep = true;
        'post_loop: while keep {
            post = add(post, 1);
            loop {
                keep = post < add(base % 3, 1);
                continue 'post_loop;
            }
        }
        let mut selected = 1;
        'done: {
            if base % 2 == 0 {
                break 'done;
            }
            selected = 2;
        }
        let mut once = 0;
        'first: for value in [base, add(base, 1)] {
            once = add(once, value);
            loop {
                break 'first;
            }
        }
        output.push(Array(vec![
            owned(trace),
            owned(updates.to_string()),
            owned(count.to_string()),
            owned(post.to_string()),
            owned(selected.to_string()),
            owned(once.to_string()),
            owned(format!("{value2}{base}")),
        ]));
    }
    output
}
fn main() {}
