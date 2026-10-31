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
fn grade(value: i64) -> &'static str {
    if value < 20 {
        "low"
    } else if value < 60 {
        "mid"
    } else {
        "high"
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let mut output = Vec::new();
    for i in 0..value0 {
        let base = add(mul(i, 13), value1) % 97;
        let mut sum = 0;
        for k in 0..add(base % 5, 1) {
            sum = add(sum, k);
        }
        let mut text = Text::new();
        for part in [value2.repeat(1), base.to_string()] {
            text.push_str(&part);
        }
        let mut odd = 0;
        for a in 0..3 {
            for b in 0..3 {
                if a == b {
                    odd = add(odd, 1);
                } else if a + b == 2 {
                    odd = add(odd, 10);
                }
            }
        }
        let mut down = add(base % 4, 1);
        while down > 1 {
            down -= 1;
        }
        loop {
            down = add(down, 2);
            if down >= 4 {
                break;
            }
        }
        output.push(Array(vec![
            String::Borrowed(grade(base)),
            owned(sum.to_string()),
            owned(text),
            owned(odd.to_string()),
            owned(down.to_string()),
        ]));
    }
    output
}
fn main() {}
