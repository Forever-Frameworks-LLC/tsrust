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
