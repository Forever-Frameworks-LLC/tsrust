type String=std::borrow::Cow<'static,str>;
// Direct std lexical control for this fixed absolute-base/parent corpus.
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let root=if cfg!(windows){r"C:\base"}else{"/base"};let mut checksum=value1;
 for _ in 0..value0 {let source=std::path::Path::new(root).join(value2.as_ref());let mut result=std::path::PathBuf::new();for part in source.components(){match part{std::path::Component::ParentDir=>{result.pop();},std::path::Component::CurDir=>{},value=>result.push(value.as_os_str())}}let text=result.into_os_string().into_string().unwrap();assert!(text.ends_with("file.txt"));checksum+=i64::try_from(text.len()).unwrap();}checksum
}fn main(){}
