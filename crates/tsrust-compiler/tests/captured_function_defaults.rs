mod support;
#[test]
fn captured_function_values_apply_defaults_once_per_omitted_argument() {
    let directory=support::TempDirectory::new();
    let path=directory.source(include_str!("fixtures/captured-function-defaults/evaluation.tsr"));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn captured_optional_values_and_owned_defaults_preserve_call_contracts() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {strictEqual} from 'std:testing';function main(){{const saved='hello';function size(value:string=saved):int{return value.length;}const action=size;strictEqual(action(),5);}{const base=10;function optional(step?:int):int{return base+(step??2);}const action=optional;strictEqual(action(),12);strictEqual(action(3),13);strictEqual(action(undefined),12);function dependent(left:int,right:int=left+base):int{return right;}const pair=dependent;strictEqual(pair(2),12);strictEqual(pair(2,9),9);}}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    let error=tsrust_compiler::check("function main(){const saved='hello';function take(value:string=saved):string{return value;}const action=take;action();action();}").err().expect("owned default capture is consumed once");
    assert_eq!(error.code,"closure-consumed","{}: {}",error.code,error.message);
}

#[test]
fn array_callbacks_supply_index_instead_of_applying_a_default() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {strictEqual} from 'std:testing';function main(){const base=[10];function pick(value:int,extra:int=base[0]):int{return value+extra;}const mapped=[1,2].map(pick);strictEqual(mapped[0],1);strictEqual(mapped[1],3);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn supplied_arguments_skip_throwing_defaults_and_omission_propagates_failure() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {strictEqual} from 'std:testing';function main(){const base=1;function fail():int{throw 'bad';}function add(step:int=fail()):int{return base+step;}const action=add;strictEqual(action(2),3);let caught=false;try{action();}catch(error){if(typeof error==='string'){strictEqual(error,'bad');}else{throw error;}caught=true;}strictEqual(caught,true);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    for (call,code) in [("action('wrong');","type"),("action(1,2);","arity")] {
        let source=format!("function main(){{const base=1;function add(step:int=2):int{{return base+step;}}const action=add;{call}}}");
        let error=tsrust_compiler::check(&source).err().expect("argument contract remains checked");
        assert_eq!(error.code,code,"{}: {}",error.code,error.message);
    }
}

#[test]
fn returned_defaulted_values_keep_independent_owned_factory_state() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {strictEqual} from 'std:testing';function make(start:int):(step?:int)=>int{let value=start;function next(step:int=2):int{value+=step;return value;}return next;}function main(){const first=make(10);const second=make(20);strictEqual(first(),12);strictEqual(second(3),23);strictEqual(first(undefined),14);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
