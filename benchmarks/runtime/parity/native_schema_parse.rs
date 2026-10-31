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
fn validate(text: std::string::String) -> Result<std::string::String, Failure> {
    if text.is_empty() {
        Err(failure(Code::TooSmall, Kind::String, Kind::String))
    } else {
        Ok(text)
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let text = validate(value2.as_ref().to_owned()).expect("Expected success");
        assert_eq!(text, "hello");
        checksum = add(checksum, len(&text));
        let mut rejected = false;
        match validate(std::string::String::new()) {
            Ok(_) => {}
            Err(error) => {
                assert!(!error.truncated && error.issues.len() == 1);
                for item in error.issues {
                    assert_eq!(item.code, Code::TooSmall);
                    assert!(item.path.is_empty() && !item.path_truncated);
                }
                rejected = true;
                checksum = add(checksum, 1);
            }
        }
        assert!(rejected);
    }
    checksum
}
fn main() {}
