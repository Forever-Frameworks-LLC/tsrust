// Independent std filesystem operations with owned values and fixed observation rows.
use std::io;
type String = std::borrow::Cow<'static,str>;
struct Array<T>(Vec<T>);
impl<T> Array<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}
struct NumericArray<T>([T; 4]);
impl<T> NumericArray<T>{fn iter(&self)->std::slice::Iter<'_,T>{self.0.iter()}}

struct Entry {name:std::string::String,path:std::string::String,file:bool,directory:bool,link:bool}
fn entries(path:&str)->io::Result<Vec<Entry>> {
 std::fs::read_dir(path)?.map(|entry|{
  let entry=entry?;let kind=entry.file_type()?;
  let unicode=|value:std::ffi::OsString|value.into_string().map_err(|_|io::Error::new(io::ErrorKind::InvalidData,"non-Unicode path"));
  Ok(Entry{name:unicode(entry.file_name())?,path:unicode(entry.path().into_os_string())?,file:kind.is_file(),directory:kind.is_dir(),link:kind.is_symlink()})
 }).collect()
}
fn sample(path:&str,tag:i64)->io::Result<NumericArray<i64>> {
 let entries=entries(path)?;let mut files=0i64;let mut bytes=0i64;let mut observed=tag;
 for entry in &entries {
  let metadata=std::fs::metadata(&entry.path)?;
  if metadata.is_file()!=entry.file || metadata.is_dir()!=entry.directory {return Err(io::Error::new(io::ErrorKind::InvalidData,"file kind changed"));}
  if entry.file {files+=1;bytes=bytes.checked_add(i64::try_from(metadata.len()).unwrap()).unwrap();}
  if entry.directory {observed+=1;}
  if entry.link {observed+=16;}
  if entry.name=="é-🦀.txt" {observed+=7;}
  if entry.name==".hidden" {observed+=1;}
 }
 Ok(NumericArray([i64::try_from(entries.len()).unwrap(),files,bytes,observed]))
}
fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> {
 let (n,seed,input)=(value0,value1,value2);
 let mut rows=Vec::new();for i in 0..n {rows.push(sample(input.as_ref(),(i+seed)%17).unwrap());}Array(rows)
}
fn main(){}

#[cfg(test)]
mod tests {
 use super::*;
 fn split_time(negative:bool,value:std::time::Duration)->Option<(i64,i64)> {
  let nanos=i128::from(value.as_secs())*1_000_000_000+i128::from(value.subsec_nanos());
  let nanos=if negative {-nanos} else {nanos};
  Some((i64::try_from(nanos.div_euclid(1_000_000_000)).ok()?,nanos.rem_euclid(1_000_000_000) as i64))
 }
 #[test]
 fn exact_epoch_pairs_normalize_negative_fractions_without_calendar_bounds() {
  use std::time::Duration;
  assert_eq!(split_time(false,Duration::ZERO),Some((0,0)));
  assert_eq!(split_time(false,Duration::new(7,123)),Some((7,123)));
  assert_eq!(split_time(true,Duration::from_nanos(1)),Some((-1,999999999)));
  assert_eq!(split_time(true,Duration::new(1,500000000)),Some((-2,500000000)));
  assert_eq!(split_time(true,Duration::from_secs(1u64<<63)),Some((i64::MIN,0)));
  assert_eq!(split_time(true,Duration::new(1u64<<63,1)),None);
  assert_eq!(split_time(false,Duration::from_secs(u64::MAX)),None);
 }
 #[test]
 fn owned_entry_snapshot_survives_directory_iterator_and_matches_independent_tree() {
  let root=std::env::temp_dir().join(format!("tsrust-directory-std-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
  struct Cleanup(std::path::PathBuf);impl Drop for Cleanup {fn drop(&mut self){std::fs::remove_dir_all(&self.0).unwrap();}}
  std::fs::create_dir(&root).unwrap();let _cleanup=Cleanup(root.clone());
  for i in 0..64 {std::fs::write(root.join(format!("entry-{i:02}.bin")),vec![i as u8;i%31+1]).unwrap();}
  std::fs::write(root.join(".hidden"),[]).unwrap();std::fs::write(root.join("é-🦀.txt"),[0,255,7]).unwrap();std::fs::create_dir(root.join("nested")).unwrap();
  let values=sample(root.to_str().unwrap(),7).unwrap();assert_eq!(values.0,[67,66,998,16]);
  let values=entries(root.to_str().unwrap()).unwrap();assert_eq!(values.len(),67);
  let unicode=values.iter().find(|entry|entry.name=="é-🦀.txt").unwrap();assert!(unicode.file);assert_eq!(std::fs::read(&unicode.path).unwrap(),[0,255,7]);
 }
}
