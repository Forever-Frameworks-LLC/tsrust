// Independent Rust Vec concatenation with standard owned copies and transfers.
type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Vec<String>> {let base=vec![String::Owned(value2.to_string()),String::Borrowed("é"),String::Borrowed("😀"),String::Borrowed("a\0b"),String::Borrowed("e\u{301}")];let mut output=Vec::with_capacity(usize::try_from(value0).unwrap());for i in 0..value0{let mode=i.checked_add(value1).unwrap()%4;let count=[0_usize,1,4,5][usize::try_from(mode).unwrap()];let selected=base[..count].to_vec();let result=selected;output.push(result);}output}
fn main(){}
