mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, assertions: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("reverse.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("reverse-{level}"));
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
fn reverse_uses_live_prefixes_and_keeps_length_capacity_and_owned_elements() {
    let source = r#"
        function numbers(values: int[]) { values.reverse(); }
        function strings(values: string[]) { values.reverse(); }
        function main() {}
    "#;
    let analysis = native(
        source,
        "",
        r#"
        for len in 0..=9 {
            let mut numbers = NumericArray::from([]); let mut strings = Array::from([]);
            for n in 0..len { numbers.push(n as i64); strings.push(String::Owned(n.to_string())); }
            let numeric_capacity = match &numbers { NumericArray::Small {..} => 4, NumericArray::Heap(values) => values.capacity() };
            let owned_capacity = match &strings { Array::Small {..} => 4, Array::Heap(values) => values.capacity() };
            function0(&mut numbers); function1(&mut strings);
            assert_eq!(numbers.len(), len); assert_eq!(strings.len(), len);
            assert_eq!(numbers.iter().copied().collect::<Vec<_>>(), (0..len as i64).rev().collect::<Vec<_>>());
            assert_eq!(strings.iter().map(|value| value.as_ref()).collect::<Vec<_>>(), (0..len).rev().map(|n| n.to_string()).collect::<Vec<_>>());
            match &numbers {
                NumericArray::Small {len,values} => assert!(values[*len..].iter().all(|value| *value == 0)),
                NumericArray::Heap(values) => assert_eq!(values.capacity(), numeric_capacity),
            }
            match &strings {
                Array::Small {len,values} => { assert!(values[..*len].iter().all(Option::is_some)); assert!(values[*len..].iter().all(Option::is_none)); },
                Array::Heap(values) => assert_eq!(values.capacity(), owned_capacity),
            }
            function0(&mut numbers); assert_eq!(numbers.iter().copied().collect::<Vec<_>>(), (0..len as i64).collect::<Vec<_>>());
        }
        struct Witness(std::sync::Arc<std::sync::atomic::AtomicUsize>);
        impl Drop for Witness { fn drop(&mut self) { self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst); } }
        for len in [0, 1, 4, 5, 9] {
            let drops = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
            let mut values = Array::from([]); for _ in 0..len { values.push(Witness(drops.clone())); }
            values.reverse(); values.reverse(); assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), 0);
            drop(values); assert_eq!(drops.load(std::sync::atomic::Ordering::SeqCst), len);
        }
    "#,
    );
    assert_eq!(
        analysis.contracts().get("numbers.values"),
        Some(&Mode::Mutate)
    );
    assert_eq!(
        analysis.contracts().get("strings.values"),
        Some(&Mode::Mutate)
    );
}

#[test]
fn fixed_scalar_reverse_and_owned_string_record_nested_clones_remain_independent() {
    let source = r#"
        type Node = { label: string; children: Node[]; };
        function edit(values: int[]) { values.reverse(); }
        function forward(values: int[]) { edit(values); }
        function main() {
            const empty: int[] = []; empty.reverse(); console.log(empty.length);
            const numbers = [1,2,3,4]; forward(numbers); console.log(numbers[0]); console.log(numbers[3]);
            const copy = clone(numbers); copy.reverse(); console.log(copy[0]); console.log(numbers[0]);
            const flags = [true,false,false,true,true]; flags.reverse(); console.log(flags[0]); console.log(flags[3]);
            const literals = ["a", "b"]; literals.reverse(); console.log(literals[0]);
            const strings = ["a", "ß".toUpperCase(), "😀", "z", "last"];
            const stringCopy = clone(strings); stringCopy.reverse(); stringCopy[0] = "changed";
            console.log(strings[0]); console.log(strings[4]); console.log(stringCopy[4]);
            const rows = [[1,2], [3,4,5], []]; const saved = clone(rows);
            rows.reverse(); rows[1].reverse(); rows[1][0] = 99;
            console.log(rows[0].length); console.log(rows[1][0]); console.log(saved[1][0]);
            const nodes: Node[] = [
                {label:"a",children:[{label:"A",children:[]}]},
                {label:"b",children:[{label:"B",children:[]}]},
                {label:"c",children:[{label:"C",children:[]}]}
            ];
            const nodeCopy = clone(nodes); nodeCopy.reverse(); nodeCopy[0].children[0].label = "changed";
            console.log(nodeCopy[0].label); console.log(nodes[2].children[0].label); console.log(nodes.length);
        }
    "#;
    let analysis = native(
        source,
        "0\n4\n1\n1\n4\ntrue\nfalse\nb\na\nlast\na\n0\n99\n3\nc\nC\n3\n",
        "",
    );
    assert_eq!(analysis.contracts().get("edit.values"), Some(&Mode::Mutate));
    assert_eq!(
        analysis.contracts().get("forward.values"),
        Some(&Mode::Mutate)
    );
    let emitted = analysis.emit_rust();
    // Width-preserving primitive mutation retains the private fixed ABI;
    // mutable literal strings must use the ordinary owned representation.
    assert!(emitted.contains(": [i64; 4] = ["));
    assert!(emitted.contains(": &mut [i64; 4]"));
    assert!(!emitted.contains("LiteralStringArray"));
}

#[test]
fn reverse_receiver_indices_run_once_and_mutable_capture_contracts_are_preserved() {
    let source = r#"
        type Box = { rows: int[][]; text: string; };
        function index(): int { console.log("index"); return 0; }
        function flipTwice(box: Box) {
            const flip = (): int => { box.rows[0].reverse(); return box.rows[0][0]; };
            console.log(flip()); console.log(flip());
        }
        function main() {
            const box: Box = {rows:[[1,2,3,4,5]],text:"kept"};
            box.rows[index()].reverse(); console.log(box.rows[0][0]);
            flipTwice(box);
            console.log(box.text);
        }
    "#;
    let analysis = native(source, "index\n5\n1\n5\nkept\n", "");
    assert_eq!(analysis.closures()[0].invocation, Mode::Mutate);
    assert_eq!(analysis.closures()[0].captures[0].access, Mode::Mutate);
}

#[test]
fn reverse_keeps_arity_types_void_results_source_loans_and_failure_order() {
    // Match push/remove: arity first, then named-storage eligibility, then type.
    for expression in [
        "1.reverse()",
        "true.reverse()",
        "\"x\".reverse()",
        "[1].reverse()",
    ] {
        assert_eq!(
            check(&format!("function main() {{ {expression}; }}"))
                .err()
                .unwrap()
                .code,
            "unsupported"
        );
    }
    for value in ["1", "true", "\"x\""] {
        let source = format!("function main() {{ const value = {value}; value.reverse(); }}");
        assert_eq!(check(&source).err().unwrap().code, "type");
    }
    for (statement, code) in [
        ("values.reverse(1);", "arity"),
        ("console.log(values.reverse());", "type"),
        ("const returned = values.reverse();", "type"),
    ] {
        let source = format!("function main() {{ const values = [1,2]; {statement} }}");
        let error = check(&source).err().unwrap();
        assert_eq!(error.code, code);
        if statement.starts_with("const returned") {
            assert_eq!(
                error.message,
                "Cannot bind a function with no return value."
            );
        }
    }
    let enclosing = "function mutate(values: int[]): int { values.reverse(); return 0; } function hold(values: int[], n: int) {} function main() { const values = [1,2]; hold(values, mutate(values)); }";
    assert_eq!(check(enclosing).err().unwrap().code, "conflicting-access");
    let iteration = "function main() { const values = [[1,2],[3]]; for (const row of values) { row.reverse(); } }";
    assert_eq!(check(iteration).err().unwrap().code, "borrowed-element");
    let moved = "function main() { const values = [1,2]; const moved = values; values.reverse(); }";
    assert_eq!(check(moved).err().unwrap().code, "transferred");
    // The first index runs before selecting its named root. Mutation there is
    // permitted, and the selected row must come from the updated parent.
    let receiver = "function choose(values: int[][]): int { console.log(\"choose\"); values.reverse(); return 0; } function main() { const values = [[1,2],[3,4]]; values[choose(values)].reverse(); console.log(values[0][0]); console.log(values[0][1]); console.log(values[1][0]); }";
    native(receiver, "choose\n4\n3\n1\n", "");
    // Once a parent has been selected, a later index cannot mutate it.
    let selected = "function choose(values: int[][][]): int { values.reverse(); return 0; } function main() { const values = [[[1,2]],[[3,4]]]; values[0][choose(values)].reverse(); }";
    assert_eq!(check(selected).err().unwrap().code, "conflicting-access");
    let source = "function index(): int { console.log(\"index\"); return -1; } function main() { const rows = [[1,2],[3,4]]; rows[index()].reverse(); }";
    let rust = check(source).unwrap().emit_rust();
    let directory = support::TempDirectory::new();
    let path = directory.0.join("invalid.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("invalid-{level}"));
        support::success(&support::run(
            support::rustc()
                .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(executable), 10);
        assert!(!output.status.success());
        assert_eq!(output.stdout, b"index\n");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("array index out of bounds at source bytes")
        );
    }
}
