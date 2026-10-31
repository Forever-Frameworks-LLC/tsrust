// Independent standard Rust/serde schema reference; retain typed output and observed diagnostics.
type String = std::borrow::Cow<'static, str>;
use serde_json::Value;
#[derive(Debug, PartialEq, Eq)]
enum Code {
    TypeMismatch,
    MissingField,
    UnknownField,
    TooSmall,
    LiteralMismatch,
    IntegerFractional,
    IntegerRange,
    NumberPrecision,
}
#[derive(Debug, PartialEq, Eq)]
enum Kind {
    Missing,
    String,
    Integer,
    Number,
    Boolean,
    Null,
    Array,
    Object,
}
#[derive(Debug, PartialEq, Eq)]
enum Segment {
    Field(std::string::String),
}
#[derive(Debug)]
struct Issue {
    code: Code,
    expected: Kind,
    observed: Kind,
    path: Vec<Segment>,
    path_truncated: bool,
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
fn failure(code: Code, expected: Kind, observed: Kind) -> Failure {
    Failure {
        issues: vec![Issue {
            code,
            expected,
            observed,
            path: Vec::new(),
            path_truncated: false,
        }],
        truncated: false,
    }
}
fn attach(mut error: Failure, name: &str) -> Failure {
    for issue in &mut error.issues {
        issue.path.insert(0, Segment::Field(name.to_owned()));
    }
    error
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
fn check_root(error: Failure, code: Code, expected: Kind, observed: Kind) {
    assert!(!error.truncated && error.issues.len() == 1);
    for issue in error.issues {
        assert_eq!(issue.code, code);
        assert_eq!(issue.expected, expected);
        assert_eq!(issue.observed, observed);
        assert!(issue.path.is_empty() && !issue.path_truncated);
    }
}
fn native_number(value: i64) -> Result<f64, Failure> {
    let converted = value as f64;
    if converted as i128 != i128::from(value) {
        Err(failure(Code::NumberPrecision, Kind::Number, Kind::Integer))
    } else {
        Ok(converted)
    }
}
fn number(input: &Value) -> Result<f64, Failure> {
    let Value::Number(value) = input else {
        return Err(failure(Code::TypeMismatch, Kind::Number, kind(input)));
    };
    if let Some(integer) = value.as_i64() {
        return native_number(integer).map_err(|mut e| {
            e.issues[0].observed = Kind::Number;
            e
        });
    }
    if let Some(integer) = value.as_u64() {
        let converted = integer as f64;
        if converted as i128 != i128::from(integer) {
            return Err(failure(Code::NumberPrecision, Kind::Number, Kind::Number));
        }
        return Ok(converted);
    }
    Ok(value.as_f64().unwrap())
}
#[derive(Debug, PartialEq)]
enum CodeValue {
    Low,
    High,
}
fn validate(input: Value) -> Result<CodeValue, Failure> {
    let value = number(&input)?;
    if value == 1.0 {
        Ok(CodeValue::Low)
    } else if value == 2.5 {
        Ok(CodeValue::High)
    } else {
        Err(failure(Code::LiteralMismatch, Kind::Number, Kind::Number))
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        assert_eq!(
            validate(parse(value2)).expect("Expected success"),
            CodeValue::High
        );
        checksum = add(checksum, 1);
        check_root(
            validate(parse("3.0")).expect_err("Expected rejection"),
            Code::LiteralMismatch,
            Kind::Number,
            Kind::Number,
        );
        checksum = add(checksum, 1);
    }
    checksum
}
fn main() {}
