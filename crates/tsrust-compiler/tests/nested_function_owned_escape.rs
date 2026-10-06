mod support;

#[test]
fn escaping_named_functions_transfer_independent_captured_owners() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/nested-function-owned-escape/take-owner.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn consuming_named_captures_reject_reuse_and_conflicting_argument_borrows() {
    for (source, code) in [
        ("type User={name:string;};function main(){const user:User={name:\"Pat\"};function take():User{return user;}const saved=take();console.log(user.name);}", "transferred"),
        ("type User={name:string;};function make(user:User):()=>User{function take():User{return user;}return take;}function main(){const take=make({name:\"Pat\"});const first=take();const second=take();}", "closure-consumed"),
    ] {
        let error = tsrust_compiler::check(source).err().expect("transferred capture or callable");
        assert_eq!(error.code, code, "{}", error.message);
    }
    let source = "type User={name:string;};function main(){const user:User={name:\"Pat\"};function take(other:ReadOnly<User>):User{console.log(other.name);return user;}const saved=take(user);}";
    let error = tsrust_compiler::check(source).err().expect("capture transfer overlaps argument borrow");
    assert_eq!(error.code, "conflicting-access", "{}", error.message);
}

#[test]
fn retained_named_callbacks_preserve_storage_recursion_and_failure() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/nested-function-owned-escape/retained.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn retained_named_callbacks_infer_callable_array_storage() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/nested-function-owned-escape/retained-inferred.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn inferred_callable_arrays_check_contracts_and_invocation() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {strictEqual} from 'std:testing';function main(){const actions=[():int=>1,():int=>2];strictEqual(actions[0](),1);strictEqual(actions[1](),2);}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    let error = tsrust_compiler::check("function main(){const actions=[():int=>1,():string=>'x'];}").err().expect("incompatible callable results");
    assert!(matches!(error.code, "callback-result" | "type"), "{}: {}", error.code, error.message);
}

#[test]
fn retained_named_environments_drop_owners_once_on_success_and_failure() {
    let source = "type Item={id:int;};function make(item:Item):()=>int{function read():int{return item.id;}return read;}function failing(item:Item):()=>int{function fail():int{throw item.id;}return fail;}function take(item:Item):()=>Item{function move():Item{return item;}return move;}function recursive(item:Item):(n:int)=>int{function even(n:int):int{if(n===0){return item.id;}return odd(n-1);}function odd(n:int):int{if(n===0){return item.id+1;}return even(n-1);}return even;}function main(){{const first=make({id:1});const second=make({id:2});if(first()!==1||second()!==2){throw 99;}}{const fail=failing({id:3});let caught=false;try{fail();}catch(error){caught=error===3;}if(!caught){throw 99;}}{const move=take({id:4});const item=move();if(item.id!==4){throw 99;}}{const rec=recursive({id:5});if(rec(4)!==5||rec(3)!==6){throw 99;}}}";
    let rust = tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()", "fn source_main()") + r#"
static DROPS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
static SEEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {
    fn drop(&mut self) {
        DROPS.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        SEEN.fetch_or(1 << self.field0, std::sync::atomic::Ordering::SeqCst);
    }
}
fn main() {
    source_main();
    assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst), 5);
    assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst), 62);
}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("owned-disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("owned-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024", "-C", &format!("opt-level={level}")]).arg(&path).arg("-o").arg(&executable), 60));
        let output = support::run(&mut std::process::Command::new(executable), 10);
        support::success(&output);
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn inferred_once_callable_arrays_move_each_retained_owner() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {strictEqual} from 'std:testing';type User={name:string;};function make(user:User):()=>User{function take():User{return user;}return take;}function main(){const actions=[make({name:'Pat'}),make({name:'Lee'})];const first=actions.remove(0);const second=actions.remove(0);const pat=first();const lee=second();strictEqual(pat.name,'Pat');strictEqual(lee.name,'Lee');strictEqual(actions.length,0);}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn unused_throwing_named_factory_emits_its_checked_error_contract() {
    let directory = support::TempDirectory::new();
    let path = directory.source("function factory(value:int):()=>int{function fail():int{throw value;}return fail;}function main(){}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn borrowed_data_cannot_be_retained_by_an_owned_named_factory() {
    let source = "type User={name:string;};function bad(user:ReadOnly<User>):()=>string{function read():string{return user.name.trim();}return read;}function main(){}";
    let error = tsrust_compiler::check(source).err().expect("borrowed data cannot escape");
    assert_eq!(error.code, "parameter-access", "{}: {}", error.code, error.message);
    assert!(error.message.contains("ReadOnly") && error.message.contains("takes ownership"));
}

#[test]
fn dead_inner_declarations_do_not_require_outer_capture_initialization() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/nested-function-owned-escape/dead-inner.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn dead_inner_declarations_still_check_invalid_bodies_and_live_calls() {
    for (source, code) in [
        ("function main(){function outer():int{function unused():int{return later.missing;}return 7;}outer();const later=[1];}", "type"),
        ("function main(){function outer():int{function used():int{return later.length;}return used();}outer();const later=[1];}", "unassigned"),
        ("function main(){function outer():()=>int{function used():int{return later.length;}return used;}const retained=outer();const later=[1];}", "unassigned"),
    ] {
        let error=tsrust_compiler::check(source).err().expect("invalid body or live capture");
        assert_eq!(error.code, code, "{}: {}", error.code, error.message);
    }
}
