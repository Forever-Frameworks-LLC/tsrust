mod support;

fn native(source: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn readonly_capture_can_be_initialized_after_the_declaration() {
    native(include_str!("fixtures/nested-function-captures/readonly-later.tsr"));
}

#[test]
fn mutable_capture_is_reused_and_released_at_block_exit() {
    native(include_str!("fixtures/nested-function-captures/mutable-block.tsr"));
}

#[test]
fn recursive_group_and_forwarder_share_a_finite_read_environment() {
    native(include_str!("fixtures/nested-function-captures/recursive-forwarding.tsr"));
}

#[test]
fn loop_entries_create_fresh_borrowed_environments() {
    native(include_str!("fixtures/nested-function-captures/fresh-loop.tsr"));
}

#[test]
fn capture_readiness_is_checked_at_the_call_with_a_binding_location() {
    let source = "function main(){function read():int{return value;} console.log(read());const value=3;}";
    let error = tsrust_compiler::check(source).err().expect("uninitialized capture");
    assert_eq!(error.code, "unassigned", "{}", error.message);
    assert_eq!(&source[error.span.start..error.span.end], "read");
    assert!(error.related.iter().any(|location| &source[location.0.start..location.0.end] == "value"));
}

#[test]
fn existing_closure_borrowing_remains_the_reference_contract() {
    native(r#"import { strictEqual } from "std:testing";
type User = { name: string; };
function main() {
    const user: User = { name: " Pat " };
    const label = (): string => user.name.trim();
    strictEqual(label(), "Pat");
    strictEqual(user.name, " Pat ");
    const counts = [0];
    {
        const tick = (): int => { counts[0] += 1; return counts[0]; };
        strictEqual(tick(), 1);
        strictEqual(tick(), 2);
    }
    strictEqual(counts[0], 2);
}"#);
}

#[test]
fn transitive_function_use_requires_later_capture_initialization() {
    let source = "function main(){function read():int{return value;}function forward():int{return read();}console.log(forward());const value=3;}";
    let error = tsrust_compiler::check(source).err().expect("uninitialized transitive capture");
    assert_eq!(error.code, "unassigned");
    assert_eq!(&source[error.span.start..error.span.end], "forward");
    assert!(error.related.iter().any(|(span,_)| &source[span.start..span.end] == "value"));
}

#[test]
fn assigning_a_shadow_does_not_initialize_the_captured_binding() {
    let source = "function main(){let value:int;function read():int{return value;}{let value=0;value=3;}console.log(read());}";
    let error = tsrust_compiler::check(source).err().expect("outer binding still unassigned");
    assert_eq!(error.code, "unassigned");
    assert_eq!(&source[error.span.start..error.span.end], "read");
}

#[test]
fn checked_call_failure_keeps_capture_unassigned_on_the_catch_path() {
    for source in [
        "function fail():int{throw 1;}function main(){let value:int;function read():int{return value;}try{fail();value=3;}catch{console.log(read());}}",
        "function fail():int{throw 1;}function main(){let value:int;function read():int{return value;}try{value=fail();}catch{console.log(read());}}",
        "function fail():int{throw 1;}function main(){const callback=():int=>{let value:int;function read():int{return value;}try{value=fail();}catch{return read();}return 0;};console.log(callback());}",
    ] {
        let error = tsrust_compiler::check(source).err().expect("capture unavailable on exceptional path");
        assert_eq!(error.code, "unassigned", "{}", error.message);
        assert_eq!(&source[error.span.start..error.span.end], "read");
        assert!(error.related.iter().any(|(span, _)| &source[span.start..span.end] == "value"));
    }
}

#[test]
fn captured_body_uses_the_typed_defining_binding_before_environment_lowering() {
    for source in [
        "type User={name:string;};function main(){const user:User={name:\"Pat\"};function label():string{return user.missing;}console.log(label());}",
        "type User={name:string;};function main(){const user:User={name:\"Pat\"};function label():string{return user.missing;}{const user=3;console.log(label());}}",
        "type User={name:string;};function main(){const user:User={name:\"Pat\"};function label():string{return user.missing;}function forward():string{return label();}console.log(forward());}",
        "type User={name:string;};function main(){const user:User={name:\"Pat\"};function label():string{return user.missing;}function forward(user:int):string{return label();}console.log(forward(3));}",
        "type User={name:string;};function main(){const user:User={name:\"Pat\"};function label(count:int=user.missing):int{return count;}console.log(label());}",
    ] {
        let error = tsrust_compiler::check(source).err().expect("invalid captured field");
        assert_eq!(error.code, "type", "{}", error.message);
        assert!(error.message.contains("missing") && error.message.contains("User"), "{}", error.message);
        assert!(source[error.span.start..error.span.end].contains("missing"));
    }
}

#[test]
fn typed_scopes_restore_outer_bindings_and_allow_arrow_parameter_shadowing() {
    native(r#"import { strictEqual } from "std:testing";
function main(){
    let value=1;
    {let value=2;value+=1;strictEqual(value,3);}
    strictEqual(value,1);
    const base=4;
    const increment=(base:int):int=>base+1;
    strictEqual(increment(5),6);
    strictEqual(base,4);
}"#);
    assert_eq!(tsrust_compiler::check("function bad(value:int){const value=3;}function main(){}").err().unwrap().code, "duplicate");
}

// Direct calls pass captured bindings as hidden arguments: scalars and
// records changed in place, forwarding, recursion, nesting, calls from arrow
// functions, defaults, exceptions, loop bindings and later initialization.
#[test]
fn direct_calls_borrow_captures_for_each_call() {
    native(include_str!("fixtures/nested-function-captures/borrowed-calls.tsr"));
}

#[test]
fn function_values_borrow_captures_like_arrows() {
    native(include_str!("fixtures/nested-function-captures/function-values.tsr"));
}

#[test]
fn rest_callback_generic_and_recursive_signatures_receive_captures() {
    native(include_str!("fixtures/nested-function-captures/signatures.tsr"));
}

#[test]
fn omitted_results_generic_and_default_function_values_execute() {
    native(include_str!("fixtures/nested-function-captures/function-value-signatures.tsr"));
}

#[test]
fn capture_borrows_follow_argument_and_view_rules() {
    for (source, code) in [
        // The function changes `counts` while an argument lends it too.
        ("function main(){const counts=[0];function grow(values:int[]):void{values.push(1);counts.push(2);}grow(counts);}", "conflicting-access"),
        // A view of an element is alive across a call that changes its owner.
        ("function main(){const names=[\"a\"];function add():void{names.push(\"b\");}const first=names[0];add();console.log(first);}", "view-conflict"),
        // An arrow that calls the function holds the capture like any closure.
        ("function main(){const counts=[0];function tick():int{counts[0]+=1;return counts[0];}const run=():int=>tick();console.log(run(),counts);}", "capture-conflict"),
        // A const scalar cannot be changed through a nested function either.
        ("function main(){const total=0;function add():void{total+=1;}add();}", "unsupported"),
        // A function value holds the captures like the arrow that calls the function.
        ("function main(){const counts=[0];function tick():int{counts[0]+=1;return counts[0];}const run=tick;console.log(run(),counts);}", "capture-conflict"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap_or_else(|| panic!("must be rejected: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
    // Reading the captured binding in an argument and lending it for reading are fine.
    for source in [
        "function main(){const counts=[1,2];function total(values:int[]):int{return values.length+counts.length;}console.log(total(counts));}",
        "function main(){const counts=[0];function tick(n:int):int{counts[0]+=n;return counts[0];}console.log(tick(counts.length));}",
    ] {
        tsrust_compiler::check(source).unwrap_or_else(|error| panic!("{source}: {}", error.message));
    }
}

#[test]
fn uncalled_capturing_functions_are_still_checked() {
    tsrust_compiler::check("function main(){const names=[\"a\"];function unused():string[]{return names;}}").unwrap();
    for (source, code) in [
        ("function main(){const base=[1];function unused():string{return base[0];}}", "type"),
        ("function main(){const base=[1];function outer():void{function inner():int{return base.missing;}}outer();}", "type"),
        ("function main(){const total=0;function unused():void{total+=1;}}", "unsupported"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap_or_else(|| panic!("must be rejected: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
        assert!(!error.message.contains("pending"), "{source}: {}", error.message);
    }
    native("function main(){const base=[1];let later:int;function unused():int{return base[0]+later+other();}function other():int{return base.length;}later=2;}");
}

#[test]
fn function_values_keep_their_captures_across_modules() {
    let directory = support::TempDirectory::new();
    std::fs::write(directory.0.join("helper.tsr"), include_str!("fixtures/nested-function-captures/modules/helper.tsr")).unwrap();
    let main = directory.0.join("main.tsr");
    std::fs::write(&main, include_str!("fixtures/nested-function-captures/modules/main.tsr")).unwrap();
    let output = support::run(support::cli().arg("run").arg(main), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
