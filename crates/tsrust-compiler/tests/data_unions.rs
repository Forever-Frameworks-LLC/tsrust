mod support;
use support::{execute,reject};

#[test]
fn common_union_fields_clone_without_record_parameter_storage_assumptions(){
 execute(r#"
import {strictEqual} from 'std:testing';
type First={code:string;offset:int;};
type Second={code:string;retry:boolean;};
function choose(first:boolean):First|Second{return first?{code:'first'.repeat(1),offset:1}:{code:'second'.repeat(1),retry:false};}
function copyCode(value:ReadOnly<First|Second>):string{return clone(value.code);}
function main(){
 const first=choose(true);const second=choose(false);
 strictEqual(copyCode(first),'first');strictEqual(copyCode(second),'second');
 strictEqual(first.code,'first');strictEqual(second.code,'second');
}
"#, "",None);
}

#[test]
fn data_unions_preserve_primitive_tags_and_canonical_transport() {
 execute(r#"
function choose(flag:boolean):int|string {return flag?7:"text".repeat(1);}
function display(value:ReadOnly<string|int>):string {return String(value);}
function main(){
 const a:int|string=choose(true);const b:string|int=choose(false);
 console.log(display(a),display(b),typeof a,typeof b,Boolean(a),Boolean(b));
 const c:int|string|boolean=false;
 console.log(String(c),typeof c,Boolean(c));
 const copy=clone(b);console.log(String(copy),String(b));
}
"#, "7 text number string true true\nfalse boolean false\ntext text\n",None);
}

#[test]
fn data_unions_non_null_readonly_subsets_preserve_source_owners() {
 execute(r#"
function choose(flag:boolean):int|string|null {return flag?"owned".repeat(1):4;}
function inspect(value:ReadOnly<int|string|null>):string {return String(value!);}
function main(){
 const value:int|string|null=choose(true);
 console.log(inspect(value),String(value),String(value??"fallback"));
 const missing:int|string|null=null;
 console.log(String(missing??"fallback"));
}
"#, "owned owned owned\nfallback\n",None);
}

#[test]
fn data_unions_clone_record_array_and_string_alternatives() {
 execute(r#"
type Leaf={label:string;};
type Node={value:int|Leaf;children:Node[]|int[];};
function main(){
 const none:Node[]=[];
 const leaf:Leaf={label:"leaf".repeat(1)};
 const child:Node={value:leaf,children:none};
 const parent:Node={children:[child],value:3};
 const copied=clone(parent);
 console.log(String(copied.value),String(copied.children),Object.keys(copied).join(","));
 const data:string|int[]|Leaf=[4,5];
 const copy=clone(data);
 console.log(String(copy),String(data),typeof data);
}
"#, "3 [object Object] children,value\n4,5 4,5 object\n",None);
}

#[test]
fn data_unions_reject_void_callable_mixes_and_direct_layout_cycles(){
 reject("type Bad={next:Bad|string;};function main(){}","recursive-layout");
 reject("function main(){const value:int|void=1;}","unsupported");
 reject("function main(){const value:int|(Function<int>)=1;}","unsupported");
}

#[test]
fn data_unions_typeof_guards_narrow_and_invalidate_stable_places(){
 execute(r#"
type Row={value:int|string|boolean;};
function inspect(value:ReadOnly<int|string|boolean>):string {
 if(typeof value==="string"){return value.toUpperCase();}
 if("number"===typeof value){return String(value+1);}
 return String(value);
}
function show(value:ReadOnly<int|string>):string{return String(value);}
function main(){
 const a:int|string|boolean="hello".repeat(1);const b:int|string|boolean=7;const c:int|string|boolean=false;
 console.log(inspect(a),inspect(b),inspect(c));
 const row:Row={value:"item"};
 if(typeof row.value!=="boolean"){console.log(show(row.value),String(row.value));}
 row.value=5;
 if(typeof row.value==="number"){console.log(row.value+2);}
 console.log(String(a),String(b),String(c));
}
"#, "HELLO 8 false\nitem item\n7\nhello 7 false\n",None);
}

#[test]
fn data_unions_fallbacks_keep_all_present_tags_and_assignment_is_lazy(){
 execute(r#"
function supply(log:int[]):string {log.push(1);return "fallback";}
function show(value:ReadOnly<int|string|boolean|null>,log:int[]):string{return String(value??supply(log));}
function main(){
 const log:int[]=[];
 const a:int|string|boolean|null="word";const b:int|string|boolean|null=true;const c:int|string|boolean|null=null;
 console.log(show(a,log),show(b,log),show(c,log),log.length);
 let value:int|string|boolean|null=null;
 console.log(String(value??=42),String(value));
 console.log(String(value??=supply(log)),log.length);
}
"#, "word true fallback 1\n42 42\n42 1\n",None);
 reject("function read(value:ReadOnly<string|int[]|null>):int{return value?.length??0;}function main(){}","unsupported");
}

#[test]
fn data_unions_required_and_defaulted_mutable_parameters_keep_tags(){
 execute(r#"
function show(value:Mutable<int|string>="default"):string{return String(value);}
function append(value:Mutable<int|int[]>){if(typeof value==="object"){value.push(8);}}
function main(){
 let value:int|string="owned".repeat(1);
 console.log(show(),show(value),String(value));
 const data:int|int[]=[1];append(data);console.log(String(data));
}
"#, "default owned owned\n1,8\n",None);
}

#[test]
fn data_unions_multiple_owned_default_borrows_clone_and_fallbacks(){
 execute(r#"
function operate(value:Mutable<string|int[]>=[1]):string {
 if(typeof value==="object"){value.push(9);}
 const copied=clone(value);return String(copied);
}
function fallback(value:Mutable<string|int[]|null>=null):string{return String(value??[4]);}
function present(value:Mutable<string|int[]|null>=[2]):string{return String(clone(value!));}
function main(){
 const words:string|int[]="x".repeat(1);const nums:string|int[]=[3];
 console.log(operate(),operate(words),operate(nums),String(nums),String(words));
 console.log(fallback(),fallback(words),fallback(nums),present(),present(words));
}
"#, "1,9 x 3,9 3,9 x\n4 x 3,9 2 x\n",None);
}

#[test]
fn data_unions_generic_patterns_keep_concrete_and_inferred_alternatives(){
 execute(r#"
function echo<T>(value:ReadOnly<T|int>):string{return String(value);}
function nullable<T>(value:ReadOnly<T|null>):string{return String(value);}
function main(){
 const value:int|string|boolean="text";
 console.log(echo(value),echo<string>(4),nullable(value));
}
"#, "text 4 text\n",None);
}

#[test]
fn data_unions_copy_subsets_and_scalar_fallback_views_keep_owners(){
 execute(r#"
function scalar(value:int|boolean):string{return String(value);}
function inspect(value:ReadOnly<int|string|boolean>):string{
 if(typeof value!=="string"){console.log(scalar(value));}
 return String(value);
}
function text(value:ReadOnly<int|string>):string{return String(value);}
function mutable(value:Mutable<int|string|boolean>=0):string{
 if(typeof value!=="boolean"){return text(value);}return String(value);
}
function main(){
 const value:int|string|boolean=7;console.log(inspect(value),String(value));
 const copy:int|null=4;const missing:int|null=null;
 console.log(String(copy??"fallback"),String(missing??"fallback"));
 const textValue:int|string|boolean="owned".repeat(1);
 console.log(mutable(textValue),String(textValue),mutable());
}
"#, "7\n7 7\n4 fallback\nowned owned 0\n",None);
}

#[test]
fn data_unions_mutation_invalidates_typeof_proofs_and_recursive_depth_is_bounded(){
 reject("type Row={value:int|string;};function change(row:Row){row.value=7;}function main(){const row:Row={value:\"x\"};if(typeof row.value===\"string\"){change(row);row.value.toUpperCase();}}","type");
 execute(r#"
type Node={children:Node[]|int[];value:string|int;};
function count(node:ReadOnly<Node>,depth:int):int{
 if(depth===0){return 1;}
 const empty:Node[]=[];return count({children:empty,value:depth},depth-1)+1;
}
function main(){const empty:Node[]=[];const node:Node={value:"x",children:empty};const copied=clone(node);console.log(count(copied,100),String(copied.value));}
"#, "101 x\n", None);
}

#[test]
fn data_unions_nested_calls_preserve_normal_worker_stack(){
 let depth=100;
 let source=format!("function identity(value:int|string):int|string{{return value;}}function main(){{console.log(String({}\"deep\".repeat(1){}));}}","identity(".repeat(depth),")".repeat(depth));
 execute(&source,"deep\n",None);
}

#[test]
fn data_unions_recursive_alternative_edges_use_iterative_clone_and_drop(){
 execute(r#"
type Node={children:int[]|Node[];value:string|int;};
function main(){
 const empty:Node[]=[];let root:Node={children:empty,value:"start"};let i=0;
 while(i<12000){root={value:i,children:[root]};i=i+1;}
 const copied=clone(root);
 console.log(String(copied.value),Array.isArray(copied.children),Object.keys(copied).length);
}
"#, "11999 true 2\n",None);
}

#[test]
fn data_unions_spread_arguments_keep_distinct_payload_tags(){
 execute(r#"
function text(value:ReadOnly<int|string>):string{return String(value);}
function pair(first:ReadOnly<int|string>,second:ReadOnly<int|string>=0):string{return String(first)+":"+String(second);}
function main(){const a:int|string=7;const b:int|string="x";const empty:int[]=[];console.log(text(a,...empty),text(b,...empty));const values=[3];console.log(pair(...values,"tail"));}
"#, "7 x\n3:tail\n",None);
}

#[test]
fn data_unions_spread_mutable_slots_preserve_owner_borrows(){
 execute(r#"
type Row={value:int;};
function pair(first:Mutable<Row|string>,second:ReadOnly<Row|string>="end"):string{
 if(typeof first==="object"){first.value=first.value+1;return String(first.value)+":"+String(second);}
 return String(first)+":"+String(second);
}
function main(){const rows:Row[]=[{value:3}];console.log(pair(...rows,"tail"),rows[0].value);}
"#, "4:tail 4\n",None);
}

#[test]
fn data_unions_narrowed_coalesce_can_add_a_fallback_alternative(){
 execute(r#"
function describe(value:ReadOnly<int|string|boolean|null>):string{
 if(typeof value==="string"){return value.toUpperCase();}
 return String(value??"missing");
}
function main(){console.log(describe(7),describe(false),describe(null),describe("text"));}
"#, "7 false missing TEXT\n",None);
}
