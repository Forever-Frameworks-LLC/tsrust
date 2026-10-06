mod support;
#[test]
fn comma_selected_receivers_evaluate_prefixes_once(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/comma-contexts/receivers.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}
#[test]
fn comma_sentinel_operand_preserves_narrowing(){
 tsrust_compiler::check("function use(value:int|null):int{if(value!==(0,null)){return value+1;}return 0;}function main(){use(2);}").unwrap();
}
#[test]
fn comma_typeof_operand_preserves_narrowing(){
 tsrust_compiler::check("function use(value:int|string):int{if((0,typeof value)===\"string\"){return value.length;}return value;}function main(){use(2);}").unwrap();
}

#[test]
fn comma_guard_effects_do_not_narrow_a_changed_source(){
 let source="type Row={value:int|null;};function reset(row:Row):int{row.value=null;return 0;}function main(){const row:Row={value:3};if(row.value!==(reset(row),null)){console.log(row.value+1);}}";
 assert!(tsrust_compiler::check(source).is_err(),"a copied comparison operand must not narrow storage mutated later");
}

#[test]
fn primitive_equality_does_not_narrow_a_changed_source(){
 let source="type Row={value:int|null;};function reset(row:Row):int{row.value=null;return 3;}function main(){const row:Row={value:3};if(row.value===reset(row)){console.log(row.value+1);}}";
 assert!(tsrust_compiler::check(source).is_err(),"equality must not narrow storage changed after its value was copied");
}

#[test]
fn typeof_guard_does_not_narrow_a_changed_source(){
 let source="type Row={value:int|string;};function reset(row:Row):int{row.value=2;return 0;}function main(){const row:Row={value:\"text\"};if(typeof row.value===(reset(row),\"string\")){console.log(row.value.length);}}";
 assert!(tsrust_compiler::check(source).is_err(),"typeof observes the old tag, not the subsequently changed source");
}
#[test]
fn comma_guard_fixture_runs(){
 let directory=support::TempDirectory::new();let path=directory.source(include_str!("fixtures/comma-contexts/guards.tsr"));let output=support::run(support::cli().arg("run").arg(path),60);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[test]
fn guard_snapshots_survive_reads_and_are_joined_across_rhs_branches(){
 tsrust_compiler::check("function read(value:int|null):int{return 0;}function use(value:int|null,flag:boolean):int{if(value!==(flag?read(value):0,null)){return value+1;}return 0;}function main(){use(2,true);}").unwrap();
 tsrust_compiler::check("function use(value:int|string):int{if(typeof value===(0,\"string\")){return value.length;}return value;}function main(){use(2);}").unwrap();
 let source="type Row={value:int|null;};function reset(row:Row):int{row.value=null;return 0;}function use(flag:boolean){const row:Row={value:3};if(row.value!==(flag?reset(row):0,null)){console.log(row.value+1);}}function main(){use(true);}";
 assert!(tsrust_compiler::check(source).is_err());
}
#[test]
fn indexed_depth_canary_keeps_analysis_and_emission_bounded() {
 let source=format!("function main(){{const a=[0];console.log({}0{});}}","a[".repeat(100),"]".repeat(100));
 let checked=tsrust_compiler::check(&source).unwrap();
 checked.emit_rust();
 let invalid=format!("function main(){{const a=[0];console.log({}0{});}}","a[".repeat(200),"]".repeat(200));assert!(tsrust_compiler::check(&invalid).is_err());
}
#[test]
fn comma_receivers_keep_existing_source_loans() {
 for source in [
  "function main(){const values=[[1,2]];const view=values[0]; (0,view).push(3);console.log(view);}",
  "function main(){const values=[[1],[2]];(0,values[0]).push((values.push([3]),4));}",
 ] {assert!(tsrust_compiler::check(source).is_err(),"comma selection must preserve source access: {source}");}
}
