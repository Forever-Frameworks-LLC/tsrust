mod support;
use std::process::Command;
use support::{TempDirectory, execute, reject, run, rustc, success};
use tsrust_compiler::{Mode, check};

#[test]
fn primitive_search_returns_exact_types_and_leaves_empty_slots_unobservable() {
    execute(
        r#"
        function main() {
            const empty: int[] = []; console.log(empty.includes(0)); console.log(empty.indexOf(0));
            const flags: boolean[] = []; console.log(flags.includes(false)); flags.push(true);
            console.log(flags.includes(false)); console.log(flags.indexOf(true));
            const values = [0, -7, 9, -7]; const found: boolean = values.includes(-7);
            const index: int = values.indexOf(-7); console.log(found); console.log(index);
            console.log(values.indexOf(-7, 2)); console.log(values.indexOf(-7, -1));
            console.log(values.includes(0, 4)); console.log(values.indexOf(0, 99));
            console.log(values.indexOf(0, -99)); console.log(values.length);
        }
    "#,
        "false\n-1\nfalse\nfalse\n0\ntrue\n1\n3\n3\nfalse\n-1\n0\n4\n",
        None,
    );
}

#[test]
fn offsets_clamp_signed_extremes_without_overflow() {
    let rust = execute(
        r#"
        function main() {
            const values = [7, 8, 7];
            console.log(values.includes(7, -9223372036854775808));
            console.log(values.indexOf(7, -9223372036854775808));
            console.log(values.includes(7, 9223372036854775807));
            console.log(values.indexOf(7, 9223372036854775807));
            console.log(values.indexOf(7, -3)); console.log(values.indexOf(7, -2));
            console.log(values.indexOf(7, -0));
            const names = ["one", "two", "one"];
            console.log(names.indexOf("one", -9223372036854775808));
            console.log(names.indexOf("one", 9223372036854775807));
        }
    "#,
        "true\n0\nfalse\n-1\n0\n2\n0\n0\n-1\n",
        None,
    );
    optimized(&rust, "true\n0\nfalse\n-1\n0\n2\n0\n0\n-1\n");
}

#[test]
fn strings_compare_contents_without_normalization_or_ownership_transfer() {
    execute(
        r#"
        function main() {
            const names = ["Pat", "pat", "😀", "\u0000", "é", "e\u0301", ""];
            const needle = "Pat"; console.log(names.indexOf(needle)); console.log(needle);
            console.log(names.indexOf(needle.toLowerCase())); console.log(names.indexOf("😀"));
            console.log(names.indexOf("\u0000")); console.log(names.indexOf("é"));
            console.log(names.indexOf("e\u0301")); console.log(names.indexOf(""));
            console.log(names.includes(" PAT ")); console.log(names.includes("😀", -5));
            const short = ["a", "a"]; console.log(short.indexOf("a", 1));
            console.log(short.indexOf("", -100));
        }
    "#,
        "0\nPat\n1\n2\n3\n4\n5\n6\nfalse\ntrue\n1\n-1\n",
        None,
    );
}

#[test]
fn mixed_receivers_and_operands_evaluate_once_in_source_order() {
    execute(
        r#"
        type Box = { rows: int[][]; words: string[]; };
        function mark(n: int): int { console.log(n); return n; }
        function word(s: string): string { console.log(s); return clone(s); }
        function main() {
            const boxes: Box[] = [{rows: [[1, 2, 3]], words: ["yes", "no"]}];
            console.log(boxes[mark(0)].rows[mark(0)].includes(mark(2), mark(-2)));
            console.log(boxes[mark(0)].words.indexOf(word("no"), mark(1)));
        }
    "#,
        "0\n0\n2\n-2\ntrue\n0\nno\n1\n1\n",
        Some(&[("mark.n", Mode::Copy), ("word.s", Mode::Read)]),
    );
}

#[test]
fn temporary_receivers_keep_independent_storage_before_argument_effects() {
    execute(
        r#"
        function make(): int[] { console.log(1); return [3, 4]; }
        function mark(n: int): int { console.log(n); return n; }
        function change(values: int[]): int { values.push(9); return 9; }
        function main() {
            console.log([1, 2].includes(2)); console.log(make().indexOf(mark(4), mark(0)));
            const values = [1]; console.log(clone(values).includes(change(values)));
            console.log(values.length); console.log(clone(values).indexOf(values.remove(0)));
            console.log(values[0]);
        }
    "#,
        "true\n1\n4\n0\n1\nfalse\n2\n0\n9\n",
        Some(&[("mark.n", Mode::Copy), ("change.values", Mode::Mutate)]),
    );
}

#[test]
fn receiver_loans_reject_mutation_transfer_and_replacement_during_arguments() {
    for operation in [
        "values.includes(change(values))",
        "values.indexOf(1, change(values))",
    ] {
        let error = reject(
            &format!(
                "function change(values: int[]): int {{ values.push(9); return 0; }} function main() {{ const values = [1]; console.log({operation}); }}"
            ),
            "conflicting-access",
        );
        assert!(!error.related.is_empty());
    }
    reject(
        "function take(values: int[]): int { const moved = values; return 0; } function main() { const values = [1]; console.log(values.includes(take(values))); }",
        "conflicting-access",
    );
    reject(
        "type Box = { rows: int[][]; }; function change(box: Box): int { box.rows[0] = [2]; return 0; } function main() { const box: Box = {rows: [[1]]}; console.log(box.rows[0].indexOf(1, change(box))); }",
        "conflicting-access",
    );
}

#[test]
fn string_needle_loans_extend_through_from_index_then_end() {
    reject(
        "function take(word: string): int { const moved = word; return 0; } function main() { const values = [\"yes\"]; const word = \"yes\"; console.log(values.indexOf(word, take(word))); }",
        "conflicting-access",
    );
    reject(
        "type Box = { word: string; other: string; }; function take(s: string): int { const moved = s; return 0; } function main() { const values = [\"yes\"]; const box: Box = {word: \"yes\", other: \"no\"}; console.log(values.includes(box.word, take(box.other))); }",
        "conflicting-access",
    );
    execute(
        "function take(s: string): string { return s; } function main() { const values = [\"yes\"]; const word = \"yes\"; console.log(values.includes(word)); console.log(take(word)); values.push(\"no\"); console.log(values.indexOf(\"no\")); }",
        "true\nyes\n1\n",
        Some(&[("take.s", Mode::Consume)]),
    );
}

#[test]
fn searches_work_through_iteration_closures_callbacks_and_owned_factories() {
    execute(
        r#"
        function query(values: string[], word: string): int { return values.indexOf(word, -2); }
        function invoke(f: (values: ReadOnly<string[]>, word: ReadOnly<string>) => int, values: string[], word: string): int { return f(values, word); }
        function make(values: int[]): () => int { return (): int => { return values.indexOf(7); }; }
        function main() {
            const rows = [[1, 2], [3, 4]]; for (const row of rows) { console.log(row.includes(4)); }
            const words = ["a", "b", "c"]; const word = "b";
            const read = (): int => { return query(words, word); }; console.log(read()); console.log(read());
            console.log(invoke((values: string[], needle: string): int => { return values.indexOf(needle); }, words, word));
            const first = make([7, 9]); const second = make([9, 7]); console.log(first()); console.log(second());
        }
    "#,
        "false\ntrue\n1\n1\n1\n0\n1\n",
        None,
    );
    reject(
        "function main() { const values = [1]; const read = (): boolean => { return values.includes(1); }; values.push(2); console.log(read()); }",
        "capture-conflict",
    );
}

#[test]
fn searches_preserve_partial_field_availability_and_readonly_specialization() {
    let rust = execute(
        r#"
        type Node = { name: string; words: string[]; children: Node[]; };
        function read(node: Node): boolean { return node.children[0].words.includes("yes"); }
        function main() {
            const node: Node = {name: "root", words: [], children: [{name: "child", words: ["yes"], children: []}]};
            console.log(read(node));
            const other: Node = {name: "second", words: ["no"], children: []};
            const name = other.name; console.log(other.words.indexOf("no")); other.name = name;
            console.log(other.name);
        }
    "#,
        "true\n0\nsecond\n",
        None,
    );
    assert!(rust.contains("FixedRecord"));
    reject(
        "type Box = { words: string[]; }; function main() { const box: Box = {words: [\"yes\"]}; const moved = box.words; console.log(box.words.includes(\"yes\")); }",
        "partial-transfer",
    );
}

#[test]
fn types_arity_and_identity_boundaries_have_source_diagnostics() {
    for expression in [
        "values.includes()",
        "values.indexOf()",
        "values.includes(1, 2, 3)",
        "values.indexOf(1, 2, 3)",
    ] {
        let error = reject(
            &format!("function main() {{ const values = [1]; console.log({expression}); }}"),
            "arity",
        );
        assert_eq!(
            error.message,
            if expression.ends_with("()") {
                "Expected 1 or 2 arguments, found 0."
            } else {
                "Expected 1 or 2 arguments, found 3."
            }
        );
    }
    for expression in [
        "values.includes(true)",
        "values.indexOf(1, false)",
        "values.includes(1, \"0\")",
    ] {
        reject(
            &format!("function main() {{ const values = [1]; console.log({expression}); }}"),
            "type",
        );
    }
    reject(
        "function main() { const values = [1]; const result: int = values.includes(1); }",
        "type",
    );
    reject(
        "function main() { const values = [1]; const result: boolean = values.indexOf(1); }",
        "type",
    );
    reject(
        "function main() { const text = \"yes\"; console.log(text.includes(1)); }",
        "type",
    );
    assert!(
        check(
            "function main() { const text = \"yes\"; const found: boolean = text.includes(\"y\"); }"
        )
        .is_ok()
    );
    reject(
        "function main() { const rows = [[1]]; console.log(rows.includes([1])); }",
        "unsupported",
    );
    reject(
        "type Item = {n: int;}; function main() { const values: Item[] = [{n: 1}]; const item: Item = {n: 1}; console.log(values.indexOf(item)); }",
        "unsupported",
    );
    assert!(check("function main() { console.log([1].includes(1)); }").is_ok());
}

#[test]
fn search_preserves_removal_growth_clone_independence_and_emptying() {
    execute(
        r#"
        function main() {
            const values = [0, 1, 2, 3]; const saved = clone(values); values.remove(0);
            console.log(values.includes(0)); console.log(saved.indexOf(0));
            values.push(9); values.push(10); console.log(values.indexOf(10));
            while (values.length > 0) { values.remove(0); } console.log(values.includes(0));
            values.push(0); console.log(values.indexOf(0));
            const words = ["a", "", "b", "c"]; words.remove(1); console.log(words.includes(""));
            words.push("d"); words.push(""); console.log(words.indexOf("", -1));
        }
    "#,
        "false\n0\n4\nfalse\n0\nfalse\n4\n",
        None,
    );
}

fn optimized(rust: &str, expected: &str) {
    let directory = TempDirectory::new();
    let path = directory.0.join("program.rs");
    std::fs::write(&path, rust).unwrap();
    for checks in ["on", "off"] {
        let binary = directory.0.join(format!("program-{checks}"));
        success(&run(
            rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg("opt-level=3")
                .arg("-C")
                .arg(format!("overflow-checks={checks}"))
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        let output = run(&mut Command::new(binary), 10);
        success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}

#[test]
fn selection_failure_precedes_search_operands_and_empty_search_evaluates_arguments() {
    let cases = [
        (
            "const rows = [[1]]; console.log(rows[mark(-1)].includes(mark(7), mark(0)));",
            "-1\n",
            "array index out of bounds",
        ),
        (
            "const values: int[] = []; console.log(values.includes(mark(7), 9223372036854775807 + 1));",
            "7\n",
            "integer addition overflow",
        ),
    ];
    for (body, expected, failure) in cases {
        let source = format!(
            "function mark(n: int): int {{ console.log(n); return n; }} function main() {{ {body} }}"
        );
        let rust = check(&source).unwrap().emit_rust();
        let directory = TempDirectory::new();
        let path = directory.0.join("failure.rs");
        std::fs::write(&path, rust).unwrap();
        for level in ["0", "3"] {
            for checks in ["on", "off"] {
                let binary = directory.0.join(format!("failure-{level}-{checks}"));
                success(&run(
                    rustc()
                        .arg("--edition=2024")
                        .arg("-A")
                        .arg("warnings")
                        .arg("-C")
                        .arg(format!("opt-level={level}"))
                        .arg("-C")
                        .arg(format!("overflow-checks={checks}"))
                        .arg(&path)
                        .arg("-o")
                        .arg(&binary),
                    60,
                ));
                let output = run(&mut Command::new(binary), 10);
                assert!(!output.status.success());
                assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
                let error = String::from_utf8(output.stderr).unwrap();
                assert!(error.contains(failure) && error.contains("source bytes"));
            }
        }
    }
}

#[test]
fn differential_search_covers_every_storage_width_and_owned_string_resources() {
    let source = r#"
        function main() { const values = [1]; const flags = [true]; const words = ["a"];
            values.push(2); flags.push(false);
            console.log(values.indexOf(1)); console.log(flags.includes(true)); console.log(words.indexOf("a")); }
    "#;
    let mut rust = check(source)
        .unwrap()
        .emit_rust()
        .replace("fn main()", "fn source_main()");
    rust.push_str(r#"
        fn expected_start(len: usize, from: i64) -> usize {
            let offset = if from < 0 { len as i128 + from as i128 } else { from as i128 };
            offset.clamp(0, len as i128) as usize
        }
        fn expected_index<T: PartialEq>(values: &[T], needle: &T, from: i64) -> i64 {
            for index in expected_start(values.len(), from)..values.len() {
                if &values[index] == needle { return index as i64; }
            }
            -1
        }
        fn main() {
            let labels = ["", "a", " A ", "é", "e\u{301}", "😀", "\0", "duplicate"];
            for width in 0..=80 {
                let mut numbers = NumericArray::from([]); let mut integer_model = Vec::new();
                let mut flags = NumericArray::from([]); let mut boolean_model = Vec::new();
                let mut words = Array::from([]); let mut string_model = Vec::new();
                for index in 0..width {
                    let number = if index % 17 == 0 { i64::MIN } else if index % 19 == 0 { i64::MAX } else { index as i64 % 7 - 3 };
                    numbers.push(number); integer_model.push(number);
                    flags.push(index % 5 == 0); boolean_model.push(index % 5 == 0);
                    words.push(String::from(labels[index % labels.len()].to_owned()));
                    string_model.push(labels[index % labels.len()].to_owned());
                }
                if width > 0 {
                    numbers.remove(width / 2); integer_model.remove(width / 2);
                    flags.remove(width / 2); boolean_model.remove(width / 2);
                    words.remove(width / 2); string_model.remove(width / 2);
                }
                for from in [i64::MIN, -100, -(width as i64), -5, -1, 0, 1, 4, width as i64, 100, i64::MAX] {
                    for needle in [i64::MIN, i64::MAX, -4, -3, 0, 3, 4] {
                        let expected = expected_index(&integer_model, &needle, from);
                        assert_eq!(numbers.tsr_index_of(&needle, from, "integer range"), expected);
                        assert_eq!(numbers.tsr_includes(&needle, from), expected >= 0);
                    }
                    for needle in [false, true] {
                        let expected = expected_index(&boolean_model, &needle, from);
                        assert_eq!(flags.tsr_index_of(&needle, from, "boolean range"), expected);
                        assert_eq!(flags.tsr_includes(&needle, from), expected >= 0);
                    }
                    for text in labels {
                        let needle = String::Owned(text.to_owned());
                        let pointer = needle.as_ptr();
                        let expected = expected_index(&string_model, &text.to_owned(), from);
                        assert_eq!(words.tsr_index_of(&needle, from, |a, b| a == b, "string range"), expected);
                        assert_eq!(words.tsr_includes(&needle, from, |a, b| a == b), expected >= 0);
                        assert_eq!(needle.as_ptr(), pointer);
                    }
                }
                let mut copy = words.clone();
                copy.push(String::from("independent".to_owned()));
                assert!(!words.tsr_includes(&String::from("independent".to_owned()), 0, |a,b| a == b));
            }
        }
    "#);
    let directory = TempDirectory::new();
    let path = directory.0.join("model.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("model-{level}"));
        success(&run(
            rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg(format!("opt-level={level}"))
                .arg("-C")
                .arg("overflow-checks=on")
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        success(&run(&mut Command::new(binary), 30));
    }
}

#[test]
fn borrowed_and_owned_search_preserves_exact_utf8_content_equality() {
    let source = r#"
        function main() {
            const words = ["é", "other", "é"];
            console.log(words.indexOf("é", 1)); console.log(words.includes("é"));
        }
    "#;
    let mut rust = check(source)
        .unwrap()
        .emit_rust()
        .replace("fn main()", "fn source_main()");
    rust.push_str(r#"
        fn main() {
            let a = String::Borrowed("é"); let b = String::Borrowed("other");
            let other = String::Borrowed("é"); let basic = String::Borrowed("é");
            let dynamic = String::Owned("é".to_owned());
            assert_eq!(a, a); assert_ne!(a, b);
            for equal in [&other, &basic, &dynamic] { assert_eq!(&a, equal); assert_eq!(equal, &a); }
            let words = Array::from([a, b]);
            for needle in [&other, &basic, &dynamic] {
                assert_eq!(words.tsr_index_of(needle, 0, |a,b| a == b, "range"), 0);
                assert!(words.tsr_includes(needle, 0, |a,b| a == b));
                assert_eq!(words.tsr_index_of(needle, 1, |a,b| a == b, "range"), -1);
            }
            let borrowed = String::Borrowed("é");
            let mut mixed = Array::from([String::Borrowed("other"), other.clone(), basic.clone(), dynamic.clone(), borrowed.clone(), String::Borrowed("other")]);
            let pointer = dynamic.as_ptr();
            for needle in [&borrowed, &other, &basic, &dynamic] {
                for (from, expected) in [(0, 1), (2, 2), (3, 3), (4, 4), (5, -1), (i64::MIN, 1), (i64::MAX, -1)] {
                    assert_eq!(mixed.tsr_index_of(needle, from, |a,b| a == b, "range"), expected);
                    assert_eq!(mixed.tsr_includes(needle, from, |a,b| a == b), expected >= 0);
                }
            }
            assert_eq!(dynamic.as_ptr(), pointer);
            mixed.get_mut(3).unwrap().to_mut().push('!');
            assert_eq!(dynamic.as_ref(), "é"); assert_eq!(mixed.remove(3).as_ref(), "é!");
            drop(mixed); assert_eq!(dynamic.as_ref(), "é");
            assert_eq!(std::mem::size_of::<String>(), std::mem::size_of::<std::borrow::Cow<'static, str>>());
        }
    "#);
    let directory = TempDirectory::new();
    let path = directory.0.join("content-search.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("content-search-{level}"));
        success(&run(
            rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg(format!("opt-level={level}"))
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        success(&run(&mut Command::new(binary), 10));
    }
}

#[test]
fn unrolled_search_preserves_exact_comparison_order_and_immediate_exit() {
    let mut rust =
        check(r#"function main() { const words = ["a"]; console.log(words.indexOf("a")); }"#)
            .unwrap()
            .emit_rust()
            .replace("fn main()", "fn source_main()");
    rust.push_str(r#"
        #[derive(Clone)]
        struct Probe { id: usize, value: i64 }
        fn main() {
            let visited = std::cell::RefCell::new(Vec::new());
            for width in 0..=80 {
                let mut array = Array::from([]); let mut model = Vec::new();
                for id in 0..width {
                    let value = id as i64 % 7 - 3;
                    array.push(Probe { id, value }); model.push(Probe { id, value });
                }
                if width > 0 { array.remove(width / 2); model.remove(width / 2); }
                for from in [i64::MIN, -99, -(width as i64), -1, 0, 1, 2, 3, 4, 7, width as i64, i64::MAX] {
                    let start = (if from < 0 { model.len() as i128 + from as i128 } else { from as i128 })
                        .clamp(0, model.len() as i128) as usize;
                    for value in -4..=4 {
                        let needle = Probe { id: usize::MAX, value };
                        let offset = model[start..].iter().position(|item| item.value == value);
                        let expected = offset.map_or(-1, |offset| (start + offset) as i64);
                        let count = offset.map_or(model.len() - start, |offset| offset + 1);
                        let trace = model[start..start + count].iter().map(|item| item.id).collect::<Vec<_>>();
                        visited.borrow_mut().clear();
                        assert_eq!(array.tsr_index_of(&needle, from, |a,b| {
                            visited.borrow_mut().push(a.id); a.value == b.value
                        }, "index model"), expected);
                        assert_eq!(*visited.borrow(), trace);
                        visited.borrow_mut().clear();
                        assert_eq!(array.tsr_includes(&needle, from, |a,b| {
                            visited.borrow_mut().push(a.id); a.value == b.value
                        }), expected >= 0);
                        assert_eq!(*visited.borrow(), trace);
                    }
                }
            }
        }
    "#);
    let directory = TempDirectory::new();
    let source = directory.0.join("comparison-order.rs");
    std::fs::write(&source, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("comparison-order-{level}"));
        success(&run(
            rustc()
                .arg("--edition=2024")
                .arg("-A")
                .arg("warnings")
                .arg("-C")
                .arg(format!("opt-level={level}"))
                .arg("-C")
                .arg("overflow-checks=on")
                .arg(&source)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        success(&run(&mut Command::new(binary), 30));
    }
}

#[test]
fn number_includes_treats_nan_as_present_while_index_of_does_not() {
    support::execute(
        r#"
function has(values: number[], value: number): boolean { return values.includes(value); }
function main() {
    const values = [1.5, Number.NaN, -0.0];
    console.log(values.includes(Number.NaN), values.indexOf(Number.NaN), values.lastIndexOf(Number.NaN));
    console.log(values.includes(0.0), values.includes(1.5), values.includes(2.5));
    console.log(values.includes(Number.NaN, 2), values.includes(Number.NaN, -2), [1.5].includes(Number.NaN));
    const grown: number[] = [];
    grown.push(Number.NaN);
    console.log(has(grown, Number.NaN), has(grown, 0.0), has([], Number.NaN));
}
"#,
        "true -1 -1\ntrue true false\nfalse true false\ntrue false false\n",
        None,
    );
}
