type String=std::borrow::Cow<'static,str>;
fn function0(value0: i64, value1: i64, value2: &String) -> i64 {
 let mut checksum=value1;
 for _ in 0..value0 {
  let path=std::path::Path::new(value2.as_ref());
  let root=if path.is_absolute(){std::path::MAIN_SEPARATOR.to_string()}else{std::string::String::new()};
  let dir=path.parent().unwrap().to_str().unwrap().to_owned();let base=path.file_name().unwrap().to_str().unwrap().to_owned();
  let ext=path.extension().map(|value|format!(".{}",value.to_str().unwrap())).unwrap_or_default();let name=path.file_stem().unwrap().to_str().unwrap().to_owned();
  assert_eq!((&*root,&*dir,&*base,&*ext,&*name),("","reports","file.tar.gz",".gz","file.tar"));
  checksum+=i64::try_from(root.len()+dir.len()+base.len()+ext.len()+name.len()).unwrap();
 }checksum
}fn main(){}
