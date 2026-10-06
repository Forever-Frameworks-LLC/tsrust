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
    let source = "function fail():int{throw 1;}function main(){let value:int;function read():int{return value;}try{fail();value=3;}catch{console.log(read());}}";
    let error = tsrust_compiler::check(source).err().expect("capture unavailable on exceptional path");
    assert_eq!(error.code, "unassigned");
    assert_eq!(&source[error.span.start..error.span.end], "read");
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
