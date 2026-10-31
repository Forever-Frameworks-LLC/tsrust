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
fn object(input: Value) -> Result<serde_json::Map<std::string::String, Value>, Failure> {
    let observed = kind(&input);
    if let Value::Object(fields) = input {
        Ok(fields)
    } else {
        Err(failure(Code::TypeMismatch, Kind::Object, observed))
    }
}
fn count(value: Option<Value>, minimum: i64) -> Result<i64, Failure> {
    let Some(value) = value else {
        return Err(attach(
            failure(Code::MissingField, Kind::Integer, Kind::Missing),
            "count",
        ));
    };
    let count = integer(&value).map_err(|e| attach(e, "count"))?;
    if count < minimum {
        Err(attach(
            failure(Code::TooSmall, Kind::Integer, Kind::Number),
            "count",
        ))
    } else {
        Ok(count)
    }
}
fn check_count(error: Failure) -> i64 {
    assert!(!error.truncated && error.issues.len() == 1);
    for issue in error.issues {
        assert_eq!(issue.code, Code::MissingField);
        assert_eq!(issue.expected, Kind::Integer);
        assert_eq!(issue.observed, Kind::Missing);
        assert_eq!(issue.path, vec![Segment::Field("count".to_owned())]);
        assert!(!issue.path_truncated);
    }
    1
}
fn label(value: Option<Value>, nullable: bool) -> Result<Option<std::string::String>, Failure> {
    match value {
        None => Err(attach(
            failure(Code::MissingField, Kind::String, Kind::Missing),
            "label",
        )),
        Some(Value::Null) if nullable => Ok(None),
        Some(Value::String(text)) => Ok(Some(text)),
        Some(value) => Err(attach(
            failure(Code::TypeMismatch, Kind::String, kind(&value)),
            "label",
        )),
    }
}
fn take<T>(value: Result<T, Failure>, issues: &mut Vec<Issue>) -> Option<T> {
    match value {
        Ok(value) => Some(value),
        Err(mut error) => {
            issues.append(&mut error.issues);
            None
        }
    }
}
fn unknown(fields: serde_json::Map<std::string::String, Value>, issues: &mut Vec<Issue>) {
    for (key, value) in fields {
        issues.extend(
            attach(
                failure(Code::UnknownField, Kind::Missing, kind(&value)),
                &key,
            )
            .issues,
        );
    }
}
fn check_presence(error: Failure) -> i64 {
    assert!(!error.truncated && error.issues.len() == 2);
    let mut count = 0;
    for issue in error.issues {
        assert!(!issue.path_truncated);
        assert_eq!(issue.path.len(), 1);
        let Segment::Field(name) = &issue.path[0];
        match name.as_str() {
            "count" => {
                assert_eq!(issue.code, Code::TypeMismatch);
                assert_eq!(issue.expected, Kind::Integer);
                assert_eq!(issue.observed, Kind::Null);
            }
            "label" => {
                assert_eq!(issue.code, Code::MissingField);
                assert_eq!(issue.expected, Kind::String);
                assert_eq!(issue.observed, Kind::Missing);
            }
            _ => panic!("Unexpected field"),
        };
        count = add(count, 1);
    }
    count
}
#[derive(Debug)]
struct Output {
    count: Option<i64>,
    label: Option<std::string::String>,
}
fn validate(input: Value) -> Result<Output, Failure> {
    let mut fields = object(input)?;
    let mut issues = Vec::new();
    let count_result = match fields.remove("count") {
        None => Ok(None),
        some => count(some, i64::MIN).map(Some),
    };
    let count = take(count_result, &mut issues);
    let label = take(label(fields.remove("label"), true), &mut issues);
    unknown(fields, &mut issues);
    if issues.is_empty() {
        Ok(Output {
            count: count.unwrap(),
            label: label.unwrap(),
        })
    } else {
        Err(Failure {
            issues,
            truncated: false,
        })
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let output = validate(parse(value2)).expect("Expected success");
        assert!(output.count.is_none() && output.label.is_none());
        checksum = add(checksum, 1);
        checksum = add(
            checksum,
            check_presence(validate(parse("{\"count\":null}")).expect_err("Expected rejection")),
        );
    }
    checksum
}
fn main() {}
