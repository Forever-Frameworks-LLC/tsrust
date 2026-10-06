mod support;
use tsrust_compiler::{Mode, check};

fn execute(source: &str, expected: &str, read_parameters: &[&str]) -> String {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let contracts = analysis.contracts();
    for name in read_parameters {
        assert_eq!(contracts.get(*name), Some(&Mode::Read));
    }
    let rust = analysis.emit_rust();
    let directory = support::TempDirectory::new();
    let path = directory.0.join("comparison.rs");
    std::fs::write(&path, &rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("comparison-{level}"));
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
    rust
}

#[test]
fn standard_string_equality_and_order_cover_unicode_nul_prefixes_and_empty_values() {
    let rust = execute(
        r#"
        function main() {
            const a = "alpha"; const b = "alpha";
            console.log(a === b); console.log(a !== "Alpha"); console.log(a < "beta");
            console.log(a <= b); console.log("beta" > a); console.log(a >= b);
            console.log("" < a); console.log("a" < "aa"); console.log("a\u0000" > "a");
            console.log("é" === "e\u0301"); console.log("é" > "e"); console.log("😀" === "😀");
            console.log("\uE000" < "😀"); console.log("😀" > "\uE000");
            console.log(a); console.log(b);
        }
    "#,
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\nalpha\nalpha\n",
        &[],
    );
    assert!(!rust.contains("Clone::clone("));
    assert!(rust.contains("let string_left: &String ="));
}

#[test]
fn comparison_operands_evaluate_once_in_order_and_keep_independent_temporaries() {
    execute(
        r#"
        type Box = { text: string; };
        function operand(label: string): string { console.log(label); return label; }
        function rewrite(box: Box): string { box.text = "other"; console.log("rewrite"); return "saved"; }
        function main() {
            console.log(operand("left") < operand("right"));
            const box: Box = { text: "saved" };
            console.log(clone(box.text) === rewrite(box)); console.log(box.text);
            console.log("A😀B".slice(1,5) === "😀");
            console.log("A😀B".charAt(1) === "😀");
        }
    "#,
        "left\nright\ntrue\nrewrite\ntrue\nother\ntrue\ntrue\n",
        &[],
    );
}

#[test]
fn comparisons_infer_read_contracts_and_reusable_closure_captures() {
    let rust = execute(
        r#"
        function equal(left: string, right: string): boolean { return left === right; }
        function forward(left: string, right: string): boolean { return equal(left, right); }
        function main() {
            const first = "é"; const second = "é".toLowerCase();
            const same = (): boolean => { return first === second; };
            console.log(forward(first, second)); console.log(same()); console.log(same());
            if (first <= second) { console.log(first); } else { console.log(second); }
            console.log(first); console.log(second);
        }
    "#,
        "true\ntrue\ntrue\né\né\né\n",
        &["equal.left", "equal.right", "forward.left", "forward.right"],
    );
    assert!(!rust.contains("Clone::clone("));
}

#[test]
fn comparisons_work_through_fields_array_elements_and_borrowed_iteration() {
    execute(
        r#"
        type Box = { text: string; };
        function read(box: Box, other: string): boolean { return box.text >= other; }
        function main() {
            const box: Box = { text: "z" }; const values = ["a", "é", "😀"];
            console.log(read(box, values[0])); console.log(values[1] > values[0]);
            console.log(values[2] !== box.text);
            for (const value of values) { console.log(value >= "a"); }
            box.text = "a"; console.log(box.text === values[0]); console.log(values[1]);
        }
    "#,
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\né\n",
        &["read.box", "read.other"],
    );
}

#[test]
fn comparisons_retain_left_and_enclosing_loans_and_reject_mixed_types() {
    for operation in ["===", "!==", "<", "<=", ">", ">="] {
        for (left, right, declarations) in [
            (
                "value",
                "take(value)",
                "function take(value: string): string { return value; }",
            ),
            (
                "box.text",
                "change(box)",
                "function change(box: Box): string { box.text = \"changed\"; return \"changed\"; }",
            ),
            (
                "values[0]",
                "changeArray(values)",
                "function changeArray(values: string[]): string { values[0] = \"changed\"; return \"changed\"; }",
            ),
        ] {
            let source = format!(
                "type Box = {{text: string;}}; {declarations} function main() {{ const value = \"old\"; const box: Box = {{text: \"old\"}}; const values = [\"old\"]; console.log({left} {operation} {right}); }}"
            );
            let error = check(&source)
                .err()
                .expect("left source borrow must remain active");
            assert_eq!(error.code, "conflicting-access");
            assert!(!error.related.is_empty());
        }
    }
    for expression in [
        "\"x\" === 1",
        "true !== \"true\"",
        "\"x\" < false",
        "true > false",
        "[1] === [1]",
        "\"x\" + 1",
    ] {
        let source = format!("function main() {{ console.log({expression}); }}");
        assert_eq!(check(&source).err().unwrap().code, "type");
    }
    let source = r#"
        type Box = {text: string;};
        function change(box: Box): string { box.text = "changed"; return "changed"; }
        function hold(box: Box, same: boolean) { console.log(box.text); }
        function main() { const box: Box = {text: "old"}; hold(box, "old" === change(box)); }
    "#;
    assert_eq!(check(source).err().unwrap().code, "conflicting-access");
}

#[test]
fn captured_strings_coerce_to_standard_borrows_against_literals_and_locals() {
    execute(
        r#"
        function main() {
            const captured = "é".toLowerCase();
            const compare = (local: string): boolean => {
                console.log(captured === "é"); console.log("é" === captured);
                console.log(captured !== local); console.log(local !== captured);
                console.log(captured < "😀"); console.log("a" < captured);
                console.log(captured <= local); console.log(local >= captured);
                console.log(captured > "a"); console.log("😀" > captured);
                console.log(captured >= local); console.log(local <= captured);
                return captured === local;
            };
            const local = "é";
            console.log(compare(local)); console.log(compare(local));
            console.log(captured); console.log(local);
        }
    "#,
        "true\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\né\né\n",
        &[],
    );
}
