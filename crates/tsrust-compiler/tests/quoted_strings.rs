mod support;
use std::process::Command;
use support::{TempDirectory, run, rustc, success};

fn emitted(literal: &str) -> String {
    let source = format!("function value(): string {{ return {literal}; }} function main() {{}}");
    tsrust_compiler::check(&source)
        .unwrap_or_else(|e| panic!("{}", e.render("quoted.tsr", &source)))
        .emit_rust()
}

fn equal(literal: &str, expected: &str) {
    assert_eq!(
        emitted(literal),
        emitted(&tsrust_compiler::json_string(expected)),
        "{literal:?}"
    );
}

#[test]
fn both_quote_styles_decode_strict_escapes_to_identical_native_literals() {
    let cells = [
        ("", ""),
        ("é😀中", "é😀中"),
        (r"\b\f\n\r\t\v\0", "\u{8}\u{c}\n\r\t\u{b}\0"),
        (r"\x00\x7f\x80\xff", "\0\u{7f}\u{80}ÿ"),
        (r"\u0000\u00e9\uD83D\uDE00", "\0é😀"),
        (
            r"\u{0}\u{7f}\u{80}\u{FFFF}\u{10000}\u{10ffff}",
            "\0\u{7f}\u{80}\u{ffff}\u{10000}\u{10ffff}",
        ),
        (r"\u{d83d}\u{de00}\uD83D\u{de00}\u{d83d}\uDE00", "😀😀😀"),
        (r"\a\z\é\😀\$\/\`\{\}", "azé😀$/`{}"),
        (r"\'", "'"),
        ("\u{2028}\u{2029}\t\0", "\u{2028}\u{2029}\t\0"),
        (
            "// /* } ) ; function fake() {}",
            "// /* } ) ; function fake() {}",
        ),
    ];
    for quote in ['\'', '"'] {
        for (raw, expected) in cells {
            equal(&format!("{quote}{raw}{quote}"), expected);
        }
        equal(&format!("{quote}\\\"{quote}"), "\"");
        equal(&format!("{quote}\\\\{quote}"), "\\");
        let opposite = if quote == '\'' { '"' } else { '\'' };
        equal(&format!("{quote}{opposite}{quote}"), &opposite.to_string());
        for cp in [
            0, 0x7f, 0x80, 0x7ff, 0x800, 0xd7ff, 0xe000, 0xffff, 0x10000, 0x10ffff,
        ] {
            equal(
                &format!("{quote}\\u{{{cp:x}}}{quote}"),
                &char::from_u32(cp).unwrap().to_string(),
            );
        }
    }
}

#[test]
fn all_continuations_are_empty_and_long_code_point_escapes_remain_iterative() {
    for quote in ['\'', '"'] {
        for terminator in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
            equal(
                &format!("{quote}before\\{terminator}after{quote}"),
                "beforeafter",
            );
            equal(&format!("{quote}\\{terminator}{quote}"), "");
            equal(
                &format!("{quote}\\uD83D\\{terminator}\\u{{de00}}{quote}"),
                "😀",
            );
            equal(
                &format!("{quote}\\uD83D\\{terminator}\\{terminator}\\uDE00{quote}"),
                "😀",
            );
            equal(&format!("{quote}\\0\\{terminator}1{quote}"), "\u{0}1");
            for raw in [
                format!("\\uD800\\{terminator}a"),
                format!("\\uDC00\\{terminator}\\uD800"),
            ] {
                let source = format!("function main() {{ console.log({quote}{raw}{quote}); }}");
                assert_eq!(
                    tsrust_compiler::check(&source).err().unwrap().message,
                    "Unpaired surrogate escape is not valid Unicode in a UTF-8 string."
                );
            }
        }
        equal(
            &format!("{quote}\\u{{{}10FFFF}}{quote}", "0".repeat(100_000)),
            "\u{10ffff}",
        );
        equal(
            &format!("{quote}{}{quote}", "\\é".repeat(20_000)),
            &"é".repeat(20_000),
        );
    }
}

#[test]
fn malformed_escapes_strict_decimal_and_unpaired_surrogates_are_source_errors() {
    for quote in ['\'', '"'] {
        for raw in [
            r"\x",
            r"\x0",
            r"\xGG",
            r"\u",
            r"\u000",
            r"\u00G0",
            r"\u{}",
            r"\u{g}",
            r"\u{110000}",
            r"\u{FFFFFFFFFFFFFFFF}",
            r"\u{0",
            r"\01",
            r"\00",
            r"\08",
            r"\09",
            r"\1",
            r"\7",
            r"\8",
            r"\9",
            "raw\nline",
            "raw\rline",
        ] {
            let source = format!("function main() {{ console.log({quote}{raw}{quote}); }}");
            let e = tsrust_compiler::check(&source).err().expect(raw);
            assert_eq!(e.code, "syntax", "{raw}");
            assert!(source.is_char_boundary(e.span.start) && source.is_char_boundary(e.span.end));
        }
        for raw in [
            r"\uD800",
            r"\uDC00",
            r"\u{d800}",
            r"\u{dfff}",
            r"\uD800\x41",
            r"\uD800\u{10000}",
            r"\u{d800}\uD800",
        ] {
            let source = format!("function main() {{ console.log({quote}{raw}{quote}); }}");
            let e = tsrust_compiler::check(&source).err().expect(raw);
            assert_eq!(
                e.message,
                "Unpaired surrogate escape is not valid Unicode in a UTF-8 string."
            );
            assert!(source[e.span.start..e.span.end].starts_with(r"\u"));
        }
    }
    for literal in ["'unfinished", "\"unfinished", "'trailing\\", "\"trailing\\"] {
        assert_eq!(
            tsrust_compiler::check(&format!("function main() {{ {literal}"))
                .err()
                .unwrap()
                .code,
            "syntax"
        );
    }
    let source = "function main() { const text = 'é\\\n😀'; console.log(missing); }";
    let e = tsrust_compiler::check(source).err().unwrap();
    assert_eq!(&source[e.span.start..e.span.end], "missing");
    assert_eq!(tsrust_compiler::line_column(source, e.span).0, 2);
}

#[test]
fn escaped_values_in_calls_arrays_captures_and_methods_execute_at_o0_and_o3() {
    let source = r#"
function values(): string[] {
    const text = 'é\u{1f600}\x00\'"';
    const captured = (): string => { return clone(text); };
    const result: string[] = [captured(), text.concat('!'), text.charAt(1), text.slice(2, 6), '\uD83D\u{de00}'];
    return result;
}
function main() {}
"#;
    let generated = tsrust_compiler::check(source).unwrap().emit_rust();
    let code = generated.replace("fn main()", "fn source_main()")
        + r#"
fn main() {
    let result = function0();
    let expected = ["é😀\0'\"", "é😀\0'\"!", "😀", "😀", "😀"];
    assert_eq!(result.len(), expected.len());
    for (index, value) in expected.iter().enumerate() { assert_eq!(result.get(index).unwrap().as_ref(), *value); }
    assert!(matches!(result.get(0).unwrap(), String::Borrowed(_)));
    assert!(matches!(result.get(1).unwrap(), String::Owned(_)));
    assert!(matches!(result.get(3).unwrap(), String::Borrowed(_)));
    drop(result);
}
"#;
    let dir = TempDirectory::new();
    let file = dir.0.join("quoted.rs");
    std::fs::write(&file, code).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("quoted-{level}"));
        success(&run(
            rustc()
                .args([
                    "--edition=2024",
                    "-A",
                    "warnings",
                    "--crate-name",
                    "quoted",
                    "-C",
                    &format!("opt-level={level}"),
                ])
                .arg(&file)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        success(&run(&mut Command::new(binary), 30));
    }
}
