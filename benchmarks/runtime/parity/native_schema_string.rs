// Independently written Rust schema validation; keep observed typed data and diagnostics.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Code {
    TypeMismatch,
    IntegerFractional,
    IntegerRange,
    NumberPrecision,
    TooSmall,
    TooLarge,
    RefinementFailed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    String,
    Boolean,
    Integer,
    Number,
    Null,
    Array,
    Object,
}
#[derive(Debug, PartialEq, Eq)]
enum Segment {
    Index(usize),
}
#[derive(Debug)]
struct Issue {
    code: Code,
    expected: Kind,
    observed: Kind,
    path: Vec<Segment>,
    path_truncated: bool,
    message: Option<std::string::String>,
}
#[derive(Debug)]
struct Failure {
    issues: Vec<Issue>,
    truncated: bool,
}
fn kind(value: &Value) -> Kind {
    match value {
        Value::Null => Kind::Null,
        Value::Bool(_) => Kind::Boolean,
        Value::Number(_) => Kind::Number,
        Value::String(_) => Kind::String,
        Value::Array(_) => Kind::Array,
        Value::Object(_) => Kind::Object,
    }
}
fn issue(code: Code, expected: Kind, observed: Kind) -> Issue {
    Issue {
        code,
        expected,
        observed,
        path: Vec::new(),
        path_truncated: false,
        message: None,
    }
}
fn failure(code: Code, expected: Kind, observed: Kind) -> Failure {
    Failure {
        issues: vec![issue(code, expected, observed)],
        truncated: false,
    }
}
fn parse(input: &str) -> Value {
    serde_json::from_str(input).expect("Invalid JSON")
}
fn add(a: i64, b: i64) -> i64 {
    a.checked_add(b).unwrap()
}
fn len(text: &str) -> i64 {
    i64::try_from(text.len()).unwrap()
}
fn check_failure(error: Failure, code: Code, expected: Kind, observed: Kind) -> i64 {
    assert!(!error.truncated && error.issues.len() == 1);
    let mut count = 0;
    for item in error.issues {
        assert_eq!(item.code, code);
        assert_eq!(item.expected, expected);
        assert_eq!(item.observed, observed);
        assert!(item.path.is_empty() && !item.path_truncated);
        count = add(count, 1);
    }
    count
}
fn string(
    input: Value,
    min: Option<usize>,
    max: Option<usize>,
    messages: bool,
) -> Result<std::string::String, Failure> {
    let observed = kind(&input);
    let Value::String(text) = input else {
        return Err(failure(Code::TypeMismatch, Kind::String, observed));
    };
    let count = if min.is_some() || max.is_some() {
        text.chars().count()
    } else {
        0
    };
    let code = if min.is_some_and(|n| count < n) {
        Some(Code::TooSmall)
    } else if max.is_some_and(|n| count > n) {
        Some(Code::TooLarge)
    } else {
        None
    };
    if let Some(code) = code {
        let mut error = failure(code, Kind::String, Kind::String);
        if messages {
            error.issues[0].message = Some(
                if code == Code::TooSmall {
                    "Too short"
                } else {
                    "Too long"
                }
                .to_owned(),
            );
        }
        return Err(error);
    }
    Ok(text)
}
fn validate(value: Value) -> Result<std::string::String, Failure> {
    string(value, None, None, false)
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let text = validate(parse(value2)).expect("Expected string success");
        checksum = add(checksum, len(&text));
        let error = validate(parse("42")).expect_err("Expected rejection");
        checksum = add(
            checksum,
            check_failure(error, Code::TypeMismatch, Kind::String, Kind::Number),
        );
    }
    checksum
}
fn main() {}
