mod support;
use support::{execute, reject};

// Expected output was produced by Node for the same values and radices.
#[test]
fn number_to_string_matches_reference_output_in_every_tested_radix() {
    execute(
        include_str!("fixtures/number-to-string/radix.tsr"),
        include_str!("fixtures/number-to-string/radix.stdout"),
        None,
    );
}

// 400 pseudo-random large, fractional, tiny and negative values in nine radices.
#[test]
fn number_to_string_matches_reference_output_for_generated_values() {
    execute(
        include_str!("fixtures/number-to-string/rand.tsr"),
        include_str!("fixtures/number-to-string/rand.stdout"),
        None,
    );
}

#[test]
fn number_to_string_checks_its_radix() {
    for (source, code) in [
        ("function main() { const v = 1.5; console.log(v.toString(2, 3)); }", "arity"),
        ("function main() { const v = 1.5; console.log(v.toString(2.5)); }", "type"),
        ("function main() { const v = 1.5; console.log(v.toString(\"2\")); }", "type"),
        ("function main() { const v = 1.5; const n: number = v.toString(); }", "type"),
    ] {
        reject(source, code);
    }
    for radix in [1, 37, 0, -2] {
        let source = format!("function main() {{ const v = 1.5; const radix = {radix}; console.log(v.toString(radix)); }}");
        let analysis = tsrust_compiler::check(&source).unwrap();
        let directory = support::TempDirectory::new();
        let path = directory.0.join("radix.rs");
        std::fs::write(&path, analysis.emit_rust()).unwrap();
        let binary = directory.0.join("radix");
        support::success(&support::run(support::rustc().arg("--edition=2024").arg(&path).arg("-o").arg(&binary), 60));
        let output = support::run(&mut std::process::Command::new(&binary), 10);
        assert!(!output.status.success(), "radix {radix}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("number.toString radix must be between 2 and 36"), "radix {radix}");
    }
}
