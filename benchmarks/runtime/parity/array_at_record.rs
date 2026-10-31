// Independent safe Rust Vec removal and optional selection with retained outputs.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn relative_index(index:i64,len:usize)->Option<usize>{let index=if index<0{i64::try_from(len).unwrap().checked_add(index)?}else{index};usize::try_from(index).ok().filter(|&index|index<len)}
#[derive(Clone)]struct Record{id:i64,label:std::string::String}
fn contents(count:i64,seed:i64,input:&str)->Vec<Record>{let mut values=Vec::new();if count>0{values.push(Record{id:seed,label:input.repeat(1)});}if count>1{values.push(Record{id:seed.checked_add(1).unwrap(),label:input.to_string()+"!"});}if count>2{values.push(Record{id:seed.checked_add(2).unwrap(),label:"é😀e\u{301}\0".to_owned()});}values}
fn describe(value:&Record)->String{String::Owned(value.id.to_string()+":"+&value.label)}
fn selection(output:&mut Vec<String>,selected:Option<&Record> ){output.push(String::Borrowed(if selected.is_some(){"object"}else{"undefined"}));output.push(selected.map(describe).unwrap_or(String::Borrowed("undefined")));}
fn snapshot(output:&mut Vec<String>,values:&[Record],count:usize){output.push(String::Owned(values.len().to_string()));for at in 0..count{output.push(values.get(at).map(describe).unwrap_or(String::Borrowed("<absent>")));}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<String> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(6).unwrap()).unwrap());for i in 0..value0{let count=i.checked_add(value1).unwrap()%4;let mut values=contents(count,value1,value2);let index=(i.checked_mul(5).unwrap().checked_add(value1).unwrap()%9).checked_sub(4).unwrap();let selected=relative_index(index,values.len()).map(|at|values[at].clone());selection(&mut output,selected.as_ref());snapshot(&mut output,&values,3);}Array(output)}
fn main(){}
