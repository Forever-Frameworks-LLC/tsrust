mod support;

fn native(source: &str, stdout: &str) {
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), stdout);
}

#[test]
fn generic_named_values_keep_polymorphic_calls_and_captured_state() {
    native(include_str!("fixtures/generic-function-values/polymorphic.tsr"), "");
}

#[test]
fn generic_named_values_specialize_from_fixed_callable_context() {
    native("import {strictEqual} from 'std:testing';function main(){const base=3;function choose<T>(value:T):T{strictEqual(base,3);return value;}const numberValue:(value:int)=>int=choose;const stringValue:(value:Owned<string>)=>string=choose;strictEqual(numberValue(7),7);strictEqual(stringValue('text'),'text');}", "");
}

#[test]
fn returned_generic_values_keep_independent_factory_environments() {
    native("import {strictEqual} from 'std:testing';function make(start:int){let calls=start;function select<T>(value:T):T{calls+=1;console.log(calls);return value;}return select;}function main(){const first=make(10);const second=make(20);strictEqual(first(1),1);strictEqual(second('x'),'x');strictEqual(first('y'),'y');}", "11\n21\n12\n");
}

#[test]
fn captured_generic_value_shares_one_environment_across_data_specializations() {
    native("import {strictEqual} from 'std:testing';function main(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}{const action=select;strictEqual(action(3),3);strictEqual(action('value'),'value');}strictEqual(calls,2);}", "");
}

#[test]
fn contextual_values_preserve_capture_free_arrays_and_result_only_inference() {
    native("import {strictEqual} from 'std:testing';function identity<T>(value:T):T{return value;}function empty<T>():T[]{return [];}function main(){const plain:(value:int)=>int=identity;strictEqual(plain(7),7);const strings:(value:Owned<string>)=>string=identity;strictEqual(strings('text'),'text');const fresh:()=>int[]=empty;strictEqual(fresh().length,0);const base=3;function choose<T>(values:T[]):T{strictEqual(base,3);return values.remove(0);}const head:(values:Mutable<int[]>)=>int=choose;const values=[7,8];strictEqual(head(values),7);strictEqual(values.length,1);const arrays:(value:Owned<int[]>)=>int[]=identity;const copied=arrays([5,6]);strictEqual(copied[1],6);}", "");
}

#[test]
fn contextual_values_keep_mutable_factories_and_array_callback_captures() {
    native("import {strictEqual} from 'std:testing';function make(start:int):(value:int)=>int{let calls=start;function select<T>(value:T):T{calls+=1;console.log(calls);return value;}return select;}function main(){const first=make(10);const second=make(20);strictEqual(first(1),1);strictEqual(second(2),2);strictEqual(first(3),3);const seen:int[]=[];function note<T>(value:T):T{seen.push(1);return value;}{const mapped=[1,2].map(note);strictEqual(mapped[0],1);strictEqual(mapped[1],2);}strictEqual(seen.length,2);}", "11\n21\n12\n");
}

#[test]
fn contextual_generic_values_reject_incompatible_relations() {
    for source in [
        "function main(){const base=1;function same<T>(left:T,right:T):T{console.log(base);return left;}const action:(left:int,right:Owned<string>)=>int=same;}",
        "function identity<T>(value:T):T{return value;}function main(){const action:(value:int)=>string=identity;}",
        "function main(){const base=1;function select<T>(value:T):T{console.log(base);return value;}const action:(value:int)=>int=select;action('wrong');}",
    ] {
        let error=tsrust_compiler::check(source).err().expect("context must preserve generic type relation");
        assert!(matches!(error.code,"generic-inference"|"type"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn contextual_owned_values_keep_argument_and_capture_transfers() {
    native("import {strictEqual} from 'std:testing';function main(){const saved='owner';function take<T>(value:T):string{return saved;}const once:Once<Function<string,[value:int]>>=take;strictEqual(once(1),'owner');}", "");
    for source in [
        "function identity<T>(value:T):T{return value;}function main(){const action:(value:Owned<string>)=>string=identity;const text='value'.repeat(1);action(text);console.log(text);}",
        "function main(){const saved='owner';function take<T>(value:T):string{return saved;}const action:Once<Function<string,[value:int]>>=take;action(1);action(2);}",
    ] {
        let error=tsrust_compiler::check(source).err().expect("transferred owner must stay unavailable");
        assert!(matches!(error.code,"transferred"|"closure-consumed"),"{}",error.render("input.tsr",source));
    }
}

#[test]
fn contextual_borrowed_and_owned_values_keep_distinct_capture_storage() {
    native("import {strictEqual} from 'std:testing';function main(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}{const action:Mutable<Function<int,[value:int]>>=select;strictEqual(action(1),1);strictEqual(action(2),2);}strictEqual(calls,2);{const owned:(value:int)=>int=select;strictEqual(owned(3),3);strictEqual(owned(4),4);}strictEqual(calls,2);}", "");
}

#[test]
fn capture_free_contexts_keep_defaults_and_rest_after_source_solving() {
    native(include_str!("fixtures/generic-function-values/context-default-rest.tsr"), "");
}

#[test]
fn capture_free_deferred_values_keep_distinct_types_and_argument_ownership() {
    native(include_str!("fixtures/generic-function-values/deferred-capture-free.tsr"), "default\n");
}

#[test]
fn deferred_values_reject_inconsistent_types_and_reused_owners() {
    let cases = [
        ("function identity<T>(value:T):T{return value;}function accept(action:()=>void):void{}function main(){const action=identity;accept(action);}", &["unsupported", "type"][..]),
        ("type Holder={action:()=>void;};function identity<T>(value:T):T{return value;}function main(){const action=identity;const holder:Holder={action:action};}", &["unsupported", "type"][..]),
        ("function identity<T>(value:T):T{return value;}function main(){const action=identity;const invalid:()=>void=action;}", &["unsupported", "type"][..]),
        ("function same<T>(left:T,right:T):T{return left;}function main(){const action=same;action(1,'wrong');}", &["type", "generic-inference"][..]),
        ("function identity<T>(value:T):T{return value;}function main(){const action=identity;const values=[1,2];const returned=action(values);console.log(values.length);}", &["transferred"][..]),
        ("function identity<T>(value:T):T{return value;}function main(){const action=identity;const other=action;action(1);}", &["closure-consumed"][..]),
        ("function identity<T>(value:T):T{return value;}function main(){const action=identity;const value:string=action(1);}", &["type"][..]),
        ("function identity<T>(value:T):T{return value;}function main(){const action=identity;action();}", &["arity"][..]),
    ];
    for (source, codes) in cases {
        let error=tsrust_compiler::check(source).err().expect("invalid generic value");
        assert!(codes.contains(&error.code), "{}: {}",error.code,error.message);
    }
}

#[test]
fn deferred_captured_values_reject_owner_reuse_and_live_mutable_capture_conflicts() {
    for source in [
        "function main(){const saved='owner';function take<T>(value:T):string{return saved;}const action=take;action(1);action('again');}",
        "type Item={id:int;};function make(item:Item){function read<T>(value:T):int{return item.id;}return read;}function main(){const item:Item={id:1};const action=make(item);console.log(item.id);}",
        "function main(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}{const action=select;console.log(calls);action(1);}}",
    ] {
        let error=tsrust_compiler::check(source).err().unwrap_or_else(||panic!("retained capture ownership must hold: {source}"));
        assert!(matches!(error.code,"transferred"|"closure-consumed"|"borrow-conflict"|"capture-conflict"),"{}: {}",error.code,error.message);
    }
}

#[test]
fn deferred_captured_environments_drop_each_owner_once_across_types_and_failures() {
    let source="type Item={id:int;};function make(item:Item){function read<T>(value:T):int{return item.id;}return read;}function failing(item:Item){function fail<T>(value:T):int{throw item.id;}return fail;}function take(item:Item){function move<T>(value:T):Item{return item;}return move;}function recursive(item:Item){function even<T>(value:T,n:int):int{if(n===0){return item.id;}return odd(value,n-1);}function odd<T>(value:T,n:int):int{if(n===0){return item.id+1;}return even(value,n-1);}return even;}function main(){{const first=make({id:1});const second=make({id:2});if(first(1)!==1||first('a')!==1||second('b')!==2){throw 99;}}{const fail=failing({id:3});let caught=0;try{fail(1);}catch(error){if(error===3){caught+=1;}}try{fail('x');}catch(error){if(error===3){caught+=1;}}if(caught!==2){throw 99;}}{const move=take({id:4});const item=move('x');if(item.id!==4){throw 99;}}{const rec=recursive({id:5});if(rec(1,4)!==5||rec('x',3)!==6){throw 99;}}}";
    let rust=tsrust_compiler::check(source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),5);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),62);}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.0.join("deferred-disposal.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("deferred-disposal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
        let output=support::run(&mut std::process::Command::new(binary),10);support::success(&output);assert!(output.stdout.is_empty());
    }
}

#[test]
fn captured_result_only_generic_value_can_be_stored_before_invocation_context() {
    native("function main(){const label='retained';function empty<T>():T[]{console.log(label);return [];}const action=empty;}","");
}

#[test]
fn deferred_generic_aliases_infer_result_only_types_without_sharing_methods() {
    native(include_str!("fixtures/generic-function-values/result-context.tsr"),"11\n12\n3 x\n");
}
