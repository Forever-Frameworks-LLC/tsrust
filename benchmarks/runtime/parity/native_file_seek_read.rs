// Independent std cursor operations; the observation row owns its fixed storage.
use std::io::{Read,Seek,SeekFrom};
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>([T;2]);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
 let (n,seed,input)=(value0,value1,value2);let mut file=std::fs::File::open(input.as_ref()).unwrap();let mut rows=Vec::new();
 for i in 0..n {
  let offset=if i%8==0 {4096} else {(i*97+seed)%4096};file.seek(SeekFrom::Start(u64::try_from(offset).unwrap())).unwrap();
  let mut bytes=vec![0;64];let count=loop {match file.read(&mut bytes){Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,value=>break value.unwrap()}};
  let observed=i64::try_from(file.stream_position().unwrap()).unwrap();let sum=bytes[..count].iter().map(|byte|i64::from(*byte)).sum();rows.push(NumericArray([observed,sum]));
 }
 Array(rows)
}
fn main(){}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn start_relative_end_positions_and_eof_follow_the_plain_file_cursor() {
  let path=std::env::temp_dir().join(format!("tsrust-seek-std-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
  struct Cleanup(std::path::PathBuf);impl Drop for Cleanup{fn drop(&mut self){std::fs::remove_file(&self.0).unwrap();}}
  std::fs::write(&path,b"abcd").unwrap();let _cleanup=Cleanup(path.clone());let mut file=std::fs::File::open(&path).unwrap();
  assert_eq!(file.stream_position().unwrap(),0);assert_eq!(file.seek(SeekFrom::Start(2)).unwrap(),2);assert_eq!(file.seek(SeekFrom::Current(-1)).unwrap(),1);assert_eq!(file.seek(SeekFrom::End(-1)).unwrap(),3);
  assert_eq!(file.seek(SeekFrom::Start(8)).unwrap(),8);assert_eq!(file.read(&mut[0;64]).unwrap(),0);assert_eq!(file.stream_position().unwrap(),8);assert!(file.seek(SeekFrom::Current(-9)).is_err());assert_eq!(file.stream_position().unwrap(),8);
  std::fs::write(&path,b"0123456789abcdef".repeat(256)).unwrap();drop(file);let input=std::borrow::Cow::Owned(path.to_str().unwrap().to_owned());let rows=function0(2,7,&input);assert_eq!(rows.0[0].0,[4096,0]);assert_eq!(rows.0[1].0,[168,4488]);
 }
}
