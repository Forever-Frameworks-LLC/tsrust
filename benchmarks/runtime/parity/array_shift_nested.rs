// Independent safe Rust Vec removal and optional selection with retained outputs.
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn relative_index(index:i64,len:usize)->Option<usize>{let index=if index<0{i64::try_from(len).unwrap().checked_add(index)?}else{index};usize::try_from(index).ok().filter(|&index|index<len)}
fn contents(count:i64,seed:i64)->Vec<Vec<i64>>{let mut values=Vec::new();if count>0{values.push(vec![seed,1]);}if count>1{values.push(vec![]);}if count>2{values.push(vec![seed.checked_add(2).unwrap(),3]);}values}
fn describe(value:&Vec<i64>)->String{String::Owned(value.len().to_string()+":"+&value.iter().map(i64::to_string).collect::<Vec<_>>().join(","))}
fn selection(output:&mut Vec<String>,selected:Option<&Vec<i64>> ){output.push(String::Borrowed(if selected.is_some(){"object"}else{"undefined"}));output.push(selected.map(describe).unwrap_or(String::Borrowed("undefined")));}
fn snapshot(output:&mut Vec<String>,values:&[Vec<i64>],count:usize){output.push(String::Owned(values.len().to_string()));for at in 0..count{output.push(values.get(at).map(describe).unwrap_or(String::Borrowed("<absent>")));}}
fn function0(value0: i64, value1: i64) -> Array<String> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(10).unwrap()).unwrap());for i in 0..value0{let count=i.checked_add(value1).unwrap()%4;let mut values=contents(count,value1);for _ in 0..2{let removed=if values.is_empty(){None}else{Some(values.remove(0))};selection(&mut output,removed.as_ref());snapshot(&mut output,&values,2);}}Array(output)}
fn main(){}
