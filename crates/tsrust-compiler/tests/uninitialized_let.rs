mod support;

// `let name: Type;` declares a variable that is assigned later. Every read
// must come after an assignment on every path. The independent Node run of
// this source passes.
#[test]
fn let_without_initializer_is_assigned_before_use() {
    let source = r#"import { strictEqual } from "std:testing";
type Point = { x: int; y: int; };
function grade(score: int): string {
    let label: string;
    if (score >= 90) { label = "high"; } else if (score >= 50) { label = "mid"; } else { label = "low"; }
    return label;
}
function pick(flag: boolean): int {
    let value: int;
    if (flag) { value = 1; } else { return -1; }
    return value;
}
function firstEven(values: int[]): int {
    let found: int;
    found = -1;
    for (const value of values) {
        if (value % 2 === 0) { found = value; break; }
    }
    return found;
}
function main() {
    let count: int;
    count = 3;
    count += 1;
    strictEqual(count, 4);
    strictEqual(grade(95) + grade(60) + grade(1), "highmidlow");
    strictEqual(pick(true) + pick(false), 0);
    strictEqual(firstEven([3, 8, 6]), 8);
    let point: Point;
    point = { x: 1, y: 2 };
    strictEqual(point.x + point.y, 3);
    let words: string[];
    words = ["a"];
    words.push("b");
    strictEqual(words.join(""), "ab");
    let maybe: string | undefined;
    strictEqual(maybe === undefined, true);
    maybe = "set";
    strictEqual(maybe === "set", true);
    let first: int, second: int = 5;
    first = second + 1;
    strictEqual(first, 6);
    let kind: string;
    switch (count) { case 4: kind = "four"; break; default: kind = "other"; }
    strictEqual(kind, "four");
    let late: int;
    late = 2;
    const read = (): int => late + 1;
    strictEqual(read(), 3);
    let text: string;
    {
        const prefix = "in";
        text = prefix + "ner";
    }
    strictEqual(text, "inner");
    let twice: string;
    if (count > 3) { twice = "a"; }
    twice = "b";
    strictEqual(twice, "b");
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn let_without_initializer_keeps_static_contracts() {
    for (source, code) in [
        ("function main() { let a: int; console.log(a); }", "unassigned"),
        ("function main() { let a: int; if (true) { a = 1; } console.log(a); }", "unassigned"),
        ("function main() { let a: int; while (false) { a = 1; } console.log(a); }", "unassigned"),
        ("function main() { let a: int; for (const v of [1]) { a = v; } console.log(a); }", "unassigned"),
        ("function main() { let a: int; a += 1; }", "unassigned"),
        ("function main() { let a: int; a = a + 1; }", "unassigned"),
        ("function main() { let a: int; const f = (): int => a; a = 1; console.log(f()); }", "unassigned"),
        ("function main() { let a: int; if (a > 0) { a = 1; } else { a = 2; } }", "unassigned"),
        ("function main() { let a: string; a = 1; }", "type"),
        ("function main() { let a; a = 1; }", "annotation-required"),
        ("function main() { const a: int; }", "syntax"),
        ("function main() { let a: int, b; }", "annotation-required"),
        ("function main() { let a: int; switch (1) { case 1: a = 1; break; } console.log(a); }", "unassigned"),
        ("function main() { let f: Function<int, [int]>; f = (v: int): int => v; }", "unsupported"),
        ("function main() { let [x]: int[]; }", "syntax"),
        ("function main() { let a: int; { console.log(a); a = 1; } }", "unassigned"),
    ] {
        support::reject(source, code);
    }
}
