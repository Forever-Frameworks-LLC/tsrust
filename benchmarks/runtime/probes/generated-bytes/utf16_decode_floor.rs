// Independent std floor; setup allocations are reported separately by n=0.
type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let size=if value2.ends_with("4k"){4096}else if value2.ends_with("1m"){1048576}else{32};
 let cjk=value2.starts_with("cjk");let text=if cjk{"汉字".repeat(size/4)}else{"é😀Ab".repeat(size/8)};
 let little=value1%2==0;let mut checksum=0i64;
 let units:Vec<u16>=text.encode_utf16().collect();
 for _ in 0..value0 {
  let result=std::string::String::from_utf16(&units).unwrap();checksum+=result.len() as i64+i64::from(result.starts_with(if cjk{"汉"}else{"é"}))+i64::from(result.ends_with(if cjk{"字"}else{"Ab"}));
 }checksum
}
fn main(){}
