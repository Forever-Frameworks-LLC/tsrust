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
fn check_array(error: Failure, code: Code, expected: Kind, count: usize) -> i64 {
    assert!(!error.truncated && error.issues.len() == count);
    let mut result = 0;
    for (index, item) in error.issues.into_iter().enumerate() {
        assert_eq!(item.code, code);
        assert_eq!(item.expected, expected);
        assert_eq!(item.observed, Kind::Integer);
        assert!(!item.path_truncated);
        assert_eq!(item.path, vec![Segment::Index(index)]);
        result = add(result, 1);
    }
    result
}
fn validate(input: Vec<i64>) -> Result<Vec<i64>, Failure> {
    let mut issues = Vec::new();
    for (index, &value) in input.iter().enumerate() {
        if value < 2 {
            let mut item = issue(Code::TooSmall, Kind::Integer, Kind::Integer);
            item.path.push(Segment::Index(index));
            issues.push(item);
        }
    }
    if issues.is_empty() {
        Ok(input)
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
        let output = validate(vec![2, 3]).expect("Expected success");
        checksum = add(checksum, i64::try_from(output.len()).unwrap());
        checksum = add(
            checksum,
            check_array(
                validate(vec![1, 0]).expect_err("Expected rejection"),
                Code::TooSmall,
                Kind::Integer,
                2,
            ),
        );
    }
    checksum
}
fn main() {}
