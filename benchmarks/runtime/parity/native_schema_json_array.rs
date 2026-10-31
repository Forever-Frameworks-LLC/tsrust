// Hand-written Rust: parse an owned JSON value, validate each array element,
// retain ordered indexed errors, and construct an owned integer output.
type String = std::borrow::Cow<'static, str>;

#[derive(Debug, PartialEq)]
enum Code { TooSmall, Type }
#[derive(Debug, PartialEq)]
enum Kind { Integer, Number, Other }
#[derive(Debug)]
struct Issue { code: Code, expected: Kind, observed: Kind, path: Vec<usize>, path_truncated: bool }
#[derive(Debug)]
struct Failure { issues: Vec<Issue>, truncated: bool }

fn validate(input: serde_json::Value) -> Result<Vec<i64>, Failure> {
    let serde_json::Value::Array(items) = input else { panic!("Expected array input") };
    let mut values = Vec::with_capacity(items.len());
    let mut issues = Vec::new();
    for (index, item) in items.into_iter().enumerate() {
        let value = item.as_i64();
        let code = match value {
            Some(value) if value >= 2 => { if issues.is_empty() { values.push(value); } continue; }
            Some(_) => Code::TooSmall,
            None => Code::Type,
        };
        values.clear();
        issues.push(Issue { code, expected: Kind::Integer,
            observed: if item.is_number() { Kind::Number } else { Kind::Other },
            path: vec![index], path_truncated: false });
    }
    if issues.is_empty() { Ok(values) } else { Err(Failure { issues, truncated: false }) }
}

fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let input = serde_json::from_str(value2).expect("Invalid JSON");
        let value = validate(input).expect("Expected success");
        checksum = checksum.checked_add(i64::try_from(value.len()).unwrap()).expect("integer addition overflow");
        let input = serde_json::from_str("[1,0]").expect("Invalid JSON");
        let error = validate(input).expect_err("Expected rejection");
        assert!(!error.truncated && error.issues.len() == 2, "Invalid count");
        for (index, issue) in error.issues.into_iter().enumerate() {
            assert!(issue.code == Code::TooSmall && issue.expected == Kind::Integer
                && issue.observed == Kind::Number && issue.path == [index]
                && !issue.path_truncated, "Invalid issue");
            checksum = checksum.checked_add(1).expect("integer addition overflow");
        }
    }
    checksum
}
fn main() {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_error_order_and_checks_integer_type() {
        let error = validate(serde_json::from_str("[2,1,3.5,4,0]").unwrap()).unwrap_err();
        assert_eq!(error.issues.iter().map(|i| i.path[0]).collect::<Vec<_>>(), [1,2,4]);
        assert_eq!(error.issues[1].code, Code::Type);
        assert_eq!(validate(serde_json::from_str("[2,3]").unwrap()).unwrap(), [2,3]);
        assert!(validate(serde_json::from_str("[]").unwrap()).unwrap().is_empty());
    }
}
