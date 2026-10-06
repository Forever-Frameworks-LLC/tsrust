mod support;
use support::{execute, reject};

// Expected output was produced by Node for the same values: grid edges, ties,
// subnormal results, overflow and 300 generated values.
#[test]
fn math_f16round_matches_reference_output() {
    execute(
        include_str!("fixtures/math-f16round/values.tsr"),
        include_str!("fixtures/math-f16round/values.stdout"),
        None,
    );
}

#[test]
fn math_f16round_requires_one_number() {
    execute(
        "function half(value: number): number { return Math.f16round(value); }\nfunction main() { const count = 2049; console.log(Math.f16round(Number(count)), half(0.1), Math.f16round(...[1.337])); }\n",
        "2048 0.0999755859375 1.3369140625\n",
        None,
    );
    for (source, code) in [
        ("function main() { Math.f16round(); }", "arity"),
        ("function main() { Math.f16round(1, 2); }", "arity"),
        ("function main() { Math.f16round(\"1\"); }", "type"),
        ("function main() { Math.f16round(true); }", "type"),
        ("function main() { const count = 2; Math.f16round(count); }", "type"),
        ("function main() { const value: int = Math.f16round(1.5); }", "type"),
    ] {
        reject(source, code);
    }
}
