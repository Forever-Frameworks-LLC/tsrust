mod support;

// One declaration can introduce several names, and a classic for update can
// hold several expressions. The independent Node run of this source passes.
#[test]
fn declaration_lists_and_comma_updates_run_in_source_order() {
    let source = r#"import { strictEqual } from "std:testing";
type Point = { x: int; y: int; };
function next(trace: int[], value: int): int { trace.push(value); return value; }
function main() {
    const trace: int[] = [];
    let a = next(trace, 1), b = next(trace, a + 1), text: string = "t".repeat(b);
    strictEqual(trace.join(","), "1,2");
    strictEqual(text, "tt");
    a = a + b;
    strictEqual(a, 3);
    const origin: Point = { x: 7, y: 8 };
    const first = 4, [second, third] = [5, 6], { x, y: z } = origin, last = first + second + third + x + z;
    strictEqual(last, 30);
    let total = 0;
    for (let i = 0, j = 10; i < j; i++, j--) { total += j - i; }
    strictEqual(total, 30);
    const seen: string[] = [];
    for (let [low, high] = [0, 3], step = 1; low < high; low += step, seen.push(String(low))) { if (low === 1) { continue; } }
    strictEqual(seen.join(","), "1,2,3");
    let count = 0, limit = 3;
    for (; count < limit; count++, limit--) { }
    strictEqual(count + limit, 3);
    const point: Point = { x: 1, y: 2 }, sum = point.x + point.y;
    strictEqual(sum, 3);
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn declaration_lists_keep_scope_and_syntax_contracts() {
    for (source, code) in [
        ("function main() { let a = 1, a = 2; }", "duplicate"),
        ("function main() { const a = b, b = 1; }", "name"),
        ("function main() { let a = 1, ; }", "syntax"),
        ("function main() { let a = 1, b; }", "annotation-required"),
        ("function main() { const a = 1, b: int; }", "syntax"),
        ("function main() { for (let i = 0, j = 2; i < j; i++, ) { } }", "syntax"),
        ("function main() { for (let i = 0; i < 2; i++, let k = 1) { } }", "syntax"),
        ("function main() { for (let i = 0, j = 2; i < j; i++) { } console.log(j); }", "name"),
        ("function main() { const a = 1, b: string = 2; }", "type"),
    ] {
        support::reject(source, code);
    }
}
