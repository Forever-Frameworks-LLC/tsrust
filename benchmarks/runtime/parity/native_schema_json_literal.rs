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
fn validate(input: Value, allowed: &[&str]) -> Result<std::string::String, Failure> {
    let observed = kind(&input);
    let Value::String(text) = input else {
        return Err(failure(Code::TypeMismatch, Kind::String, observed));
    };
    if allowed.contains(&text.as_str()) {
        Ok(text)
    } else {
        Err(failure(Code::LiteralMismatch, Kind::String, Kind::String))
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let allowed = ["ready"];
    let mut checksum = value1;
    for _ in 0..value0 {
        let value = validate(parse(value2), &allowed).expect("Expected success");
        assert_eq!(value, "ready");
        checksum = add(checksum, len(&value));
        check_root(
            validate(parse("\"waiting\""), &allowed).expect_err("Expected rejection"),
            Code::LiteralMismatch,
            Kind::String,
            Kind::String,
        );
        checksum = add(checksum, 1);
    }
    checksum
}
fn main() {}
