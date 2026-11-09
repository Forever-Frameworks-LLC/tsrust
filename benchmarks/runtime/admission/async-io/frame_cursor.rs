// Separate HTTP-adapter shape probe: retained bytes::Bytes frames, no next()->Vec
// bridge. The source is synthetic, so this proves cursor/storage behavior only;
// actual HTTP frame limits, trailers, errors and cancellation remain admission work.
use bytes::{Buf,Bytes};
pub struct FrameCursor{frames:std::collections::VecDeque<Bytes>,current:Bytes}
impl FrameCursor{
 pub fn new(frames:impl IntoIterator<Item=Bytes>)->Self{Self{frames:frames.into_iter().collect(),current:Bytes::new()}}
 pub fn read_into(&mut self,destination:&mut[u8])->usize{
  if destination.is_empty(){return 0;}
  while self.current.is_empty(){let Some(frame)=self.frames.pop_front()else{return 0;};self.current=frame;}
  let count=destination.len().min(self.current.len());destination[..count].copy_from_slice(&self.current[..count]);self.current.advance(count);count
 }
 pub fn remaining_pointer(&self)->*const u8{self.current.as_ptr()}
}
