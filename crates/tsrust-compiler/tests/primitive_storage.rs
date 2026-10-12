// Other integration binaries exercise the remaining runtime fragments.
#[allow(dead_code)]
#[path = "../src/runtime/mod.rs"]
mod runtime;
mod support;
use support::{execute, success};

#[test]
fn inline_arrays_spill_remove_clone_and_remain_independent() {
    execute(
        r#"function main() {
 const values: int[] = []; let i = 0;
 while (i < 33) { values.push(i); i = i + 1; }
 const copy = clone(values); console.log(values.remove(0)); console.log(values.remove(15));
 console.log(values.remove(values.length - 1)); values[0] = 99;
 console.log(copy[0]); console.log(copy[16]); console.log(copy[32]);
 console.log(values.length); console.log(values[0]); console.log(values[15]);
 const small = [1, 2, 3, 4]; console.log(small.remove(1)); small.push(5); small.push(6);
 for (const value of small) { console.log(value); }
 const literal = [1, 2, 3, 4, 5]; console.log(literal.remove(2)); console.log(literal[2]);
}"#,
        "0\n16\n32\n0\n16\n32\n30\n99\n17\n2\n1\n3\n4\n5\n6\n3\n4\n",
        None,
    );
}

#[test]
fn static_and_owned_unicode_strings_survive_input_replacement() {
    execute(
        r#"function main() {
 let text = "  İΣÉ HELLO  "; const trimmed = text.trim();
 const lower = trimmed.toLowerCase(); const copy = clone(lower);
 const independent = lower.trim(); text = "replacement";
 console.log(trimmed); console.log(lower); console.log(copy); console.log(independent); console.log(text);
 const words = [" A ".trim().toLowerCase(), "lower", " B ".trim()];
 const saved = clone(words); words[0] = "changed";
 for (const word of saved) { console.log(word); }
}"#,
        "İΣÉ HELLO\ni\u{307}σé hello\ni\u{307}σé hello\ni\u{307}σé hello\nreplacement\na\nlower\nB\n",
        None,
    );
}

#[test]
fn runtime_arrays_match_vec_across_boundaries_and_drop_exactly_once() {
    let dir = support::TempDirectory::new();
    let source = format!(
        "{}\n{}\n{}",
        runtime::STRINGS,
        runtime::ARRAYS,
        r#"
use std::rc::Rc;
use std::cell::Cell;
#[derive(Clone)]
struct Value { number: usize, drops: Rc<Cell<usize>> }
impl Drop for Value { fn drop(&mut self) { self.drops.set(self.drops.get() + 1); } }
fn main() {
 let literal = String::Borrowed("  lower  ");
 assert_eq!(tsr_trim(&literal).as_ref(), "lower");
 let lower = String::Borrowed("lower");
 assert_eq!(tsr_lower(&lower).as_ref(), "lower");
 let owned = String::from("  HELLO  ".to_owned());
 let trimmed = tsr_trim(&owned); let lower = tsr_lower(&trimmed);
 drop(owned); drop(trimmed); assert_eq!(lower.as_ref(), "hello");
 let drops = Rc::new(Cell::new(0));
 let mut made = 0;
 for size in 0..40 {
  let mut array = Array::from([]); let mut model = Vec::new();
  for n in 0..size { array.push(Value { number: n, drops: drops.clone() }); model.push(n); made += 1; }
  for step in 0..size {
   assert_eq!(array.len(), model.len());
   assert_eq!(array.iter().map(|v| v.number).collect::<Vec<_>>(), model);
   assert_eq!(array.get(model.len()).map(|v| v.number), None);
   let index = (step * 7) % model.len();
   array.get_mut(index).unwrap().number += 100; model[index] += 100;
   assert_eq!(array.remove(index).number, model.remove(index));
  }
  assert_eq!(array.len(), 0); assert!(array.get(0).is_none());
  array.push(Value { number: 42, drops: drops.clone() }); made += 1;
  let copy = array.clone(); made += 1; drop(array); assert_eq!(copy.get(0).unwrap().number, 42); drop(copy);
 }
 assert_eq!(drops.get(), made);
}
"#,
    );
    let path = dir.0.join("runtime.rs");
    std::fs::write(&path, &source).unwrap();
    for optimized in [false, true] {
        let binary = dir.0.join(if optimized { "optimized" } else { "debug" });
        let mut command = support::rustc();
        command
            .arg("--edition=2024")
            .arg(&path)
            .arg("-o")
            .arg(&binary);
        if optimized {
            command.arg("-C").arg("opt-level=3");
        }
        success(&support::run(&mut command, 30));
        success(&support::run(&mut std::process::Command::new(binary), 10));
    }
}

#[test]
fn inline_record_and_nested_array_layouts_remain_bounded() {
    let rust = execute(
        r#"type Tree = { value: int; children: Tree[]; };
 type Small = { name: string; score: int; };
 type Large = { a: string; b: string; c: string; d: string; e: string; };
 function countTrees(values: Tree[]): int { return values.length; }
 function countSmall(values: Small[]): int { return values.length; }
 function countLarge(values: Large[]): int { return values.length; }
 function nested(values: int[][][]): int { return values[0][0][0]; }
 function main() {
 const tree: Tree = { value: 1, children: [{ value: 2, children: [] }] };
 console.log(countTrees(tree.children));
 const values: Small[] = [{ name: "Pat", score: 1 }];
 const saved = clone(values); values[0].name = "Sam";
 console.log(countSmall(values)); console.log(saved[0].name);
 console.log(countLarge([{ a: "a", b: "b", c: "c", d: "d", e: "e" }]));
 console.log(nested([[[7]]]));
 }"#,
        "1\n1\nPat\n1\n7\n",
        None,
    );
    assert!(rust.contains("struct RecordArray0(Vec<Record0>)"));
    assert!(rust.contains("type RecordArray1 = Array<Record1>"));
    assert!(rust.contains("type RecordArray2 = Vec<Record2>"));
    assert!(rust.contains("Vec<Array<NumericArray<i64>>>"));
    assert!(!rust.contains("unsafe"));
}

#[test]
fn record_layout_budget_accounts_for_inline_array_storage() {
    let mut source = String::from("type R0 = { values: int[][]; }; ");
    for index in 1..=25 {
        source.push_str(&format!(
            "type R{index} = {{ left: R{}; right: R{}; }}; ",
            index - 1,
            index - 1
        ));
    }
    source.push_str("function main() {}");
    let diagnostic = tsrust_compiler::check(&source)
        .err()
        .expect("inline array storage exceeds record budget");
    assert_eq!(diagnostic.code, "unsupported");
    assert!(diagnostic.message.contains("1 GiB"));
}

#[test]
fn literal_transform_paths_use_standard_utf8_operations() {
    let words = (0..256)
        .map(|n| format!("\"  LABEL {n:04} ABC XYZ  \""))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "function main() {{ const words = [{words}]; for (const word of words) {{ console.log(word.trim().toLowerCase()); }} }}"
    );
    let expected = (0..256)
        .map(|n| format!("label {n:04} abc xyz\n"))
        .collect::<String>();
    let rust = execute(&source, &expected, None);
    assert!(rust.contains("std::borrow::Cow"));
    assert!(!rust.contains("struct Literal {"));
}

#[test]
fn literal_transforms_cover_functions_closures_and_context() {
    let rust = execute(
        r#"function lower(s: string): string { return s.toLowerCase(); }
        function main() {
            const trimmed = (): string => { return "  ΟΣ  ".trim(); };
            console.log(lower("  ΟΣ  ")); console.log(trimmed());
            console.log("  İ  ".trim().toLowerCase());
            console.log("tsr_lower(".length);
        }"#,
        "  ος  \nΟΣ\ni\u{0307}\n10\n",
        None,
    );
    assert!(rust.contains("tsr_lower"));
    assert!(rust.contains("tsr_trim"));
}

#[test]
fn large_literal_pools_and_oversized_strings_preserve_unicode_context() {
    let words = (0..1100)
        .map(|n| format!("\"  LABEL {n:04} ABC XYZ  \""))
        .collect::<Vec<_>>()
        .join(",");
    let oversized = "A".repeat(70_000);
    let source = format!(
        "function main() {{ const words = [{words}]; console.log(words[0].trim().toLowerCase()); console.log(words[1099].trim().toLowerCase()); const large = \"{oversized}\"; const saved = clone(large); console.log(saved); console.log(\" ΟΣ \".trim().toLowerCase()); }}"
    );
    let rust = execute(
        &source,
        &format!("label 0000 abc xyz\nlabel 1099 abc xyz\n{oversized}\nος\n"),
        None,
    );
    assert!(rust.contains("String::Borrowed"));
    assert!(!rust.contains("struct Literal {"));
}

#[test]
fn runtime_strings_and_mixed_array_operations_match_standard_models() {
    let dir = support::TempDirectory::new();
    let path = dir.0.join("models.rs");
    let source = format!(
        "{}\n{}\n{}",
        runtime::STRINGS,
        runtime::ARRAYS,
        r#"
fn main() {
 fn transferable<T: Send + Sync>() {} transferable::<String>();
 for text in ["", " ", "\t\r\n", "already lower 0123", "  MiXeD  ", "İΣÉ", "ΟΣ", "ΟΣ Α", "𐐀ßẞ", "\u{2003}HELLO\u{a0}", "nul\0ABC", "🦀 XYZ"] {
  let original = String::from(text.to_owned());
  let clone = original.clone(); let trimmed = tsr_trim(&original);
  let lower = tsr_lower(&original); let normalized = tsr_normalize(&original);
  assert_eq!(trimmed.as_ref(), text.trim()); assert_eq!(lower.as_ref(), text.to_lowercase());
  assert_eq!(normalized.as_ref(), text.trim().to_lowercase());
  assert_eq!(tsr_lower(&trimmed).as_ref(), normalized.as_ref());
  drop(original); assert_eq!(clone.as_ref(), text); drop(clone);
  assert_eq!(normalized.clone().as_ref(), text.trim().to_lowercase());
 }
 let lower = String::from("already lower".to_owned());
 let mut normalized = tsr_normalize(&lower); normalized.to_mut().push('!');
 assert_eq!(lower.as_ref(), "already lower"); assert_eq!(normalized.as_ref(), "already lower!");
 let mut state = 0x12345678_u64;
 let mut array = Array::from([]); let mut model = Vec::new();
 std::panic::set_hook(Box::new(|_| {}));
 for step in 0..20000 {
  state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
  let index = if model.is_empty() { 0 } else { (state as usize) % model.len() };
  match (state >> 32) % 5 {
   0 | 1 => { array.push(step); model.push(step); }
   2 if !model.is_empty() => assert_eq!(array.remove(index), model.remove(index)),
   3 if !model.is_empty() => { *array.get_mut(index).unwrap() = -step; model[index] = -step; }
   _ => {
    let copy = array.clone();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| array.remove(model.len()))).is_err());
    assert_eq!(copy.iter().copied().collect::<Vec<_>>(), model);
   }
  }
  assert_eq!(array.len(), model.len()); assert_eq!(array.iter().len(), model.len());
  assert_eq!(array.iter().copied().collect::<Vec<_>>(), model);
 }
}
"#
    );
    std::fs::write(&path, source).unwrap();
    for optimization in [0, 3] {
        let binary = dir.0.join(format!("models-{optimization}"));
        success(&support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg(format!("opt-level={optimization}"))
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            30,
        ));
        success(&support::run(&mut std::process::Command::new(binary), 10));
    }
}

#[test]
fn numeric_slots_match_vec_across_boundaries_and_preserve_borrowed_slices() {
    let dir = support::TempDirectory::new();
    let path = dir.0.join("numeric-models.rs");
    let source = format!(
        "{}\n{}",
        runtime::NUMERIC_ARRAYS,
        r#"
fn check<T: Copy + Default + Eq + std::fmt::Debug>(value: impl Fn(usize) -> T) {
 fn constructor<T: Copy + Default + Eq + std::fmt::Debug, const N: usize>(input: [T; N]) {
  let array = NumericArray::from(input);
  assert_eq!(array.as_slice(), &input);
  assert_eq!(array.len(), N);
  assert!(array.get(N).is_none());
 }
 constructor::<T, 0>([]); constructor([value(0)]); constructor([value(0), value(1)]);
 constructor([value(0), value(1), value(2)]);
 constructor([value(0), value(1), value(2), value(3)]);
 constructor([value(0), value(1), value(2), value(3), value(4)]);
 for width in 0..=4 {
  let mut small = NumericArray::from([]); let mut expected = Vec::new();
  for i in 0..width { small.push(value(i)); expected.push(value(i)); }
  assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| small.remove(width))).is_err());
  assert_eq!(small.as_slice(), expected);
  for index in 0..width {
   let mut copy = small.clone(); let mut model = expected.clone();
   assert_eq!(copy.remove(index), model.remove(index));
   assert_eq!(copy.as_slice(), model); assert_eq!(small.as_slice(), expected);
   assert!(copy.get(width - 1).is_none());
  }
 }
 let mut array = NumericArray::from([]); let mut model = Vec::new();
 // Empty, every inline width, first spill, and later heap growth are observed.
 for i in 0..70 {
  let item = value(i); array.push(item); model.push(item);
  let mut saved = array.clone();
  assert_eq!(saved.as_slice(), model);
  let index = i / 2;
  *saved.get_mut(index).unwrap() = value(i + 111);
  assert_eq!(array.as_slice(), model);
  assert!(saved.get_mut(i + 1).is_none());
 }
 let mut state = 0x17345678_u64;
 for step in 0..20000 {
  state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
  let index = if model.is_empty() { 0 } else { (state as usize) % model.len() };
  match (state >> 32) % 6 {
   0 | 1 => { let item = value(step); array.push(item); model.push(item); }
   2 if !model.is_empty() => assert_eq!(array.remove(index), model.remove(index)),
   3 if !model.is_empty() => { let item = value(step + 1); array.as_mut_slice()[index] = item; model[index] = item; }
   4 => {
    let saved = array.clone();
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| array.remove(model.len()))).is_err());
    assert_eq!(saved.as_slice(), model);
   }
   _ => { while !model.is_empty() { assert_eq!(array.remove(0), model.remove(0)); } }
  }
  assert_eq!(array.len(), model.len()); assert_eq!(array.iter().len(), model.len());
  assert_eq!(array.as_slice(), model);
  assert!(array.get(model.len()).is_none());
 }
}
fn main() {
 std::panic::set_hook(Box::new(|_| {}));
 check(|i| match i % 5 { 0 => i64::MIN, 1 => i64::MAX, 2 => -1, 3 => 0, _ => i as i64 });
 check(|i| i % 3 == 0);
}
"#,
    );
    std::fs::write(&path, source).unwrap();
    for optimization in [0, 3] {
        let binary = dir.0.join(format!("numeric-models-{optimization}"));
        success(&support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg(format!("opt-level={optimization}"))
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            30,
        ));
        success(&support::run(&mut std::process::Command::new(binary), 10));
    }
}

#[test]
fn standard_normalization_covers_all_ascii_bytes_and_unicode_boundaries() {
    let directory = support::TempDirectory::new();
    let source = directory.0.join("normalization-model.rs");
    let mut rust = runtime::STRINGS.to_owned();
    rust.push_str(r#"
        fn verify(text: &str) {
            let value = String::Owned(text.to_owned());
            let trimmed = tsr_trim(&value);
            let expected = text.trim().to_lowercase();
            let normalized = tsr_normalize(&value);
            assert_eq!(normalized.as_ref(), expected);
            assert_eq!(tsr_lower(&trimmed).as_ref(), expected);
            assert_eq!(tsr_normalize(&normalized).as_ref(), expected);
            assert_eq!(value.as_ref(), text);
            drop(trimmed); drop(normalized); drop(value);

        }
        fn main() {
            for byte in 0_u8..=127 {
                for width in [0, 1, 2, 3, 4, 7, 8, 15, 16, 31, 32, 63, 64, 65, 127, 128, 129, 255, 256, 511] {
                    let mut text = std::string::String::new();
                    text.push_str(" \t");
                    for index in 0..width { text.push((if index % 3 == 0 { byte } else { b'A' + (index % 26) as u8 }) as char); }
                    text.push_str("\r\n "); verify(&text);
                }
            }
            for token in ["İ", "ΟΣ", "ΟΣ Α", "Σ\u{301}", "𐐀ßẞ", "🦀", "é", "e\u{301}", "K", "\0", "\u{2003}", "\u{a0}", "\u{feff}"] {
                for prefix in ["", "  ", "A", "\tMiXeD\u{2003}"] {
                    for suffix in ["", " ", "Z", "\u{a0}\r\n"] {
                        verify(&format!("{prefix}{token}{suffix}"));
                    }
                }
            }
        }
    "#);
    std::fs::write(&source, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("normalization-model-{level}"));
        success(&support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg(format!("opt-level={level}"))
                .arg(&source)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        success(&support::run(&mut std::process::Command::new(binary), 30));
    }
}
