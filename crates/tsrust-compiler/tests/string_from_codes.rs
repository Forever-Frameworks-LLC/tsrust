mod support;
use support::{execute, reject};

// Expected output was produced by Node for the same program. A surrogate code
// with no partner prints as U+FFFD in both.
#[test]
fn string_from_code_point_and_char_code_build_text_from_numbers() {
    execute(
        include_str!("fixtures/string-from-codes/values.tsr"),
        include_str!("fixtures/string-from-codes/values.stdout"),
        None,
    );
}

#[test]
fn string_from_codes_require_int_arguments_and_return_a_string() {
    for (source, code) in [
        ("function main() { console.log(String.fromCodePoint(65.5)); }", "type"),
        ("function main() { console.log(String.fromCharCode(\"A\")); }", "type"),
        ("function main() { const codes = [\"a\"]; console.log(String.fromCodePoint(...codes)); }", "type"),
        ("function main() { const value: int = String.fromCodePoint(65); }", "type"),
        ("function main() { console.log(String.fromCodePoints(65)); }", "unsupported"),
    ] {
        reject(source, code);
    }
}

#[test]
fn string_from_code_point_stops_on_a_code_outside_unicode() {
    for code in ["-1", "1114112"] {
        let source = format!("function main() {{ const code = {code}; console.log(String.fromCodePoint(65, code)); }}");
        let analysis = tsrust_compiler::check(&source).unwrap();
        let directory = support::TempDirectory::new();
        let path = directory.0.join("code.rs");
        std::fs::write(&path, analysis.emit_rust()).unwrap();
        let binary = directory.0.join("code");
        support::success(&support::run(support::rustc().arg("--edition=2024").arg(&path).arg("-o").arg(&binary), 60));
        let output = support::run(&mut std::process::Command::new(&binary), 10);
        assert!(!output.status.success(), "code {code}");
        assert!(output.stdout.is_empty(), "code {code}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("String.fromCodePoint code point must be between 0 and 1114111"), "code {code}");
    }
}
