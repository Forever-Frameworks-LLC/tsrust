// Independent Rust Vec concatenation with standard owned copies and transfers.
type String=std::borrow::Cow<'static,str>;
#[derive(Clone)]struct Record0{field0:i64,field1:String,field2:Vec<i64>}
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<Record0>> {let base=vec![Record0{field0:value1,field1:String::Owned(value2.to_string()),field2:vec![value1,1]},Record0{field0:2,field1:String::Borrowed("é"),field2:vec![]},Record0{field0:3,field1:String::Borrowed("a\0b"),field2:vec![2,3]},Record0{field0:4,field1:String::Borrowed("😀"),field2:vec![4]},Record0{field0:5,field1:String::Borrowed("e\u{301}"),field2:vec![5,6]}];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%4;let count=[0_usize,1,4,5][usize::try_from(mode).unwrap()];let selected=base[..count].to_vec();let result=selected;output.push(result);}output}
fn main(){}
