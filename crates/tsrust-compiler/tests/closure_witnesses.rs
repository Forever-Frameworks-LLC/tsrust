mod support;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use support::closure_corpus::{BackendExpectation, CASES, SourceExpectation};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/closures")
}

#[test]
fn closure_inventory_separates_source_cases_from_backend_controls() {
    let mut names = BTreeSet::new();
    let mut expected = BTreeSet::new();
    for case in CASES {
        assert!(names.insert(case.name), "Duplicate case: {}", case.name);
        assert!(!case.claim.is_empty());
        expected.insert(format!("{}.rs", case.name));
        match case.source {
            SourceExpectation::BackendOnly => (),
            SourceExpectation::Accept => {
                expected.insert(format!("{}.tsr", case.name));
            }
            SourceExpectation::Reject(code) => {
                assert!(!code.is_empty());
                expected.insert(format!("{}.tsr", case.name));
            }
        }
    }
    let actual = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_closure_lowerings_match_backend_expectations() {
    let directory = support::TempDirectory::new();
    for case in CASES {
        let binary = directory
            .0
            .join(format!("{}{}", case.name, std::env::consts::EXE_SUFFIX));
        let compiled = support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg("--error-format=short")
                .arg(fixtures().join(format!("{}.rs", case.name)))
                .arg("-o")
                .arg(&binary),
            60,
        );
        match case.backend {
            BackendExpectation::Output(expected) => {
                support::success(&compiled);
                let output = support::run(&mut Command::new(&binary), 10);
                support::success(&output);
                assert_eq!(
                    output.stdout,
                    expected.as_bytes(),
                    "{}: {}",
                    case.name,
                    case.claim
                );
                assert!(output.stderr.is_empty());
            }
            BackendExpectation::Reject(code) => {
                assert!(!compiled.status.success(), "{} must be rejected", case.name);
                let diagnostic = String::from_utf8(compiled.stderr).unwrap();
                assert!(
                    diagnostic.contains(&format!("error[{code}]")),
                    "{}: {diagnostic}",
                    case.name
                );
            }
        }
    }
}
