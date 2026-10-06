mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, assertions: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    let rust = emitted.replace("fn main()", "fn source_main()")
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("uppercase.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("uppercase-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(&executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn uppercase_uses_standard_unicode_expansion_and_independent_owned_results() {
    let source = r#"
        function upper(text: string): string { return text.toUpperCase(); }
        function main() {
            console.log("Straße".toUpperCase()); console.log("ﬃ".toUpperCase());
            console.log("a\u0000z".toUpperCase()); console.log("e\u0301😀".toUpperCase());
            console.log("𐐨".toUpperCase()); console.log("".toUpperCase());
        }
    "#;
    let analysis = native(
        source,
        "STRASSE\nFFI\nA\0Z\nE\u{301}😀\n𐐀\n\n",
        r#"
        for text in ["", "aZ012", "Straße", "ﬃ", "a\0z", "e\u{301}😀", "𐐨", "ΟΣİ"] {
            for owned in [false, true] {
                let original = if owned { String::Owned(text.to_owned()) } else { String::Borrowed(text) };
                let result = function0(&original);
                assert!(matches!(result, String::Owned(_)));
                assert_eq!(result.as_ref(), text.to_uppercase()); assert_eq!(original.as_ref(), text);
                let repeated = function0(&original); assert_eq!(repeated, result);
                drop(original); assert_eq!(result.as_ref(), text.to_uppercase());
            }
        }
    "#,
    );
    assert_eq!(analysis.contracts().get("upper.text"), Some(&Mode::Read));
}

#[test]
fn uppercase_reads_reusable_values_fields_array_elements_and_captures_once() {
    let source = r#"
        type Box = { text: string; };
        function receiver(): string { console.log("receiver"); return "straße"; }
        function upper(text: string): string { return text.toUpperCase(); }
        function field(box: Box): string { return box.text.toUpperCase(); }
        function main() {
            console.log(receiver().toUpperCase());
            const text = "ß"; const action = (): string => { return text.toUpperCase(); };
            console.log(action()); console.log(action()); console.log(text);
            const box: Box = { text: "é" }; const values = ["ﬃ", "a\u0000z"];
            console.log(field(box)); console.log(values[0].toUpperCase());
            console.log(values[1].toUpperCase()); console.log(upper(box.text));
            console.log(box.text); console.log(values[0]);
            const chained = "  Straße  ";
            console.log(chained.trim().toUpperCase());
            console.log(chained.toUpperCase().toLowerCase());
            console.log(chained);
        }
    "#;
    let analysis = native(
        source,
        "receiver\nSTRASSE\nSS\nSS\nß\nÉ\nFFI\nA\0Z\nÉ\né\nﬃ\nSTRASSE\n  strasse  \n  Straße  \n",
        "",
    );
    assert_eq!(analysis.contracts().get("upper.text"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("field.box"), Some(&Mode::Read));
    let closure = &analysis.closures()[0];
    assert_eq!(closure.invocation, Mode::Read);
    assert_eq!(closure.captures[0].access, Mode::Read);
}

#[test]
fn uppercase_matches_lowercase_type_arity_and_enclosing_loan_diagnostics() {
    for expression in [
        "(1).toUpperCase()",
        "true.toUpperCase()",
        "[1].toUpperCase()",
    ] {
        let source = format!("function main() {{ console.log({expression}); }}");
        assert_eq!(check(&source).err().unwrap().code, "type");
    }
    for expression in ["\"x\".toUpperCase(1)", "\"x\".toUpperCase(\"y\", \"z\")"] {
        let source = format!("function main() {{ console.log({expression}); }}");
        assert_eq!(check(&source).err().unwrap().code, "arity");
    }
    let source = r#"
        type Box = {text: string;};
        function change(box: Box): string { box.text = "changed"; return box.text.toUpperCase(); }
        function hold(box: Box, text: string) { console.log(text); }
        function main() { const box: Box = {text: "old"}; hold(box, change(box).toUpperCase()); }
    "#;
    let error = check(source).err().unwrap();
    assert_eq!(error.code, "conflicting-access");
    assert!(!error.related.is_empty());
    let moved = "function main() { const text = \"old\"; const moved = text; console.log(text.toUpperCase()); }";
    assert_eq!(check(moved).err().unwrap().code, "transferred");
}
