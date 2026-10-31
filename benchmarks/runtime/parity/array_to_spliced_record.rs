// Independent safe standard Rust Vec mutation and owned result transfer.
// Per-iteration receivers are dead after the operation, so Rust may consume
// their storage while preserving all observable values and independent owners.
type String = std::borrow::Cow<'static, str>;
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn copy_owned<T:Clone>(values:&mut[T],target:usize,start:usize,end:usize){let count=end.saturating_sub(start).min(values.len()-target);if target>start{for offset in (0..count).rev(){values[target+offset]=values[start+offset].clone();}}else{for offset in 0..count{values[target+offset]=values[start+offset].clone();}}}
// Nominal field names satisfy the observer ABI; storage is ordinary Rust.
#[derive(Clone)]struct Record0{field0:i64,field1:String,field2:Vec<i64>}
fn item(base:i64,input:&str)->Record0{Record0{field0:base%1009,field1:String::Owned(input.to_owned()),field2:vec![base%17,0,base.checked_add(1).unwrap()%17]}}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<Record0>> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(1).unwrap()).unwrap());for i in 0..value0{let base=value1.checked_add(i).unwrap();let mut values=Vec::with_capacity(6);for offset in 0..6{values.push(item(base.checked_add(offset).unwrap(),value2));}let mode=i.checked_add(value1).unwrap()%10;if mode!=0{let starts=[-9,-6,-1,0,2,5,6,99];let at=usize::try_from(i.checked_mul(3).unwrap().checked_add(value1).unwrap()%8).unwrap();let start=bound(starts[at],values.len());let deletion=match mode{1|5|9=>values.len(),2|3|7=>0,4=>2,6=>1,_=>3};let end=start.checked_add(deletion.min(values.len()-start)).unwrap();let inserted=match mode{6=>vec![item(base.checked_add(11).unwrap(),value2)],7|9=>vec![item(base.checked_add(11).unwrap(),value2),item(base.checked_add(13).unwrap(),value2)],8=>vec![item(base.checked_add(11).unwrap(),value2),item(base.checked_add(13).unwrap(),value2),item(base.checked_add(17).unwrap(),value2)],_=>vec![]};drop(values.splice(start..end,inserted));}output.push(values);}output}
fn main(){}
