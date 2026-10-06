use tsrust_compiler::{Mode, check, line_column};

const RECORD: &str = "type User = { name: string; };\n";

#[test]
fn callee_contracts_are_inferred_before_forward_callers() {
    let source = format!(
        "{RECORD} function outer(u: User): User {{ return inner(u); }} function inner(u: User): User {{ return u; }} function main() {{}}"
    );
    let result = check(&source).unwrap_or_else(|e| panic!("{}", e.message));
    assert_eq!(result.contracts()["outer.u"], Mode::Consume);
    assert_eq!(result.contracts()["inner.u"], Mode::Consume);
}

#[test]
fn nested_argument_access_preserves_outer_loans() {
    let source = format!(
        "{RECORD}
function change(u: User): string {{ u.name = \"Sam\"; return \"done\"; }}
function both(u: User, s: string) {{ console.log(u.name); }}
function main() {{ const u: User = {{ name: \"Pat\" }}; both(u, change(u)); }}"
    );
    let error = check(&source).err().expect("conflict");
    assert_eq!(error.code, "conflicting-access");
    assert_eq!(&source[error.span.start..error.span.end], "u");
}

#[test]
fn diagnostic_offsets_are_utf8_bytes_and_columns_are_characters() {
    let source = "function main() { console.log(\"😀\"); console.log(missing); }";
    let error = check(source).err().expect("unknown value");
    assert_eq!(&source[error.span.start..error.span.end], "missing");
    assert_eq!(error.span.start, source.find("missing").unwrap());
    assert_eq!(
        line_column(source, error.span),
        (1, source[..error.span.start].chars().count() + 1)
    );
    assert!(error.to_json(source).contains("\"code\":\"name\""));
}

#[test]
fn unicode_escapes_are_validated_and_decoded() {
    let source = r#"function main() { console.log("\uD83D\uDE00"); }"#;
    assert!(
        check(source)
            .unwrap_or_else(|e| panic!("{}", e.message))
            .emit_rust()
            .contains('😀')
    );
    for value in [r"\uD800", r"\uDC00", r"\uD800\u0000"] {
        let source = format!("function main() {{ console.log(\"{value}\"); }}");
        let error = check(&source)
            .err()
            .expect("unpaired surrogate is not Unicode");
        assert_eq!(error.code, "syntax");
        assert_eq!(
            error.message,
            "Unpaired surrogate escape is not valid Unicode in a UTF-8 string."
        );
        assert_eq!(&source[error.span.start..error.span.end], &value[..6]);
    }
    for value in [r"\uZ000", r"\xG0"] {
        let error = check(&format!("function main() {{ console.log(\"{value}\"); }}"))
            .err()
            .expect("invalid escape");
        assert_eq!(error.code, "syntax");
    }
}

#[test]
fn explicit_limits_reject_deep_source_without_panicking() {
    let source = format!(
        "function main() {{ {}\"x\"{}; }}",
        "(".repeat(200),
        ")".repeat(200)
    );
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
    let mut nested = "\"x\"".to_owned();
    for _ in 0..40 {
        nested = format!("({nested}){}", ".trim()".repeat(10));
    }
    let source = format!("function main() {{ {nested}; }}");
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
    let source = format!("function main() {{ \"x\"{}; }}", ".trim()".repeat(200));
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
    let mut source = String::from("function main() {}\n");
    for i in 0..200 {
        source.push_str(&format!("function f{i}() {{ f{}(); }}\n", i + 1));
    }
    source.push_str("function f200() {}\n");
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
}

#[test]
fn malformed_token_streams_return_diagnostics_without_panicking() {
    // Deterministic smoke coverage; this is not a substitute for sustained fuzzing.
    let pieces = [
        "function", "main", "type", "const", "return", "x", "string", "{", "}", "(", ")", ";", ":",
        "=", ",", ".", "\"a\"", "/*", "😀", "\\",
    ];
    let mut state = 17u64;
    for _ in 0..1500 {
        let mut source = String::new();
        for _ in 0..24 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            source.push_str(pieces[(state >> 32) as usize % pieces.len()]);
            source.push(' ');
        }
        let _ = check(&source);
    }
}

#[test]
fn unused_functions_are_checked_and_duplicate_types_rejected() {
    assert_eq!(
        check("function bad() { console.log(missing); } function main() {}")
            .err()
            .unwrap()
            .code,
        "name"
    );
    assert_eq!(
        check("type U = {}; type U = {}; function main() {}")
            .err()
            .unwrap()
            .code,
        "duplicate"
    );
    assert_eq!(
        check("type U = { x: string; x: string; }; function main() {}")
            .err()
            .unwrap()
            .code,
        "duplicate"
    );
}

#[test]
fn repository_and_backend_pins_agree() {
    let pin = include_str!("../../../rust-toolchain.toml");
    assert!(
        pin.lines().any(|line| line.trim()
            == format!("channel = \"{}\"", tsrust_compiler::RUST_BACKEND_VERSION))
    );
}
