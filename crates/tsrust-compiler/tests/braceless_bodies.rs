mod support;

// A single statement can be the body of if, else, while, do, for and for...of
// without braces. The independent Node run of this source passes.
#[test]
fn single_statement_bodies_run_like_braced_blocks() {
    let source = r#"import { strictEqual } from "std:testing";
function sign(value: int): string {
    if (value < 0) return "negative";
    else if (value === 0) return "zero";
    else return "positive";
}
function main() {
    strictEqual(sign(-2) + sign(0) + sign(5), "negativezeropositive");
    let total = 0;
    for (let i = 0; i < 4; i++) total += i;
    strictEqual(total, 6);
    for (const value of [10, 20]) total += value;
    strictEqual(total, 36);
    let n = 0;
    while (n < 3) n++;
    strictEqual(n, 3);
    do n--; while (n > 1);
    strictEqual(n, 1);
    const seen: string[] = [];
    for (let i = 0; i < 3; i++)
        for (let j = 0; j < 3; j++)
            if (i === j) seen.push(String(i));
            else if (i + j === 2) seen.push("x");
    strictEqual(seen.join(""), "0x1x2");
    let nearest = "";
    if (total > 0) if (total > 100) nearest = "big"; else nearest = "small";
    strictEqual(nearest, "small");
    let count = 0;
    for (let i = 0; i < 5; i++) { if (i === 1) continue; if (i === 4) break; count++; }
    strictEqual(count, 3);
    let [a, b] = [1, 2];
    if (a < b) [a, b] = [b, a];
    strictEqual(a * 10 + b, 21);
    if (a > 100) strictEqual(a, 0); else { strictEqual(b, 1); }
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn single_statement_bodies_keep_scope_and_syntax_contracts() {
    for (source, code) in [
        ("function main() { if (true) const a = 1; }", "syntax"),
        ("function main() { for (let i = 0; i < 2; i++) let k = i; }", "syntax"),
        ("function main() { if (true) ; }", "syntax"),
        ("function main() { while (true) }", "syntax"),
        ("function main() { if (true) console.log(1) else console.log(2); }", "syntax"),
        ("function main() { for (let i = 0; i < 2; i++) console.log(i); console.log(i); }", "name"),
        ("function f(): int { if (true) return \"x\"; return 1; } function main() {}", "type"),
    ] {
        support::reject(source, code);
    }
    // Nesting without braces is bounded like nesting with braces.
    let deep = format!("function main() {{ {} console.log(1); }}", "if (true) ".repeat(200));
    support::reject(&deep, "unsupported");
}
