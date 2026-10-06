mod support;

fn assert_fixture(source: &str) {
 let directory=support::TempDirectory::new();
 let path=directory.source(source);
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
 assert!(output.stdout.is_empty(),"Assertion fixtures should not log results");
 assert!(output.stderr.is_empty(),"{}",String::from_utf8_lossy(&output.stderr));
}

#[test]
fn defaults_select_only_undefined_and_run_lazily(){
 assert_fixture(include_str!("fixtures/destructuring-defaults/declarations.tsr"));
}
#[test]
fn defaults_work_in_parameters_callbacks_and_loops(){
 assert_fixture(include_str!("fixtures/destructuring-defaults/parameters.tsr"));
}
#[test]
fn defaults_preserve_source_access_without_cloning(){
 assert_fixture(include_str!("fixtures/destructuring-defaults/owners.tsr"));
}
#[test]
fn defaults_keep_tuple_bounds_missing_fields_and_source_evaluation(){
 assert_fixture(include_str!("fixtures/destructuring-defaults/edges.tsr"));
}
#[test]
fn defaults_keep_borrowing_transfer_and_binding_order_contracts(){
 use support::reject;
 reject("function take(text:string):string{return text;}function main(){const words:string[]=[];const [text=\"x\".repeat(1)]=words;take(text);}","borrowed-place");
 reject("function main(){const words:string[]=[];const [text=\"x\"]=words;words.push(\"y\");console.log(text);}","view-conflict");
 reject("function main(){const [first=later,later=2]:int[]=[];console.log(first);}","name");
 reject("function main(){const [first=first]:int[]=[];console.log(first);}","name");
 assert_fixture(r#"import { strictEqual } from "std:testing";
function main(){const words:string[]=[];const text=clone(words.at(0)??"x");words.push("y");strictEqual(text,"x");strictEqual(words.length,1);}"#);
}
#[test]
fn defaults_reject_mutating_the_borrowed_source_during_fallback(){
 support::reject("function mutate(words:string[]):string{words.push(\"changed\");return \"fallback\";}function main(){const words:string[]=[];const [text=mutate(words)]=words;console.log(text);}","conflicting-access");
}
#[test]
fn defaults_validate_absent_field_sources_and_hold_fresh_readonly_owners(){
 support::reject("type Row={text:string;};function take(row:Row):Row{return row;}function main(){const row:Row={text:\"x\"};const saved=take(row);const {missing=1}=row;console.log(missing,saved.text);}","transferred");
 assert_fixture(r#"import { strictEqual } from "std:testing";
type Row={value:int;};function make():string{return "fresh".repeat(2);}function read({missing=make()}:Row):int{return missing.length;}function main(){strictEqual(read({value:1}),10);}"#);
}
#[test]
fn defaults_compose_with_whole_parameter_defaults_and_owned_records(){
 assert_fixture(include_str!("fixtures/destructuring-defaults/whole-parameters.tsr"));
}
#[test]
fn defaults_specialize_generic_nullable_array_elements(){
 assert_fixture(r#"import { strictEqual } from "std:testing";
function get<T>([value=null]:(T|null|undefined)[]):string{return String(value);}function main(){const words:(string|null|undefined)[]=["x"];const numbers:(int|null|undefined)[]=[];strictEqual(String(get<string>(words)),"x");strictEqual(String(get<int>(numbers)),"null");strictEqual(String(words[0]),"x");}"#);
}

#[test]
fn defaults_initialize_parameter_patterns_left_to_right(){
 assert_fixture(include_str!("fixtures/destructuring-defaults/parameter-order.tsr"));
}
#[test]
fn defaults_cannot_observe_later_parameters(){
 support::reject("function bad([first=later]:int[],later:int):int{return first;}function main(){console.log(bad([],2));}","name");
 support::reject("function main(){const bad=([first=later]:int[],later:int):int=>first;console.log(bad([],2));}","name");
}

#[test]
fn defaults_keep_callable_result_discovery(){
 assert_fixture(r#"import { strictEqual } from "std:testing";
function make([value=3]:int[]):()=>int{return ():int=>value;}function main(){const action=make([]);strictEqual(action(),3);}"#);
}

#[test]
fn defaults_keep_nested_arrow_parameter_pattern_boundaries(){
 assert_fixture(r#"import { strictEqual } from "std:testing";
function invoke([first=3]:int[],action:Function<int>=():int=>first):int{return action();}function main(){strictEqual(invoke([]),3);}"#);
}

#[test]
fn defaults_allow_readonly_later_parameters_to_borrow_earlier_pattern_names(){
 assert_fixture(r#"import { strictEqual } from "std:testing";
type Row={text:string|undefined;};function read({text="none"}:Row,last:ReadOnly<string>=text):int{return last.length;}function main(){strictEqual(read({text:undefined}),4);strictEqual(read({text:"hello"}),5);}"#);
}

#[test]
fn defaults_keep_loans_from_earlier_owned_pattern_names(){
 support::reject(r#"type Row={text:string;};function consume(text:string):string{return text;}function main(){const read=({text}:Owned<Row>,last:ReadOnly<string>=text):string=>{const saved=consume(text);console.log(last);return saved;};console.log(read({text:"hello"}));}"#,"view-conflict");
 support::reject(r#"type Row={text:string;};function consume(text:string):string{return text;}function read({text}:Owned<Row>,last:ReadOnly<string>=text):string{const saved=consume(text);console.log(last);return saved;}function main(){console.log(read({text:"hello"}));}"#,"view-conflict");
}

#[test]
fn defaults_allow_readonly_arrow_parameters_to_borrow_earlier_pattern_names(){
 assert_fixture(r#"import { strictEqual } from "std:testing";
type Row={text:string|undefined;};function main(){const read=({text="none"}:Row,last:ReadOnly<string>=text):int=>last.length;strictEqual(read({text:undefined}),4);strictEqual(read({text:"hello"}),5);}"#);
}
