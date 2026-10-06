mod support;
use std::process::Command;
use support::{TempDirectory, run, rustc, success};
use tsrust_compiler::check;

fn execute(source: &str, expected: &str) -> String {
    let rust = check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)))
        .emit_rust();
    let directory = TempDirectory::new();
    let path = directory.0.join("program.rs");
    std::fs::write(&path, &rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("program-{level}"));
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
        let output = run(&mut Command::new(binary), 30);
        success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
    rust
}

#[test]
fn literal_string_slices_preserve_borrows_forwarding_and_owned_clones() {
    let rust = execute(
        r#"
        function count(values: string[], depth: int): int {
            if (depth > 0) { return forward(values, depth - 1); }
            let total = 0; for (const value of values) { total = total + 1; }
            return total + values.indexOf("Å") + values.indexOf(values[2], -2);
        }
        function forward(values: string[], depth: int): int { return count(values, depth); }
        function duplicate(values: string[]): string[] { return clone(values); }
        function main() {
            const values = ["", "Å", "a", "Å", "😀"];
            console.log(count(values, 4));
            const copy = duplicate(values); copy[1] = "changed"; copy.push("tail");
            console.log(values[1]); console.log(copy[1]); console.log(values.length);
            console.log(values.includes("Å", -2)); console.log(copy.length);
            const empty: string[] = []; console.log(empty.length); console.log(empty.indexOf(""));
        }
    "#,
        "5\nÅ\nchanged\n5\ntrue\n6\n0\n-1\n",
    );
    assert!(rust.contains("LiteralStringArray::new"));
    assert!(rust.contains("literal_iter()"));
    assert!(rust.contains("fixed_function"));
    assert!(rust.contains("clone_owned()"));
}

#[test]
fn literal_string_slice_proof_rejects_moves_mutation_captures_and_dynamic_values() {
    for (source, expected) in [
        (
            r#"function main() { const values = ["a"]; values.push("b"); console.log(values.length); }"#,
            "2\n",
        ),
        (
            r#"function main() { const values = ["a"]; values[0] = "b"; console.log(values[0]); }"#,
            "b\n",
        ),
        (
            r#"function main() { let values = ["a"]; values = ["b"]; console.log(values[0]); }"#,
            "b\n",
        ),
        (
            r#"function make(flag: boolean): string[] { const values = ["a"]; if (flag) { return values; } return ["b"]; } function main() { const result = make(true); console.log(result[0]); }"#,
            "a\n",
        ),
        (
            r#"function main() { const values = ["a"]; const read = (): int => { return values.length; }; console.log(read()); }"#,
            "1\n",
        ),
        (
            r#"function make(): string { console.log(7); return "a"; } function main() { const values = [make()]; console.log(values[0]); }"#,
            "7\na\n",
        ),
        (
            r#"function edit(values: string[]) { values.push("b"); } function main() { const values = ["a"]; edit(values); console.log(values.length); }"#,
            "2\n",
        ),
    ] {
        // Some legal temporary syntax is deliberately left to ordinary storage.
        let rust = execute(source, expected);
        assert!(!rust.contains("LiteralStringArray::new"), "{source}");
    }
}

#[test]
fn literal_string_slice_budgets_fail_closed() {
    let items = vec!["\"a\""; 510].join(",");
    let source =
        format!("function main() {{ const values = [{items}]; console.log(values.length); }}");
    assert!(!execute(&source, "510\n").contains("LiteralStringArray::new"));
    let readers = (0..70)
        .map(|i| format!("function read{i}(values: string[]): int {{ return values.length; }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let calls = (0..70)
        .map(|i| format!("console.log(read{i}(values));"))
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!("{readers} function main() {{ const values = [\"a\"]; {calls} }}");
    assert!(!execute(&source, &"1\n".repeat(70)).contains("LiteralStringArray::new"));
}

#[test]
fn literal_string_slices_match_utf8_content_model_and_signed_offsets() {
    let mut rust =
        check(r#"function main() { const values = ["Å"]; console.log(values.includes("Å")); }"#)
            .unwrap()
            .emit_rust()
            .replace("fn main()", "fn source_main()");
    rust.push_str(
        r#"
        static EMPTY: &str = "";
        static WORD: &str = "Å";
        static EMOJI: &str = "😀";
        static FOREIGN: &str = "Å";
        static BASIC: &str = "Å";
        fn main() {
    "#,
    );
    for name in ["UNIFORM", "MIXED", "BASIC_VALUES"] {
        let items = (0..83)
            .map(|i| match i % 4 {
                0 => "EMPTY",
                1 => "WORD",
                2 => "EMOJI",
                _ => match name {
                    "UNIFORM" => "WORD",
                    "MIXED" => "FOREIGN",
                    _ => "BASIC",
                },
            })
            .collect::<Vec<_>>()
            .join(",");
        rust.push_str(&format!("static {name}: [&str; 83] = [{items}];\n"));
    }
    rust.push_str(r#"
        for values in [&UNIFORM[..], &MIXED[..], &BASIC_VALUES[..]] {
            for width in 0..=83 {
                let view = LiteralStringArray::new(&values[..width]);
                assert_eq!(view.len(), width);
                assert!(view.get(width).is_none());
                for from in [i64::MIN, -100, -(width as i64), -5, -1, 0, 1, 4, width as i64, 100, i64::MAX] {
                    let start = if from < 0 { width.saturating_sub(usize::try_from(from.unsigned_abs()).unwrap_or(usize::MAX)) } else { width.min(usize::try_from(from).unwrap_or(usize::MAX)) };
                    for needle in [String::Borrowed(EMPTY), String::Borrowed(WORD), String::Borrowed(EMOJI), String::Borrowed(FOREIGN), String::Borrowed(BASIC), String::from("Å".to_owned()), String::from("missing".to_owned())] {
                        let expected = values[start..width].iter().position(|item| *item == needle.as_ref()).map_or(-1, |offset| (start + offset) as i64);
                        assert_eq!(view.tsr_index_of(&needle, from, |a,b| a == b, "range"), expected);
                        assert_eq!(view.tsr_includes(&needle, from, |a,b| a == b), expected >= 0);
                    }
                }
                let mut copy = view.clone_owned();
                assert_eq!(copy.len(), width);
                for index in 0..width { assert_eq!(copy.get(index).unwrap().as_ref(), view.get(index).unwrap().as_ref()); }
                if width > 0 { *copy.get_mut(0).unwrap() = String::from("changed".to_owned()); assert_eq!(view.get(0).unwrap().as_ref(), ""); copy.remove(0); }
                copy.push(String::from("tail".to_owned()));
                assert_eq!(view.len(), width);
            }
        }
        }
    "#);
    let directory = TempDirectory::new();
    let path = directory.0.join("literal-slices.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("literal-slices-{level}"));
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
fn literal_string_slices_preserve_basic_literals_with_large_source_pools() {
    let large = "x".repeat(20_000);
    let source = format!(
        r#"function main() {{ const values = ["{large}", "{large}", "small"]; console.log(values.indexOf("{large}")); console.log(values.indexOf("{large}", 1)); console.log(values.includes("small")); console.log(values.length); }}"#
    );
    let rust = execute(&source, "0\n1\ntrue\n3\n");
    assert!(rust.contains("LiteralStringArray::new"));
    assert!(!rust.contains("struct Literal {"));
}

#[test]
fn literal_string_slice_bounds_fail_before_later_operands() {
    for index in ["-1", "1", "9223372036854775807"] {
        let source = format!(
            r#"function later(): int {{ console.log(99); return 0; }} function main() {{ const values = ["a"]; console.log(7); console.log(values.includes(values[{index}], later())); }}"#
        );
        let rust = check(&source).unwrap().emit_rust();
        assert!(rust.contains("LiteralStringArray::new"));
        let access = format!("values[{index}]");
        let start = source.find(&access).unwrap();
        let failure = format!(
            "array index out of bounds at source bytes {start}..{}",
            start + access.len()
        );
        let directory = TempDirectory::new();
        let path = directory.0.join("literal-bounds.rs");
        std::fs::write(&path, rust).unwrap();
        for level in ["0", "3"] {
            let binary = directory.0.join(format!("literal-bounds-{level}"));
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
            let output = run(&mut Command::new(binary), 10);
            assert!(!output.status.success());
            assert_eq!(String::from_utf8(output.stdout).unwrap(), "7\n");
            assert!(String::from_utf8(output.stderr).unwrap().contains(&failure));
        }
    }
}

#[test]
fn fixed_fields_preserve_every_payload_and_independent_owned_clones() {
    let rust = execute(
        r#"
        type Node = { value: int; rows: int[][]; text: string; children: Node[]; };
        type Group = { name: string; left: Node[]; right: Node[]; count: int; };
        function rows(nodes: Node[]): int { let sum = 0; for (const node of nodes) { sum = sum + node.value + node.rows[0][1]; } return sum; }
        function duplicate(group: Group): Group { return clone(group); }
        function main() {
            const group: Group = {right: [], count: 19, name: " hello ", left: [
                {text: "alpha", children: [], rows: [[1, 2]], value: 3},
                {value: 5, rows: [[7, 11]], children: [], text: "beta"}]};
            const copy = duplicate(group); copy.left[0].rows[0][1] = 100;
            const children = clone(group.left); children[1].value = 17;
            console.log(rows(group.left)); console.log(rows(copy.left));
            console.log(rows(children)); console.log(group.name.trim());
            console.log(group.left[1].text); console.log(group.count + group.right.length);
        }
    "#,
        "21\n119\n33\nhello\nbeta\n19\n",
    );
    assert!(rust.contains("struct FixedRecord"));
    assert!(rust.contains("clone_owned()"));
}

#[test]
fn readonly_specialization_handles_recursive_and_multiple_parameter_calls() {
    let rust = execute(
        r#"
        type Node = { value: int; children: Node[]; };
        function walk(node: Node, depth: int): int { if (depth === 0) { return node.value + node.children.length; } return forward(node, depth - 1); }
        function forward(node: Node, depth: int): int { return walk(node, depth); }
        function compare(left: Node, right: Node): int { return walk(left, 3) + forward(right, 2); }
        function main() {
            const left: Node = {value: 7, children: [{value: 2, children: []}]};
            const right: Node = {children: [], value: 11};
            console.log(compare(left, right)); console.log(left.children[0].value);
        }
    "#,
        "19\n2\n",
    );
    assert!(rust.contains("fixed_function"));
    assert!(rust.contains("[Record0; 0]"));
    assert!(rust.contains("[Record0; 1]"));
}

#[test]
fn dynamic_elements_preserve_constructor_order_and_failure_locations() {
    let rust = execute(
        r#"
        type Node = { value: int; children: Node[]; };
        function make(value: int): Node { console.log(value); return {children: [], value: value}; }
        function main() {
            const root: Node = {children: [make(10), make(20)], value: 30};
            console.log(root.children[0].value + root.children[1].value + root.value);
        }
    "#,
        "10\n20\n60\n",
    );
    assert!(rust.contains("FixedRecord"));
    let source = r#"
        type Node = { value: int; children: Node[]; };
        function make(value: int): Node { console.log(value); return {children: [], value: value}; }
        function main() { const root: Node = {children: [make(10), make(20)], value: 30}; console.log(root.children[-1].value); }
    "#;
    let analysis = check(source).unwrap();
    let rust = analysis.emit_rust();
    let start = source.find("root.children[-1]").unwrap();
    let failure = format!(
        "array index out of bounds at source bytes {start}..{}",
        start + "root.children[-1]".len()
    );
    assert!(rust.contains(&failure));
    let directory = TempDirectory::new();
    let path = directory.0.join("failure.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("failure-{level}"));
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
        let output = run(&mut Command::new(binary), 10);
        assert!(!output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "10\n20\n");
        assert!(String::from_utf8(output.stderr).unwrap().contains(&failure));
    }
}

#[test]
fn mutation_rebinding_capture_and_escape_retain_owned_fallback() {
    let prefix = "type Node = { value: int; children: Node[]; };";
    for (body, expected) in [
        (
            "function main() { const node: Node = {value: 1, children: []}; node.value = 2; console.log(node.value); }",
            "2\n",
        ),
        (
            "function main() { const node: Node = {value: 1, children: []}; node.children.push({value: 2, children: []}); console.log(node.children.length); }",
            "1\n",
        ),
        (
            "function main() { let node: Node = {value: 1, children: []}; node = {value: 2, children: []}; console.log(node.value); }",
            "2\n",
        ),
        (
            "function main() { const node: Node = {value: 1, children: []}; const read = (): int => { return node.value; }; console.log(read()); }",
            "1\n",
        ),
        (
            "function make(): Node { const node: Node = {value: 1, children: []}; return node; } function main() { const result = make(); console.log(result.value); }",
            "1\n",
        ),
        (
            "function main() { const node: Node = {value: 1, children: []}; const children = node.children; children.push({value: 2, children: []}); console.log(children.length); console.log(node.value); }",
            "1\n1\n",
        ),
        (
            "function edit(node: Node) { node.value = 3; } function main() { const node: Node = {value: 1, children: []}; edit(node); console.log(node.value); }",
            "3\n",
        ),
    ] {
        let rust = execute(&format!("{prefix} {body}"), expected);
        assert!(!rust.contains("FixedRecord"), "{body}");
    }
}

#[test]
fn moves_on_exited_branches_and_owned_length_fields_do_not_escape_proof() {
    let rust = execute(
        r#"
        type Node = { length: string; children: Node[]; };
        function choose(flag: boolean): string {
            const node: Node = {length: "value", children: []};
            if (flag) { return node.length; } return "other";
        }
        function main() { console.log(choose(true)); console.log(choose(false)); }
    "#,
        "value\nother\n",
    );
    assert!(!rust.contains("FixedRecord"));
}

#[test]
fn large_literals_and_specialization_growth_fall_back_without_type_changes() {
    let children = (0..200)
        .map(|i| format!("{{value: {i}, children: []}}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "type Node = {{value: int; children: Node[];}}; function main() {{ const node: Node = {{value: 7, children: [{children}]}}; console.log(node.children.length); }}"
    );
    assert!(!execute(&source, "200\n").contains("FixedRecord"));
    let functions = (0..70).map(|i| format!("function make{i}(): int {{ const node: Node = {{value: {i}, children: []}}; return read(node); }}")).collect::<Vec<_>>().join("\n");
    // Different callees exceed the variant budget even when their shapes match.
    let functions = functions.replace("return read(node)", "return read0(node)");
    let readers = (0..70)
        .map(|i| format!("function read{i}(node: Node): int {{ return node.value; }}"))
        .collect::<Vec<_>>()
        .join("\n");
    let functions = functions
        .lines()
        .enumerate()
        .map(|(i, f)| f.replace("read0(node)", &format!("read{i}(node)")))
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "type Node = {{value: int; children: Node[];}}; {readers} {functions} function main() {{ console.log(make69()); }}"
    );
    assert!(!execute(&source, "69\n").contains("FixedRecord"));
}

#[test]
fn fixed_root_keeps_iterative_cleanup_for_deep_dynamic_descendants() {
    let rust = execute(
        r#"
        type Node = { value: int; children: Node[]; };
        function make(n: int): Node {
            let result: Node = {value: 0, children: []}; let i = 0;
            while (i < n) { result = {value: i, children: [result]}; i = i + 1; }
            return result;
        }
        function duplicate(node: Node): Node { return clone(node); }
        function main() {
            const root: Node = {value: 7, children: [make(100000)]};
            const copy = duplicate(root); copy.children[0].value = 5;
            console.log(root.children[0].value); console.log(copy.children[0].value);
        }
    "#,
        "99999\n5\n",
    );
    assert!(rust.contains("FixedRecord"));
}

#[test]
fn fixed_storage_releases_dynamic_strings_and_preserves_mutable_clone_storage() {
    let source = r#"
        type Node = { text: string; numbers: int[]; children: Node[]; };
        function main() {
            const root: Node = {text: "root", numbers: [1], children: [{text: "child", numbers: [2], children: []}]};
            console.log(root.numbers[0] + root.children[0].numbers[0]);
        }
    "#;
    let mut rust = check(source)
        .unwrap()
        .emit_rust()
        .replace("fn main()", "fn source_main()");
    rust.push_str(r#"
        fn main() {
            let shared = std::string::String::from("dynamic payload");
            let root = FixedRecord0 {
                field0: String::Owned(shared.clone()), field1: NumericArray::from([1]),
                field2: [Record0 { field0: String::Owned(shared.clone()), field1: NumericArray::from([2]), field2: RecordArray0::from([]) }],
            };
            assert_eq!(shared, "dynamic payload");
            let mut copy = root.clone_owned();
            assert_ne!(copy.field0.as_ptr(), root.field0.as_ptr());
            copy.field0.to_mut().push('!'); assert_eq!(root.field0.as_ref(), "dynamic payload");
            let mut children = RecordArray0::from(root.field2.clone());
            assert_eq!(shared, "dynamic payload");
            *copy.field1.get_mut(0).unwrap() = 17;
            *copy.field2[0].field1.get_mut(0).unwrap() = 19;
            *children[0].field1.get_mut(0).unwrap() = 23;
            assert_eq!(*root.field1.get(0).unwrap(), 1);
            assert_eq!(*root.field2[0].field1.get(0).unwrap(), 2);
            assert_eq!(*copy.field2[0].field1.get(0).unwrap(), 19);
            assert_eq!(*children[0].field1.get(0).unwrap(), 23);
            drop(children); assert_eq!(shared, "dynamic payload");
            drop(copy); assert_eq!(shared, "dynamic payload");
            drop(root); assert_eq!(shared, "dynamic payload");
        }
    "#);
    let directory = TempDirectory::new();
    let path = directory.0.join("resources.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("resources-{level}"));
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
fn deeply_nested_transitive_proofs_fall_back_before_native_stack_exhaustion() {
    let functions = (0..8)
        .map(|index| {
            let result = if index == 7 {
                "node.value".to_owned()
            } else {
                format!("read{}(node)", index + 1)
            };
            let expression = format!("{}{}{}", "(1 + ".repeat(10), result, ")".repeat(10));
            format!("function read{index}(node: Node): int {{ return {expression}; }}")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let source = format!(
        "type Node = {{value: int; children: Node[];}}; {functions} function main() {{ const node: Node = {{value: 7, children: []}}; console.log(read0(node)); }}"
    );
    assert!(!execute(&source, "87\n").contains("FixedRecord"));
}

#[test]
fn scalar_literals_use_rust_arrays_with_runtime_items_search_and_independent_clones() {
    let rust = execute(
        r#"
        function item(value: int): int { console.log(value); return value; }
        function main() {
            const values = [item(3), item(1), item(3)];
            const flags = [true, false, values[0] === 3];
            console.log(values.length); let sum = 0;
            for (const value of values) { sum = sum + value; } console.log(sum);
            console.log(values.includes(3, -1)); console.log(values.indexOf(3, 1));
            console.log(flags.includes(false)); console.log(flags.indexOf(true, 1));
            const copy = clone(values); copy[1] = 9;
            console.log(values[1]); console.log(copy[1]);
            const copiedFlags = clone(flags); copiedFlags[0] = false;
            console.log(flags[0]); console.log(copiedFlags[0]);
        }
    "#,
        "3\n1\n3\n3\n7\ntrue\n2\ntrue\n2\n1\n9\ntrue\nfalse\n",
    );
    assert!(rust.contains(": [i64; 3] = ["));
    assert!(rust.contains(": [bool; 3] = ["));
    assert!(rust.contains(": [i64; 3] = Clone::clone("));
    assert!(rust.contains(": [bool; 3] = Clone::clone("));
    assert!(rust.contains("search_array[tsr_search_start("));
}

#[test]
fn scalar_array_proof_follows_stable_readers_and_indexed_mutation_calls() {
    let rust = execute(
        r#"
        function edit(values: int[], delta: int) { values[1] = values[0] + delta; }
        function forward(values: int[], delta: int) { edit(values, delta); }
        function read(values: int[], depth: int): int {
            if (depth > 0) { return read(values, depth - 1); }
            let sum = 0; for (const value of values) { sum = sum + value; }
            return sum + values.indexOf(9);
        }
        function flip(flags: boolean[]) { flags[0] = !flags[1]; }
        function main() {
            const values = [2, 4, 8]; forward(values, 7); console.log(read(values, 2));
            const flags = [false, false]; flip(flags); console.log(flags[0]);
        }
    "#,
        "20\ntrue\n",
    );
    assert!(rust.contains(": [i64; 3] = ["));
    assert!(rust.contains(": &mut [i64; 3]"));
    assert!(rust.contains(": &[i64; 3]"));
    assert!(rust.contains(": &mut [bool; 2]"));
    // Top-level record literals reuse the existing read-only fixed-array shape.
    let rust = execute(
        r#"
        type Item = { value: int; };
        function sum(items: Item[]): int { let result = 0; for (const item of items) { result = result + item.value; } return result; }
        function main() { const items: Item[] = [{ value: 2 }, { value: 5 }]; console.log(sum(items) + items[0].value); }
    "#,
        "9\n",
    );
    assert!(rust.contains(": [Record0; 2] = ["));
}

#[test]
fn scalar_array_proof_rejects_resize_rebind_moves_returns_and_captures() {
    for (source, expected) in [
        (
            "function main() { const values = [1,2,3]; values.push(4); console.log(values.length); }",
            "4\n",
        ),
        (
            "function main() { const values = [1,2,3]; console.log(values.remove(1)); console.log(values.length); }",
            "2\n2\n",
        ),
        (
            "function main() { let values = [1,2,3]; values = [4,5]; console.log(values.length); }",
            "2\n",
        ),
        (
            "function grow(values: int[]) { values.push(4); } function main() { const values = [1,2,3]; grow(values); console.log(values.length); }",
            "4\n",
        ),
        (
            "function grow(values: int[]) { values.push(4); } function relay(values: int[]) { grow(values); } function main() { const values = [1,2,3]; relay(values); console.log(values.length); }",
            "4\n",
        ),
        (
            "function replace(values: int[]): int[] { const moved = values; return [4,5]; } function relay(values: int[]): int[] { return replace(values); } function main() { let values = [1,2,3]; values = relay(values); console.log(values.length); }",
            "2\n",
        ),
        (
            "function make(): int[] { const values = [1,2,3]; return values; } function main() { const result = make(); console.log(result.length); }",
            "3\n",
        ),
        (
            "function main() { const values = [1,2,3]; const moved = values; console.log(moved.length); }",
            "3\n",
        ),
        (
            "function main() { const values = [1,2,3]; const read = (): int => { return values.length; }; console.log(read()); }",
            "3\n",
        ),
    ] {
        let rust = execute(source, expected);
        assert!(!rust.contains(": [i64; 3] = ["), "{source}");
    }
    // A clone that resizes stays in ordinary storage while its stable source
    // can remain fixed; cloning still produces independent owned elements.
    let rust = execute(
        "function main() { const values = [1,2,3]; const copy = clone(values); copy.push(4); console.log(values.length); console.log(copy.length); }",
        "3\n4\n",
    );
    assert!(rust.contains(": [i64; 3] = ["));
    assert!(rust.contains("NumericArray::from(Clone::clone("));
}

#[test]
fn scalar_arrays_keep_empty_lengths_and_search_offsets_exact() {
    let rust = execute(
        r#"
        function main() {
            const empty: int[] = []; const flags: boolean[] = [];
            console.log(empty.length); console.log(empty.includes(1)); console.log(empty.indexOf(1));
            console.log(flags.length); console.log(flags.includes(true));
            const values = [1, 2, 1];
            console.log(values.indexOf(1, -100)); console.log(values.indexOf(1, -1));
            console.log(values.indexOf(1, 3)); console.log(values.includes(1, 100));
        }
    "#,
        "0\nfalse\n-1\n0\nfalse\n0\n2\n-1\nfalse\n",
    );
    assert!(rust.contains(": [i64; 0] = []"));
    assert!(rust.contains(": [bool; 0] = []"));
    // The receiver snapshot is independently owned before the later operands
    // mutate the fixed source array; each operand executes once in order.
    let rust = execute(
        r#"
        function needle(values: int[]): int { console.log(1); values[0] = 9; return 9; }
        function start(values: int[]): int { console.log(2); values[1] = 7; return 1; }
        function main() {
            const values = [1,2,3];
            console.log(clone(values).includes(needle(values), start(values)));
            console.log(clone(values).indexOf(needle(values), start(values)));
            console.log(values[0]); console.log(values[1]);
        }
    "#,
        "1\n2\nfalse\n1\n2\n-1\n9\n7\n",
    );
    assert!(rust.contains(": [i64; 3] = ["));
    assert!(rust.contains("NumericArray::from(Clone::clone("));
    // A named receiver is loaned across its operands by the source checker,
    // including length-preserving writes. Fixed storage keeps that contract.
    for operation in [
        "values.includes(change(values))",
        "values.includes(1, change(values))",
        "values.indexOf(change(values))",
        "values.indexOf(1, change(values))",
    ] {
        let source = format!(
            "function change(values: int[]): int {{ values[0] = 9; return 0; }} function main() {{ const values = [1,2,3]; console.log({operation}); }}"
        );
        let error = match check(&source) {
            Err(error) => error,
            Ok(_) => panic!("mutating search receiver unexpectedly accepted"),
        };
        assert_eq!(error.code, "conflicting-access");
        assert!(!error.related.is_empty());
    }
}

#[test]
fn scalar_array_budget_counts_raw_elements_and_fails_closed() {
    let ints = vec!["1"; 513].join(",");
    let source =
        format!("function main() {{ const values = [{ints}]; console.log(values.length); }}");
    assert!(!execute(&source, "513\n").contains(": [i64; 513]"));
    let bools = vec!["true"; 4097].join(",");
    let source =
        format!("function main() {{ const values = [{bools}]; console.log(values.length); }}");
    assert!(!execute(&source, "4097\n").contains(": [bool; 4097]"));
}

#[test]
fn fixed_scalar_index_writes_preserve_operand_order_and_checked_bounds() {
    let source = r#"
        function operand(value: int): int { console.log(value); return value; }
        function main() {
            const values = [1,2,3]; values[operand(-1)] = operand(9);
        }
    "#;
    let rust = check(source).unwrap().emit_rust();
    assert!(rust.contains(": [i64; 3] = ["));
    let directory = TempDirectory::new();
    let path = directory.0.join("bounds.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("bounds-{level}"));
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
        let output = run(&mut Command::new(binary), 30);
        assert!(!output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "-1\n");
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("array index out of bounds at source bytes")
        );
    }
}
