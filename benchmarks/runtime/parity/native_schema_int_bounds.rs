// Hand-written Rust for native_schema_int_bounds, written the way a careful Rust
// programmer would: serde_json for parsing, an enum for issue codes, checked
// arithmetic to match Tsrust's overflow semantics. Same observable checksum.
type String = std::borrow::Cow<'static, str>;

#[derive(PartialEq)]
enum Code { InvalidType, TooSmall, TooLarge }

struct Issue { code: Code, path: Vec<std::string::String> }

fn check(text: &str) -> Result<i64, Vec<Issue>> {
    let invalid = || Err(vec![Issue { code: Code::InvalidType, path: Vec::new() }]);
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(serde_json::Value::Number(number)) => match number.as_i64() {
            Some(value) if value < 2 => Err(vec![Issue { code: Code::TooSmall, path: Vec::new() }]),
            Some(value) if value > 9 => Err(vec![Issue { code: Code::TooLarge, path: Vec::new() }]),
            Some(value) => Ok(value),
            None => invalid(),
        },
        _ => invalid(),
    }
}

fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        match check(value2) {
            Ok(value) => checksum = checksum.checked_add(value).expect("integer addition overflow"),
            Err(_) => panic!("Expected integer success"),
        }
        match check("10") {
            Ok(_) => panic!("Expected upper-bound rejection"),
            Err(issues) => {
                assert_eq!(issues.len(), 1, "Invalid error count");
                for issue in &issues {
                    assert!(issue.code == Code::TooLarge && issue.path.is_empty(), "Invalid issue");
                    checksum = checksum.checked_add(1).expect("integer addition overflow");
                }
            }
        }
    }
    checksum
}

fn main() {}
