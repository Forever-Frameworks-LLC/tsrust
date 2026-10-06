mod support;

use std::collections::BTreeSet;
use std::process::Command;
use support::corpus::{CASES, Expected};

#[test]
fn corpus_inventory_is_complete_and_unique() {
    let mut names = BTreeSet::new();
    let mut expected = BTreeSet::new();
    for case in CASES {
        assert!(names.insert(case.name), "Duplicate case: {}", case.name);
        assert!(!case.claim.is_empty());
        expected.insert(format!("{}.tsr", case.name));
        expected.insert(format!("{}.rs", case.name));
    }
    let actual = std::fs::read_dir(support::fixture_dir())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected);
}

#[test]
fn handwritten_rust_witnesses() {
    let directory = support::TempDirectory::new();
    for case in CASES {
        let source = support::fixture_dir().join(format!("{}.rs", case.name));
        let binary = directory
            .0
            .join(format!("{}{}", case.name, std::env::consts::EXE_SUFFIX));
        let compiled = support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg("--error-format=short")
                .arg(&source)
                .arg("-o")
                .arg(&binary),
            60,
        );
        match case.expected {
            Expected::Output(stdout) => {
                support::success(&compiled);
                let output = support::run(&mut Command::new(&binary), 10);
                support::success(&output);
                assert_eq!(
                    output.stdout,
                    stdout.as_bytes(),
                    "{}: {}",
                    case.name,
                    case.claim
                );
                assert!(output.stderr.is_empty());
            }
            Expected::Rejected { rust_code, .. } => {
                assert!(!compiled.status.success(), "{} must be rejected", case.name);
                let diagnostics = String::from_utf8(compiled.stderr).unwrap();
                assert!(
                    diagnostics.contains(&format!("error[{rust_code}]")),
                    "{}: {diagnostics}",
                    case.name
                );
            }
        }
    }
}
