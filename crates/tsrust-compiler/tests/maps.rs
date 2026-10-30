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

#[path = "../src/runtime/sets.rs"] mod runtime_sets;
#[path = "../src/runtime/maps.rs"] mod runtime_maps;

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


#[test]
fn small_and_indexed_collections_agree_across_the_switch() {
    // Up to 8 slots a Map or Set scans its entries; the 9th builds the hash
    // index. Results, order and set algebra must not depend on the form.
    let source = r#"function main() {
  const out: string[] = [];
  for (const total of [7, 8, 9, 10, 20]) {
    const map = new Map<string, int>();
    for (let i = 0; i < total; i++) { map.set(`k${i}`, i); }
    map.delete("k1");
    map.set("k1", 100);
    map.set("k0", 50);
    let sum = 0;
    for (const [key, value] of map) { sum += value; }
    out.push(`${total}:${map.size}:${map.get("k1") ?? -1}:${map.get(`k${total - 1}`) ?? -1}:${map.has("k1")}:${map.has("zz")}:${sum}`);
  }
  const map = new Map<int, int>();
  for (let i = 0; i < 12; i++) { map.set(i, i * i); }
  map.clear();
  map.set(3, 9);
  out.push(`cleared:${map.size}:${map.get(3) ?? -1}:${map.has(11)}`);
  const small = new Set<int>([1, 2, 3, 4]);
  const large = new Set<int>();
  for (let i = 0; i < 30; i += 2) { large.add(i); }
  const both = small.intersection(large);
  const either = large.intersection(small);
  const only = small.difference(large);
  const sym = small.symmetricDifference(new Set<int>([3, 4, 5]));
  out.push(`sets:${[...both].join(",")}|${[...either].join(",")}|${[...only].join(",")}|${[...sym].join(",")}|${small.isSubsetOf(large)}|${new Set<int>([2, 4]).isSubsetOf(large)}|${small.isDisjointFrom(new Set<int>([9]))}`);
  console.log(out.join(" "));
}"#;
    support::execute(&support::format_round_trip(source), "7:7:100:6:true:false:170 8:8:100:7:true:false:177 9:9:100:8:true:false:185 10:10:100:9:true:false:194 20:20:100:19:true:false:339 cleared:1:9:false sets:2,4|2,4|1,3|1,2,5|false|true|true
", None);
}
