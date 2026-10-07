mod support;

fn native(source: &str, expected: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn generic_value_adapters_preserve_borrowed_and_owned_factory_state() {
    native(include_str!("fixtures/generic-value-adapters/contracts.tsr"), "11\n12\n13\n");
}

#[test]
fn generic_value_adapters_supply_contextual_array_callbacks() {
    native(include_str!("fixtures/generic-value-adapters/callbacks.tsr"), "11\n12\n13\n");
}

#[test]
fn generic_value_adapters_support_record_and_array_storage() {
    native(include_str!("fixtures/generic-value-adapters/storage.tsr"), "11\n21\n22\n");
}

#[test]
fn generic_value_adapters_preserve_transfer_and_type_constraints() {
    let source="function make(){const saved='owner';function select<T>(value:T):T{console.log(saved);return value;}return select;}function main(){const action=make();const fixed:Function<int,[value:int]>=action;action(1);}";
    let error=tsrust_compiler::check(source).err().expect("owned adapter transfers its environment");
    assert!(matches!(error.code,"transferred"|"closure-consumed"),"{}",error.render("input.tsr",source));
    for source in [
        "function identity<T>(value:T):T{return value;}function main(){const action=identity;const fixed:Function<string,[value:int]>=action;}",
        "function identity<T>(value:T):T{return value;}function main(){const action=identity;const fixed:Function<string,[value:ReadOnly<string>]>=action;}",
    ] {
        let error=tsrust_compiler::check(source).err().expect("adapter must retain source type and access constraints");
        assert!(matches!(error.code,"type"|"generic-inference"|"callback-contract"|"callback-access"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn generic_value_adapters_reject_live_exclusive_borrows_and_consuming_reuse() {
    let source="function make(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}return select;}function main(){const action=make();const fixed:Mutable<Function<int,[value:int]>>=action;action(1);fixed(2);}";
    let error=tsrust_compiler::check(source).err().expect("mutable adapter excludes direct access while live");
    assert!(matches!(error.code,"capture-conflict"|"conflicting-access"|"borrow"),"{}",error.render("input.tsr",source));
    let source="type Item={id:int;};function make(item:Item){function take<T>(value:T):Item{return item;}return take;}function main(){const action=make({id:1});const fixed:Once<Function<Item,[value:int]>>=action;fixed(1);fixed(2);}";
    let error=tsrust_compiler::check(source).err().expect("consuming adapter cannot be invoked twice");
    assert!(matches!(error.code,"closure-consumed"|"transferred"),"{}",error.render("input.tsr",source));
}

#[test]
fn generic_value_adapters_drop_each_retained_owner_once_on_success_and_failure() {
    let source="type Item={id:int;};type Holder={run:Function<int,[value:int]>;};function make(item:Item){function select<T>(value:T):T{if(item.id<0){throw item.id;}return value;}return select;}function fail(item:Item){function select<T>(value:T):T{throw item.id;}return select;}function main(){{const action=make({id:1});{const view:Mutable<Function<int,[value:int]>>=action;if(view(7)!==7){throw 99;}}const owned:Function<string,[value:Owned<string>]>=action;if(owned('a')!=='a'){throw 99;}}{const action=make({id:2});const holder:Holder={run:action};if(holder.run(3)!==3){throw 99;}}{const action=fail({id:3});const owned:Function<int,[value:int]>=action;let caught=0;try{owned(1);}catch(error){if(error===3){caught+=1;}}if(caught!==1){throw 99;}}}";
    let rust=tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),3);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),14);}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.0.join("adapter-disposal.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("adapter-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
        let output=support::run(&mut std::process::Command::new(binary),10);
        support::success(&output);assert!(output.stdout.is_empty());
    }
}

#[test]
fn generic_value_adapters_specialize_nominal_generic_results() {
    native("type Receipt<T>={value:T;calls:int;};function make(){let calls=0;function select<T>(value:T):Receipt<T>{calls+=1;return {value:value,calls:calls};}return select;}function main(){const action=make();{const view:Mutable<Function<Receipt<int>,[value:int]>>=action;const first=view(7);console.log(first.value,first.calls);}const second=action('x');console.log(second.value,second.calls);}", "7 1\nx 2\n");
}

#[test]
fn generic_value_adapters_reduce_existing_environments() {
    native(include_str!("fixtures/generic-value-adapters/reduction.tsr"), "1\n2\n3\n9\n4\nafter\n");
}

#[test]
fn generic_value_adapters_sort_and_search_existing_environments() {
    native(include_str!("fixtures/generic-value-adapters/sort-search.tsr"), "1\nfalse\n");
}

#[test]
fn generic_value_adapters_preserve_default_effects_and_rest_packs() {
    native(include_str!("fixtures/generic-value-adapters/default-rest.tsr"), "1\n1\n9\n2\n2\n3\n3\n1 0\n2 2\n3 1\n");
}

#[test]
fn generic_value_adapter_owned_rest_disposes_once_on_success_and_failure() {
    let source = "type Item={id:int;};function make(item:Item){function select<T>(fallback:T,...values:T[]):T{if(item.id===4){throw 4;}return fallback;}return select;}function main(){{const action=make({id:1});const owned:Function<Item,[fallback:Owned<Item>,...values:Item[]]>=action;const result=owned({id:2},{id:3});if(result.id!==2){throw 99;}}{const action=make({id:4});const owned:Function<Item,[fallback:Owned<Item>,...values:Item[]]>=action;let caught=false;try{owned({id:5},{id:6});}catch(error){caught=error===4;}if(!caught){throw 99;}}}";
    let rust = tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()", "fn source_main()") + r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),6);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),126);}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("owned-rest-disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("owned-rest-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024", "-C", &format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary), 60));
        support::success(&support::run(&mut std::process::Command::new(binary), 10));
    }
}

#[test]
fn generic_value_adapter_owned_rest_rejects_transferred_sources_and_elements() {
    let prefix="type Item={id:int;};function make(){const marker='owner';function select<T>(fallback:T,...values:T[]):T{console.log(marker);return fallback;}return select;}";
    for body in [
        "const action=make();const owned:Function<Item,[fallback:Owned<Item>,...values:Item[]]>=action;action(1);",
        "const action=make();const owned:Function<Item,[fallback:Owned<Item>,...values:Item[]]>=action;const tail:Item={id:3};owned({id:2},tail);console.log(tail.id);",
    ] {
        let source=format!("{prefix}function main(){{{body}}}");
        let error=tsrust_compiler::check(&source).err().expect("owned adapter and rest elements transfer once");
        assert!(matches!(error.code,"transferred"|"closure-consumed"),"{}",error.render("input.tsr",&source));
    }
}

#[test]
fn generic_value_adapters_solve_result_only_and_nullable_contracts() {
    native("function make(){let calls=0;function fresh<T>():T[]{calls+=1;console.log(calls);return [];}return fresh;}function main(){const action=make();{const view:Mutable<Function<int[],[]>>=action;const values=view();console.log(values.length);}const texts:string[]=action();console.log(texts.length);const optional:Function<int[],[]>|undefined=action;const values=optional!();console.log(values.length);}", "1\n0\n2\n0\n3\n0\n");
}

#[test]
fn generic_value_adapters_contextualize_factory_call_results() {
    native(include_str!("fixtures/generic-value-adapters/factory-expression.tsr"), "10\n11\n12\n20\n21\n22\n");
}

#[test]
fn fresh_generic_adapters_dispose_environments_on_factory_and_callback_failure() {
    let source="type Item={id:int;};function make(item:Item){if(item.id===3){throw 3;}function select<T>(value:T):T{if(item.id===2){throw 2;}return value;}return select;}function main(){{const fixed:Function<int,[value:int]>=make({id:1});if(fixed(7)!==7){throw 99;}}let caught=0;try{const fixed:Function<int,[value:int]>=make({id:2});fixed(7);}catch(error){if(error===2){caught+=1;}}try{const fixed:Function<int,[value:int]>=make({id:3});fixed(7);}catch(error){if(error===3){caught+=1;}}if(caught!==2){throw 99;}}";
    let rust=tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),3);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),14);}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("fresh-adapter-disposal.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("fresh-adapter-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),10));
    }
}

#[test]
fn fresh_generic_adapters_reject_borrowed_temporary_escape() {
    let source="function make(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}return select;}function main(){const view:Mutable<Function<int,[value:int]>>=make();view(7);}";
    let error=tsrust_compiler::check(source).err().expect("temporary owner cannot outlive borrowed facade");
    assert_eq!(error.code,"closure-escape","{}",error.render("input.tsr",source));
}

#[test]
fn fresh_generic_adapters_accept_local_factory_aliases() {
    native("function make(start:int){console.log(start);let calls=start;function select<T>(value:T):T{calls+=1;console.log(calls);return value;}return select;}function main(){const factory=make;const fixed:Function<int,[value:int]>=factory(10);if(fixed(7)!==7){throw 99;}const mapped=[3,4].map(factory(20));if(mapped[1]!==4){throw 99;}}", "10\n11\n20\n21\n22\n");
}

#[test]
fn fresh_generic_adapters_accept_selected_factory_calls() {
    native("function make(start:int){console.log(start);let calls=start;function select<T>(value:T):T{calls+=1;console.log(calls);return value;}return select;}function main(){const factory=make;const fixed:Function<int,[value:int]>=(true?factory:factory)(10);if(fixed(7)!==7){throw 99;}const mapped=[3,4].map((false?factory:factory)(20));if(mapped[1]!==4){throw 99;}}", "10\n11\n20\n21\n22\n");
}

#[test]
fn fresh_generic_adapters_specialize_generic_factories_once() {
    native("function make<T>(seed:T){console.log(seed);function select<U>(value:U):U{console.log(seed);return value;}return select;}function main(){const fixed:Function<int,[value:int]>=make('owner');if(fixed(7)!==7){throw 99;}const mapped=[3,4].map(make(10));if(mapped[1]!==4){throw 99;}}", "owner\nowner\n10\n10\n10\n");
}

#[test]
fn fresh_generic_adapters_select_different_factory_origins_once() {
    native(include_str!("fixtures/generic-value-adapters/selected-factories.tsr"), "left 1\n2\nright 2\n4\n6\n");
}

#[test]
fn selected_generic_factory_adapters_reject_incompatible_results_and_borrowed_escape() {
    let declarations = "function left(){function identity<T>(value:T):T{return value;}return identity;}function right(){function stringify<T>(value:T):string{return String(value);}return stringify;}";
    let source = format!("{declarations}function main(){{const fixed:Function<int,[value:int]>=(true?left:right)();}}");
    let error = tsrust_compiler::check(&source).err().expect("both branches must satisfy the result contract");
    assert!(matches!(error.code, "type" | "generic-inference"), "{}", error.render("input.tsr", &source));
    let source = "function left(){let calls=0;function identity<T>(value:T):T{calls+=1;return value;}return identity;}function right(){let calls=10;function identity<T>(value:T):T{calls+=1;return value;}return identity;}function main(){const view:Mutable<Function<int,[value:int]>>=(true?left:right)();view(7);}";
    let error = tsrust_compiler::check(source).err().expect("fresh selected owner cannot escape through a borrowed facade");
    assert_eq!(error.code, "closure-escape", "{}", error.render("input.tsr", source));
}

#[test]
fn selected_generic_factories_drop_only_selected_owners_once() {
    let source = "type Item={id:int;};function left(item:Item){function select<T>(value:T):T{if(item.id===2){throw 2;}return value;}return select;}function right(item:Item){if(item.id===3){throw 3;}function select<T>(value:T):T{if(item.id===2){throw 2;}return value;}return select;}function main(){{const action:Function<int,[value:int]>=(true?left:right)({id:1});if(action(7)!==7){throw 99;}}let caught=0;try{const mapped=[7].map((false?left:right)({id:2}));}catch(error){if(error===2){caught+=1;}}try{const action:Function<int,[value:int]>=(false?left:right)({id:3});}catch(error){if(error===3){caught+=1;}}if(caught!==2){throw 99;}}";
    let rust = tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()", "fn source_main()") + r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),3);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),14);}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("selected-disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("selected-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024", "-C", &format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary), 60));
        support::success(&support::run(&mut std::process::Command::new(binary), 10));
    }
}

#[test]
fn selected_generic_factory_callbacks_cover_traversal_contracts() {
    native(include_str!("fixtures/generic-value-adapters/selected-traversal.tsr"), "");
}

#[test]
fn fresh_generic_adapters_cannot_retain_borrowed_factory_data() {
    let source = "type User={name:string;};function make(user:ReadOnly<User>){function select<T>(value:T):T{console.log(user.name);return value;}return select;}function main(){const user:User={name:'owner'};const fixed:Function<int,[value:int]>=make(user);fixed(7);}";
    let error = tsrust_compiler::check(source).err().expect("an owned generic environment cannot retain borrowed parameter data");
    assert_eq!(error.code, "parameter-access", "{}: {}", error.code, error.message);
}

#[test]
fn nested_selected_generic_factories_evaluate_only_selected_predicates() {
    native(include_str!("fixtures/generic-value-adapters/nested-selected-factories.tsr"), "left 1\n2\nright 2\n4\n6\n");
}

#[test]
fn fresh_generic_factory_adapters_support_aggregate_destinations() {
    native(include_str!("fixtures/generic-value-adapters/fresh-storage.tsr"), "10\n11\n20\n30\n21\n22\n32\n");
}

#[test]
fn fresh_generic_factory_adapters_infer_omitted_results() {
    native(include_str!("fixtures/generic-value-adapters/inferred-factory-result.tsr"), "10\n11\n12\n20\n21\n22\n");
}

#[test]
fn fresh_generic_result_inference_does_not_relax_explicit_result_contracts() {
    let source = "function make(){function identity<T>(value:T){return value;}return identity;}function main(){const fixed:Function<string,[value:int]>=make();}";
    let error = tsrust_compiler::check(source).err().expect("inferred integer result cannot satisfy a string contract");
    assert_eq!(error.code, "type", "{}: {}", error.code, error.message);
}

#[test]
fn fresh_generic_adapters_preserve_nested_callable_parameter_contracts() {
    native(include_str!("fixtures/generic-value-adapters/nested-callable-input.tsr"), "1\n2\n");
}

#[test]
fn fresh_generic_adapters_preserve_void_callable_inputs() {
    native("function observe(value:int):void{console.log(value);}function make(){let calls=0;function invoke<T>(action:Function<void,[value:T]>,value:T):void{calls+=1;action(value);console.log(calls);}return invoke;}function main(){const fixed:Function<void,[action:Function<void,[value:int]>,value:int]>=make();fixed(observe,7);fixed(observe,8);}", "7\n1\n8\n2\n");
}

#[test]
fn fresh_generic_adapters_preserve_nested_default_and_rest_contracts() {
    native("function sum(value:int=4,...tail:int[]):int{let result=value;for(const item of tail){result+=item;}return result;}function make(){let calls=0;function invoke<T>(action:Function<T,[value?:T,...tail:T[]]>,value:T):T{calls+=1;console.log(calls);return action(value,value);}return invoke;}function main(){const fixed:Function<int,[action:Function<int,[value?:int,...tail:int[]]>,value:int]>=make();console.log(fixed(sum,7),fixed(sum,8));}", "1\n2\n14 16\n");
}

#[test]
fn fresh_generic_adapters_preserve_nested_callable_results() {
    native("function increment(value:int):int{return value+1;}function make(){let calls=0;function select<T>(action:Function<T,[value:T]>):Function<T,[value:T]>{calls+=1;console.log(calls);return action;}return select;}function main(){const fixed:Function<Function<int,[value:int]>,[action:Function<int,[value:int]>]>=make();const first=fixed(increment);const second=fixed(increment);console.log(first(7),second(8));}", "1\n2\n8 9\n");
}

#[test]
fn fresh_generic_adapters_preserve_borrowed_callback_inputs() {
    native("function make(){let calls=0;function invoke<T>(action:ReadOnly<Function<T,[value:T]>>,value:T):T{calls+=1;console.log(calls);return action(value);}return invoke;}function main(){const action=(value:int):int=>value+1;const fixed:Function<int,[action:ReadOnly<Function<int,[value:int]>>,value:int]>=make();console.log(fixed(action,7),fixed(action,8),action(9));}", "1\n2\n8 9 10\n");
}

#[test]
fn nested_callable_results_reject_incompatible_invocation_contracts() {
    let source = "function make(){let calls=0;function select<T>(action:Once<Function<T,[value:T]>>,value:T):Once<Function<T,[value:T]>>{calls+=1;console.log(calls);return action;}return select;}function main(){const fixed:Function<Function<int,[value:int]>,[action:Once<Function<int,[value:int]>>,value:int]>=make();}";
    let error = tsrust_compiler::check(source).err().expect("single-use results cannot satisfy reusable callable contracts");
    assert_eq!(error.code, "callback-contract", "{}: {}", error.code, error.message);
}

#[test]
fn nullable_fresh_generic_adapters_preserve_presence_and_state() {
    native(include_str!("fixtures/generic-value-adapters/nullable-factory.tsr"), "1\n2\n");
}

#[test]
fn selected_generic_factories_join_heterogeneous_results() {
    native(include_str!("fixtures/generic-value-adapters/heterogeneous-factories.tsr"), "1\n1\n");
}

#[test]
fn generic_nullable_constraints_accept_existing_payload_members() {
    native("import { strictEqual } from 'std:testing';function main(){const values=[7];const value=values.pop();strictEqual(value,7);}", "");
}

#[test]
fn generic_factory_values_preserve_callable_type_payloads() {
    native(include_str!("fixtures/generic-value-adapters/callable-payload.tsr"), "1\n2\n");
}

#[test]
fn generic_callable_payload_signatures_keep_one_environment_and_distinct_methods() {
    native(include_str!("fixtures/generic-value-adapters/callable-payload-signatures.tsr"), "1\n2\n");
}

#[test]
fn generic_callable_payload_signatures_reject_different_argument_access() {
    let source = include_str!("fixtures/generic-value-adapters/callable-payload-signatures.tsr")
        .replace("import { strictEqual } from 'std:testing';", "function strictEqual<T>(actual:T,expected:T):void{}")
        .replace("(value: Owned<string>): string =>", "(value: string): string =>");
    let error = tsrust_compiler::check(&source).err().expect("borrowed argument callback cannot become consuming");
    assert_eq!(error.code, "type", "{}: {}", error.code, error.message);
}

#[test]
fn generic_callable_payloads_drop_original_captures_once_on_success_and_failure() {
    let source = "type Item={id:int;};function make(marker:Item){function identity<T>(value:T):T{if(marker.id===2){throw 2;}return value;}return identity;}function main(){{const fixed:Function<Function<int,[]>,[value:Function<int,[]>]>=make({id:1});const item:Item={id:3};const action:Function<int,[]>=():int=>item.id;const result=fixed(action);if(result()!==3){throw 99;}}{const fixed:Function<Function<int,[]>,[value:Function<int,[]>]>=make({id:2});const item:Item={id:4};const action:Function<int,[]>=():int=>item.id;let caught=false;try{const result=fixed(action);result();}catch(error){caught=error===2;}if(!caught){throw 99;}}}";
    let rust = tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()", "fn source_main()") + r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),4);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),30);}
"#;
    let directory = support::TempDirectory::new();
    let path = directory.0.join("callable-payload-disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("callable-payload-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024", "-C", &format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary), 60));
        support::success(&support::run(&mut std::process::Command::new(binary), 10));
    }
}

#[test]
fn generic_callable_payloads_preserve_once_invocation() {
    native(include_str!("fixtures/generic-value-adapters/callable-payload-once.tsr"), "1\n");
}

#[test]
fn generic_callable_payloads_borrow_readonly_environments() {
    native(include_str!("fixtures/generic-value-adapters/callable-payload-readonly.tsr"), "1 5\n2 5\n5\n");
}

#[test]
fn generic_callable_payloads_borrow_mutable_environments() {
    native(include_str!("fixtures/generic-value-adapters/callable-payload-mutable.tsr"), "1 1\n2 2\n3\n");
}

#[test]
fn generic_callable_payloads_reject_repeated_once_invocation() {
    let source = include_str!("fixtures/generic-value-adapters/callable-payload-once.tsr")
        .replace("import { strictEqual } from 'std:testing';", "function strictEqual(actual:int,expected:int):void{}")
        .replace("const result = action();", "const result = action(); action();");
    let error = tsrust_compiler::check(&source).err().expect("a transferred once callback cannot be invoked twice");
    assert_eq!(error.code, "closure-consumed", "{}: {}", error.code, error.message);
}

#[test]
fn generic_callable_payloads_support_explicit_source_type_applications() {
    native(include_str!("fixtures/generic-value-adapters/explicit-callable-payload.tsr"), "1\n2\n");
}

#[test]
fn explicit_generic_values_preserve_constraints_and_reject_nongeneric_calls() {
    for source in [
        "function identity<T>(value:T):T{return value;}function main(){const action=identity;action<int>('wrong');}",
        "function identity<T>(value:T):T{return value;}function main(){const action=identity;action<int,string>(1);}",
        "function pair<T>(first:T,second:T):T{return first;}function main(){const action=pair;action<int>(1,'wrong');}",
        "function main(){const action=(value:int):int=>value;action<int>(1);}",
        "function identity<T>(value:T):T{return value;}function main(){const action=identity;const text='owner'.repeat(1);action<string>(text);console.log(text);}",
    ] {
        let error=tsrust_compiler::check(source).err().expect("explicit applications must preserve constraints");
        assert!(matches!(error.code,"type"|"arity"|"transferred"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn explicit_generic_values_substitute_enclosing_types_and_result_only_arguments() {
    native("function identity<T>(value:T):T{return value;}function wrap<T>(value:T):T{const action=identity;return action<T>(value);}function empty<T>():T[]{return [];}function main(){console.log(wrap(7),wrap('owner'));const action=empty;const ints=action<int>();const strings=action<string>();console.log(ints.length,strings.length);}", "7 owner\n0 0\n");
}

#[test]
fn explicit_generic_values_keep_one_captured_environment_across_types() {
    native("function make(){let calls=0;function identity<T>(value:T):T{calls+=1;console.log(calls);return value;}return identity;}function main(){const action=make();console.log(action<int>(7));console.log(action<string>('owner'));console.log(action(8));}", "1\n7\n2\nowner\n3\n8\n");
}

#[test]
fn explicit_generic_values_forward_defaults_and_rest_without_reconstruction() {
    native("function make(){let calls=0;function collect<T>(fallback:T,offset:int=4,...values:T[]):T{calls+=1;console.log(calls,offset,values.length);return fallback;}return collect;}function main(){const action=make();console.log(action<int>(7));console.log(action<int>(8,undefined,9,10));console.log(action<string>('owner',5,'extra'));}", "1 4 0\n7\n2 4 2\n8\n3 5 1\nowner\n");
}

#[test]
fn explicit_generic_values_do_not_repeat_consuming_capture_calls() {
    let source="type Item={id:int;};function make(item:Item){function take<T>(value:T):Item{return item;}return take;}function main(){const action=make({id:7});action<int>(1);action<int>(2);}";
    let error=tsrust_compiler::check(source).err().expect("explicit calls do not restore consumed captures");
    assert!(matches!(error.code,"closure-consumed"|"transferred"),"{}",error.render("input.tsr",source));
}

#[test]
fn optional_explicit_generic_values_skip_absent_arguments_and_retain_state() {
    native(include_str!("fixtures/generic-value-adapters/optional-explicit-generic.tsr"), "1\n2\n");
}

#[test]
fn optional_explicit_generic_aliases_skip_absent_arguments() {
    native("function make(){function identity<T>(value:T):T{return value;}return identity;}function main(){let effects=0;const candidate=make();const absent=false?candidate:undefined;const result=absent?.<int>(effects++);console.log(result??-1,effects);}", "-1 0\n");
}

#[test]
fn selected_generic_calls_do_not_recheck_owned_factory_arguments() {
    native("type Item={id:int;};function make(item:Item){function identity<T>(value:T):T{console.log(item.id);return value;}return identity;}function main(){const item:Item={id:7};console.log(make(item)(8));}", "7\n8\n");
}

#[test]
fn optional_generic_calls_keep_result_only_context() {
    native("function make(){function empty<T>():T[]{return [];}return empty;}function main(){const action=true?make():undefined;const values:int[]|undefined=action?.();console.log(values?.length??-1);}", "0\n");
}

#[test]
fn optional_generic_calls_preserve_failures_and_reuse_the_original_environment() {
    native("function make(){let calls=0;function select<T>(value:T):T{calls+=1;if(calls===2){throw calls;}return value;}return select;}function main(){const action=true?make():undefined;try{action?.<int>(1);action?.<int>(2);}catch(error){console.log(error);}console.log(action?.<int>(3)??-1);}", "2\n3\n");
}

#[test]
fn optional_generic_calls_preserve_type_and_receiver_loan_constraints() {
    for source in [
        "function main(){const candidate=(value:int):int=>value;const action=false?candidate:undefined;action?.<int>(1);}",
        "function identity<T>(value:T):T{return value;}function main(){const candidate=identity;const action=false?candidate:undefined;action?.<int>('wrong');}",
        "function identity<T>(value:T):T{return value;}function main(){const candidate=identity;const action=true?candidate:undefined;action?.<int,string>(1);}",
        "function make(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}return select;}function main(){const action=true?make():undefined;action?.<int>(action?.<int>(1)??0);}",
    ] {
        let error=tsrust_compiler::check(source).err().expect("optional calls retain type and receiver constraints");
        assert!(matches!(error.code,"type"|"arity"|"conflicting-access"|"borrow"|"capture-conflict"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn optional_generic_calls_drop_original_owners_once_on_success_and_failure() {
    let source="type Item={id:int;};function make(item:Item){let calls=0;function identity<T>(value:T):T{calls+=1;if(item.id===1&&calls===2){throw calls;}return value;}return identity;}function main(){const action=true?make({id:1}):undefined;const first=action?.<Item>({id:2});if((first?.id??0)!==2){throw 99;}let caught=0;try{action?.<Item>({id:3});}catch(error){if(error===2){caught+=1;}}if(caught!==1){throw 99;}}";
    let rust=tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),3);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),14);}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("optional-generic-disposal.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("optional-generic-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),10));
    }
}

#[test]
fn selected_generic_calls_keep_consuming_capture_ownership() {
    native("type Item={id:int;};function make(item:Item){function take<T>(value:T):Item{return item;}return take;}function main(){const result=make({id:7})(1);console.log(result.id);}", "7\n");
    let source="type Item={id:int;};function make(item:Item){function take<T>(value:T):Item{return item;}return take;}function main(){const action=true?make({id:7}):undefined;action?.<int>(1);}";
    let error=tsrust_compiler::check(source).err().expect("a named optional selection cannot consume a borrowed owner");
    assert!(matches!(error.code,"borrowed-place"|"borrowed-view"|"type"),"{}",error.render("input.tsr",source));
}

#[test]
fn callable_payload_results_keep_default_effects_and_spread_rest_contracts() {
    native(include_str!("fixtures/generic-value-adapters/callable-payload-default-rest.tsr"), "1\ndefault\ndefault\n");
}

#[test]
fn unused_generic_alias_applications_still_validate_type_annotations() {
    let source="function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{const alias=identity;return alias<Missing>(value);}function main(){}";
    let error=tsrust_compiler::check(source).err().expect("unused templates still validate explicit annotations");
    assert_eq!(error.code,"type","{}",error.render("input.tsr",source));
}

#[test]
fn generic_adapters_preserve_owned_array_parameter_patterns() {
    native(include_str!("fixtures/generic-value-adapters/array-parameter-pattern.tsr"), "1\n2\n");
}

#[test]
fn generic_adapters_preserve_owned_record_parameter_patterns() {
    native(include_str!("fixtures/generic-value-adapters/record-parameter-pattern.tsr"), "1\n2\n");
}

#[test]
fn owned_pattern_adapters_dispose_each_original_record_once_on_failure() {
    let source="type Item={id:int;};function make(){let calls=0;function first<T>([first,second]:Owned<T[]>):T{calls+=1;return first;}return first;}function main(){const action=make();{const fixed:Mutable<Function<Item,[values:Owned<Item[]>]>>=action;const values:Item[]=[{id:1},{id:2}];const result=fixed(values);if(result.id!==1){throw 99;}}let caught=0;try{const values:Item[]=[{id:3}];action(values);}catch(error){caught+=1;}if(caught!==1){throw 99;}}";
    let rust=tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),3);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),14);}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("owned-pattern-disposal.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("owned-pattern-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),10));
    }
}

#[test]
fn pattern_adapters_supply_record_array_literal_context() {
    native("type Item={id:int;};function make(){let calls=0;function first<T>([first,second]:Owned<T[]>):T{calls+=1;return first;}return first;}function main(){const action=make();const fixed:Mutable<Function<Item,[values:Owned<Item[]>]>>=action;const result=fixed([{id:7},{id:8}]);console.log(result.id);}", "7\n");
}

#[test]
fn owned_pattern_contracts_match_the_typescript_reference() {
    native(include_str!("fixtures/generic-value-adapters/owned-pattern-contracts.tsr"), "");
}

#[test]
fn generic_adapters_allow_discarding_returned_callable_values() {
    native("function sum(value:int=4,...tail:int[]):int{return value;}function make(){let calls=0;function identity<T>(value:T):T{calls+=1;console.log(calls);return value;}return identity;}function main(){const action=make();{const view:Mutable<Function<Function<int,[value?:int,...tail:int[]]>,[action:Function<int,[value?:int,...tail:int[]]>]>>=action;view(sum);}action(7);}", "1\n2\n");
}

#[test]
fn discarded_factory_call_disposes_its_owned_environment_once() {
    let source="type Item={id:int;};function make(item:Item){return ():int=>item.id;}function main(){make({id:1});}";
    let rust=tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){assert_eq!(self.field0,1);DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),1);}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("discarded-factory.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("discarded-factory-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),10));
    }
}

#[test]
fn unused_generic_aliases_validate_explicit_arity_and_monomorphic_targets() {
    let mut accepted = Vec::new();
    for source in [
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{const alias=identity;return alias<T,T>(value);}function main(){}",
        "function exact(value:int):int{return value;}function unused<T>(value:T):int{const alias=exact;return alias<T>(1);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{const alias=identity;return alias(value,value);}function main(){}",
    ] {
        match tsrust_compiler::check(source) {
            Ok(_) => accepted.push(source),
            Err(error) => assert!(matches!(error.code,"arity"|"type"),"{}",error.render("input.tsr",source)),
        }
    }
    assert!(accepted.is_empty(), "Unused aliases accepted invalid calls: {accepted:#?}");
}

#[test]
fn generic_adapters_reject_borrowed_results_escaping_as_owned_values() {
    for source in [
        "function lend<T>(value:ReadOnly<T>):T{return value;}function main(){const action=lend;const fixed:Function<string,[value:ReadOnly<string>]>=action;}",
        "function make(){let calls=0;function lend<T>(value:ReadOnly<T>):T{calls+=1;return value;}return lend;}function main(){const action=make();const fixed:Mutable<Function<string,[value:ReadOnly<string>]>>=action;}",
    ] {
        let error=tsrust_compiler::check(source).err().expect("owned results cannot escape borrowed inputs");
        assert!(matches!(error.code,"borrowed-place"|"borrowed-view"|"borrowed-element"|"borrow"|"parameter-access"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn unused_generic_alias_validation_preserves_scopes_defaults_and_spreads() {
    for source in [
        "function recurse<T>(value:T):T{const alias=recurse;return alias<T>(value);}function main(){}",
        "function unused<T>(value:T):T{const alias=identity;return alias<T>(value);}function identity<T>(value:T):T{return value;}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{const first=identity;const second=first;return second<T>(value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{const alias=identity;{const alias=1;}return alias<T>(value);}function main(){}",
        "function collect<T>(value:T,offset:int=4,...tail:T[]):T{return value;}function unused<T>(value:T):T{const alias=collect;alias(value);return alias(value,undefined,...[value,value]);}function main(){}",
        "function collect<T>(...values:T[]):int{return values.length;}function unused<T>(values:T[]):int{const alias=collect;return alias(...values);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{const alias=identity;return alias(...[value]);}function main(){}",
    ] { tsrust_compiler::check(source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",source))); }
}

#[test]
fn unused_generic_calls_validate_callback_parameters_and_arrow_aliases() {
    for source in [
        "function unused<T>(action:(value:T)=>T,value:T):T{return action(value,value);}function main(){}",
        "function unused<T>(value:T):int{const alias=(item:int):int=>item;return alias<T>(1);}function main(){}",
    ] {
        let error=tsrust_compiler::check(source).err().expect("known callback contracts validate unused calls");
        assert!(matches!(error.code,"arity"|"type"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn unused_alias_contracts_match_the_typescript_reference() {
    native(include_str!("fixtures/generic-value-adapters/unused-alias-contracts.tsr"), "");
}

#[test]
fn unused_closure_callbacks_and_dynamic_spreads_reject_guaranteed_excess() {
    let mut accepted=Vec::new();
    for source in [
        "function unused<T>(value:T):void{const apply=(action:(item:T)=>T):T=>action(value,value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T,tail:T[]):T{const alias=identity;return alias(value,value,...tail);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T,tail:T[]):T{const alias=identity;return alias(...[value,...tail,value]);}function main(){}",
    ] {
        match tsrust_compiler::check(source) {
            Ok(_)=>accepted.push(source),
            Err(error)=>assert_eq!(error.code,"arity","{}",error.render("input.tsr",source)),
        }
    }
    assert!(accepted.is_empty(),"Invalid unused calls accepted: {accepted:#?}");
}

#[test]
fn unused_call_cardinality_keeps_unknown_lengths_and_callback_rest_contracts() {
    for source in [
        "function identity<T>(value:T):T{return value;}function unused<T>(values:T[]):T{const alias=identity;return alias(...values);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T,tail:T[]):T{const alias=identity;return alias(...[...tail,value]);}function main(){}",
        "function unused<T>(value:T):void{const apply=(action:(item:T,...tail:T[])=>T):T=>action(value,...[value]);}function main(){}",
        "function unused<T>(value:T):void{const apply=(action:(item?:T)=>T):T=>action();}function main(){}",
    ] { tsrust_compiler::check(source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",source))); }
    let source="function identity<T>(value:T):T{return value;}function unused<T>():T{const alias=identity;return alias(...[...[]]);}function main(){}";
    let error=tsrust_compiler::check(source).err().expect("nested empty literal spreads have exact zero cardinality");
    assert_eq!(error.code,"arity","{}",error.render("input.tsr",source));
}

#[test]
fn unused_mutable_and_selected_aliases_validate_known_call_contracts() {
    let mut accepted=Vec::new();
    for source in [
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{let alias=identity;return alias(value,value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{const alias=flag?identity:identity;return alias(value,value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function other<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{let alias=identity;if(flag){alias=other;}return alias(value,value);}function main(){}",
        "function exact(value:int):int{return value;}function other(value:int):int{return value;}function unused<T>(flag:boolean):int{let alias=exact;if(flag){alias=other;}return alias<T>(1);}function main(){}",
    ] {
        match tsrust_compiler::check(source) {
            Ok(_)=>accepted.push(source),
            Err(error)=>assert!(matches!(error.code,"arity"|"type"),"{}",error.render("input.tsr",source)),
        }
    }
    assert!(accepted.is_empty(),"Invalid mutable/selected calls accepted: {accepted:#?}");
}

#[test]
fn source_target_joins_preserve_common_shapes_and_self_assignment() {
    for source in [
        "function identity<T>(value:T):T{return value;}function other<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{let alias=identity;if(flag){alias=other;}return alias<T>(value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{let alias=identity;alias=alias;return alias<T>(value);}function main(){}",
    ] { tsrust_compiler::check(source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",source))); }
}

#[test]
fn unused_direct_selected_calls_validate_common_source_contracts() {
    let source="function identity<T>(value:T):T{return value;}function other<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{return (flag?identity:other)(value,value);}function main(){}";
    let error=tsrust_compiler::check(source).err().expect("direct selected calls retain common source cardinality");
    assert_eq!(error.code,"arity","{}",error.render("input.tsr",source));
}

#[test]
fn mutable_source_contracts_cover_loops_closure_writes_and_shadowing() {
    let mut accepted=Vec::new();
    for source in [
        "function identity<T>(value:T):T{return value;}function other<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{let alias=identity;while(flag){alias=other;}return alias(value,value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function other<T>(value:T):T{return value;}function unused<T>(value:T):T{let alias=identity;const change=():void=>{alias=other;};return alias(value,value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function other<T>(value:T):T{return value;}function unused<T>(value:T):T{let alias=identity;const next=(alias=other);return alias(value,value);}function main(){}",
    ] {
        match tsrust_compiler::check(source) {
            Ok(_)=>accepted.push(source),
            Err(error)=>assert_eq!(error.code,"arity","{}",error.render("input.tsr",source)),
        }
    }
    assert!(accepted.is_empty(),"Invalid calls accepted: {accepted:#?}");
    let source="function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{let alias=identity;{const alias=(left:T,right:T):T=>left;alias(value,value);}return alias(value);}function main(){}";
    tsrust_compiler::check(source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",source)));
}

#[test]
fn unused_mutable_calls_before_different_shape_writes_still_validate() {
    let source="function one<T>(value:T):T{return value;}function two<T>(left:T,right:T):T{return left;}function unused<T>(value:T):T{let alias=one;const result=alias(value,value);alias=two;return result;}function main(){}";
    let error=tsrust_compiler::check(source).err().expect("later writes cannot hide an invalid earlier call");
    assert_eq!(error.code,"arity","{}",error.render("input.tsr",source));
}

#[test]
fn unused_definite_callable_overwrite_uses_the_new_contract() {
    let source="function one<T>(value:T):T{return value;}function two<T>(left:T,right:T):T{return left;}function unused<T>(value:T):T{let alias=one;alias=two;return alias(value);}function main(){}";
    let error=tsrust_compiler::check(source).err().expect("definite overwrite changes the call contract");
    assert_eq!(error.code,"arity","{}",error.render("input.tsr",source));
}

#[test]
fn immutable_callable_snapshot_keeps_the_contract_at_initialization() {
    let source="function one<T>(value:T):T{return value;}function two<T>(left:T,right:T):T{return left;}function unused<T>(value:T):T{let alias=one;const saved=alias;alias=two;return saved(value,value);}function main(){}";
    let error=tsrust_compiler::check(source).err().expect("saved callable keeps its original contract");
    assert_eq!(error.code,"arity","{}",error.render("input.tsr",source));
}

#[test]
fn unused_branch_flow_uses_definite_replacement_contracts() {
    let mut accepted=Vec::new();
    for source in [
        "function one<T>(value:T):T{return value;}function two<T>(left:T,right:T):T{return left;}function unused<T>(value:T):T{let alias=one;{alias=two;}return alias(value);}function main(){}",
        "function one<T>(value:T):T{return value;}function two<T>(left:T,right:T):T{return left;}function unused<T>(flag:boolean,value:T):T{let alias=one;if(flag){alias=two;return alias(value);}return value;}function main(){}",
        "function one<T>(value:T):T{return value;}function two<T>(left:T,right:T):T{return left;}function unused<T>(flag:boolean,value:T):T{let alias=one;if(flag){alias=two;}else{alias=two;}return alias(value);}function main(){}",
    ] {
        match tsrust_compiler::check(source) {
            Ok(_)=>accepted.push(source),
            Err(error)=>assert_eq!(error.code,"arity","{}",error.render("input.tsr",source)),
        }
    }
    assert!(accepted.is_empty(),"Invalid branch calls accepted: {accepted:#?}");
}

#[test]
fn branch_contract_flow_preserves_valid_shadowing_and_owned_self_assignment() {
    for source in [
        "function identity<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{let alias=identity;if(flag){alias=alias;}else{alias=alias;}return alias<T>(value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{const alias=identity;if(flag){const alias=1;return value;}else{const alias=2;}return alias<T>(value);}function main(){}",
        "function identity<T>(value:T):T{return value;}function unused<T>(flag:boolean,value:T):T{if(flag){const alias=identity;return alias<T>(value);}const alias=identity;return alias<T>(value);}function main(){}",
    ] { tsrust_compiler::check(source).unwrap_or_else(|error|panic!("{}",error.render("input.tsr",source))); }
}

#[test]
fn branch_alias_contracts_match_the_typescript_reference() {
    native(include_str!("fixtures/generic-value-adapters/branch-alias-contracts.tsr"), "");
}
