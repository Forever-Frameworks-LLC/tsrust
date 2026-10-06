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
