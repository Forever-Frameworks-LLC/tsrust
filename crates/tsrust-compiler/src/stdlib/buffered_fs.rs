//! Owned, bounded file buffers. The underlying File has exactly one owner.
use super::{fs::{FileHandle,FsFailure,MAX_READ_BYTES,validate_write_length},resource::{Backend,Resource}};
use std::io::{Read,Write};
pub const DEFAULT_CAPACITY:i64=8192;
pub const MAX_CAPACITY:i64=1048576;
pub struct BufferedReaderBackend {file:FileHandle,buffer:Vec<u8>,start:usize,end:usize}
pub struct BufferedWriterBackend {file:FileHandle,buffer:Vec<u8>,capacity:usize}
pub type BufferedReader=Resource<BufferedReaderBackend>;
pub type BufferedWriter=Resource<BufferedWriterBackend>;
fn buffer(capacity:i64,operation:&'static str)->Result<Vec<u8>,FsFailure> {
    if !(1..=MAX_CAPACITY).contains(&capacity) {return Err(FsFailure{code:"capacity-range",operation,message:"Buffer capacity must be between 1 and 1048576 bytes.".into(),os_code:None});}
    let mut bytes=Vec::new();bytes.try_reserve_exact(capacity as usize).map_err(|_|FsFailure{code:"allocation",operation,message:"Unable to allocate file buffer.".into(),os_code:None})?;Ok(bytes)
}
pub fn buffer_reader(file:FileHandle,capacity:i64)->Result<BufferedReader,FsFailure> {
    let mut buffer=buffer(capacity,"bufferReader")?;buffer.resize(capacity as usize,0);
    Ok(Resource::new(BufferedReaderBackend {file,buffer,start:0,end:0}))
}
pub fn buffer_writer(file:FileHandle,capacity:i64)->Result<BufferedWriter,FsFailure> {
    let buffer=buffer(capacity,"bufferWriter")?;
    Ok(Resource::new(BufferedWriterBackend {file,buffer,capacity:capacity as usize}))
}
impl Backend for BufferedReaderBackend {
    type Error=FsFailure;
    fn close(self)->Result<(),FsFailure> {self.file.close()}
}
impl Backend for BufferedWriterBackend {
    type Error=FsFailure;
    fn close(mut self)->Result<(),FsFailure> {
        let flushed=flush_backend(&mut self);
        let closed=self.file.close();
        flushed?;closed
    }
}
pub fn buffered_read(reader:&mut BufferedReader,length:i64)->Result<Vec<u8>,FsFailure> {
    let length=usize::try_from(length).ok().filter(|length|*length<=MAX_READ_BYTES)
        .ok_or_else(||FsFailure::read_error("length-range","Read length must be between 0 and 67108864 bytes."))?;
    if length==0 {return Ok(Vec::new());}
    let reader=reader.get_mut();
    // Reserve the result before performing I/O or consuming buffered bytes.
    let mut result=Vec::new();result.try_reserve_exact(length.min(reader.buffer.len()))
        .map_err(|_|FsFailure::read_error("allocation","Unable to allocate read buffer."))?;
    if reader.start==reader.end {
        let count=loop {match reader.file.get_mut().0.read(&mut reader.buffer) {
            Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,
            result=>break result.map_err(|error|FsFailure::io("read",error))?,
        }};
        reader.start=0;reader.end=count;
    }
    let count=length.min(reader.end-reader.start);
    result.extend_from_slice(&reader.buffer[reader.start..reader.start+count]);reader.start+=count;Ok(result)
}
fn flush_buffer(output:&mut impl Write,buffer:&mut Vec<u8>)->Result<(),FsFailure> {
    let mut written=0;
    while written<buffer.len() {
        let result=match output.write(&buffer[written..]) {
            Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,
            Ok(0)=>Err(std::io::ErrorKind::WriteZero.into()),
            result=>result,
        };
        match result {
            Ok(count)=>written+=count,
            Err(error)=>{buffer.drain(..written);return Err(FsFailure::io("flush",error));}
        }
    }
    buffer.clear();output.flush().map_err(|error|FsFailure::io("flush",error))
}
fn flush_backend(writer:&mut BufferedWriterBackend)->Result<(),FsFailure> {flush_buffer(&mut writer.file.get_mut().0,&mut writer.buffer)}
pub fn flush(writer:&mut BufferedWriter)->Result<(),FsFailure> {flush_backend(writer.get_mut())}
pub fn buffered_write(writer:&mut BufferedWriter,bytes:&[u8])->Result<i64,FsFailure> {
    validate_write_length(bytes.len(),"write")?;
    if bytes.is_empty() {return Ok(0);}
    let writer=writer.get_mut();
    if bytes.len()>writer.capacity-writer.buffer.len() {flush_backend(writer).map_err(|mut error|{error.operation="write";error})?;}
    if bytes.len()>=writer.capacity {return super::fs::write(&mut writer.file,bytes);}
    writer.buffer.extend_from_slice(bytes);Ok(bytes.len() as i64)
}
pub fn buffered_write_all(writer:&mut BufferedWriter,bytes:&[u8])->Result<(),FsFailure> {
    validate_write_length(bytes.len(),"writeAll")?;
    let mut written=0;
    while written<bytes.len() {
        let count=buffered_write(writer,&bytes[written..]).map_err(|mut error|{error.operation="writeAll";error})? as usize;
        if count==0 {return Err(FsFailure::io("writeAll",std::io::ErrorKind::WriteZero.into()));}
        written+=count;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    static LOCK:std::sync::Mutex<()>=std::sync::Mutex::new(());
    fn path(label:&str)->std::path::PathBuf {std::env::temp_dir().join(format!("tsrust-buffered-{}-{label}",std::process::id()))}
    #[test]
    fn reader_preserves_read_ahead_and_writer_flushes_only_when_requested_or_closed() {
        let _guard=LOCK.lock().unwrap();
        let path=path("roundtrip");std::fs::write(&path,[0,1,2,3,4,5]).unwrap();
        let name=path.to_str().unwrap();
        let mut reader=buffer_reader(super::super::fs::open(name).unwrap(),4).unwrap();
        assert_eq!(buffered_read(&mut reader,2).unwrap(),[0,1]);
        assert!(buffered_read(&mut reader,0).unwrap().is_empty());
        assert_eq!(buffered_read(&mut reader,3).unwrap(),[2,3]);
        assert_eq!(buffered_read(&mut reader,3).unwrap(),[4,5]);
        assert!(buffered_read(&mut reader,1).unwrap().is_empty());reader.close().unwrap();
        let mut writer=buffer_writer(super::super::fs::create(name).unwrap(),4).unwrap();
        assert_eq!(buffered_write(&mut writer,&[7,8]).unwrap(),2);
        assert!(std::fs::read(&path).unwrap().is_empty());
        flush(&mut writer).unwrap();assert_eq!(std::fs::read(&path).unwrap(),[7,8]);
        buffered_write_all(&mut writer,&[9,10,11,12,13]).unwrap();
        buffered_write_all(&mut writer,&[14]).unwrap();writer.close().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(),[7,8,9,10,11,12,13,14]);
        std::fs::remove_file(path).unwrap();
    }
    struct PartialSink {bytes:Vec<u8>,calls:usize,fail:bool}
    impl Write for PartialSink {
        fn write(&mut self,input:&[u8])->std::io::Result<usize> {
            self.calls+=1;
            if self.calls==1 {return Err(std::io::ErrorKind::Interrupted.into());}
            if self.fail && self.bytes.len()==2 {return Err(std::io::ErrorKind::PermissionDenied.into());}
            let count=input.len().min(2);self.bytes.extend_from_slice(&input[..count]);Ok(count)
        }
        fn flush(&mut self)->std::io::Result<()> {Ok(())}
    }
    #[test]
    fn flush_retains_only_unwritten_bytes_for_retry() {
        let mut sink=PartialSink {bytes:vec![],calls:0,fail:true};let mut buffer=vec![1,2,3,4,5];
        assert!(flush_buffer(&mut sink,&mut buffer).is_err());
        assert_eq!(sink.bytes,[1,2]);assert_eq!(buffer,[3,4,5]);assert_eq!(sink.calls,3);
        sink.fail=false;flush_buffer(&mut sink,&mut buffer).unwrap();
        assert_eq!(sink.bytes,[1,2,3,4,5]);assert!(buffer.is_empty());
    }
    #[test]
    fn zero_progress_flush_preserves_the_buffer_and_reports_write_zero() {
        struct Zero;
        impl Write for Zero {
            fn write(&mut self,_:&[u8])->std::io::Result<usize> {Ok(0)}
            fn flush(&mut self)->std::io::Result<()> {panic!("failed data writes must not reach flush")}
        }
        let mut bytes=vec![1,2];let error=flush_buffer(&mut Zero,&mut bytes).unwrap_err();
        assert_eq!((error.code,error.operation),("write-zero","flush"));assert_eq!(bytes,[1,2]);
    }
    #[test]
    fn scope_exit_flushes_and_closes_a_buffered_writer() {
        let _guard=LOCK.lock().unwrap();let path=path("drop");
        {let mut writer=buffer_writer(super::super::fs::create(path.to_str().unwrap()).unwrap(),8).unwrap();buffered_write_all(&mut writer,&[9,8,7]).unwrap();}
        assert_eq!(std::fs::read(&path).unwrap(),[9,8,7]);std::fs::remove_file(path).unwrap();
    }
    #[test]
    fn invalid_capacity_and_failed_close_release_the_consumed_descriptor() {
        use std::os::fd::AsRawFd;
        unsafe extern "C" {fn fcntl(fd:i32,command:i32,...)->i32;}
        let _guard=LOCK.lock().unwrap();
        let path=path("failure");std::fs::write(&path,[1,2]).unwrap();let name=path.to_str().unwrap();
        for capacity in [0,1048577] {
            let file=super::super::fs::open(name).unwrap();let fd=file.get().0.as_raw_fd();
            assert!(buffer_reader(file,capacity).is_err());assert_eq!(unsafe{fcntl(fd,1)},-1);
        }
        let file=super::super::fs::open(name).unwrap();let fd=file.get().0.as_raw_fd();
        let mut writer=buffer_writer(file,8).unwrap();buffered_write_all(&mut writer,&[3]).unwrap();
        assert!(flush(&mut writer).is_err());assert_eq!(writer.get().buffer,[3]);
        assert!(writer.close().is_err());assert_eq!(unsafe{fcntl(fd,1)},-1);
        assert_eq!(std::fs::read(&path).unwrap(),[1,2]);std::fs::remove_file(path).unwrap();
    }
}
