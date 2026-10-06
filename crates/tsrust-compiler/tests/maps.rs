mod support;

#[test]
fn map_documented_examples_run_in_one_native_program(){
 let directory=support::TempDirectory::new();
 let path=directory.source(include_str!("fixtures/maps/documented.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
 assert_eq!(String::from_utf8(output.stdout).unwrap(),include_str!("fixtures/maps/documented.stdout"));
}

#[test]
fn map_literal_entries_infer_key_and_value_types(){
 let directory=support::TempDirectory::new();
 let path=directory.source("import {strictEqual} from \"std:testing\";function main(){const values=new Map([['a',1],['b',2]]);strictEqual(values.get('b')??0,2);}");
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
}

#[test]
fn maps_keep_key_identity_and_read_owned_values_without_implicit_clones(){
 let directory=support::TempDirectory::new();
 let path=directory.source(include_str!("fixtures/maps/values.tsr"));
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
 assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
}

#[path = "../src/runtime_sets.rs"] mod runtime_sets;
#[path = "../src/runtime_maps.rs"] mod runtime_maps;

#[test]
fn native_map_storage_keeps_order_and_drops_replaced_owners() {
 let directory=support::TempDirectory::new();
 let source=format!(r#"
use std::borrow::Cow;
type String=Cow<'static,str>;
#[derive(Clone)] struct TsrNull;
#[derive(Clone)] struct TsrUndefined;
{}
{}
struct Owner(std::rc::Rc<std::cell::Cell<usize>>);
impl Drop for Owner{{fn drop(&mut self){{self.0.set(self.0.get()+1);}}}}
fn main(){{
 let mut values:TsrMap<i64,i64>=TsrMap::from_values([(3i64,30i64),(1,10),(3,31)]);
 assert_eq!(values.len(),2);
 assert_eq!(values.get(3),Some(&31));
 assert_eq!(values.iter().map(|(k,_)|*k).collect::<Vec<_>>(),vec![3,1]);
 assert!(values.delete(3));assert!(!values.delete(3));values.set(3,32);
 assert_eq!(values.iter().map(|(k,_)|*k).collect::<Vec<_>>(),vec![1,3]);
 let mut numbers:TsrMap<f64,i64>=TsrMap::new();numbers.set(f64::NAN,1i64);numbers.set(f64::NAN,2);numbers.set(-0.0,3);numbers.set(0.0,4);
 assert_eq!(numbers.len(),2);assert_eq!(numbers.get(f64::NAN),Some(&2));assert_eq!(numbers.get(-0.0),Some(&4));
 assert!(!numbers.iter().find(|(k,_)|**k==0.0).unwrap().0.is_sign_negative());
 let dropped=std::rc::Rc::new(std::cell::Cell::new(0));
 let mut owners:TsrMap<i64,Owner>=TsrMap::new();owners.set(1i64,Owner(dropped.clone()));owners.set(1,Owner(dropped.clone()));
 assert_eq!(dropped.get(),1);let read=owners.get(1).unwrap();assert_eq!(read.0.get(),1);
 assert!(owners.delete(1));assert_eq!(dropped.get(),2);
 owners.set(2,Owner(dropped.clone()));owners.clear();assert_eq!(dropped.get(),3);
}}
"#,runtime_sets::SET_RUNTIME,runtime_maps::MAP_RUNTIME);
 let path=directory.0.join("runtime.rs");std::fs::write(&path,source).unwrap();
 let binary=directory.0.join("runtime");
 support::success(&support::run(support::rustc().arg(path).arg("-o").arg(&binary),60));
 support::success(&support::run(std::process::Command::new(binary).args(std::iter::empty::<&str>()),60));
}

#[test]
fn map_methods_enforce_types_transfers_and_read_loans(){
 for (source,message) in [
  ("function main(){const values=new Map<string,int>();values.set(1,2);}","type"),
  ("function main(){const values=new Map<int,string>();values.set(1,2);}","type"),
  ("function main(){const values=new Map<int,string>();const value=String('x');values.set(1,value);console.log(value);}","transfer"),
  ("function main(){const values=new Map<int,string>();values.set(1,'x');const value=values.get(1);values.clear();console.log(value??'');}","borrow"),
  ("function main(){const values=new Map<int,int>();values.size=1;}","read"),
  ("function main(){const values=new Map<int,int>();const Map=1;}","shadow"),
 ] {
  let error=tsrust_compiler::check(source).err().expect("unsafe Map accepted");
  assert!(error.code.contains(message)||error.message.to_lowercase().contains(message),"{}: {}",error.code,error.message);
 }
}

#[test]
fn map_display_and_string_conversion_preserve_observable_values(){
 let directory=support::TempDirectory::new();
 let path=directory.source("function main(){const values=new Map<string,int>([['a',1],['b',2]]);console.log(values);console.log(String(values));console.log([values].join(','));}");
 let output=support::run(support::cli().arg("run").arg(path),60);
 support::success(&output);
 assert_eq!(String::from_utf8(output.stdout).unwrap(),"Map(2) { 'a' => 1, 'b' => 2 }\n[object Map]\n[object Map]\n");
}
