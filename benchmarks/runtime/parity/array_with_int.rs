// Independent safe standard Rust Vec mutation and owned result transfer.
// Per-iteration receivers are dead after the operation, so Rust may consume
// their storage while preserving all observable values and independent owners.
type String = std::borrow::Cow<'static, str>;
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn copy_owned<T:Clone>(values:&mut[T],target:usize,start:usize,end:usize){let count=end.saturating_sub(start).min(values.len()-target);if target>start{for offset in (0..count).rev(){values[target+offset]=values[start+offset].clone();}}else{for offset in 0..count{values[target+offset]=values[start+offset].clone();}}}
fn item(base:i64)->i64{base%1009}
fn function0(value0: i64, value1: i64) -> Vec<i64> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(6).unwrap()).unwrap());for i in 0..value0{let base=value1.checked_add(i).unwrap();let mut values=Vec::with_capacity(6);for offset in 0..6{values.push(item(base.checked_add(offset).unwrap()));}let indices=[-6,-1,0,5,-3,2,-2,1];let index=indices[usize::try_from(i.checked_add(value1).unwrap()%8).unwrap()];let len=i64::try_from(values.len()).unwrap();let index=if index<0{len.checked_add(index).unwrap()}else{index};let index=usize::try_from(index).unwrap();assert!(index<values.len());let replacement=item(base.checked_add(11).unwrap());values[index]=replacement;output.extend(values);}output}
fn main(){}
