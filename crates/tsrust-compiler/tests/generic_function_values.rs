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
    native("import {strictEqual} from 'std:testing';function main(){const base=3;function choose<T>(value:T):T{strictEqual(base,3);return value;}const numberValue:(value:int)=>int=choose;const stringValue:(value:string)=>string=choose;strictEqual(numberValue(7),7);strictEqual(stringValue('text'),'text');}", "");
}

#[test]
fn returned_generic_values_keep_independent_factory_environments() {
    native("import {strictEqual} from 'std:testing';function make(start:int){let calls=start;function select<T>(value:T):T{calls+=1;console.log(calls);return value;}return select;}function main(){const first=make(10);const second=make(20);strictEqual(first(1),1);strictEqual(second('x'),'x');strictEqual(first('y'),'y');}", "11\n21\n12\n");
}

#[test]
fn captured_generic_value_shares_one_environment_across_data_specializations() {
    native("import {strictEqual} from 'std:testing';function main(){let calls=0;function select<T>(value:T):T{calls+=1;return value;}{const action=select;strictEqual(action(3),3);strictEqual(action('value'),'value');}strictEqual(calls,2);}", "");
}
