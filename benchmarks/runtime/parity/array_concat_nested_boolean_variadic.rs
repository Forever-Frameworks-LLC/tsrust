// Independent Rust Vec concatenation with standard owned copies and transfers.
type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64) -> Vec<Vec<Vec<bool>>> {let base=vec![vec![true,false],vec![],vec![false],vec![true],vec![true,true,false]];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%4;let count=[0_usize,1,4,5][usize::try_from(mode).unwrap()];let selected=base[..count].to_vec();let first=base[1..usize::try_from(mode%3+2).unwrap()].to_vec();let second=base[..usize::try_from(mode%2).unwrap()].to_vec();let empty:Vec<Vec<bool>>=Vec::new();let mut result=selected;result.extend(first);result.extend(empty);result.extend(second);output.push(result);}output}
fn main(){}
