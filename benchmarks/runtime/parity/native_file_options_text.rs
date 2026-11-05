// Independent std filesystem operations with owned values and fixed observation rows.
use std::io;
type String = std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>([T; 3]);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}

use std::io::Write;
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
 let (n,seed,input)=(value0,value1,value2);
 let destination=format!("{input}.out");let initial="hé🦀";let newline=b"\n".to_vec();let content=initial.as_bytes().to_vec();let mut rows=Vec::new();
 for i in 0..n {
  std::fs::write(&destination,initial).unwrap();
  {let mut file=std::fs::OpenOptions::new().append(true).create(true).open(&destination).unwrap();file.write_all(&newline).unwrap();}
  let mut file=std::fs::OpenOptions::new().read(true).write(true).open(&destination).unwrap();let before=file.metadata().unwrap().len();
  file.set_len(0).unwrap();file.write_all(&content).unwrap();drop(file);
  let observed=std::fs::read_to_string(&destination).unwrap();assert_eq!(observed,initial);
  rows.push(NumericArray([i64::try_from(before).unwrap(),i64::try_from(std::fs::metadata(&destination).unwrap().len()).unwrap(),(i+seed)%17]));
 }
 Array(rows)
}
fn main(){}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn open_options_validate_before_truncation_and_create_new_overrides_truncate() {
  let path=std::env::temp_dir().join(format!("tsrust-options-std-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
  struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){std::fs::remove_file(&self.0).unwrap();}}
  std::fs::write(&path,b"preserve").unwrap();let _cleanup=Cleanup(path.clone());
  assert_eq!(std::fs::OpenOptions::new().truncate(true).open(&path).unwrap_err().kind(),io::ErrorKind::InvalidInput);
  assert_eq!(std::fs::read(&path).unwrap(),b"preserve");
  assert_eq!(std::fs::OpenOptions::new().write(true).create_new(true).truncate(true).open(&path).unwrap_err().kind(),io::ErrorKind::AlreadyExists);
  assert_eq!(std::fs::read(&path).unwrap(),b"preserve");
  std::fs::remove_file(&path).unwrap();
  let file=std::fs::OpenOptions::new().append(true).create_new(true).truncate(true).open(&path).unwrap();file.sync_all().unwrap();file.sync_data().unwrap();drop(file);
  let input=std::borrow::Cow::Owned(path.to_str().unwrap().to_owned());let rows=function0(2,7,&input);assert_eq!(rows.0[0].0,[8,7,7]);assert_eq!(rows.0[1].0,[8,7,8]);
  std::fs::remove_file(format!("{}.out",path.display())).unwrap();
 }
}
