// Independent standard Serde reference for the workload's exact integer corpus.
type String = std::borrow::Cow<'static, str>;
use std::collections::BTreeMap;
#[derive(serde::Deserialize)]
struct Input {
    count: Option<i64>,
    #[serde(flatten)]
    extra: BTreeMap<std::string::String, serde_json::Value>,
}
struct Output {
    count: i64,
    extra: BTreeMap<std::string::String, serde_json::Value>,
}
struct Segment {
    field: Option<&'static str>,
    index: Option<usize>,
}
struct Issue {
    code: &'static str,
    expected: &'static str,
    observed: &'static str,
    path: Vec<Segment>,
    path_truncated: bool,
}
struct Failure {
    issues: Vec<Issue>,
    truncated: bool,
}
fn decode(text: &str) -> Result<Output, Failure> {
    let input: Input = serde_json::from_str(text).expect("Invalid corpus JSON");
    match input.count {
        Some(count) => {
            assert!(count >= 2, "Invalid benchmark input");
            Ok(Output {
                count,
                extra: input.extra,
            })
        }
        None => Err(Failure {
            issues: vec![Issue {
                code: "missing-field",
                expected: "integer",
                observed: "missing",
                path: vec![Segment {
                    field: Some("count"),
                    index: None,
                }],
                path_truncated: false,
            }],
            truncated: false,
        }),
    }
}
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
    let mut checksum = value1;
    for _ in 0..value0 {
        let value = match decode(value2) {
            Ok(value) => value,
            Err(_) => panic!("Expected source success"),
        };
        assert_eq!(value.extra.len(), 1);
        assert_eq!(
            serde_json::to_string(value.extra.get("extra").expect("Missing extra")).unwrap(),
            "{\"tag\":\"ready\",\"values\":[1,2,3]}"
        );
        checksum = checksum
            .checked_add(
                value
                    .count
                    .checked_add(i64::try_from(value.extra.len()).unwrap())
                    .unwrap(),
            )
            .unwrap();
        let failure = match decode("{\"extra\":true}") {
            Err(failure) => failure,
            Ok(_) => panic!("Expected source rejection"),
        };
        assert!(!failure.truncated && failure.issues.len() == 1);
        for issue in failure.issues {
            assert_eq!(
                (issue.code, issue.expected, issue.observed),
                ("missing-field", "integer", "missing")
            );
            assert!(!issue.path_truncated && issue.path.len() == 1);
            assert_eq!(issue.path[0].field, Some("count"));
            assert_eq!(issue.path[0].index, None);
            checksum = checksum.checked_add(1).unwrap();
        }
    }
    checksum
}
fn main() {}
