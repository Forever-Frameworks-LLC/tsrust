type String=std::borrow::Cow<'static,str>;
fn function0(value0:i64,value1:i64,value2:&String)->i64 {
 let dir=value2.as_ref().to_owned();let name="file.tar";let ext="gz";let mut checksum=value1;
 for _ in 0..value0 {let capacity=dir.len().checked_add(1).and_then(|n|n.checked_add(name.len())).and_then(|n|n.checked_add(1+ext.len())).unwrap();let mut text=std::string::String::with_capacity(capacity);text.push_str(&dir);text.push(std::path::MAIN_SEPARATOR);text.push_str(name);text.push('.');text.push_str(ext);assert_eq!(text.len(),19);assert!(text.ends_with("file.tar.gz"));checksum+=i64::try_from(text.len()).unwrap();}checksum
}fn main(){}
