mod support;

#[test]
fn result_callable_payloads_preserve_owned_environments_and_invocation_contracts() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";
function counter(start: int): Result<Function<int>, string> {
    let value = start;
    return Ok((): int => { value += 1; return value; });
}
function message(text: string): Result<Once<Function<string>>, int> {
    return Ok((): string => { return text; });
}
function main() {
    const first = counter(10);
    const second = counter(20);
    match (first) { Ok(action) => { strictEqual(action(), 11); strictEqual(action(), 12); }, Err(_) => { strictEqual(1, 0); } }
    match (second) { Ok(action) => { strictEqual(action(), 21); }, Err(_) => { strictEqual(1, 0); } }
    const once = message("owned");
    match (once) { Ok(action) => { strictEqual(action(), "owned"); }, Err(_) => { strictEqual(1, 0); } }
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn explicit_result_constructors_substitute_generics_and_preserve_shadowing() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";
function wrap<T>(value: T): Result<T, string> { return Ok<T, string>(value); }
function identity<T>(value: T): T { return value; }
function main() {
    const success = Ok<int, string>(9);
    match (success) { Ok(value) => { strictEqual(value, 9); }, Err(_) => { strictEqual(1, 0); } }
    const failure = Err<int, string>("failed");
    match (failure) { Ok(_) => { strictEqual(1, 0); }, Err(error) => { strictEqual(error, "failed"); } }
    const generic = wrap("owned");
    match (generic) { Ok(value) => { strictEqual(value, "owned"); }, Err(_) => { strictEqual(1, 0); } }
    const empty = Ok<void, string>();
    match (empty) { Ok(_) => {}, Err(_) => { strictEqual(1, 0); } }
    { const Ok = identity; strictEqual(Ok<int>(4), 4); }
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn explicit_result_constructor_types_are_checked_without_generic_declarations() {
    for (source, code) in [
        ("function main(){const value=Ok<int,string>(5);}", None),
        ("function main(){const value=Err<int,string>(\"error\");}", None),
        ("function Ok<T>(value:T):T{return value;} function main(){const value=Ok<int>(5);}", None),
        ("function main(){const value=Ok<int>(5);}", Some("arity")),
        ("function main(){const value=Ok<int,string,boolean>(5);}", Some("arity")),
        ("function main(){const value=Ok<string,int>(5);}", Some("type")),
        ("function main(){const value:Result<string,int>=Ok<int,string>(5);}", Some("type")),
        ("function Ok(value:int):int{return value;} function main(){const value=Ok<int,string>(5);}", Some("type")),
    ] {
        if let Some(code) = code { support::reject(source, code); }
        else { tsrust_compiler::check(&support::format_round_trip(source)).unwrap(); }
    }
}

#[test]
fn result_type_identity_supports_nested_payloads_and_forwarding() {
    for source in [
        "function relay(value: Result<int, string>): Result<int, string> { return value; } function main() {}",
        "function relay(value: Result<int, int>): Result<int, int> { return value; } function main() {}",
        "function relay(value: Result<void, string>): Result<void, string> { return value; } function main() {}",
        "type User = { name: string; }; function relay(value: Result<User[], string | null>): Result<User[], string | null> { return value; } function main() {}",
        "type Box<T> = { value: Result<T, string>; }; function relay(value: Box<int>): Box<int> { return value; } function main() {}",
        "function relay(value: Result<Result<int, string>, boolean>): Result<Result<int, string>, boolean> { return value; } function main() {}",
    ] {
        let formatted = support::format_round_trip(source);
        tsrust_compiler::check(&formatted).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", &formatted)));
    }
}

#[test]
fn result_type_parameters_are_required_and_preserve_nominal_identity() {
    for source in [
        "function relay(value: Result): void {} function main() {}",
        "function relay(value: Result<int>): void {} function main() {}",
        "function relay(value: Result<int, string, boolean>): void {} function main() {}",
    ] {
        support::reject(source, "arity");
    }
    for source in [
        "function relay(value: Result<int, string>): Result<string, int> { return value; } function main() {}",
        "type A = { value: int; }; type B = { value: int; }; function relay(value: Result<A, string>): Result<B, string> { return value; } function main() {}",
    ] {
        support::reject(source, "type");
    }
}

#[test]
fn contextual_result_constructors_compile_and_drop_owned_payloads_natively() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";
type User = { name: string; };
function success(): Result<User[], string> { return Ok([{ name: "Pat" }]); }
function failure(): Result<int, string> { return Err("zero"); }
function unit(): Result<void, string> { return Ok(); }
function main() {
    const first: Result<int, string> = Ok(5);
    const second: Result<int, string> = Err("zero");
    const nested: Result<Result<int, string>, boolean> = Ok(Err("nested"));
    const absent: Result<string | null, int> = Ok(null);
    const users = success();
    const error = failure();
    const empty = unit();
    strictEqual(2 + 3, 5);
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn contextual_result_constructors_check_payloads_arity_and_ownership() {
    for (source, code) in [
        ("function main() { const result = Ok(1); }", "annotation-required"),
        ("function main() { const result: Result<int, string> = Ok(\"bad\"); }", "type"),
        ("function main() { const result: Result<int, string> = Err(2); }", "type"),
        ("function main() { const result: Result<int, string> = Ok(); }", "arity"),
        ("function main() { const result: Result<void, string> = Ok(1); }", "arity"),
        ("function main() { const text = \"owned\"; const result: Result<string, int> = Ok(text); console.log(text); }", "transferred"),
    ] {
        support::reject(source, code);
    }
}

#[test]
fn result_matches_require_exhaustive_reachable_variants() {
    for (arms, code) in [
        ("Ok(value) => {}", "match-exhaustiveness"),
        ("", "match-exhaustiveness"),
        ("Ok(a) => {}, Ok(b) => {}, Err(e) => {}", "match-unreachable"),
        ("_ => {}, Err(e) => {}", "match-unreachable"),
        ("Other(value) => {}, _ => {}", "match-pattern"),
    ] {
        let source = format!("function main() {{ const result: Result<int,string> = Ok(1); match (result) {{ {arms} }} }}");
        support::reject(&source, code);
    }
}

#[test]
fn matches_evaluate_once_nest_and_propagate_owned_errors() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";
type Counter = { calls: int; };
function make(counter: Mutable<Counter>, fail: boolean): Result<int, string> {
    counter.calls += 1;
    if (fail) { return Err("failed"); }
    return Ok(7);
}
function double(counter: Mutable<Counter>, fail: boolean): Result<int, string> {
    match (make(counter, fail)) {
        Ok(value) => { return Ok(value * 2); },
        Err(error) => { return Err(error); },
    }
}
function main() {
    const counter: Counter = { calls: 0 };
    match (double(counter, false)) {
        Ok(value) => { strictEqual(value, 14); },
        Err(error) => { strictEqual(error, "unexpected"); },
    }
    strictEqual(counter.calls, 1);
    match (double(counter, true)) {
        Err(error) => { strictEqual(error, "failed"); },
        _ => { strictEqual(counter.calls, -1); },
    }
    strictEqual(counter.calls, 2);
    const nested: Result<Result<string, int>, boolean> = Ok(Ok("inside"));
    match (nested) {
        Ok(inner) => {
            match (inner) {
                Ok(text) => { strictEqual(text, "inside"); },
                Err(code) => { strictEqual(code, -1); },
            }
        },
        Err(flag) => { strictEqual(flag, true); },
    }
    const empty: Result<void, string> = Ok();
    match (empty) { Ok(_) => {}, Err(error) => { strictEqual(error, "unexpected"); } }
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn result_construction_and_exhaustive_match_execute() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";

function divide(a: int, b: int): Result<int, string> {
    if (b === 0) { return Err("zero"); }
    return Ok(a / b);
}

function main() {
    match (divide(10, 2)) {
        Ok(value) => { strictEqual(value, 5); },
        Err(error) => { strictEqual(error, "unexpected"); },
    }
    match (divide(10, 0)) {
        Ok(value) => { strictEqual(value, -1); },
        Err(error) => { strictEqual(error, "zero"); },
    }
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
