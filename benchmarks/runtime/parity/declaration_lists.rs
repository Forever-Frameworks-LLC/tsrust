// Hand-written Rust for declaration_lists, written the way a careful Rust
// programmer would: owned std strings, no intermediate `words` array, moves
// instead of copies, checked arithmetic to match Tsrust's overflow semantics.
// Same observable result as the Tsrust workload.
type String = std::borrow::Cow<'static, str>;

pub struct Array<T>(Vec<T>);

impl<T> Array<T> {
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.0.iter()
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> {
    let (n, seed, input) = (value0, value1, value2);
    let mut output = Vec::new();
    let mut i = 0_i64;
    while i < n {
        let base = i.checked_mul(13).and_then(|value| value.checked_add(seed)).expect("integer overflow") % 97;
        let low = base % 5;
        let high = low + 6;
        let first: &str = input;
        let second = base.to_string();
        let (mut sum, mut steps) = (0_i64, 0_i64);
        let (mut a, mut b) = (low, high);
        while a < b {
            sum = sum.checked_add(b - a).expect("integer overflow");
            steps += 1;
            a += 1;
            b -= 1;
        }
        // The loop's update appends `first` after each of its two iterations.
        let mut text = std::string::String::with_capacity(first.len() * 2);
        for _ in 0..2 {
            text.push_str(first);
        }
        output.push(Array(vec![
            String::Owned(sum.to_string()),
            String::Owned(steps.to_string()),
            String::Owned(text),
            String::Owned(second),
            String::Owned((low + high).to_string()),
        ]));
        i += 1;
    }
    output
}

fn main() {}
