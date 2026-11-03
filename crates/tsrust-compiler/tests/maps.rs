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
 let numbers:TsrMap<f64,u64>=TsrMap::from_mapped(numbers,|value|value as u64);
 assert_eq!(numbers.len(),2);assert_eq!(numbers.get(f64::NAN),Some(&2));assert_eq!(numbers.get(-0.0),Some(&4));
 assert!(!numbers.iter().find(|(k,_)|**k==0.0).unwrap().0.is_sign_negative());
 let dropped=std::rc::Rc::new(std::cell::Cell::new(0));
 let mut owners:TsrMap<i64,Owner>=TsrMap::new();owners.set(1i64,Owner(dropped.clone()));owners.set(1,Owner(dropped.clone()));
 assert_eq!(dropped.get(),1);let read=owners.get(1).unwrap();assert_eq!(read.0.get(),1);
 assert!(owners.delete(1));assert_eq!(dropped.get(),2);
 owners.set(2,Owner(dropped.clone()));owners.clear();assert_eq!(dropped.get(),3);
 remap_witness();
}}
"#,runtime_sets::SET_RUNTIME,"{MAP_FIRST}",runtime_maps::MAP_INLINE_RUNTIME);
 let source=source+r#"
struct WrappedOwner(Owner);
fn remap_witness(){
 for count in [0_i64,1,8,9,17,64]{
  let mut values:TsrMap<i64,i64>=TsrMap::new();for key in 0..count{values.set(key,key*10);}
  for key in 0..count{if key%3==1{assert!(values.delete(key));}}
  let expected=values.iter().map(|(key,_)|*key).collect::<Vec<_>>();let pointer=values.slots.as_ptr() as usize;let mut calls=0;
  let mut mapped:TsrMap<i64,u64>=TsrMap::from_mapped(values,|value|{calls+=1;(value+1) as u64});
  assert_eq!(calls,expected.len());assert_eq!(mapped.len(),expected.len());assert_eq!(mapped.slots.len()+usize::from(mapped.first.as_ref().is_some()),expected.len());
  if count>1 && count<=TSR_SMALL_SLOTS as i64{assert_eq!(mapped.slots.as_ptr() as usize,pointer);}
  assert_eq!(mapped.iter().map(|(key,_)|*key).collect::<Vec<_>>(),expected);
  for key in expected{assert_eq!(mapped.get(key),Some(&((key*10+1) as u64)));}
  mapped.set(count+3,999);assert_eq!(mapped.get(count+3),Some(&999));
  mapped.set(0,999);assert!(mapped.delete(0));mapped.set(0,1000);assert_eq!(*mapped.iter().last().unwrap().0,0);
  mapped.clear();assert_eq!(mapped.len(),0);mapped.set(99,12);assert_eq!(mapped.get(99),Some(&12));
 }
 let values:TsrMap<i64,i64>=TsrMap::from_values([(1,11),(2,22)]);
 let wider:TsrMap<i64,[i64;8]>=TsrMap::from_mapped(values,|value|[value;8]);assert_eq!(wider.get(2),Some(&[22;8]));
 let narrower:TsrMap<i64,i64>=TsrMap::from_mapped(wider,|value|value[7]);assert_eq!(narrower.get(1),Some(&11));
 let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
 for count in [3,17]{
  let dropped=std::rc::Rc::new(std::cell::Cell::new(0));let mut values:TsrMap<i64,Owner>=TsrMap::new();
  for key in 0..count{values.set(key,Owner(dropped.clone()));}assert!(values.delete(0));let mut calls=0;
  let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
   let mapped:TsrMap<i64,WrappedOwner>=TsrMap::from_mapped(values,|value|{calls+=1;if calls==2{panic!("conversion");}WrappedOwner(value)});drop(mapped);
  }));assert!(result.is_err());assert_eq!(dropped.get(),count as usize);
 }
 std::panic::set_hook(hook);
}
"#;
 let source=source+r#"
fn singleton_storage_witness(){
 let mut values:TsrMap<i64,i64>=TsrMap::new();values.set(7,11);
 assert_eq!(values.slots.capacity()==0,TSR_MAP_INLINE_FIRST);
 assert_eq!(values.get_mut(7).map(|value|{*value+=2;*value}),Some(13));
 let mut values:TsrMap<i64,u64>=TsrMap::from_mapped(values,|value|value as u64);
 assert_eq!(values.get(7),Some(&13));
 assert_eq!(values.clone().into_value(7),Some(13));
 assert!(values.delete(7));assert!(!values.delete(7));
 values.set(9,19);values.set(2,22);
 assert_eq!(values.iter().map(|(key,_)|*key).collect::<Vec<_>>(),vec![9,2]);
 assert_eq!(values.slots.iter().flatten().count(),2);
 values.clear();values.set(3,33);assert_eq!(values.into_value(3),Some(33));
 let mut inline:TsrMap<i64,i64,(i64,i64),Option<(i64,i64)>>=TsrMap::new();inline.set(5,55);
 let plain:TsrMap<i64,u64,(i64,u64),TsrMapNoFirst<(i64,u64)>>=TsrMap::from_mapped(inline,|v|v as u64);
 assert_eq!(plain.get(5),Some(&55));assert_eq!(plain.slots.len(),1);
 let inline:TsrMap<i64,i64,(i64,i64),Option<(i64,i64)>>=TsrMap::from_mapped(plain,|v|v as i64);
 assert_eq!(inline.get(5),Some(&55));assert_eq!(inline.slots.capacity(),0);
 let dropped=std::rc::Rc::new(std::cell::Cell::new(0));
 let mut owners:TsrMap<i64,Owner>=TsrMap::new();owners.set(1,Owner(dropped.clone()));
 let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
 let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{
  let _:TsrMap<i64,WrappedOwner>=TsrMap::from_mapped(owners,|_|panic!("singleton conversion"));
 }));std::panic::set_hook(hook);assert!(result.is_err());assert_eq!(dropped.get(),1);
}
"#;
 let source=source.replace(" remap_witness();"," remap_witness();singleton_storage_witness();");
 let path=directory.0.join("runtime.rs");let binary=directory.0.join("runtime");
 for first in [runtime_maps::MAP_INLINE_FIRST,"const TSR_MAP_INLINE_FIRST:bool=false;type TsrMapFirst<P>=TsrMapNoFirst<P>;"]{
  std::fs::write(&path,source.replace("{MAP_FIRST}",first)).unwrap();
  for level in ["0","3"]{
   support::success(&support::run(support::rustc().arg(&path).args(["-C",&format!("opt-level={level}")]).arg("-o").arg(&binary),60));
   support::success(&support::run(std::process::Command::new(&binary).args(std::iter::empty::<&str>()),60));
  }
 }
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

#[test]
fn inline_map_entries_keep_recursive_and_owned_capture_layouts_finite(){
 for (source,inline,expected) in [
  ("type Holder={run:ReadOnly<Function<int>>;};function main(){const source=[7];const map=new Map<string,Holder>();map.set('one',{run:():int=>source[0]});const plain=new Map<string,int>();plain.set('n',2);console.log(map.get('one')?.run()??0,plain.size);}",true,"7 1\n"),
  ("type Node={children:Map<string,Node>;};function main(){const root:Node={children:new Map<string,Node>()};root.children.set('child',{children:new Map<string,Node>()});console.log(root.children.size);}",false,"1\n"),
  ("type Node={next:Map<string,Node>|undefined;};function main(){const root:Node={next:new Map<string,Node>()};console.log(root.next?.size??-1);}",false,"0\n"),
  ("function main(){const inner=new Map<string,int>();inner.set('one',7);const outer=new Map<string,Map<string,int>>();outer.set('child',inner);console.log(outer.get('child')?.size??-1);}",false,"1\n"),
  ("type Handler={run:Once<Function<int>>;};function wrap(values:Map<string,Handler>):Handler{return {run:():int=>values.size};}function main(){const children=new Map<string,Handler>();children.set('child',wrap(new Map<string,Handler>()));const handler=wrap(children);console.log(handler.run());}",false,"1\n"),
 ] {
  let source=support::format_round_trip(source);let checked=tsrust_compiler::check(&source).unwrap();let rust=checked.emit_rust();
  assert_eq!(rust.contains("const TSR_MAP_INLINE_FIRST:bool=true;"),inline);
  if inline{assert!(rust.lines().any(|line|line.starts_with("type Map") && line.contains("TsrMapNoFirst")),"ordinary Maps keep Vec storage alongside scoped Maps");}else{assert!(rust.contains(runtime_maps::MAP_RUNTIME),"fallback programs retain the original Vec runtime");}
  let directory=support::TempDirectory::new();let path=directory.0.join("map-layout.rs");std::fs::write(&path,rust).unwrap();
  for level in ["0","3"] {
   let binary=directory.0.join("map-layout");
   support::success(&support::run(support::rustc().args(["--edition=2024","-A","warnings","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));
   let output=support::run(std::process::Command::new(&binary).args(std::iter::empty::<&str>()),60);support::success(&output);assert_eq!(String::from_utf8(output.stdout).unwrap(),expected);
  }
 }
}
