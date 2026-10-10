mod support;

#[test]
fn mutable_result_matches_update_selected_payload_and_release_the_borrow() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";
type User = { name: string; count: int; };
function update(result: Mutable<Result<User, int[]>>): void {
    match (result) {
        Ok(user) => { user.name = "updated"; user.count += 1; },
        Err(values) => { values.push(7); },
    }
}
function nested(result: Mutable<Result<Result<User, int[]>, string>>): void {
    match (result) {
        Ok(inner) => { match (inner) { Ok(user) => { user.count += 2; }, Err(values) => { values.push(9); } } },
        Err(_) => {},
    }
}
function scalar(result: Mutable<Result<int, string>>): int {
    match (result) { Ok(value) => { return value + 1; }, Err(error) => { return error.length; } }
}
function counterResult(): Result<Function<int>, string> {
    let value = 10;
    return Ok((): int => { value += 1; return value; });
}
function tick(result: Mutable<Result<Function<int>, string>>): int {
    match (result) { Ok(action) => { return action(); }, Err(error) => { return error.length; } }
}
function main() {
    const success: Result<User, int[]> = Ok({ name: "Pat", count: 1 });
    update(success);
    update(success);
    match (success) {
        Ok(user) => { strictEqual(user.name, "updated"); strictEqual(user.count, 3); },
        Err(_) => { strictEqual(1, 0); },
    }
    const failure: Result<User, int[]> = Err([4]);
    update(failure);
    update(failure);
    match (failure) {
        Ok(_) => { strictEqual(1, 0); },
        Err(values) => { strictEqual(values.length, 3); strictEqual(values[1], 7); },
    }
    const wrapped: Result<Result<User, int[]>, string> = Ok(Ok({ name: "nested", count: 4 }));
    nested(wrapped);
    match (wrapped) { Ok(inner) => { match (inner) { Ok(user) => { strictEqual(user.count, 6); }, Err(_) => { strictEqual(1, 0); } } }, Err(_) => { strictEqual(1, 0); } }
    const integer: Result<int, string> = Ok(4);
    strictEqual(scalar(integer), 5);
    strictEqual(scalar(integer), 5);
    const edit = (result: Mutable<Result<User, int[]>>): void => {
        match (result) { Ok(user) => { user.count += 3; }, Err(values) => { values.push(8); } }
    };
    const viaArrow: Result<User, int[]> = Ok({ name: "arrow", count: 1 });
    edit(viaArrow);
    match (viaArrow) { Ok(user) => { strictEqual(user.count, 4); }, Err(_) => { strictEqual(1, 0); } }
    const counted = counterResult();
    strictEqual(tick(counted), 11);
    strictEqual(tick(counted), 12);
    match (counted) { Ok(action) => { strictEqual(action(), 13); }, Err(_) => { strictEqual(1, 0); } }
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn mutable_result_payload_borrows_cannot_escape_or_overlap_the_owner() {
    for source in [
        "function escape(result:Mutable<Result<string,int>>):string{match(result){Ok(value)=>{return value;},Err(_)=>{return \"fallback\";}}}function main(){}",
        "function escape(result:Mutable<Result<string,int>>):Function<string>{match(result){Ok(value)=>{return ():string=>value;},Err(_)=>{return ():string=>\"fallback\";}}}function main(){}",
        "function invoke(result:Mutable<Result<Once<Function<int>>,string>>):int{match(result){Ok(action)=>{return action();},Err(_)=>{return 0;}}}function main(){}",
        "function overlap(result:Mutable<Result<string,int>>):void{match(result){Ok(value)=>{console.log(result);console.log(value);},Err(_)=>{}}}function main(){}",
        "function clear(result:Mutable<Result<string,int>>):void{}function replace(result:Mutable<Result<string,int>>):void{match(result){Ok(value)=>{clear(result);console.log(value);},Err(_)=>{}}}function main(){}",
    ] {
        let formatted = support::format_round_trip(source);
        let error = tsrust_compiler::check(&formatted).err().expect("exclusive payload borrow must remain scoped");
        assert!(matches!(error.code, "borrowed-place" | "view-conflict"), "{}: {}", error.code, error.message);
    }
}

#[test]
fn readonly_result_matches_lend_payloads_without_consuming_the_caller() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";
type User = { name: string; };
function measure(result: ReadOnly<Result<User, string>>): int {
    match (result) {
        Ok(user) => { return user.name.length; },
        Err(error) => { return error.length; },
    }
}
function copied(result: ReadOnly<Result<User, string>>): string {
    match (result) { Ok(user) => { return clone(user.name); }, Err(error) => { return clone(error); } }
}
function main() {
    const user: Result<User, string> = Ok({ name: "Pat" });
    strictEqual(measure(user), 3);
    strictEqual(copied(user), "Pat");
    strictEqual(measure(user), 3);
    match (user) { Ok(value) => { strictEqual(value.name, "Pat"); }, Err(_) => { strictEqual(1, 0); } }
    const error: Result<User, string> = Err("failed");
    strictEqual(measure(error), 6);
    strictEqual(copied(error), "failed");
    strictEqual(measure(error), 6);
    const rows: Result<User, string>[] = [Ok({ name: "Pat" }), Err("failed")];
    let total = 0;
    for (const row of rows) {
        match (row) { Ok(value) => { total += value.name.length; }, Err(error) => { total += error.length; } }
    }
    strictEqual(total, 9);
    strictEqual(rows.length, 2);
    rows.pop();
    strictEqual(rows.length, 1);
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn readonly_result_match_payloads_cannot_escape_or_mutate_their_owner() {
    for source in [
        "function escape(result:ReadOnly<Result<string,int>>):string{match(result){Ok(value)=>{return value;},Err(_)=>{return \"fallback\";}}}function main(){}",
        "type User={name:string;};function edit(result:ReadOnly<Result<User,string>>):void{match(result){Ok(user)=>{user.name=\"changed\";},Err(_)=>{}}}function main(){}",
        "function edit(result:ReadOnly<Result<int[],string>>):void{match(result){Ok(values)=>{values.push(4);},Err(_)=>{}}}function main(){}",
        "function invoke(result:ReadOnly<Result<Once<Function<int>>,string>>):void{match(result){Ok(action)=>{action();},Err(_)=>{}}}function main(){}",
    ] {
        let formatted = support::format_round_trip(source);
        let error = tsrust_compiler::check(&formatted).err().expect("borrowed payload must not become an owned or mutable value");
        assert!(matches!(error.code, "borrowed-view" | "borrowed-place" | "readonly" | "callback-access"), "{}: {}", error.code, error.message);
    }
}

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
function relay(value: Result<Function<int, [value: int]>, string>): Result<Function<int, [input: int]>, string> { return value; }
function main() {
    const first = counter(10);
    const second = counter(20);
    match (first) { Ok(action) => { strictEqual(action(), 11); strictEqual(action(), 12); }, Err(_) => { strictEqual(1, 0); } }
    match (second) { Ok(action) => { strictEqual(action(), 21); }, Err(_) => { strictEqual(1, 0); } }
    const once = message("owned");
    match (once) { Ok(action) => { strictEqual(action(), "owned"); }, Err(_) => { strictEqual(1, 0); } }
    const explicit = Ok<Function<int, [value: int]>, string>((value: int): int => value + 3);
    const forwarded = relay(explicit);
    match (forwarded) { Ok(action) => { strictEqual(action(4), 7); }, Err(_) => { strictEqual(1, 0); } }
    const callableError: Result<int, Function<string>> = Err((): string => "error");
    match (callableError) { Ok(_) => { strictEqual(1, 0); }, Err(action) => { strictEqual(action(), "error"); } }
}
"#;
    let path = directory.source(&support::format_round_trip(source));
    support::success(&support::run(support::cli().arg("run").arg(path), 60));
}

#[test]
fn result_callable_payloads_reject_borrowed_storage_and_preserve_once_and_shape_errors() {
    for (source, code) in [
        ("function main(){const result:Result<ReadOnly<Function<int>>,string>=Ok(():int=>1);}", "callback-escape"),
        ("function main(){const result:Result<Mutable<Function<int>>,string>=Ok(():int=>1);}", "callback-escape"),
        ("function wrap(borrowed:ReadOnly<Function<int>>,owned:Function<int>):Result<Function<int>,string>{return Ok(borrowed);}function main(){}", "callback-contract"),
        ("function main(){const result:Result<Once<Function<string>>,int>=Ok(():string=>\"owned\");match(result){Ok(action)=>{action();action();},Err(_)=>{}}}", "closure-consumed"),
        ("function relay(value:Result<Function<int>,string>):Result<Once<Function<int>>,string>{return value;}function main(){}", "type"),
        ("function relay(value:Result<Function<int>,string>):Result<Function<string>,string>{return value;}function main(){}", "type"),
    ] { support::reject(source, code); }
}

#[test]
fn generic_result_wrappers_accept_explicit_callable_type_arguments() {
    let directory = support::TempDirectory::new();
    let source = r#"
import { strictEqual } from "std:testing";
function wrap<T>(value: T): Result<T, string> { return Ok(value); }
function main() {
    const result = wrap<Function<int>>((): int => 7);
    match (result) { Ok(action) => { strictEqual(action(), 7); }, Err(_) => { strictEqual(1, 0); } }
    let value = 40;
    const captured = wrap<Function<int>>((): int => { value += 1; return value; });
    match (captured) { Ok(action) => { strictEqual(action(), 41); strictEqual(action(), 42); }, Err(_) => { strictEqual(1, 0); } }
    const text = "once";
    const once = wrap<Once<Function<string>>>((): string => text);
    match (once) { Ok(action) => { strictEqual(action(), "once"); }, Err(_) => { strictEqual(1, 0); } }
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
