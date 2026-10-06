mod support;

// switch selects the first matching case by strict equality. The independent
// Node run of this source passes.
#[test]
fn switch_selects_cases_groups_defaults_and_keeps_loop_control() {
    let source = r#"import { strictEqual } from "std:testing";
function name(code: int): string {
    switch (code) {
        case 1: return "one";
        case 2:
        case 3: return "few";
        default: return "many";
    }
}
function kind(value: ReadOnly<int | string | boolean>): string {
    switch (typeof value) {
        case "string": return "text";
        case "number": return "count";
        default: return "flag";
    }
}
function next(trace: int[], value: int): int { trace.push(value); return value; }
function main() {
    strictEqual(name(1) + name(2) + name(3) + name(9), "onefewfewmany");
    strictEqual(kind("a") + kind(1) + kind(true), "textcountflag");
    const trace: int[] = [];
    let label = "";
    switch (next(trace, 2)) {
        case next(trace, 1): label = "a"; break;
        case next(trace, 2): { label = "b"; break; }
        case next(trace, 3): label = "c"; break;
    }
    strictEqual(label, "b");
    strictEqual(trace.join(","), "2,1,2");
    let total = 0;
    for (let i = 0; i < 6; i++) {
        switch (i % 3) {
            case 0: continue;
            case 1: total += 10; break;
            default:
                total += 1;
                if (i === 5) total += 100;
        }
        total += 1000;
    }
    strictEqual(total, 4122);
    const word = "go".repeat(1);
    let speed = 0;
    switch (word) {
        default: speed = -1; break;
        case "stop": speed = 0; break;
        case "go": speed = 5; break;
    }
    strictEqual(speed, 5);
    strictEqual(word.length, 2);
    const mixed: int | string = total > 0 ? "x" : 1;
    let seen = "";
    switch (mixed) {
        case 1: seen = "int"; break;
        case "x": seen = mixed.toUpperCase(); break;
    }
    strictEqual(seen, "X");
    let untouched = 7;
    switch (untouched) { }
    switch (untouched) { case 8: untouched = 0; }
    strictEqual(untouched, 7);
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn switch_keeps_static_contracts() {
    for (source, code) in [
        // A case that runs into the next one must be written as a group or end explicitly.
        ("function main() { let n = 1; switch (n) { case 1: n = 2; case 2: n = 3; break; } }", "unsupported"),
        ("function main() { let n = 1; switch (n) { case 1: if (n > 0) { break; } n = 2; break; } }", "unsupported"),
        ("function main() { const n = 1; switch (n) { default: break; default: break; } }", "syntax"),
        ("function main() { const n = 1; switch (n) { n; } }", "syntax"),
        ("function main() { const n = 1; switch (n) { case 1 break; } }", "syntax"),
        ("function main() { const n = 1; switch (n) { case \"a\": break; } }", "type"),
        ("function f(n: int): int { switch (n) { case 1: return 1; } } function main() {}", "type"),
        ("function main() { const n = 1; switch (n) { case 1: const a = 1; break; case 2: console.log(a); break; } }", "name"),
    ] {
        support::reject(source, code);
    }
}
