#[allow(dead_code)]
#[path = "../src/runtime/mod.rs"]
mod runtime;
mod support;
use support::{execute, reject, success};

#[test]
fn byte_length_slices_and_character_defaults_preserve_unicode() {
    execute(
        r#"function main() {
 const text = "A😀e\u0301\u0000Z";
 console.log(text.length); console.log("😀".length); console.log(text.slice().length);
 console.log(text.slice(1, 5)); console.log(text.slice(-1)); console.log(text.slice(5, 2).length);
 console.log(text.charAt(1)); console.log(text.charAt(2));
 console.log(text.charAt()); console.log(text.charAt(-1).length); console.log(text.charAt(999).length);
 console.log(text.slice(-9223372036854775808, 9223372036854775807).length);
 console.log(text.slice(9223372036854775807).length);
 console.log(text.slice(0, -9223372036854775808).length);
 console.log("\uD83D\uDE00".length);
 }"#,
        "10\n4\n10\n😀\nZ\n0\n😀\ne\nA\n0\n0\n10\n0\n0\n4\n",
        None,
    );
}

#[test]
fn rust_whitespace_and_contextual_case_conversion_preserve_unicode() {
    execute(
        r#"function main() {
 console.log("\u0085A\u0085".trim());
 console.log("\uFEFF A \uFEFF".trim().length);
 const text = "\u2003 ΟΣ İ 𐐀 \u2003";
 const lower = text.trim().toLowerCase();
 const expected = ["ος i\u0307 𐐨"];
 console.log(expected.includes(lower)); console.log(lower.length);
 const copied = clone(lower); console.log(expected.includes(copied));
 console.log("e\u0301".length); console.log("é".length);
 }"#,
        "A\n9\ntrue\n13\ntrue\n3\n2\n",
        None,
    );
}

#[test]
fn slices_clones_captures_and_array_search_remain_independent() {
    execute(
        r#"function take(s: string): string { return s; }
 function saved(s: string): () => string { return (): string => { return s; }; }
 function main() {
 let text = "😀é"; const first = text.charAt(0); const second = text.charAt(1);
 const copy = clone(first); const action = saved(first); text = "replaced";
 const result = action(); const moved = take(result);
 const words = [copy, second, moved]; const independent = clone(words); words[0] = "changed";
 console.log(independent.indexOf("😀")); console.log(independent.indexOf("é"));
 console.log(independent.includes("\uFFFD")); console.log(text);
 }"#,
        "0\n1\nfalse\nreplaced\n",
        None,
    );
}

#[test]
fn string_methods_evaluate_receiver_then_arguments_and_hold_loan() {
    execute(
        r#"function receiver(): string { console.log(1); return "abcd"; }
 function offset(n: int): int { console.log(n); return n; }
 function main() { console.log(receiver().slice(offset(2), offset(3))); console.log(receiver().charAt(offset(2))); }"#,
        "1\n2\n3\nc\n1\n2\nc\n",
        None,
    );
    reject(
        r#"function take(s: string): int { const moved = s; return 0; } function main() { const s = "abc"; s.slice(take(s)); }"#,
        "conflicting-access",
    );
    reject(
        r#"type Box = {text: string;}; function change(b: Box): int { b.text = "other"; return 0; } function main() { const b: Box = {text: "abc"}; b.text.charAt(change(b)); }"#,
        "conflicting-access",
    );
}

#[test]
fn string_api_type_arity_and_immutability_diagnostics() {
    for source in [
        r#"function main() { "a".slice(true); }"#,
        r#"function main() { "a".charAt("0"); }"#,
        r#"function main() { const a = [1]; a.charAt(0); }"#,
    ] {
        reject(source, "type");
    }
    for source in [
        r#"function main() { "a".slice(0, 1, 2); }"#,
        r#"function main() { "a".charAt(0, 1); }"#,
    ] {
        reject(source, "arity");
    }
    reject(
        r#"function main() { const s = "abc"; s.length = 3; }"#,
        "unsupported",
    );
    reject(
        r#"function main() { const s = "abc"; const moved = s; console.log(s.length); }"#,
        "transferred",
    );
}

#[test]
fn runtime_utf8_model_covers_offsets_layout_and_independent_lifetimes() {
    verify_runtime(
        r#"
fn main() {
 std::panic::set_hook(Box::new(|_| {}));
 fn transferable<T: Send + Sync>() {} transferable::<String>();
 assert_eq!(std::mem::size_of::<String>(), std::mem::size_of::<std::borrow::Cow<'static, str>>());
 let offsets = [i64::MIN, -99, -8, -3, -1, 0, 1, 2, 3, 8, 99, i64::MAX];
 for text in ["", "\0", "AB", "😀", "A😀é中\0Z", "e\u{301}😀Z", "𐀀𐀀\u{10ffff}"] {
  let chars: Vec<_> = text.chars().collect();
  let value = String::Owned(text.to_owned());
  assert_eq!(tsr_length(&value), text.len() as i64);
  for start in offsets { for end in offsets {
   let bound = |index: i64| (if index < 0 { text.len() as i128 + index as i128 } else { index as i128 }).clamp(0, text.len() as i128) as usize;
   let first = bound(start); let last = bound(end).max(first);
   if first == last {
    assert!(tsr_slice(&value, start, Some(end)).is_empty());
   } else if let Some(expected) = text.get(first..last) {
    assert_eq!(tsr_slice(&value, start, Some(end)).as_ref(), expected);
   } else {
    assert!(std::panic::catch_unwind(|| tsr_slice(&value, start, Some(end))).is_err());
   }
  } }
  for index in offsets {
   let expected = if index < 0 { None } else { chars.get(index as usize) };
   assert_eq!(tsr_char_at(&value, index).as_ref(), expected.into_iter().collect::<std::string::String>());
  }
  let mut copied = value.clone();
  if !text.is_empty() { assert_ne!(copied.as_ptr(), value.as_ptr()); }
  copied.to_mut().push('!'); assert_eq!(value.as_ref(), text);
  drop(value); assert_eq!(copied.as_ref(), format!("{text}!"));
 }
 for text in ["ABCD", " A B ", "abcdefg", "ABCDEFGH", "😀Z", "éééé", " ΟΣ İ "] {
  let original = String::Owned(text.to_owned());
  let end = text.char_indices().last().unwrap().0;
  let expected = &text[..end];
  let sliced = tsr_slice(&original, 0, Some(end as i64));
  assert_eq!(sliced.as_ref(), expected);
  assert_eq!(tsr_lower(&sliced).as_ref(), expected.to_lowercase());
  assert_eq!(tsr_normalize(&sliced).as_ref(), expected.trim().to_lowercase());
 }
}
"#,
    );
}

#[test]
fn character_access_across_long_ascii_prefix_and_scalar_widths() {
    verify_runtime(
        r#"
fn main() {
 for suffix in ["", "\u{7f}", "\u{80}", "\u{7ff}", "\u{800}", "\u{d7ff}", "\u{e000}", "\u{ffff}", "\u{10000}", "😀", "\u{10ffff}"] {
  let text = format!("{}{}Z", "AbCd0123".repeat(8192), suffix);
  let expected: Vec<char> = text.chars().collect();
  let value = String::Owned(text);
  for index in [0, 1, 7, 31, 8191, 65534, 65535, 65536, 65537, 65538, 65539] {
   assert_eq!(tsr_char_at(&value, index as i64).as_ref(), expected.get(index).into_iter().collect::<std::string::String>());
  }
  for start in 65532..=value.len() { for end in start..=value.len() {
   if let Some(expected) = value.get(start..end) { assert_eq!(tsr_slice(&value, start as i64, Some(end as i64)).as_ref(), expected); }
  } }
  let tail = tsr_slice(&value, 65536, None); drop(value);
  assert_eq!(tail.as_ref(), expected[65536..].iter().collect::<std::string::String>());
 }
 for text in ["é", "éééé", "😀", "😀😀😀", "A😀éZ", "é\0😀\u{800}𐀀Z", "😀ABC", "ABC😀", "é😀A𐀀é"] {
  let expected: Vec<char> = text.chars().collect(); let value = String::Owned(text.to_owned());
  for index in 0..expected.len() {
   assert_eq!(tsr_char_at(&value, index as i64).as_ref(), expected[index].to_string());
   let (byte, scalar) = value.char_indices().nth(index).unwrap();
   assert_eq!(tsr_slice(&value, byte as i64, Some((byte + scalar.len_utf8()) as i64)).as_ref(), expected[index].to_string());
  }
  for start in 0..=expected.len() { for end in start..=expected.len() {
   let first = value.char_indices().nth(start).map_or(value.len(), |(i, _)| i);
   let last = value.char_indices().nth(end).map_or(value.len(), |(i, _)| i);
   assert_eq!(tsr_slice(&value, first as i64, Some(last as i64)).as_ref(), expected[start..end].iter().collect::<std::string::String>());
  } }
 }
 let text = "é😀\u{800}\0".repeat(4096); let expected: Vec<_> = text.chars().collect(); let value = String::Owned(text);
 for center in [0, expected.len() / 2 - 4, expected.len() - 8] { for index in center..center + 8 {
  assert_eq!(tsr_char_at(&value, index as i64).as_ref(), expected[index].to_string());
  let (byte, scalar) = value.char_indices().nth(index).unwrap();
   assert_eq!(tsr_slice(&value, byte as i64, Some((byte + scalar.len_utf8()) as i64)).as_ref(), expected[index].to_string());
 } }
}
"#,
    );
}

fn verify_runtime(body: &str) {
    let dir = support::TempDirectory::new();
    let path = dir.0.join("scalar-model.rs");
    std::fs::write(&path, format!("{}\n{body}", runtime::STRINGS)).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("scalar-model-{level}"));
        success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-A",
                    "warnings",
                    "-C",
                    &format!("opt-level={level}"),
                ])
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        success(&support::run(&mut std::process::Command::new(binary), 30));
    }
}

#[test]
fn array_search_matches_scalars_across_borrowed_owned_strings_and_signed_offsets() {
    execute(
        r#"function main() {
 const text = "A\u0000😀Z";
 const values = [text.charAt(0), text.charAt(1), text.charAt(2), text.charAt(3), text.charAt(99)];
 console.log(values.indexOf("A")); console.log(values.indexOf("\u0000"));
 console.log(values.indexOf("😀")); console.log(values.indexOf("Z")); console.log(values.indexOf(""));
 console.log(values.includes("\uFFFD")); console.log(values.indexOf(text.charAt(2), -3));
 console.log(values.indexOf(text.charAt(2), -2));
 console.log(values.indexOf("A", -9223372036854775808)); console.log(values.includes("A", 9223372036854775807));
 const literals = ["A", "\u0000", "😀", "Z", ""];
 console.log(literals.indexOf(text.charAt(2))); console.log(literals.includes(text.charAt(3)));
 const owned = " A ".trim(); console.log(values.indexOf(owned));
 const sliced = "abcdefgh".slice(1, 6); const slicedValues = [sliced];
 console.log(slicedValues.includes("bcdef")); console.log(slicedValues.indexOf("bcdef")); console.log(slicedValues.includes("abcde"));
 const mixed = [text.slice(2, 6), text.charAt(2), "éX"];
 console.log(mixed.includes("😀")); console.log(mixed.indexOf("😀")); console.log(mixed.includes("éX")); console.log(mixed.includes("XY"));
 }"#,
        "0\n1\n2\n3\n4\nfalse\n2\n-1\n0\nfalse\n2\ntrue\n0\ntrue\n0\nfalse\ntrue\n0\ntrue\nfalse\n",
        None,
    );
}

#[test]
fn byte_slices_fail_at_invalid_utf8_boundaries_after_argument_evaluation() {
    for source in [
        r#"function main() { console.log("😀".slice(1, 4)); }"#,
        r#"function main() { console.log("é".slice(0, 1)); }"#,
        r#"function receiver(): string { console.log(1); return "A😀Z"; }
        function offset(n: int): int { console.log(n); return n; }
        function main() { console.log(receiver().slice(offset(2), offset(5))); }"#,
    ] {
        let dir = support::TempDirectory::new();
        let path = dir.0.join("invalid-boundary.rs");
        std::fs::write(&path, tsrust_compiler::check(source).unwrap().emit_rust()).unwrap();
        for level in ["0", "3"] {
            let binary = dir.0.join(format!("invalid-boundary-{level}"));
            success(&support::run(
                support::rustc()
                    .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                    .arg(&path)
                    .arg("-o")
                    .arg(&binary),
                60,
            ));
            let result = support::run(&mut std::process::Command::new(binary), 10);
            assert!(!result.status.success());
            assert_eq!(
                result.stdout,
                if source.contains("receiver") {
                    b"1\n2\n5\n".as_slice()
                } else {
                    b"".as_slice()
                }
            );
            assert!(
                std::string::String::from_utf8_lossy(&result.stderr).contains("UTF-8 boundaries")
            );
        }
    }
}
