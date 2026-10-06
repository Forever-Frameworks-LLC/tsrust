mod support;

#[test]
fn sets_preserve_membership_order_same_value_zero_and_ownership() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/sets/values.tsr"));
    let output = support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn set_keys_are_typed_and_iteration_protects_the_receiver() {
    for (source, message) in [
        ("function main(){const values=new Set<int>();values.add('x');}", "type"),
        ("function main(){const source:int[]=[1];const values=new Set<number>(source);}", "type"),
        ("function main(){const values=new Set<int>();for(const value of values){values.add(value);}}", "borrow"),
        ("function main(){const values=new Set<string>();const text=String('x');values.add(text);console.log(text);}", "transfer"),
        ("function main(){const values=new Set<int>();values.size=1;}", "read"),
        ("function main(){const values=new Set<int>();const Set=1;}", "shadow"),
        ("function Set():int{return 1;}function main(){const values=new Set<int>();}", "shadow"),
    ] {
        let error=tsrust_compiler::check(source).err().expect("unsafe set accepted");
        assert!(error.code.contains(message)||error.message.to_lowercase().contains(message),"{}: {}",error.code,error.message);
    }
}

#[test]
fn set_display_and_explicit_string_conversion_are_checked() {
    let directory = support::TempDirectory::new();
    let path = directory.source("function main(){const values=new Set<int>([2,1,2]);console.log(values);console.log(String(values));}");
    let output = support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "Set(2) { 2, 1 }\n[object Set]\n");
}

#[test]
fn set_sources_adapt_widened_keys_without_transferring_the_source() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {strictEqual} from \"std:testing\";function main(){const source:int[]=[1,2];const alternatives=new Set<int|string>(source);strictEqual(alternatives.has(2),true);const widened=new Set<int|string>(new Set<int>([3]));strictEqual(widened.has(3),true);strictEqual(source.length,2);}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
}

#[test]
fn set_documented_examples_run_in_one_native_program() {
    let directory=support::TempDirectory::new();
    let path=directory.source(include_str!("fixtures/sets/documented.tsr"));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),include_str!("fixtures/sets/documented.stdout"));
}

#[test]
fn set_storage_in_records_arrays_and_unions_preserves_ownership() {
    let directory=support::TempDirectory::new();
    let path=directory.source("import {strictEqual} from \"std:testing\";type Bag={values:Set<string>;};function main(){const bag:Bag={values:new Set<string>(['x'])};strictEqual(bag.values.has('x'),true);const groups:Set<int>[]=[new Set<int>([1]),new Set<int>([2])];strictEqual(groups[0].has(1),true);console.log(groups.join(','));const maybe:Set<string>|undefined=new Set<string>(['a']);strictEqual(String(maybe),'[object Set]');}");
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(),"[object Set],[object Set]\n");
}
