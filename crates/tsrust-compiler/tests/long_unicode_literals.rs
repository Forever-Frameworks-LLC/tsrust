mod support;
use std::process::Command;
use support::{TempDirectory, run, rustc, success};

fn verify(source: &str, assertions: &str) -> String {
    let emitted = tsrust_compiler::check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)))
        .emit_rust();
    let dir = TempDirectory::new();
    let file = dir.0.join("literal-model.rs");
    let code = emitted.replace("fn main()", "fn source_main()")
        + &format!(
            r#"
fn main() {{
    fn transferable<T: Send + Sync>() {{}} transferable::<String>();
    assert_eq!(std::mem::size_of::<String>(), std::mem::size_of::<std::borrow::Cow<'static, str>>());
    {assertions}
}}
"#
        );
    std::fs::write(&file, code).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("literal-model-{level}"));
        success(&run(
            rustc()
                .args([
                    "--edition=2024",
                    "-A",
                    "warnings",
                    "--crate-name",
                    "long_literal_model",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&file)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        success(&run(&mut Command::new(binary), 30));
    }
    emitted
}

#[test]
fn direct_long_literal_all_scalar_positions_and_byte_slices_are_exact() {
    let text = format!("{}😀{}", "a".repeat(63), "A😀é中Σ".repeat(96));
    let source = format!(
        r#"function collect(p: int, start: int, end: int): string[] {{
        const text = "{text}";
        return [text.charAt(p), text.slice(start, end), text.slice(63, 67), clone(text), text.trim().toLowerCase()];
    }} function main() {{}}"#
    );
    verify(
        &source,
        &format!(
            r#"
        let text = {text:?}; let expected: Vec<_> = text.chars().collect();
        let mut boundaries: Vec<_> = text.char_indices().map(|(i, _)| i).collect(); boundaries.push(text.len());
        for p in 0..=expected.len() {{
            let start = boundaries[p]; let end = boundaries.get(p + 1).copied().unwrap_or(start);
            let result = function0(p as i64, start as i64, end as i64);
            assert_eq!(result.get(0).unwrap().as_ref(), expected.get(p).into_iter().collect::<std::string::String>());
            assert_eq!(result.get(1).unwrap().as_ref(), &text[start..end]);
            assert_eq!(result.get(2).unwrap().as_ref(), "😀");
            assert!(matches!(result.get(3).unwrap(), String::Borrowed(_)));
            assert_eq!(result.get(3).unwrap().as_ref(), text);
            assert_eq!(result.get(4).unwrap().as_ref(), text.to_lowercase());
        }}
        let original = function0(0, 0, 0); let child = tsr_slice(original.get(3).unwrap(), 63, Some(67));
        let mut saved = child.clone(); saved.to_mut().push('!'); drop(original); drop(child);
        assert_eq!(saved.as_ref(), "😀!");
    "#
        ),
    );
}

#[test]
fn long_literal_arrays_forward_scalar_access_and_valid_byte_ranges() {
    let text = format!("{}😀{}", "a".repeat(127), "Α😀ΟΣİ".repeat(64));
    let source = format!(
        r#"function collect(p: int): string[] {{
        const values = ["short", "{text}", ""]; return scan(values, p);
    }} function scan(values: string[], p: int): string[] {{
        const output: string[] = []; for (const value of values) {{ output.push(value.charAt(p)); }}
        output.push(values[1].slice(127, 131)); return output;
    }} function main() {{}}"#
    );
    verify(
        &source,
        &format!(
            r#"
        let expected: Vec<_> = {text:?}.chars().collect();
        for p in [0, 63, 64, 126, 127, 128, 129, expected.len()/2, expected.len()*3/4, expected.len()-1, expected.len()] {{
            let result = function0(p as i64);
            assert_eq!(result.get(0).unwrap().as_ref(), "short".chars().nth(p).into_iter().collect::<std::string::String>());
            assert_eq!(result.get(1).unwrap().as_ref(), expected.get(p).into_iter().collect::<std::string::String>());
            assert!(result.get(2).unwrap().is_empty()); assert_eq!(result.get(3).unwrap().as_ref(), "😀");
        }}
    "#
        ),
    );
}

#[test]
fn lowercase_expansion_uses_standard_owned_utf8() {
    let text = "İ".repeat(128);
    let lower = text.to_lowercase();
    let source = format!(
        r#"function collect(): string[] {{
        const text = "{text}"; return [clone(text), text.toLowerCase(), text.trim().toLowerCase()];
    }} function main() {{}}"#
    );
    let rust = verify(
        &source,
        &format!(
            r#"
        let result = function0(); assert!(matches!(result.get(0).unwrap(), String::Borrowed(_)));
        for index in [1, 2] {{ assert!(matches!(result.get(index).unwrap(), String::Owned(_))); assert_eq!(result.get(index).unwrap().as_ref(), {lower:?}); }}
    "#
        ),
    );
    assert!(!rust.contains("struct Literal {"));
}

#[test]
fn large_literal_pools_keep_long_unicode_access_and_transforms_uniform() {
    let words = (0..1100)
        .map(|n| format!("\"  LABEL {n:04} ABC XYZ  \""))
        .collect::<Vec<_>>()
        .join(",");
    let text = "A😀é中Σ".repeat(128);
    let source = format!(
        r#"function collect(p: int): string[] {{
        const words = [{words}]; const text = "{text}";
        return [words[1099].trim().toLowerCase(), text.charAt(p), clone(text)];
    }} function main() {{}}"#
    );
    let rust = verify(
        &source,
        &format!(
            r#"
        let chars: Vec<_> = {text:?}.chars().collect();
        for p in [63,64,127,128,chars.len()/2,chars.len()*7/8,chars.len()-1] {{
            let result = function0(p as i64); assert_eq!(result.get(0).unwrap().as_ref(), "label 1099 abc xyz");
            assert_eq!(result.get(1).unwrap().as_ref(), chars[p].to_string());
            assert!(matches!(result.get(2).unwrap(), String::Borrowed(_)));
        }}
    "#
        ),
    );
    assert!(!rust.contains("struct Literal {"));
}

#[test]
fn literal_sizes_and_unicode_widths_use_the_same_standard_representation() {
    let ascii = "A".repeat(512);
    let short = "😀".repeat(64);
    let long = "😀".repeat(65);
    let source = format!(
        r#"function collect(): string[] {{ return ["{ascii}", "{short}", "{long}"]; }} function main() {{}}"#
    );
    verify(
        &source,
        r#"
        let result = function0(); for value in result.iter() { assert!(matches!(value, String::Borrowed(_))); }
        assert_eq!(tsr_length(result.get(0).unwrap()), 512); assert_eq!(tsr_length(result.get(1).unwrap()), 256); assert_eq!(tsr_length(result.get(2).unwrap()), 260);
    "#,
    );
}
