mod support;
use support::{execute, reject};

#[test]
fn object_keys_preserves_each_constructor_order_and_copies_keys() {
    execute(r#"
type Row = { first: int; second: string; third: boolean; };
function make(log: int[]): Row { log.push(1); return {third:true,first:7,second:"x"}; }
function keys(value: ReadOnly<Row>): string[] { return Object.keys(value); }
function main() {
    const left: Row = { second:"a", third:false, first:1 };
    const right: Row = { first:2, second:"b", third:true };
    console.log(keys(left).join(","), keys(right).join(","));
    const copy = clone(left);
    copy.first = 8;
    console.log(Object.keys(copy).join(","));
    const names = Object.keys(left);
    names[0] = "changed";
    console.log(names.join(","), Object.keys(left).join(","));
    const log: int[] = [];
    console.log(Object.keys(make(log)).join(","), log.length);
}
"#, "second,third,first first,second,third\nsecond,third,first\nchanged,third,first second,third,first\nthird,first,second 1\n", None);
}

#[test]
fn object_keys_rejects_wrong_arity_and_nullable_receivers() {
    reject("function main() { Object.keys(); }", "arity");
    reject("function main() { Object.keys(1,2); }", "arity");
    reject("type Row = {n:int;}; function main() { const row:Row|null=null; Object.keys(row); }", "unsupported");
}

#[test]
fn object_keys_arrays_tuples_utf16_strings_and_primitives() {
 execute(r#"
function main() {
 const data=[10,20];
 const pair:[int,string]=[1,"x"];
 console.log(Object.keys(data).join(","),Object.keys(pair).join(","));
 console.log(Object.keys("é😀").join(","));
 console.log(Object.keys(12).length,Object.keys(true).length,Object.keys(Number.NaN).length);
 const empty:int[]=[];
 console.log(Object.keys(empty).length);
}
"#, "0,1 0,1\n0,1,2\n0 0 0\n0\n",None);
}

#[test]
fn object_keys_recursive_clones_and_fixed_record_storage_keep_order() {
 execute(r#"
type Node={value:int;children:Node[];};
type Box={rows:Node[];text:string;};
function main(){
 const child:Node={children:[],value:2};
 const tree:Node={value:1,children:[child]};
 const copy=clone(tree);
 console.log(Object.keys(copy).join(","),Object.keys(copy.children[0]).join(","));
 const box:Box={text:"x",rows:[{children:[],value:3}]};
 const other=clone(box);
 console.log(Object.keys(other).join(","),Object.keys(other.rows[0]).join(","));
}
"#, "value,children children,value\ntext,rows children,value\n",None);
}

#[test]
fn object_keys_metadata_is_absent_from_unobserved_record_types() {
 let source="type Row={a:int;b:string;};function main(){const row:Row={b:\"x\",a:1};console.log(row.a);}";
 let native=tsrust_compiler::check(source).unwrap().emit_rust();
 assert!(!native.contains("key_order"));
}

#[test]
fn object_keys_generic_records_and_owned_receiver_restore_keep_order() {
 execute(r#"
type Box<T>={label:string;item:T;};
function labels<T>(value:ReadOnly<T>):string[]{return Object.keys(value);}
function main(){
 const box:Box<string>={item:"value".repeat(1),label:"label"};
 const taken=box.item;
 box.item="restored";
 console.log(labels(box).join(","),taken);
 const copied=clone(box);
 console.log(labels(copied).join(","));
}
"#, "item,label value\nitem,label\n", None);
}

#[test]
fn object_keys_direct_collection_keeps_heap_and_inline_results_independent() {
 execute(r#"
type Many={a:int;b:int;c:int;d:int;e:int;f:int;g:int;h:int;};
function main(){
 const row:Many={h:8,g:7,f:6,e:5,d:4,c:3,b:2,a:1};
 console.log(Object.keys(row).join('|'));
 const first=Object.keys('0123456789é🙂');const second=Object.keys('0123456789é🙂');
 const copied=clone(first);first[0]+='x';first[10]+='y';
 console.log(first[0],first[10],second[0],second[10],copied[0],copied[10]);
 console.log(second.join('|'));
 const values=[1,2,3,4,5,6,7,8,9,10,11];
 const keys=Object.keys(values);keys[10]+='z';
 console.log(keys[10],Object.keys(values)[10],values.length);
 const pair:[int,string,boolean,int,string]=[1,'x',true,2,'y'];
 console.log(Object.keys(pair).join('|'));
}
"#, "h|g|f|e|d|c|b|a\n0x 10y 0 10 0 10\n0|1|2|3|4|5|6|7|8|9|10|11|12\n10z 10 11\n0|1|2|3|4\n",None);
}

#[test]
fn object_keys_collection_avoids_staging_storage_and_drops_iterator_values() {
 let source=support::format_round_trip(r#"
type Row={first:int;second:string;third:boolean;};type Many={a:int;b:int;c:int;d:int;e:int;f:int;g:int;h:int;};type Empty={};
function small():string[]{const row:Row={third:true,first:1,second:'x'};return Object.keys(row);}
function large():string[]{const row:Many={h:8,g:7,f:6,e:5,d:4,c:3,b:2,a:1};return Object.keys(row);}
function text(value:string):string[]{return Object.keys(value);}
function tuple():string[]{const value:[int,string]=[1,'x'];return Object.keys(value);}
function empty():string[]{const value:Empty={};return Object.keys(value);}
function main(){}
"#);
 let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();assert!(!emitted.contains("unsafe"));
 let allocator=include_str!("../../../benchmarks/runtime/allocation-driver.rs").split("fn main()").next().unwrap();
 let harness=r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
struct DropValue(usize);
impl Drop for DropValue{fn drop(&mut self){DROPS.fetch_add(1,Relaxed);}}
fn main(){
 for (function,count,allocations) in [(function0 as fn()->Array<String>,3,0),(function1,8,1),(function3,2,0),(function4,0,0)]{
  let before=ALLOCS.load(Relaxed);let live=LIVE.load(Relaxed);let result=function();
  assert_eq!(result.len(),count);assert_eq!(ALLOCS.load(Relaxed)-before,allocations);
  drop(result);assert_eq!(LIVE.load(Relaxed),live);
 }
 for count in [0,1,4,5,8,9,10,11,17,33,129]{
  let text=String::Owned("x".repeat(count));let before=ALLOCS.load(Relaxed);let reallocated=REALLOCS.load(Relaxed);let live=LIVE.load(Relaxed);
  let result=function2(&text);
  assert_eq!(result.len(),count);assert_eq!(ALLOCS.load(Relaxed)-before,usize::from(count>4)+count.saturating_sub(10));
  assert_eq!(REALLOCS.load(Relaxed),reallocated,"no key array growth");
  for (index,key) in result.iter().enumerate(){assert_eq!(key.as_ref(),index.to_string());}
  drop(result);assert_eq!(LIVE.load(Relaxed),live);
 }
 for count in [0,1,4,5,8,33,129]{
  for exact in [true,false]{
   let dropped=DROPS.load(Relaxed);let live=LIVE.load(Relaxed);
   let values:Array<DropValue>=if exact{(0..count).map(DropValue).collect()}else{(0..count).filter(|_|true).map(DropValue).collect()};
   assert_eq!(values.len(),count);assert_eq!(values.iter().map(|value|value.0).sum::<usize>(),(0..count).sum::<usize>());
   drop(values);assert_eq!(DROPS.load(Relaxed)-dropped,count);assert_eq!(LIVE.load(Relaxed),live);
  }
 }
}
"#;
 let directory=support::TempDirectory::new();let path=directory.0.join("object-key-collection.rs");
 std::fs::write(&path,emitted.replace("fn main()","fn source_main()")+allocator+harness).unwrap();
 for level in ["0","3"]{
  let executable=directory.0.join(format!("object-key-collection-{level}"));
  support::success(&support::run(support::rustc().args(["--edition=2024","-A","warnings","-C",&format!("opt-level={level}"),"-C","overflow-checks=on"]).arg(&path).arg("-o").arg(&executable),60));
  let output=support::run(&mut std::process::Command::new(executable),30);support::success(&output);assert!(output.stdout.is_empty());assert!(output.stderr.is_empty());
 }
}
