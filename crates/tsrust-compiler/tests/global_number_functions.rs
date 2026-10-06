mod support;

// `parseInt`, `parseFloat`, `isNaN`, `isFinite`, `NaN` and `Infinity` are
// global names for the matching `Number` members. The independent Node run of
// this source passes.
#[test]
fn global_number_names_match_number_members() {
    let source = r#"import { strictEqual } from "std:testing";
function fallback(value: number = NaN): boolean { return isNaN(value); }
function main() {
    strictEqual(parseInt("42px"), 42.0);
    strictEqual(parseInt("ff", 16), 255.0);
    strictEqual(parseInt("  -17"), Number.parseInt("  -17"));
    strictEqual(parseFloat("1.5x"), 1.5);
    strictEqual(isNaN(parseInt("abc")), true);
    strictEqual(isNaN(1.5), false);
    strictEqual(isFinite(2.5), true);
    strictEqual(isFinite(Infinity), false);
    strictEqual(isFinite(-Infinity), false);
    strictEqual(isNaN(NaN), true);
    strictEqual(NaN === NaN, false);
    strictEqual(Infinity > Number.MAX_VALUE, true);
    strictEqual(-Infinity, Number.NEGATIVE_INFINITY);
    strictEqual(fallback(), true);
    strictEqual(fallback(1.0), false);
    const values = ["1", "x", "3"].map((text) => parseInt(text)).filter((value) => !isNaN(value));
    strictEqual(values.length, 2);
    let limit = Infinity;
    for (const value of [4.5, 2.5, 8.0]) { if (value < limit) { limit = value; } }
    strictEqual(limit, 2.5);
    const lowest = -Infinity;
    strictEqual(lowest < -Number.MAX_VALUE, true);
    strictEqual(isNaN(-NaN), true);
    const negative = -parseInt("4");
    strictEqual(negative, -4.0);
    strictEqual(typeof NaN, "number");
    strictEqual(typeof Infinity, "number");
    strictEqual(`${NaN} ${-Infinity}`, "NaN -Infinity");
    const picked = isNaN(negative) ? NaN : Infinity;
    strictEqual(picked, Infinity);
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

// A declaration with the same name takes precedence over the global name.
#[test]
fn declarations_shadow_global_number_names() {
    let source = r#"import { strictEqual } from "std:testing";
function parseInt(text: string): int { return text.length; }
function main() {
    strictEqual(parseInt("42px"), 4);
    const NaN = 7;
    strictEqual(NaN, 7);
    const isFinite = (value: int): int => value + 1;
    strictEqual(isFinite(1), 2);
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn global_number_names_keep_static_contracts() {
    for (source, code) in [
        ("function main() { const a = parseInt(42); }", "type"),
        ("function main() { const a = parseInt(); }", "arity"),
        ("function main() { const a = isNaN(\"x\"); }", "type"),
        ("function main() { const a = isFinite(1, 2); }", "arity"),
        ("function main() { const a: int = parseInt(\"4\"); }", "type"),
        ("function main() { const a: int = NaN; }", "type"),
        ("function main() { NaN = 1; }", "name"),
        ("function main() { const a = parseBool(\"4\"); }", "name"),
        ("function main() { const a = parseInt; }", "unsupported"),
        ("function main() { const a = [\"1\"].map(parseInt); }", "unsupported"),
        ("function main() { Infinity += 1.0; }", "name"),
        ("function main() { const a = typeof parseInt; }", "unsupported"),
    ] {
        support::reject(source, code);
    }
}
