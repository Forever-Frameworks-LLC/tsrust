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
