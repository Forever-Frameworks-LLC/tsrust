mod support;

#[test]
fn captured_rest_function_values_forward_empty_and_ordered_packs() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/captured-function-rest/forwarding.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn captured_rest_direct_spread_calls_prepare_capture_context() {
    let source = "function main(){const base=10;function sum(...values:int[]):int{return values.reduce((total:int,value:int):int=>{return total+value;},base);}console.log(sum(...[2,3]));}";
    if let Err(error) = tsrust_compiler::check(source) { panic!("{}", error.render("input.tsr", source)); }
}

#[test]
fn captured_rest_values_forward_without_generic_method_adaptation() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {strictEqual} from 'std:testing';function main(){const base=10;function sum(...values:int[]):int{let total=base;for(const value of values){total+=value;}return total;}const action=sum;strictEqual(action(),10);strictEqual(action(2,3),15);const values=[4,5];strictEqual(action(...values),19);}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
}

#[test]
fn captured_rest_values_keep_defaults_callbacks_and_factory_state() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {strictEqual} from 'std:testing';function make(start:int):(...values:int[])=>int{let total=start;function add(...values:int[]):int{for(const value of values){total+=value;}return total;}return add;}function apply(action:ReadOnly<Function<int,[value:int,index:int]>>):int{return action(3,4);}function main(){const first=make(10);const second=make(20);strictEqual(first(),10);strictEqual(first(2,3),15);strictEqual(second(4),24);strictEqual(first(1),16);{const base=10;function pick(value:int=base,...extra:int[]):int{return value+extra.length;}const action=pick;strictEqual(action(),10);strictEqual(action(undefined,1,2),12);strictEqual(action(2,3),3);strictEqual(apply(pick),4);}}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn captured_rest_values_preserve_owned_arguments_and_once_captures() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {strictEqual} from 'std:testing';function main(){const base=2;function size(...values:string[]):int{return base+values[0].length+values[1].length;}const action=size;const a='ab'.repeat(1);const b='xyz'.repeat(1);strictEqual(action(a,b),7);const saved='owner';function take(...values:int[]):string{return saved;}const once=take;strictEqual(once(1,2),'owner');}");
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    for (source, code) in [
        ("function main(){const base=1;function pick(value:int,...extra:int[]):int{return base+value+extra.length;}[3,4].map(pick);}", "type"),
        ("function main(){const base=1;function size(...values:int[]):int{return base+values.length;}const action=size;action('wrong');}", "type"),
        ("function main(){const saved='owner';function take(...values:int[]):string{return saved;}const action=take;action();action();}", "closure-consumed"),
    ] {
        let error = tsrust_compiler::check(source).err().expect("rest value contract must reject");
        assert_eq!(error.code, code, "{}", error.render("input.tsr", source));
    }
}
