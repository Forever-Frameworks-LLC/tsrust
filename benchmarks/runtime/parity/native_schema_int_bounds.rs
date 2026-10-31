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
fn integer(input: &Value) -> Result<i64, Failure> {
    let Value::Number(number) = input else {
        return Err(failure(Code::TypeMismatch, Kind::Integer, kind(input)));
    };
    if let Some(value) = number.as_i64() {
        return Ok(value);
    }
    if number.is_u64() {
        return Err(failure(Code::IntegerRange, Kind::Integer, Kind::Number));
    }
    let value = number.as_f64().unwrap();
    if value.fract() != 0.0 {
        return Err(failure(
            Code::IntegerFractional,
            Kind::Integer,
            Kind::Number,
        ));
    }
    if !(-9223372036854775808.0..9223372036854775808.0).contains(&value) {
        return Err(failure(Code::IntegerRange, Kind::Integer, Kind::Number));
    }
    Ok(value as i64)
}
fn validate(input: Value) -> Result<i64, Failure> {
    let value = integer(&input)?;
    if value < 2 {
        return Err(failure(Code::TooSmall, Kind::Integer, Kind::Number));
    }
    if value > 9 {
        return Err(failure(Code::TooLarge, Kind::Integer, Kind::Number));
    }
    Ok(value)
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        checksum = add(
            checksum,
            validate(parse(value2)).expect("Expected integer success"),
        );
        checksum = add(
            checksum,
            check_failure(
                validate(parse("10")).expect_err("Expected rejection"),
                Code::TooLarge,
                Kind::Integer,
                Kind::Number,
            ),
        );
    }
    checksum
}
fn main() {}
