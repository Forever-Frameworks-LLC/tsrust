mod support;
use support::{execute, reject};

#[test]
fn array_keys_and_values_return_independent_arrays() {
    execute(
        r#"
type Row = { n: int; };
function supply(log: int[]): string[] { log.push(1); return ["a", "b", "c"]; }
function main() {
    const words = ["x".repeat(1), "y", "z"];
    const keys = words.keys();
    console.log(keys.join(","), keys.length, words.length);
    let total = 0;
    for (const index of words.keys()) { total += index; }
    console.log(total);
    const copy = words.values();
    copy[0] = "changed";
    console.log(words.join(","), copy.join(","));
    const empty: int[] = [];
    console.log(empty.keys().length, empty.values().length);
    const rows: Row[] = [{ n: 4 }, { n: 5 }];
    console.log(rows.keys().join("|"), rows.values().length);
    const nested = [[1], [2, 3]];
    console.log(nested.keys().join(","), nested.values()[1].join(","));
    const log: int[] = [];
    console.log(supply(log).keys().join(","), log.length);
    const grown = [1, 2].keys();
    grown.push(9);
    console.log(grown.join(","), Array.from(words.keys(), (key: int): int => { return key * 2; }).join(","));
    console.log([...words.keys(), 7].join(","), words.keys().map((key: int): string => { return clone(words[key]); }).join(""));
}
"#,
        "0,1,2 3 3\n3\nx,y,z changed,y,z\n0 0\n0|1 2\n0,1 2,3\n0,1,2 1\n0,1,9 0,2,4\n0,1,2,7 xyz\n",
        None,
    );
}

#[test]
fn array_keys_and_values_reject_arguments_and_non_arrays() {
    for (source, code) in [
        ("function main() { console.log([1].keys(1).length); }", "arity"),
        ("function main() { console.log([1].values(1).length); }", "arity"),
        ("function main() { const text = \"ab\"; console.log(text.keys().length); }", "unsupported"),
        ("function main() { const keys: string[] = [1].keys(); }", "type"),
    ] {
        reject(source, code);
    }
}
