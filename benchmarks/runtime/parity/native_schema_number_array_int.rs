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
fn validate(input: Vec<i64>) -> Result<Vec<f64>, Failure> {
    let mut output = Vec::with_capacity(input.len());
    let mut issues = Vec::new();
    for (index, value) in input.into_iter().enumerate() {
        match native_number(value) {
            Ok(value) => {
                if issues.is_empty() {
                    output.push(value);
                }
            }
            Err(error) => {
                output.clear();
                for mut item in error.issues {
                    item.path.push(Segment::Index(index));
                    issues.push(item);
                }
            }
        }
    }
    if issues.is_empty() {
        Ok(output)
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
        for value in validate(vec![42]).expect("Expected exact conversion") {
            assert_eq!(value, 42.0);
            checksum = add(checksum, 1);
        }
        checksum = add(
            checksum,
            check_array(
                validate(vec![9007199254740993]).expect_err("Expected rejection"),
                Code::NumberPrecision,
                Kind::Number,
                1,
            ),
        );
    }
    checksum
}
fn main() {}
