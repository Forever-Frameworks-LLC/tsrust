mod support;

// `function (...) { ... }` is an anonymous function value, like an arrow
// function with a block body. The independent Node run of this source passes.
#[test]
fn function_expressions_are_function_values() {
    let source = r#"import { strictEqual } from "std:testing";
type Item = { id: int; };
function apply(action: Function<int, [int]>, value: int): int { return action(value); }
function main() {
    const double = function (value: int): int { return value * 2; };
    strictEqual(double(4), 8);
    strictEqual(apply(double, 5), 10);
    strictEqual(apply(function (value: int): int { return value + 1; }, 1), 2);
    const offset = 3;
    const shift = function (value: int) { return value + offset; };
    strictEqual(shift(1), 4);
    const words = ["b", "a", "c"];
    const upper = words.map(function (word) { return word.toUpperCase(); });
    strictEqual(upper.join(""), "BAC");
    words.sort(function (left, right) { return left < right ? -1 : 1; });
    strictEqual(words.join(""), "abc");
    let total = 0;
    [1, 2, 3].forEach(function (value: int) { total += value; });
    strictEqual(total, 6);
    const defaults = function (value: int = 5, ...rest: int[]): int { return value + rest.length; };
    strictEqual(defaults() + defaults(1, 2, 3), 8);
    const pick = function ({ id }: Item, [first]: int[]): int { return id + first; };
    strictEqual(pick({ id: 2 }, [3]), 5);
    const make = function (step: int) { return function (value: int): int { return value + step; }; };
    const add = make(10);
    strictEqual(add(1), 11);
}"#;
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn function_expressions_keep_static_contracts() {
    for (source, code) in [
        ("function main() { const f = function named(value: int): int { return value; }; }", "unsupported"),
        ("function main() { const f = function (value: int): int => value; }", "syntax"),
        ("function main() { const f = function (value: int): int { return \"x\"; }; }", "type"),
        ("function main() { const f = function (value: int): int { return value; }; f(\"a\"); }", "type"),
        ("function main() { const f = function (value) { return value; }; }", "parameter-inference"),
        ("function main() { function (value: int): int { return value; }; }", "syntax"),
    ] {
        support::reject(source, code);
    }
}
