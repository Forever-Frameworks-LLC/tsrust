mod support;
use support::{execute,reject};
#[test]
fn array_locale_string_formats_scalars_nested_arrays_and_empty_fields() {
 execute(r#"type Row={n:int;};function main(){
 const ints=[1000,-1234567,0];console.log(ints.toLocaleString(),ints.join("|"));
 const numbers:number[]=[1234.5675,-0.0,-0.0004,Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY];console.log(numbers.toLocaleString());
 const words=["a,b","é😀",""];console.log(words.toLocaleString());console.log([true,false].toLocaleString());
 const empty:int[]=[];console.log(empty.toLocaleString().length,[null,null].toLocaleString(),[undefined,undefined].toLocaleString());
 const nested:number[][]=[[1000.5,-0.0],[],[1.2345]];console.log(nested.toLocaleString());
 const rows:Row[]=[{n:1},{n:2}];console.log(rows.toLocaleString());
 const deep=[[[1000]],[[2000,3000]]];console.log(deep.toLocaleString());
}"#, "1,000,-1,234,567,0 1000|-1234567|0\n1,234.568,-0,-0,NaN,∞,-∞\na,b,é😀,\ntrue,false\n0 , ,\n1,000.5,-0,,1.235\n[object Object],[object Object]\n1,000,2,000,3,000\n",None);
}
#[test]
fn array_locale_string_evaluates_receiver_once_and_keeps_owned_elements() {
 execute(r#"function supply(log:int[]):string[]{log.push(1);return ["é".repeat(2),"b"];}
 function main(){const log:int[]=[];console.log(supply(log).toLocaleString(),log.length);const words=["x".repeat(3),"y"];const text=words.toLocaleString();words[0]="changed";console.log(text,words.join("|"));}"#,"éé,b 1\nxxx,y changed|y\n",None);
}
#[test]
fn array_locale_string_rejects_locale_options_and_transferred_receiver() {
 reject("function main(){const values=[1];values.toLocaleString(\"en-US\");}","unsupported");
 reject("function main(){const values=[1];const moved=values;values.toLocaleString();}","transferred");
}

#[test]
fn array_locale_string_rejects_function_source_elements() {
 reject("function plus(value:int):int{return value+1;}function main(){const actions:Function<int,[int]>[]=[plus];actions.toLocaleString();}","unsupported");
}
