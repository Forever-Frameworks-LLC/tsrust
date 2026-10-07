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
