mod support;
#[test]
fn own_members_match_node_without_moving_sources(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/object-has-own/basic.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());
}
#[test]
fn invalid_inputs_are_rejected(){
 support::reject("function main(){Object.hasOwn();}","arity");
 support::reject(r#"function main(){Object.hasOwn([],"x",1);}"#,"arity");
 support::reject(r#"function main(){Object.hasOwn(null,"x");}"#,"unsupported");
 support::reject("function main(){Object.hasOwn([],[1]);}","unsupported");
}

#[test]
fn key_evaluation_cannot_mutate_a_borrowed_receiver(){
 support::reject(r#"function key(values:int[]):string{values.push(1);return "0";}function main(){const values=[1];Object.hasOwn(values,key(values));}"#,"conflicting-access");
}


#[test]
fn canonical_index_keys_and_boolean_text_need_no_allocation() {
    // Index keys are checked as text (digits, no leading zero); String(boolean)
    // borrows "true"/"false". Results match JavaScript.
    let source = r#"function flag(value: boolean | undefined): string { return String(value); }
function main() {
  const values = [10, 20, 30];
  const keys = ["0", "2", "3", "01", "+1", "00", "1e0", "length", "", "-0"];
  const owned: string[] = [];
  for (const key of keys) { owned.push(`${key}=${String(Object.hasOwn(values, key))}`); }
  console.log(owned.join(" "), String(true), String(false), flag(true), flag(undefined), Object.hasOwn(values, 2), Object.hasOwn(values, -0.0), Object.hasOwn("abc", 2));
}"#;
    let rust = tsrust_compiler::check(&support::format_round_trip(source)).unwrap().emit_rust();
    assert!(!rust.contains("index.to_string()"), "no index text is formatted for comparison");
    assert!(!rust.contains("TsrConsoleValue::Boolean(*(&builtin_argument_0))"), "String(boolean) borrows its text");
    support::execute(&support::format_round_trip(source), "0=true 2=true 3=false 01=false +1=false 00=false 1e0=false length=true =false -0=false true false true undefined true true true
", None);
}
