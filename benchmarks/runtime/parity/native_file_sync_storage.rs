// Explicit durability requests stay separate from ordinary I/O throughput.
use std::io::{Read,Seek,SeekFrom,Write};
type String=std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>([T;2]);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
 let(n,seed,input)=(value0,value1,value2);let destination=format!("{input}.out");
 let mut file=std::fs::OpenOptions::new().read(true).write(true).create(true).truncate(true).open(&destination).unwrap();let first=b"a".to_vec();let second=b"b".to_vec();let mut rows=Vec::new();
 for i in 0..n {
  file.write_all(&first).unwrap();file.sync_data().unwrap();file.write_all(&second).unwrap();file.sync_all().unwrap();file.seek(SeekFrom::Current(-2)).unwrap();
  let mut bytes=vec![0;2];let count=loop {match file.read(&mut bytes){Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,value=>break value.unwrap()}};assert_eq!(count,2);
  let sum=bytes.iter().map(|byte|i64::from(*byte)).sum::<i64>();rows.push(NumericArray([i64::try_from(file.stream_position().unwrap()).unwrap(),sum+(i+seed)%17]));
 }
 Array(rows)
}
fn main(){}
#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn explicit_syncs_preserve_complete_bytes_and_cursor_observations() {
  let input=std::env::temp_dir().join(format!("tsrust-sync-std-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));let output=format!("{}.out",input.display());
  struct Cleanup(std::path::PathBuf);impl Drop for Cleanup{fn drop(&mut self){std::fs::remove_file(&self.0).unwrap();}}let _cleanup=Cleanup(output.clone().into());
  let input=std::borrow::Cow::Owned(input.to_str().unwrap().to_owned());let rows=function0(2,7,&input);assert_eq!(rows.0[0].0,[2,202]);assert_eq!(rows.0[1].0,[4,203]);assert_eq!(std::fs::read(output).unwrap(),b"abab");
 }
}
