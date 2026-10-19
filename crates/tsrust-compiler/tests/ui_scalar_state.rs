mod support;
#[test]
fn scalar_state_keeps_exact_values_snapshots_and_direct_text_bindings() {
 let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-state");
 let output=support::run(support::cli().arg("run").arg(root.join("src/inspect.tsr")),90);support::success(&output);assert_eq!(output.stdout,b"Scalar state passed\n");
}
#[test]
fn scalar_state_rejects_incompatible_payloads_and_ownership_operations() {
 for (body,code) in [
  ("const state=signal('text');state.set(1);","type"),
  ("const state=signal([1]);","ui-state-type"),
  ("const value:Value={value:1};const state=signal(value);","ui-state-type"),
  ("const state=signal(undefined);","ui-state-type"),
  ("const state=signal(true);const copy=clone(state);","clone-type"),
 ] {
  let dir=support::TempDirectory::new();std::fs::write(dir.0.join("tsrust.config.json"),serde_json::json!({"ui":tsrust_compiler::ui_project::Config::default()}).to_string()).unwrap();let source=support::format_round_trip(&format!("import {{signal}} from 'std:ui';interface Value{{value:int;}}function main(){{{body}}}"));let path=dir.source(&source);
  for command in ["check","emit"]{let output=support::run(support::cli().arg(command).arg(&path),30);assert!(!output.status.success(),"{body}");assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("error[{code}]")),"{}",String::from_utf8_lossy(&output.stderr));}
 }
}

#[test]
fn zero_parameter_components_accept_only_empty_jsx_props() {
 let dir=support::TempDirectory::new();let path=dir.0.join("main.tsrx");
 for (call,accepted) in [("<Empty/>",true),("<Empty></Empty>",true),("Empty({})",false),("<Empty extra={1}/>",false),("<Empty>child</Empty>",false)] {
  let source=support::format_round_trip(&format!("function Empty():int{{return 7;}}function main(){{const value={call};console.log(value);}}"));std::fs::write(&path,&source).unwrap();
  let result=tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider,path.to_str().unwrap()).and_then(tsrust_compiler::project::check_graph);
  assert_eq!(result.is_ok(),accepted,"{call}: {:?}",result.err());
 }
}

#[test]
fn checked_state_reads_publish_scalar_facts_for_each_generic_specialization() {
 let dir=support::TempDirectory::new();let source=support::format_round_trip("import {Signal,signal} from 'std:ui';function read<T>(value:ReadOnly<Signal<T>>):T{return value.get();}function main(){const text=signal('value');const count=signal(7);console.log(read(text),read(count));}");let path=dir.source(&source);
 let checked=tsrust_compiler::project::check_graph(tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider,path.to_str().unwrap()).unwrap()).unwrap();
 let facts=checked.checked_type_facts();let reads=facts.method_results.iter().filter(|fact|fact.location.source.0==path.canonicalize().unwrap().to_str().unwrap() && &source[fact.location.span.start..fact.location.span.end]=="value.get()").collect::<Vec<_>>();
 let mut names=reads.iter().map(|fact|match &fact.ty{tsrust_compiler::project::ProjectType::Named{name,..}=>name.clone(),other=>panic!("hidden UI result leaked: {other:?}")}).collect::<Vec<_>>();names.sort();names.dedup();assert_eq!(names,vec!["int","string"],"{:?}",facts.method_results.iter().filter(|fact|fact.location.source.0.ends_with("input.tsr")).collect::<Vec<_>>());
}
