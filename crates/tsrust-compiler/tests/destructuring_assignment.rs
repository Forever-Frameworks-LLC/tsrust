mod support;

// Assignment to existing names and places through a pattern, and shorthand
// record properties. The independent Node run of this source passes.
#[test]
fn destructuring_assignment_replaces_existing_places_in_source_order() {
    let source = r#"import { strictEqual } from "std:testing";
type Point = { x: int; y: int; label: string; };
function pair(trace: int[]): [int, string] { trace.push(1); return [7, "seven"]; }
function main() {
    let a = "x".repeat(1), b = "y".repeat(1);
    [a, b] = [b, a];
    strictEqual(a + b, "yx");
    let i = 1, j = 2, k = 3;
    [i, j, k] = [k, i, j];
    strictEqual(i * 100 + j * 10 + k, 312);
    const trace: int[] = [];
    let count = 0, word = "";
    [count, word] = pair(trace);
    strictEqual(count, 7);
    strictEqual(word, "seven");
    strictEqual(trace.length, 1);
    const values = [0, 0, 0];
    const point: Point = { x: 1, y: 2, label: "p" };
    [values[0], point.x] = [5, 6];
    strictEqual(values[0] + point.x, 11);
    let x = 0, y = 0, text = "";
    ({ x, y, label: text } = point);
    strictEqual(x + y, 8);
    strictEqual(text, "p");
    let first = 0, rest: int[] = [];
    [first, ...rest] = [1, 2, 3];
    strictEqual(first, 1);
    strictEqual(rest.join(","), "2,3");
    let m = 0, n = 0;
    [m, [n]] = [1, [2]];
    strictEqual(m + n, 3);
    const label = text + "!";
    const copy: Point = { x, y, label };
    strictEqual(copy.x + copy.y, 8);
    strictEqual(copy.label, "p!");
    let low = 0, high = 9;
    while (low < high) { [low, high] = [low + 2, high - 1]; }
    strictEqual(low + high, 12);
    let f0 = 0, f1 = 1, steps = 0;
    for (let turn = 0; turn < 5; [f0, f1] = [f1, f0 + f1], turn++) { steps++; if (turn === 1) { continue; } }
    strictEqual(f0, 5);
    strictEqual(f1, 8);
    strictEqual(steps, 5);
    let left = "l".repeat(1), right = "r".repeat(1), turns = 0;
    for (; turns < 3; turns++, [left, right] = [right, left]) { }
    strictEqual(left + right, "rl");
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn destructuring_assignment_keeps_static_contracts() {
    for (source, code) in [
        ("function main() { let a = 1; const b = 2; [a, b] = [b, a]; }", "unsupported"),
        ("function main() { let a = 1; let b = 2; [a, b] = [b]; }", "bounds"),
        ("function main() { let a = 1; let r: int[] = []; [...r, a] = [1, 2]; }", "syntax"),
        ("function main() { for (let i = 0; i < 2; i++, let k = 1) { } }", "syntax"),
        ("function main() { let a = 1; [a, missing] = [1, 2]; }", "name"),
        ("function main() { let a = 1; let s = \"x\"; [a, s] = [s, a]; }", "type"),
        ("function main() { let a = 1; let b = 2; [a = 5, b] = [1, 2]; }", "unsupported"),
        ("function pair(): [int, string] { return [1, \"a\"]; } function main() { let a = 1; let b = 2; let c = 3; [a, b, c] = pair(); }", "type"),
        ("type P = { x: int; }; function main() { const p: P = { x: 1 }; let y = 0; ({ y } = p); }", "type"),
        ("type P = { x: int; }; function main() { const z = 1; const p: P = { z }; }", "type"),
    ] {
        support::reject(source, code);
    }
}
