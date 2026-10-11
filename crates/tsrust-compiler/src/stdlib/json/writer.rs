//! Bounded JSON Lines output with consuming ownership and explicit flush errors.
use super::{Value,stringify};
use crate::stdlib::{buffered_fs::{self,BufferedWriter},fs::FsFailure};

/// Output errors preserve stable codes, never backend messages or record contents.
#[derive(Debug,Clone,PartialEq,Eq)]
pub struct WriteError {
 pub code:&'static str,pub message:&'static str,pub operation:&'static str,
 /// The current record for write, or next record for flush/close. A flush error
 /// can concern any buffered earlier record; this is not its source location.
 pub line:usize,pub os_code:Option<i64>,
}
pub trait Output {
 fn write_all(&mut self,bytes:&[u8])->Result<(),FsFailure>;
 fn flush(&mut self)->Result<(),FsFailure>;
 fn close(self)->Result<(),FsFailure>;
}
impl Output for BufferedWriter {
 fn write_all(&mut self,bytes:&[u8])->Result<(),FsFailure> {buffered_fs::buffered_write_all(self,bytes)}
 fn flush(&mut self)->Result<(),FsFailure> {buffered_fs::flush(self)}
 fn close(self)->Result<(),FsFailure> {self.close()}
}
impl<W:Output> Output for &mut W {
 fn write_all(&mut self,bytes:&[u8])->Result<(),FsFailure> {(**self).write_all(bytes)}
 fn flush(&mut self)->Result<(),FsFailure> {(**self).flush()}
 // Finish this adapter's buffered output, without consuming the caller's owner.
 fn close(self)->Result<(),FsFailure> {self.flush()}
}
pub struct Writer<W:Output> {output:Option<W>,max_bytes:usize,line:usize,terminal:Option<WriteError>}
impl<W:Output> Writer<W> {
 pub fn new(output:W,max_bytes:usize)->Self {Self{output:Some(output),max_bytes,line:1,terminal:None}}
 fn io_error(&self,operation:&'static str,error:FsFailure)->WriteError {
  WriteError{code:error.code,message:"JSON Lines output operation failed",operation,line:self.line,os_code:error.os_code}
 }
 fn stop(&mut self,error:WriteError)->Result<(),WriteError> {self.terminal=Some(error.clone());Err(error)}
 pub fn write(&mut self,value:&Value)->Result<(),WriteError> {
  if let Some(error)=&self.terminal {return Err(error.clone());}
  let Some(next)=self.line.checked_add(1) else {
   return self.stop(WriteError{code:"position-limit",message:"JSON Lines record count limit exceeded",operation:"write",line:self.line,os_code:None});
  };
  // A rejected encoding must not leave a partial record in the destination.
  // The record budget excludes LF, matching the reader's byte budget.
  let record=match stringify(value,self.max_bytes) {Ok(record)=>record,Err(error)=>{
   return self.stop(WriteError{code:error.code,message:error.message,operation:"write",line:self.line,os_code:None});
  }};
  let output=self.output.as_mut().expect("live JSON Lines writer");
  if let Err(error)=output.write_all(record.as_bytes()).and_then(|()|output.write_all(b"\n")) {
   return self.stop(self.io_error("write",error));
  }
  self.line=next;Ok(())
 }
 pub fn flush(&mut self)->Result<(),WriteError> {
  if let Some(error)=&self.terminal {return Err(error.clone());}
  match self.output.as_mut().expect("live JSON Lines writer").flush() {
   Ok(())=>Ok(()),Err(error)=>self.stop(self.io_error("flush",error)),
  }
 }
 pub fn close(mut self)->Result<(),WriteError> {
  // Always release, including after a failed record. A close failure takes
  // precedence over an earlier failure already returned by write/flush.
  self.output.take().expect("live JSON Lines writer").close().map_err(|error|self.io_error("close",error))?;
  match self.terminal.take(){Some(error)=>Err(error),None=>Ok(())}
 }
}
impl<W:Output> Drop for Writer<W> {
 fn drop(&mut self) {if let Some(output)=self.output.take(){let _=output.close();}}
}

#[cfg(test)]
mod tests {
 use super::*;
 use std::{cell::RefCell,rc::Rc};
 #[derive(Default)]
 struct State {bytes:Vec<u8>,closes:usize,flushes:usize,fail_after:Option<usize>,fail_flush:bool,fail_close:bool}
 struct Probe(Rc<RefCell<State>>);
 fn error()->FsFailure {FsFailure{code:"permission-denied",operation:"writeAll",message:"secret output must not appear".into(),os_code:Some(13)}}
 impl Output for Probe {
  fn write_all(&mut self,bytes:&[u8])->Result<(),FsFailure> {
   let mut s=self.0.borrow_mut();
   let count=s.fail_after.map_or(bytes.len(),|max|max.saturating_sub(s.bytes.len()).min(bytes.len()));
   s.bytes.extend_from_slice(&bytes[..count]);if count<bytes.len(){Err(error())}else{Ok(())}
  }
  fn flush(&mut self)->Result<(),FsFailure> {let mut s=self.0.borrow_mut();s.flushes+=1;if s.fail_flush{Err(error())}else{Ok(())}}
  fn close(self)->Result<(),FsFailure> {let mut s=self.0.borrow_mut();s.closes+=1;if s.fail_close{Err(error())}else{Ok(())}}
 }
 #[test]
 fn compact_records_have_one_lf_and_encoding_failure_writes_nothing() {
  let state=Rc::new(RefCell::new(State::default()));let mut writer=Writer::new(Probe(state.clone()),8);
  writer.write(&Value::Null).unwrap();writer.write(&Value::from("é\n")).unwrap();
  assert_eq!(state.borrow().bytes,"null\n\"é\\n\"\n".as_bytes());
  let before=state.borrow().bytes.clone();let error=writer.write(&Value::from("too long for this record")).unwrap_err();
  assert_eq!(error.line,3);assert_eq!(error.code,"output-limit");assert_eq!(state.borrow().bytes,before);
  assert_eq!(writer.write(&Value::Null).unwrap_err(),error);
  assert_eq!(writer.close().unwrap_err(),error);assert_eq!(state.borrow().closes,1);
 }
 #[test]
 fn partial_output_failure_is_terminal_redacted_and_close_failure_is_reported() {
  let state=Rc::new(RefCell::new(State{fail_after:Some(2),fail_close:true,..State::default()}));
  let mut writer=Writer::new(Probe(state.clone()),8);
  let error=writer.write(&Value::Null).unwrap_err();
  assert_eq!((error.line,error.operation,error.code,error.os_code),(1,"write","permission-denied",Some(13)));
  assert!(!format!("{error:?}").contains("secret"));assert_eq!(writer.write(&Value::Null).unwrap_err(),error);
  assert_eq!(state.borrow().bytes,b"nu");assert_eq!(writer.close().unwrap_err().operation,"close");assert_eq!(state.borrow().closes,1);
 }
 #[test]
 fn exact_byte_limits_and_delimiter_failures_are_distinct() {
  let state=Rc::new(RefCell::new(State::default()));
  let mut writer=Writer::new(Probe(state.clone()),0);
  assert_eq!(writer.write(&Value::Null).unwrap_err().code,"output-limit");assert!(state.borrow().bytes.is_empty());
  drop(writer);assert_eq!(state.borrow().closes,1);
  let state=Rc::new(RefCell::new(State::default()));
  let mut writer=Writer::new(Probe(state.clone()),4);writer.write(&Value::Null).unwrap();writer.close().unwrap();
  assert_eq!(state.borrow().bytes,b"null\n");
  let state=Rc::new(RefCell::new(State{fail_after:Some(4),..State::default()}));
  let mut writer=Writer::new(Probe(state.clone()),4);
  let error=writer.write(&Value::Null).unwrap_err();assert_eq!(error.code,"permission-denied");assert_eq!(error.line,1);
  assert_eq!(state.borrow().bytes,b"null");assert_eq!(writer.close().unwrap_err(),error);assert_eq!(state.borrow().closes,1);
 }
 #[test]
 fn borrowed_drop_flushes_without_closing_and_position_overflow_writes_nothing() {
  let state=Rc::new(RefCell::new(State::default()));let mut probe=Probe(state.clone());
  {let mut writer=Writer::new(&mut probe,8);writer.write(&Value::Null).unwrap();}
  assert_eq!(state.borrow().flushes,1);assert_eq!(state.borrow().closes,0);
  let mut writer=Writer::new(probe,8);writer.line=usize::MAX;
  assert_eq!(writer.write(&Value::Null).unwrap_err().code,"position-limit");
  assert_eq!(state.borrow().bytes,b"null\n");drop(writer);assert_eq!(state.borrow().closes,1);
 }
 #[test]
 fn explicit_flush_failure_and_drop_cleanup_release_once() {
  let state=Rc::new(RefCell::new(State{fail_flush:true,..State::default()}));
  {let mut writer=Writer::new(Probe(state.clone()),8);writer.write(&Value::Null).unwrap();
   let error=writer.flush().unwrap_err();assert_eq!(error.operation,"flush");assert_eq!(writer.flush().unwrap_err(),error);}
  assert_eq!(state.borrow().flushes,1);assert_eq!(state.borrow().closes,1);
 }
 #[test]
 fn buffered_files_flush_close_and_borrowed_writers_remain_usable() {
  use crate::stdlib::{buffered_fs,fs};
  let path=std::env::temp_dir().join(format!("tsrust-jsonl-writer-{}",std::process::id()));
  let mut file=buffered_fs::buffer_writer(fs::create(path.to_str().unwrap()).unwrap(),16).unwrap();
  {let mut writer=Writer::new(&mut file,8);writer.write(&Value::Null).unwrap();
   assert!(std::fs::read(&path).unwrap().is_empty());writer.flush().unwrap();assert_eq!(std::fs::read(&path).unwrap(),b"null\n");
   writer.write(&Value::from(42)).unwrap();writer.close().unwrap();}
  buffered_fs::buffered_write_all(&mut file,b"true\n").unwrap();file.close().unwrap();
  assert_eq!(std::fs::read(&path).unwrap(),b"null\n42\ntrue\n");
  {let file=buffered_fs::buffer_writer(fs::create(path.to_str().unwrap()).unwrap(),16).unwrap();
   let mut writer=Writer::new(file,8);writer.write(&Value::Null).unwrap();}
  assert_eq!(std::fs::read(&path).unwrap(),b"null\n");std::fs::remove_file(path).unwrap();
 }
}
