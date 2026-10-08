mod support;
use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn arithmetic_precedence_associativity_and_signed_literals() {
    execute(
        r#"function main() {
 console.log(2 + 3 * 4); console.log((2 + 3) * 4); console.log(20 - 5 - 3);
 console.log(24 / 4 / 2); console.log(7 % 4); console.log(-7 / 3); console.log(-7 % 3);
 console.log(1-2); console.log(1 - -2); console.log(- 1); console.log(-9223372036854775808);
 const n = 4; console.log(-n * 2); console.log(-(n + 2)); console.log(- -n);
 console.log(1/* gap */+2); console.log(6/2); console.log(00042 - 2);
}"#,
        "14\n20\n12\n3\n3\n-2\n-1\n-1\n3\n-1\n-9223372036854775808\n-8\n-6\n4\n3\n3\n40\n",
        None,
    );
}

#[test]
fn comparisons_and_boolean_precedence() {
    execute(
        r#"function main() {
 console.log(1 < 2); console.log(2 <= 2); console.log(3 > 2); console.log(2 >= 3);
 console.log(2 + 3 === 5); console.log(2 * 3 !== 6); console.log(true === !false);
 console.log(false !== true); console.log(1 < 2 === true);
 console.log(true || false && false); console.log((true || false) && false);
 console.log(!false && !!true); console.log(-2 < -1);
 if (2 * 3 >= 6 && !false) { console.log("yes"); } else { console.log("no"); }
}"#,
        "true\ntrue\ntrue\nfalse\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\nfalse\ntrue\ntrue\nyes\n",
        None,
    );
}

#[test]
fn operands_run_once_in_source_order_and_release_scalar_access() {
    execute(
        r#"type S = { n: int; };
function bump(s: S): int { s.n = s.n + 1; console.log(s.n); return s.n; }
function main() { const s: S = { n: 0 }; console.log(bump(s) * 10 + bump(s)); console.log(s.n + bump(s)); console.log(bump(s) < bump(s)); console.log(-s.n); }"#,
        "1\n2\n12\n3\n5\n4\n5\ntrue\n-5\n",
        Some(&[("bump.s", Mode::Mutate)]),
    );
}

#[test]
fn short_circuiting_skips_side_effects_and_runtime_traps() {
    execute(
        r#"function mark(value: boolean): boolean { console.log(value); return value; }
function main() {
 console.log(false && mark(true)); console.log(true || mark(false));
 console.log(true && mark(false)); console.log(false || mark(true));
 console.log(false && 1 / 0 === 0); console.log(true || 9223372036854775807 + 1 === 0);
}"#,
        "false\ntrue\nfalse\nfalse\ntrue\ntrue\nfalse\ntrue\n",
        None,
    );
}

#[test]
fn short_circuit_possible_transfers_remain_unavailable() {
    for condition in [
        "false && take(user)",
        "true || take(user)",
        "flag && take(user)",
        "flag || take(user)",
        "take(user) && flag",
    ] {
        let source = format!(
            "type U = {{ n: int; }}; function take(user: U): boolean {{ user; return true; }} function f(user: U, flag: boolean) {{ const result = {condition}; console.log(user.n); }} function main() {{}}"
        );
        let error = reject(&source, "transferred");
        assert_eq!(error.related.len(), 1);
    }
    reject(
        "type U = { n: int; }; function take(user: U): boolean { user; return true; } function f(user: U, flag: boolean) { if (flag && take(user)) {} else { console.log(user.n); } } function main() {}",
        "transferred",
    );
}

#[test]
fn conditional_consumption_infers_owned_parameters_and_single_use_closures() {
    let source = r#"type U = { n: int; }; function take(user: U): boolean { user; return true; }
function maybe(user: U, flag: boolean): boolean { return flag && take(user); }
function main() { const user: U = { n: 7 }; const test = (flag: boolean): boolean => { return flag || take(user); }; console.log(test(true)); }"#;
    execute(
        source,
        "true\n",
        Some(&[
            ("take.user", Mode::Consume),
            ("maybe.user", Mode::Consume),
            ("maybe.flag", Mode::Copy),
        ]),
    );
    assert_eq!(
        check(source).unwrap().closures()[0].invocation,
        Mode::Consume
    );
    reject(
        &source.replace(
            "console.log(test(true));",
            "console.log(test(true)); test(false);",
        ),
        "closure-consumed",
    );
}

#[test]
fn operators_preserve_enclosing_argument_loans() {
    for expression in [
        "user.n + bump(user)",
        "user.n - bump(user)",
        "user.n < bump(user)",
        "false && bump(user) > 0",
        "true || bump(user) > 0",
    ] {
        let result = if expression.contains('+') || expression.contains(" - ") {
            "int"
        } else {
            "boolean"
        };
        let source = format!(
            "type U = {{ n: int; }}; function bump(user: U): int {{ user.n = user.n + 1; return user.n; }} function hold(user: U, value: {result}) {{ console.log(user.n); }} function main() {{ const user: U = {{ n: 0 }}; hold(user, {expression}); }}"
        );
        reject(&source, "conflicting-access");
    }
}

#[test]
fn operators_work_inside_retained_callbacks_and_nested_results() {
    execute(
        include_str!("fixtures/operators/counter.tsr"),
        "6\n5\nready\n",
        None,
    );
    execute(
        r#"type S = { n: int; };
function retain(action: Function<int, [step: int]>): (step: int) => int { return (step: int): int => { return action(step); }; }
function main() { const s: S = { n: 1 }; const count = retain((step: int): int => { s.n = s.n + step; return s.n; }); console.log(count(2 * 3)); console.log(count(-1));
 const make = (base: int): (n: int) => boolean => { return (n: int): boolean => { return n >= base && n % 2 === 0; }; }; const even = make(4); console.log(even(6)); console.log(even(3)); }"#,
        "7\n6\ntrue\nfalse\n",
        None,
    );
}

#[test]
fn operator_type_errors_point_to_original_operator() {
    // Logical operators select operand values; incompatible operand types
    // report the selection rule rather than a boolean operand type.
    for (expression, operator, code) in [
        ("1 + true", "+", "type"),
        ("-true", "-", "type"),
        ("true < false", "<", "type"),
        ("1 === true", "===", "type"),
        ("\"x\" !== 1", "!==", "type"),
    ] {
        let source = format!("function main() {{ /* café */ console.log({expression}); }}");
        let error = reject(&source, code);
        assert_eq!(&source[error.span.start..error.span.end], operator);
    }
    reject("function main() { console.log(1 < 2 < 3); }", "type");
    reject(
        "type U = { n: int; }; function main() { const u: U = { n: 1 }; console.log(u === u); }",
        "type",
    );
}

#[test]
fn unsupported_operators_and_missing_operands_have_frontend_diagnostics() {
    for expression in [
        "1 == 1",
        "1 != 2",
        "++1",
        "--1",
        "1 += 2",
        "1 **= 2",
        "true &= false",
        "true |= false",
        "1 <<= 2",
        "1 +",
        "!",
        "1 / / 2",
    ] {
        let source = format!("function main() {{ console.log({expression}); }}");
        let error = check(&source).err().unwrap();
        assert!(
            matches!(error.code, "unsupported" | "syntax"),
            "{source}: {}",
            error.message
        );
        reject(&source, error.code);
    }
}

#[test]
fn unary_and_binary_nesting_are_bounded() {
    for expression in [
        format!("{}true", "!".repeat(100)),
        format!("{}1", "1 + ".repeat(100)),
        format!("{}1{}", "-(".repeat(100), ")".repeat(100)),
    ] {
        let source = format!("function main() {{ console.log({expression}); }}");
        check(&source)
            .unwrap_or_else(|error| panic!("{}", error.message))
            .emit_rust();
    }
    for expression in [
        format!("{}true", "!".repeat(200)),
        format!("{}1", "1 + ".repeat(200)),
        format!("{}1{}", "-(".repeat(200), ")".repeat(200)),
    ] {
        reject(
            &format!("function main() {{ console.log({expression}); }}"),
            "unsupported",
        );
    }
}

#[test]
fn arithmetic_checks_are_identical_with_backend_overflow_checks_disabled() {
    for (expression, failure, operator) in [
        ("9223372036854775807 + 1", "addition overflow", "+"),
        ("-9223372036854775808 - 1", "subtraction overflow", " - "),
        ("9223372036854775807 * 2", "multiplication overflow", "*"),
        ("1 / 0", "division by zero or overflow", "/"),
        (
            "-9223372036854775808 / -1",
            "division by zero or overflow",
            "/",
        ),
        ("1 % 0", "remainder by zero or overflow", "%"),
        (
            "-9223372036854775808 % -1",
            "remainder by zero or overflow",
            "%",
        ),
        ("-minimum", "negation overflow", "-minimum"),
    ] {
        let source = format!(
            "function main() {{ const minimum = -9223372036854775808; console.log({expression}); }}"
        );
        let rust = check(&source).unwrap().emit_rust();
        let directory = support::TempDirectory::new();
        let path = directory.0.join("program.rs");
        std::fs::write(&path, rust).unwrap();
        let offset = source.rfind(expression).unwrap()
            + expression.find(operator).unwrap()
            + usize::from(operator.starts_with(' '));
        for optimized in [false, true] {
            let binary = directory.0.join("program");
            let output = support::run(
                support::rustc()
                    .arg("--edition=2024")
                    .arg(&path)
                    .arg("-o")
                    .arg(&binary)
                    .arg("-C")
                    .arg(if optimized {
                        "opt-level=3"
                    } else {
                        "opt-level=0"
                    })
                    .arg("-C")
                    .arg(if optimized {
                        "overflow-checks=no"
                    } else {
                        "overflow-checks=yes"
                    }),
                60,
            );
            support::success(&output);
            let result = support::run(&mut std::process::Command::new(&binary), 10);
            assert!(!result.status.success());
            assert!(result.stdout.is_empty());
            let error = String::from_utf8(result.stderr).unwrap();
            assert!(
                error.contains(&format!(
                    "integer {failure} at source bytes {offset}..{}",
                    offset + 1
                )),
                "{error}"
            );
        }
    }
}

#[test]
fn operators_check_and_emit_without_external_backend() {
    let directory = support::TempDirectory::new();
    let path = directory.source("function main() { console.log(2 + 3 * 4 === 14 && !false); }");
    for command in ["check", "emit"] {
        let result = support::run(support::cli().arg(command).arg(&path).env("PATH", ""), 10);
        support::success(&result);
    }
    let bad = directory.source("function main() { console.log(true + 1); }");
    let result = support::run(
        support::cli()
            .arg("build")
            .arg(bad)
            .arg("-o")
            .arg(directory.0.join("out"))
            .env("PATH", ""),
        10,
    );
    assert!(!result.status.success());
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("error[type]")
    );
}
