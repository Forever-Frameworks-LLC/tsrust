// Independent safe standard Rust Vec mutation and owned result transfer.
// Per-iteration receivers are dead after the operation, so Rust may consume
// their storage while preserving all observable values and independent owners.
type String = std::borrow::Cow<'static, str>;
fn bound(index:i64,len:usize)->usize{let len=i64::try_from(len).unwrap();usize::try_from(if index<0{index.saturating_add(len).max(0)}else{index.min(len)}).unwrap()}
fn copy_owned<T:Clone>(values:&mut[T],target:usize,start:usize,end:usize){let count=end.saturating_sub(start).min(values.len()-target);if target>start{for offset in (0..count).rev(){values[target+offset]=values[start+offset].clone();}}else{for offset in 0..count{values[target+offset]=values[start+offset].clone();}}}
fn item(base:i64)->bool{base%3!=0}
fn function0(value0: i64, value1: i64) -> Vec<bool> {let mut output=Vec::with_capacity(usize::try_from(value0.checked_mul(6).unwrap()).unwrap());for i in 0..value0{let base=value1.checked_add(i).unwrap();let mut values=Vec::with_capacity(6);for offset in 0..6{values.push(item(base.checked_add(offset).unwrap()));}let mode=usize::try_from(i.checked_add(value1).unwrap()%8).unwrap();let starts=[0,1,-6,-9,-9,4,-3,-9];let ends=[5,6,-1,9,9,1,9,-1];let start=bound(starts[mode],values.len());let replacement=item(base.checked_add(11).unwrap());let start=if mode==0{0}else{start};let end=if mode<2{values.len()}else{bound(ends[mode],values.len())};values[start..end.max(start)].fill(replacement);output.extend(values);}output}
fn main(){}
