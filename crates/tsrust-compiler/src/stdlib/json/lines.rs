//! Private bounded JSON Lines reader; public file adapters follow separately.
use super::{Error,Limits,Value,failure,parse};
use std::io::BufRead;
const MAX_POSITION:usize=if usize::BITS<64{usize::MAX}else{i64::MAX as usize};

#[derive(Debug,Clone,PartialEq,Eq)]
pub struct LineError {
 pub error:Error,
 pub line:usize,
 pub io_kind:Option<std::io::ErrorKind>,
 pub os_code:Option<i32>,
}
/// Native reader dispatch preserves each adapter's concrete owner or borrow.
/// No trait object, reference counting, or lifetime conversion is involved.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct CloseError {pub code:&'static str,pub os_code:Option<i64>}
pub trait Source {
 fn read_jsonl(&mut self)->Result<Option<Value>,LineError>;
 fn close_jsonl(self)->Result<(),CloseError>;
}
/// Buffer capacity belongs to the supplied reader. Borrowed adapters consume only
/// the current record, preserving unread buffered bytes when stopped early.
pub struct Lines<R:BufRead> {
 reader:Option<R>,limits:Limits,record:Vec<u8>,offset:usize,line:usize,terminal:Option<LineError>,
}
impl<R:BufRead> Lines<R> {
 pub fn new(reader:R,limits:Limits)->Result<Self,LineError> {
  if limits.depth>64 {return Err(LineError{error:failure("invalid-limits",0),line:1,io_kind:None,os_code:None});}
  Ok(Self{reader:Some(reader),limits,record:Vec::new(),offset:0,line:1,terminal:None})
 }
 pub fn close_with<E>(mut self,close:impl FnOnce(R)->Result<(),E>)->Result<(),E> {
  match self.reader.take(){Some(reader)=>close(reader),None=>Ok(())}
 }
 fn finish(&mut self) {self.reader.take();self.record=Vec::new();}
 fn stop<T>(&mut self,error:LineError)->Result<Option<T>,LineError> {
  self.finish();self.terminal=Some(error.clone());Err(error)
 }
 fn error(&self,code:&'static str,relative:usize)->LineError {
  let error=match self.offset.checked_add(relative).filter(|value|*value<=MAX_POSITION) {Some(offset)=>failure(code,offset),None=>failure("position-limit",self.offset)};
  LineError{error,line:self.line,io_kind:None,os_code:None}
 }
 fn value<T>(&mut self,decode:impl FnOnce(&[u8],Limits)->Result<T,Error>)->Result<Option<T>,LineError> {
  match decode(&self.record,self.limits) {
   Ok(value)=>Ok(Some(value)),
   Err(error)=>self.stop(self.error(error.code,error.offset)),
  }
 }
 pub fn next(&mut self)->Result<Option<Value>,LineError> {self.next_with(parse)}
 /// A direct typed decoder can construct its output without a dynamic tree.
 pub fn next_with<T>(&mut self,decode:impl FnOnce(&[u8],Limits)->Result<T,Error>)->Result<Option<T>,LineError> {
  if let Some(error)=&self.terminal {return Err(error.clone());}
  if self.reader.is_none() {return Ok(None);}
  self.record.clear();
  loop {
   let remaining=match self.reader.as_mut().unwrap().fill_buf() {
    Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,
    Err(error)=>{let mut failure=self.error("io-error",self.record.len());failure.io_kind=Some(error.kind());failure.os_code=error.raw_os_error();return self.stop(failure);}
    Ok(bytes)=>bytes,
   };
   if remaining.is_empty() {let result=if self.record.is_empty(){Ok(None)}else{self.value(decode)};self.finish();return result;}
   // Inspect at most one byte beyond the remaining record budget. A reader may
   // expose a much larger existing buffer; that does not authorize an unbounded scan.
   let inspect=remaining.len().min(self.limits.bytes.saturating_sub(self.record.len()).saturating_add(1));
   let remaining=&remaining[..inspect];
   let newline=remaining.iter().position(|byte|*byte==b'\n');let count=newline.unwrap_or(remaining.len());
   if count>self.limits.bytes-self.record.len() {return self.stop(self.error("byte-limit",self.limits.bytes));}
   let needed=self.record.len()+count;
   if self.offset.checked_add(needed).is_none_or(|value|value>MAX_POSITION){return self.stop(self.error("position-limit",0));}
   if needed>self.record.capacity() {
    let capacity=self.record.capacity().max(128).saturating_mul(2).min(self.limits.bytes).max(needed);
    if self.record.try_reserve_exact(capacity-self.record.len()).is_err(){return self.stop(self.error("allocation",self.record.len()));}
   }
   self.record.extend_from_slice(&remaining[..count]);
   self.reader.as_mut().unwrap().consume(count+usize::from(newline.is_some()));
   if newline.is_some() {
    let value=self.value(decode)?;
    let Some(offset)=self.offset.checked_add(self.record.len()).and_then(|offset|offset.checked_add(1)).filter(|value|*value<=MAX_POSITION) else {return self.stop(self.error("position-limit",0));};
    let Some(line)=self.line.checked_add(1).filter(|value|*value<=MAX_POSITION) else {return self.stop(self.error("position-limit",0));};
    self.offset=offset;self.line=line;return Ok(value);
   }
  }
 }
 #[cfg(test)]
 fn retained_bytes(&self)->usize {self.record.capacity()}
}

#[cfg(test)]
mod tests {
 use super::*;
 use crate::stdlib::json::{Limits,Value};
 use std::io::{self,Read};
 fn limits()->Limits {Limits{bytes:64,depth:4,entries:8,nodes:16}}
 #[test]
 fn public_integer_position_bounds_reject_without_wrapping() {
  let mut reader=Lines::new(b"0\n".as_slice(),limits()).unwrap();reader.offset=MAX_POSITION;
  let error=reader.next().unwrap_err();assert_eq!(error.error.code,"position-limit");assert_eq!(error.error.offset,MAX_POSITION);
  let mut reader=Lines::new(b"0\n".as_slice(),limits()).unwrap();reader.offset=MAX_POSITION-2;
  assert_eq!(reader.next().unwrap(),Some(Value::from(0)));assert_eq!(reader.offset,MAX_POSITION);assert_eq!(reader.next().unwrap(),None);
 }
 #[test]
 fn explicit_close_reports_failure_once_and_skips_already_released_readers() {
  let closes=std::cell::Cell::new(0);
  let reader=Lines::new(b"null\n".as_slice(),limits()).unwrap();
  assert_eq!(reader.close_with(|_|{closes.set(closes.get()+1);Err("close failed")}),Err("close failed"));
  let mut reader=Lines::new(b"".as_slice(),limits()).unwrap();assert_eq!(reader.next().unwrap(),None);
  assert_eq!(reader.close_with(|_|{closes.set(closes.get()+1);Err("closed twice")}),Ok(()));
  let mut reader=Lines::new(b"bad\n".as_slice(),limits()).unwrap();assert!(reader.next().is_err());
  assert_eq!(reader.close_with(|_|{closes.set(closes.get()+1);Err("closed twice")}),Ok(()));
  assert_eq!(closes.get(),1);
 }
 #[test]
 fn records_survive_every_chunk_boundary_and_eof_is_not_null() {
  let input="null\r\n{\"text\":\"é🦀\\n\\uD83E\\uDD80\"}\n42";
  for capacity in 1..=input.len() {
   let mut reader=Lines::new(std::io::BufReader::with_capacity(capacity,input.as_bytes()),limits()).unwrap();
   assert_eq!(reader.next().unwrap(),Some(Value::Null));
   assert_eq!(reader.next().unwrap().unwrap()["text"],"é🦀\n🦀");
   assert_eq!(reader.next().unwrap(),Some(Value::from(42)));
   assert_eq!(reader.next().unwrap(),None);assert_eq!(reader.next().unwrap(),None);
  }
  for input in [b"".as_slice(),b"null\n"] {
   let mut reader=Lines::new(std::io::BufReader::with_capacity(1,input),limits()).unwrap();
   if !input.is_empty(){assert_eq!(reader.next().unwrap(),Some(Value::Null));}
   assert_eq!(reader.next().unwrap(),None);
  }
 }
 #[test]
 fn malformed_records_have_absolute_offsets_and_terminal_errors() {
  for (input,code,line,offset) in [("0\n\n","unexpected-end",2,2),("0\nsecret\n","invalid-json",2,2),("\u{feff}0\n","invalid-json",1,0),("0\n[1,\n","unexpected-end",2,5)] {
   let mut reader=Lines::new(std::io::BufReader::with_capacity(2,input.as_bytes()),limits()).unwrap();
   if line==2 {reader.next().unwrap();}
   let error=reader.next().unwrap_err();assert_eq!(error.line,line);assert_eq!(error.error.code,code);assert_eq!(error.error.offset,offset);
   assert!(!format!("{error:?}").contains("secret"));assert_eq!(reader.next().unwrap_err(),error);
  }
 }
 #[test]
 fn generic_reader_dispatch_keeps_borrowed_owner_and_unread_bytes() {
  struct Borrowed<'a>(Lines<&'a mut std::io::Cursor<Vec<u8>>>);
  impl Source for Borrowed<'_> {
   fn read_jsonl(&mut self)->Result<Option<Value>,LineError>{self.0.next()}
   fn close_jsonl(self)->Result<(),CloseError>{self.0.close_with(|_|Ok(()))}
  }
  fn read(source:&mut impl Source)->Result<Option<Value>,LineError>{source.read_jsonl()}
  fn close(source:impl Source)->Result<(),CloseError>{source.close_jsonl()}
  let mut owner=std::io::Cursor::new(b"0\nbad\n42\n".to_vec());
  let mut first=Borrowed(Lines::new(&mut owner,limits()).unwrap());
  assert_eq!(read(&mut first).unwrap(),Some(Value::from(0)));
  close(first).unwrap();
  assert_eq!(owner.position(),2);
  let mut second=Borrowed(Lines::new(&mut owner,limits()).unwrap());
  let error=read(&mut second).unwrap_err();
  assert_eq!(error.error.code,"invalid-json");
  assert_eq!(read(&mut second).unwrap_err(),error);
  close(second).unwrap();
  // A fresh adapter starts at the owner's preserved next record.
  let mut third=Borrowed(Lines::new(&mut owner,limits()).unwrap());
  assert_eq!(read(&mut third).unwrap(),Some(Value::from(42)));
  assert_eq!(read(&mut third).unwrap(),None);
  close(third).unwrap();
  assert_eq!(owner.position(),9);
 }
 #[test]
 fn file_reader_preserves_read_ahead_across_jsonl_and_byte_reads() {
  use crate::stdlib::fs::runtime::{buffered_fs,fs};
  let path=std::env::temp_dir().join(format!("tsrust-jsonl-borrowed-{}",std::process::id()));
  std::fs::write(&path,b"0\n1\nnull").unwrap();
  let mut input=buffered_fs::buffer_reader(fs::open(path.to_str().unwrap()).unwrap(),8).unwrap();
  assert_eq!(input.read(&mut []).unwrap(),0);
  {let mut lines=Lines::new(&mut input,limits()).unwrap();assert_eq!(lines.next().unwrap(),Some(Value::from(0)));}
  assert_eq!(buffered_fs::buffered_read(&mut input,2).unwrap(),b"1\n");
  {let mut lines=Lines::new(&mut input,limits()).unwrap();assert_eq!(lines.next().unwrap(),Some(Value::Null));assert_eq!(lines.next().unwrap(),None);}
  input.close().unwrap();std::fs::remove_file(path).unwrap();
 }
 #[test]
 fn owned_file_reader_closes_on_eof_error_and_early_drop() {
  use crate::stdlib::fs::runtime::{buffered_fs,fs};
  unsafe extern "C" {fn fcntl(fd:i32,command:i32,...)->i32;}
  let path=std::env::temp_dir().join(format!("tsrust-jsonl-owned-{}",std::process::id()));
  for (input,mode) in [(b"null".as_slice(),0),(b"bad\n",1),(b"0\n1\n",2)] {
   std::fs::write(&path,input).unwrap();
   let file=fs::open(path.to_str().unwrap()).unwrap();let fd=crate::stdlib::fs::runtime::raw_fd(&file);
   let mut lines=Lines::new(buffered_fs::buffer_reader(file,2).unwrap(),limits()).unwrap();
   match mode {
    0=>{assert_eq!(lines.next().unwrap(),Some(Value::Null));assert_eq!(unsafe{fcntl(fd,1)},-1);},
    1=>{assert_eq!(lines.next().unwrap_err().error.code,"invalid-json");assert_eq!(unsafe{fcntl(fd,1)},-1);},
    _=>{assert_eq!(lines.next().unwrap(),Some(Value::from(0)));assert_ne!(unsafe{fcntl(fd,1)},-1);},
   }
   drop(lines);assert_eq!(unsafe{fcntl(fd,1)},-1);
  }
  std::fs::remove_file(path).unwrap();
 }
 struct Counted {reads:std::rc::Rc<std::cell::Cell<usize>>,drops:std::rc::Rc<std::cell::Cell<usize>>,bytes:&'static [u8]}
 impl Read for Counted {fn read(&mut self,out:&mut [u8])->io::Result<usize>{let count=out.len().min(self.bytes.len());out[..count].copy_from_slice(&self.bytes[..count]);self.bytes=&self.bytes[count..];self.reads.set(self.reads.get()+count);Ok(count)}}
 impl Drop for Counted {fn drop(&mut self){self.drops.set(self.drops.get()+1);}}
 #[test]
 fn early_drop_preserves_unconsumed_bytes_in_a_borrowed_reader() {
  use std::io::BufRead;
  let mut input=std::io::BufReader::with_capacity(8,b"0\n1\n".as_slice());
  {let mut reader=Lines::new(&mut input,limits()).unwrap();assert_eq!(reader.next().unwrap(),Some(Value::from(0)));}
  assert_eq!(input.fill_buf().unwrap(),b"1\n");
 }
 #[test]
 fn interruptions_retry_and_io_failures_are_redacted_terminal_errors() {
  struct Faults(u8);
  impl Read for Faults {fn read(&mut self,out:&mut [u8])->io::Result<usize>{self.0+=1;match self.0 {
   1=>Err(io::ErrorKind::Interrupted.into()),
   2=>{out[..5].copy_from_slice(b"null\n");Ok(5)},
   3=>{out[..3].copy_from_slice(b"sec");Ok(3)},
   _=>Err(io::Error::new(io::ErrorKind::PermissionDenied,"secret input must not appear")),
  }}}
  let mut reader=Lines::new(io::BufReader::with_capacity(8,Faults(0)),limits()).unwrap();
  assert_eq!(reader.next().unwrap(),Some(Value::Null));
  let error=reader.next().unwrap_err();assert_eq!(error.error.code,"io-error");assert_eq!(error.error.offset,8);assert_eq!(error.line,2);
  assert_eq!(error.io_kind,Some(io::ErrorKind::PermissionDenied));assert!(!format!("{error:?}").contains("secret"));
  assert_eq!(reader.next().unwrap_err(),error);
 }
 #[test]
 fn decoder_boundary_accepts_direct_typed_output() {
  let mut reader=Lines::new(b"42\n7".as_slice(),limits()).unwrap();
  fn integer(bytes:&[u8],_:Limits)->Result<i64,Error> {std::str::from_utf8(bytes).unwrap().parse().map_err(|_|failure("invalid-json",0))}
  assert_eq!(reader.next_with(integer).unwrap(),Some(42));assert_eq!(reader.next_with(integer).unwrap(),Some(7));assert_eq!(reader.next_with(integer).unwrap(),None);
 }
 #[test]
 fn typed_lines_own_values_and_release_reader_on_shape_failure() {
  let reads=std::rc::Rc::new(std::cell::Cell::new(0));
  let drops=std::rc::Rc::new(std::cell::Cell::new(0));
  let input=Counted{reads:reads.clone(),drops:drops.clone(),bytes:b"[\"first\"]\r\n[\"second\"]\n[42]\n"};
  let mut reader=Lines::new(std::io::BufReader::with_capacity(3,input),limits()).unwrap();
  let first=reader.next_with(super::super::typed::decode::<Vec<String>>).unwrap().unwrap();
  let second=reader.next_with(super::super::typed::decode::<Vec<String>>).unwrap().unwrap();
  assert_eq!(first,vec!["first"]);assert_eq!(second,vec!["second"]);
  let error=reader.next_with(super::super::typed::decode::<Vec<String>>).unwrap_err();
  assert_eq!(error.line,3);assert_eq!(error.error.code,"shape-mismatch");
  // Two completed records occupy 22 bytes; the numeric token ends at byte 24.
  assert_eq!(error.error.offset,24);assert_eq!(drops.get(),1);
  assert_eq!(reader.next_with(super::super::typed::decode::<Vec<String>>).unwrap_err(),error);
  drop(reader);assert_eq!(drops.get(),1);assert_eq!(first[0],"first");
 }
 #[test]
 fn reads_and_retention_are_bounded_and_owned_readers_close() {
  let reads=std::rc::Rc::new(std::cell::Cell::new(0));let drops=std::rc::Rc::new(std::cell::Cell::new(0));
  let input=Counted{reads:reads.clone(),drops:drops.clone(),bytes:b"0\n123456789012345678901234567890"};
  let mut reader=Lines::new(std::io::BufReader::with_capacity(3,input),Limits{bytes:4,..limits()}).unwrap();
  assert_eq!(reader.next().unwrap(),Some(Value::from(0)));assert!(reads.get()<=3);
  let error=reader.next().unwrap_err();assert_eq!(error.error.code,"byte-limit");assert_eq!(error.error.offset,6);
  assert!(reads.get()<=9);assert_eq!(drops.get(),1);assert!(reader.retained_bytes()<=7);
  drop(reader);assert_eq!(drops.get(),1);
  let input=Counted{reads:reads.clone(),drops:drops.clone(),bytes:b"0\n1\n"};
  let mut reader=Lines::new(std::io::BufReader::with_capacity(1,input),limits()).unwrap();reader.next().unwrap();drop(reader);assert_eq!(drops.get(),2);
  let input=Counted{reads:reads.clone(),drops:drops.clone(),bytes:b"null"};
  let mut reader=Lines::new(std::io::BufReader::with_capacity(1,input),limits()).unwrap();
  assert_eq!(reader.next().unwrap(),Some(Value::Null));assert_eq!(drops.get(),3);
  assert_eq!(reader.next().unwrap(),None);drop(reader);assert_eq!(drops.get(),3);
 }
}
