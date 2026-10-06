mod support;
use support::{execute, reject};
use tsrust_compiler::check;

#[test]
fn array_from_copies_arrays_maps_elements_and_splits_strings() {
    execute(
        r#"
type Row = { n: int; };
function double(value: int, index: int): int { return value * 2 + index; }
function main() {
    const source = [3, 1, 2];
    const copy = Array.from(source);
    copy.push(9);
    console.log(source.join(","), copy.join(","));
    console.log(Array.from(source, (value: int): int => { return value + 1; }).join(","));
    console.log(Array.from(source, (value: int, index: int): string => { return String(value) + ":" + String(index); }).join(" "));
    console.log(Array.from(source, double).join(","));
    console.log(Array.from("aé😀b").join("|"), Array.from("").length);
    console.log(Array.from("ab", (letter: ReadOnly<string>): int => { return letter.length; }).join(","));
    const words = ["x".repeat(2), "y"];
    const kept = Array.from(words);
    kept[0] = "changed";
    console.log(words.join(","), kept.join(","));
    const rows: Row[] = [{ n: 1 }, { n: 2 }];
    console.log(Array.from(rows, (row: ReadOnly<Row>): int => { return row.n * 10; }).join(","));
    const nested = [[1], [2, 3]];
    console.log(Array.from(nested).length, Array.from(Array.from(source)).length);
    const empty: int[] = [];
    console.log(Array.from(empty).length, Array.from(empty, (value: int): int => { return value; }).length);
}
"#,
        "3,1,2 3,1,2,9\n4,2,3\n3:0 1:1 2:2\n6,3,6\na|é|😀|b 0\n1,1\nxx,y changed,y\n10,20\n2 3\n0 0\n",
        None,
    );
}

#[test]
fn array_from_evaluates_its_source_once_before_mapping() {
    execute(
        r#"
function supply(log: int[]): int[] { log.push(1); return [5, 6]; }
function main() {
    const log: int[] = [];
    let calls = 0;
    const result = Array.from(supply(log), (value: int, index: int): int => { calls += 1; return value + index; });
    console.log(result.join(","), log.length, calls);
}
"#,
        "5,7 1 2\n",
        None,
    );
}

#[test]
fn array_from_rejects_unsupported_sources_and_arity() {
    for (source, code) in [
        ("function main() { console.log(Array.from().length); }", "arity"),
        ("function main() { console.log(Array.from([1], (v: int): int => { return v; }, 3).length); }", "arity"),
        ("function main() { console.log(Array.from(7).length); }", "type"),
        ("type Row = { n: int; }; function main() { const row: Row = { n: 1 }; console.log(Array.from(row).length); }", "type"),
        ("function main() { console.log(Array.from([1], (v: int, i: int, all: ReadOnly<int[]>): int => { return v; }).length); }", "arity"),
        ("function main() { console.log(Array.from([1], 5).length); }", "type"),
    ] {
        let error = reject(source, code);
        assert!(error.message.contains("Array.from") || code == "type", "{source}: {}", error.message);
    }
    assert!(check("function main() { const Array = 1; console.log(Array); }").is_err());
}
