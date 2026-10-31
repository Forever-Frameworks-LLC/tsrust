// Independently written Rust schema validation; keep observed typed data and diagnostics.
type String = std::borrow::Cow<'static, str>;
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
fn native_number(value: i64) -> Result<f64, Failure> {
    let converted = value as f64;
    if converted as i128 != i128::from(value) {
        Err(failure(Code::NumberPrecision, Kind::Number, Kind::Integer))
    } else {
        Ok(converted)
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        assert_eq!(native_number(42).expect("Expected exact integer"), 42.0);
        checksum = add(checksum, 1);
        checksum = add(
            checksum,
            check_failure(
                native_number(9007199254740993).expect_err("Expected precision rejection"),
                Code::NumberPrecision,
                Kind::Number,
                Kind::Integer,
            ),
        );
    }
    checksum
}
fn main() {}
