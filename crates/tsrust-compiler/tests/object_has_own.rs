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
