mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    definitions: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\n{definitions}\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("temporary-length.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("temporary-length-{level}"));
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
fn temporary_array_lengths_cover_native_leaf_types_ranks_closures_and_single_evaluation() {
    let source = r#"
        function ints(): int[] { console.log("ints"); return [1,2,3,4,5]; }
        function flags(): boolean[] { console.log("flags"); return [true,false]; }
        function names(): string[] { console.log("names"); return ["ß".toUpperCase(),"😀"]; }
        function rows(): int[][] { console.log("rows"); return [[1,2],[],[3]]; }
        function empty(): string[] { console.log("empty"); return []; }
        function count(values: int[]): int { return clone(values).length; }
        function main() {
            console.log([1,2,3].length); console.log([true,false].length);
            console.log(["ab","😀"].length); console.log([[1],[2,3]].length);
            console.log(ints().length); console.log(flags().length); console.log(names().length);
            console.log(rows().length); console.log(empty().length);
            const get = (): int[] => { console.log("closure"); return [9]; };
            console.log(get().length);
            const values = [7,8,9]; console.log(count(values)); values[0] = 10; console.log(values[0]);
            const nested = [[1,2],[3]]; console.log(nested.remove(0).length); console.log(nested.length);
            console.log("ß".toUpperCase().length);
        }
    "#;
    let analysis = native(
        source,
        "3\n2\n2\n2\nints\n5\nflags\n2\nnames\n2\nrows\n3\nempty\n0\nclosure\n1\n3\n10\n2\n1\n2\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("count.values"), Some(&Mode::Read));
}

#[test]
fn temporary_record_array_lengths_dispose_small_and_heap_elements_at_statement_end() {
    let source = r#"
        type Item = { n: int; };
        function small(): Item[] { console.log("small"); return [{n:1},{n:2}]; }
        function heap(): Item[] { console.log("heap"); return [{n:3},{n:4},{n:5},{n:6},{n:7}]; }
        function main() {
            console.log(small().length); console.log("after-small");
            console.log(heap().length); console.log("after-heap");
        }
    "#;
    // Add an ordinary Rust destructor only to observe generated temporary
    // lifetimes. Both inline and Vec-backed arrays must release every element.
    native(
        source,
        "small\n2\ndrop1\ndrop2\nafter-small\nheap\n5\ndrop3\ndrop4\ndrop5\ndrop6\ndrop7\nafter-heap\n",
        r#"
        static DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        impl Drop for Record0 {
            fn drop(&mut self) {
                DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                println!("drop{}", self.field0);
            }
        }
    "#,
        "assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst), 7);",
    );
}

#[test]
fn temporary_lengths_preserve_readonly_types_context_and_enclosing_loans() {
    for source in [
        "function n(): int { return 1; } function main() { console.log(n().length); }",
        "function flag(): boolean { return true; } function main() { console.log(flag().length); }",
    ] {
        let error = check(source).err().unwrap();
        assert_eq!(error.code, "type");
        assert_eq!(error.message, "Length requires a string or array.");
    }
    for source in [
        "function make(): int[] { return [1,2]; } function main() { make().length = 3; }",
        "function main() { const values = [1,2]; values.length = 3; }",
        "function main() { const value = \"abc\"; value.length = 3; }",
    ] {
        assert_eq!(check(source).err().unwrap().code, "unsupported");
    }
    assert_eq!(
        check("function main() { const count = [].length; }")
            .err()
            .unwrap()
            .code,
        "array-type"
    );
    let conflict = "function change(values: int[]): int[] { values.push(3); return [4]; } function hold(values: int[], count: int) {} function main() { const values = [1,2]; hold(values, change(values).length); }";
    assert_eq!(check(conflict).err().unwrap().code, "conflicting-access");
    let moved = "function main() { const values = [1,2]; const saved = values; console.log(clone(values).length); }";
    assert_eq!(check(moved).err().unwrap().code, "transferred");
}
