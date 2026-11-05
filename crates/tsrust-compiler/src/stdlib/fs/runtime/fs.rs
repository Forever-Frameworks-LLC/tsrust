//! Synchronous file operations, optional whole-file bounds and consuming native close.

use super::resource::{Backend, Resource};
use std::io::{Read,Write};
#[cfg(unix)]
use std::os::fd::IntoRawFd;

#[derive(Debug)]
pub struct FsFailure {
    pub code: &'static str,
    pub operation: &'static str,
    pub message: String,
    pub os_code: Option<i64>,
}
impl FsFailure {
    pub(super) fn io(operation: &'static str, error: std::io::Error) -> Self {
        use std::io::ErrorKind;
        let code = match error.kind() {
            ErrorKind::NotFound => "not-found",
            ErrorKind::PermissionDenied => "permission-denied",
            ErrorKind::InvalidInput => "invalid-input",
            ErrorKind::Interrupted => "interrupted",
            ErrorKind::IsADirectory => "is-directory",
            ErrorKind::WouldBlock => "would-block",
            ErrorKind::WriteZero => "write-zero",
            ErrorKind::OutOfMemory => "allocation",
            _ => "io",
        };
        Self { code, operation, message: error.to_string(), os_code: error.raw_os_error().map(i64::from) }
    }
    pub(super) fn read_error(code: &'static str, message: &str) -> Self {
        Self { code, operation: "read", message: message.into(), os_code: None }
    }
}

pub struct FileBackend(pub(super) std::fs::File);
pub type FileHandle = Resource<FileBackend>;
impl Backend for FileBackend {
    type Error = FsFailure;
    fn close(self) -> Result<(), FsFailure> {
        #[cfg(unix)]
        { close_file_with(self.0, close_fd) }
        // Safe standard file release on Windows does not expose OS close errors.
        #[cfg(not(unix))]
        { drop(self.0); Ok(()) }
    }
}

pub fn open(path: &str) -> Result<FileHandle, FsFailure> {
    std::fs::File::open(path).map(|file| Resource::new(FileBackend(file))).map_err(|error| FsFailure::io("open", error))
}

/// Create or truncate a write-only file. Existing permissions are preserved;
/// newly created files use the host's normal creation mode and umask.
pub fn create(path:&str)->Result<FileHandle,FsFailure> {
    std::fs::File::create(path).map(|file|Resource::new(FileBackend(file)))
        .map_err(|error|FsFailure::io("create",error))
}

pub fn write(file:&mut FileHandle,bytes:&[u8])->Result<i64,FsFailure> {
    write_with(&mut file.get_mut().0,bytes,false)
}
pub fn write_all(file:&mut FileHandle,bytes:&[u8])->Result<(),FsFailure> {
    write_with(&mut file.get_mut().0,bytes,true).map(|_|())
}
pub(super) fn validate_write_length(length:usize,operation:&'static str)->Result<(),FsFailure> {
    if i64::try_from(length).is_err() {
        return Err(FsFailure {code:"length-range",operation,
            message:"Write length must fit int.".into(),os_code:None});
    }
    Ok(())
}
/// A failing write-all operation may already have changed the file. Do not
/// retry non-interruption failures or imply rollback of successful writes.
fn write_with(output:&mut impl Write,bytes:&[u8],all:bool)->Result<i64,FsFailure> {
    let operation=if all {"writeAll"} else {"write"};
    validate_write_length(bytes.len(),operation)?;
    let mut written=0;
    while written<bytes.len() {
        let count=match output.write(&bytes[written..]) {
            Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,
            result=>result.map_err(|error|FsFailure::io(operation,error))?,
        };
        if !all {return Ok(count as i64);}
        if count==0 {return Err(FsFailure::io(operation,std::io::ErrorKind::WriteZero.into()));}
        written+=count;
    }
    Ok(written as i64)
}

/// Read through EOF; an optional ceiling is enforced independently of metadata races.
pub fn read_file(path:&str,limit:Option<i64>)->Result<Vec<u8>,FsFailure> {
    let limit=limit.map(|length|validate_read_length(length,"readFile")).transpose()?;
    let mut file=open(path).map_err(|mut error|{error.operation="readFile";error})?;
    let result=read_file_with(&mut file.get_mut().0,limit);
    finish_file(file,result,"readFile")
}
fn read_file_with(input:&mut impl Read,limit:Option<usize>)->Result<Vec<u8>,FsFailure> {
    let mut bytes=Vec::new();
    let Some(limit)=limit else {
        input.read_to_end(&mut bytes).map_err(|error|FsFailure::io("readFile",error))?;
        return Ok(bytes);
    };
    let mut chunk=[0u8;8192];
    loop {
        // One extra byte distinguishes exact-limit EOF from an oversized file.
        let size=chunk.len().min(limit-bytes.len()+1);
        let count=match input.read(&mut chunk[..size]) {
            Err(error) if error.kind()==std::io::ErrorKind::Interrupted=>continue,
            result=>result.map_err(|error|FsFailure::io("readFile",error))?,
        };
        if count==0 {return Ok(bytes);}
        let length=bytes.len()+count;
        if length>limit {return Err(FsFailure {code:"length-range",operation:"readFile",message:"File exceeds the requested byte limit.".into(),os_code:None});}
        if length>bytes.capacity() {
            let capacity=bytes.capacity().saturating_mul(2).max(length).min(limit);
            bytes.try_reserve_exact(capacity-bytes.len()).map_err(|_|FsFailure {code:"allocation",operation:"readFile",message:"Unable to allocate file buffer.".into(),os_code:None})?;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
}
/// Validate before truncation. Successful writes do not imply durable storage.
pub fn write_file(path:&str,bytes:&[u8])->Result<(),FsFailure> {
    validate_write_length(bytes.len(),"writeFile")?;
    let mut file=create(path).map_err(|mut error|{error.operation="writeFile";error})?;
    let result=write_all(&mut file,bytes).map_err(|mut error|{error.operation="writeFile";error});
    finish_file(file,result,"writeFile")
}
fn finish_file<T,B:Backend<Error=FsFailure>>(file:Resource<B>,result:Result<T,FsFailure>,operation:&'static str)->Result<T,FsFailure> {
    let closed=file.close().map_err(|mut error|{error.operation=operation;error});
    // Always attempt close once. An earlier I/O error takes precedence.
    let value=result?;closed?;Ok(value)
}

pub(super) fn validate_read_length(length:i64,operation:&'static str)->Result<usize,FsFailure> {
    usize::try_from(length).ok().filter(|length|*length<=isize::MAX as usize)
        .ok_or_else(||FsFailure {code:"length-range",operation,message:"Read length must be nonnegative and fit native storage.".into(),os_code:None})
}
/// Read at most length bytes once. Validate and reserve before changing the cursor.
pub fn read(file: &mut FileHandle, length: i64) -> Result<Vec<u8>, FsFailure> {
    let length = validate_read_length(length,"read")?;
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(|_| FsFailure::read_error("allocation", "Unable to allocate read buffer."))?;
    bytes.resize(length, 0);
    if length == 0 { return Ok(bytes); }
    let count = loop {
        match file.get_mut().0.read(&mut bytes) {
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            result => break result.map_err(|error| FsFailure::io("read", error))?,
        }
    };
    bytes.truncate(count);
    Ok(bytes)
}

#[cfg(unix)]
fn close_file_with(file: std::fs::File, release: impl FnOnce(i32) -> std::io::Result<()>) -> Result<(), FsFailure> {
    // Transfer ownership out of File before releasing. Neither an error nor the
    // Resource destructor may drop/retry the same descriptor after this call.
    release(file.into_raw_fd()).map_err(|error| FsFailure::io("close", error))
}
#[cfg(unix)]
fn close_fd(descriptor: i32) -> std::io::Result<()> {
    nix::unistd::close(descriptor).map_err(std::io::Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Seek};

    struct Directory(std::path::PathBuf);
    impl Directory {
        fn new() -> Self {
            static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("tsrust-file-backend-{}-{id}", std::process::id()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn input(&self) -> String {
            let path = self.0.join("input.bin");
            std::fs::write(&path, [0, 65, 255, 66]).unwrap();
            path.to_str().unwrap().into()
        }
    }
    impl Drop for Directory {
        fn drop(&mut self) { std::fs::remove_dir_all(&self.0).unwrap(); }
    }
    fn failure<T>(result: Result<T, FsFailure>) -> FsFailure {
        match result { Ok(_) => panic!("expected error"), Err(error) => error }
    }

    #[test]
    fn create_truncates_and_writes_preserve_bytes_and_close_ownership() {
        let directory=Directory::new();let path=directory.input();
        let mut file=create(&path).unwrap();
        assert!(std::fs::read(&path).unwrap().is_empty());
        assert_eq!(write(&mut file,&[0,65,255]).unwrap(),3);
        write_all(&mut file,&[66,0]).unwrap();
        assert_eq!(write(&mut file,&[]).unwrap(),0);
        file.close().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(),[0,65,255,66,0]);
        let mut readonly=open(&path).unwrap();
        let error=failure(write(&mut readonly,&[9]));
        assert_eq!(error.operation,"write");assert!(error.os_code.is_some());
        readonly.close().unwrap();
        assert_eq!(std::fs::read(&path).unwrap(),[0,65,255,66,0]);
    }

    #[test]
    fn full_file_helpers_handle_empty_exact_limit_errors_and_truncation() {
        let directory=Directory::new();let path=directory.input();
        assert_eq!(read_file(&path,Some(4)).unwrap(),[0,65,255,66]);
        assert_eq!(failure(read_file(&path,Some(3))).code,"length-range");
        assert_eq!(failure(read_file("missing-file",Some(-1))).code,"length-range");
        assert_eq!(failure(read_file("missing-file",Some(4))).operation,"readFile");
        write_file(&path,&[255,0]).unwrap();assert_eq!(std::fs::read(&path).unwrap(),[255,0]);
        write_file(&path,&[]).unwrap();assert_eq!(read_file(&path,Some(0)).unwrap(),Vec::<u8>::new());
        assert_eq!(failure(write_file(directory.0.to_str().unwrap(),&[1])).operation,"writeFile");
        std::fs::write(&path,[7,8]).unwrap();
        assert_eq!(read_file(&path,None).unwrap(),[7,8]);
    }
    struct ShortReader {bytes:std::io::Cursor<Vec<u8>>,interrupted:bool,fail:bool}
    impl Read for ShortReader {
        fn read(&mut self,buffer:&mut [u8])->std::io::Result<usize> {
            if !self.interrupted {self.interrupted=true;return Err(std::io::ErrorKind::Interrupted.into());}
            if self.fail && self.bytes.position()>0 {return Err(std::io::ErrorKind::PermissionDenied.into());}
            let length=buffer.len().min(2);self.bytes.read(&mut buffer[..length])
        }
    }
    #[test]
    fn full_file_reads_retry_interruption_accumulate_short_reads_and_stop_on_error() {
        for fail in [false,true] {
            let mut input=ShortReader {bytes:std::io::Cursor::new(vec![1,2,3,4,5]),interrupted:false,fail};
            let result=read_file_with(&mut input,Some(5));
            if fail {let error=failure(result);assert_eq!((error.code,error.operation),("permission-denied","readFile"));}
            else {assert_eq!(result.unwrap(),[1,2,3,4,5]);}
        }
        let mut input=std::io::Cursor::new(vec![1,2,3,4]);
        assert_eq!(failure(read_file_with(&mut input,Some(2))).code,"length-range");
        assert_eq!(input.position(),3,"read at most one byte beyond the limit");
    }

    struct ShortWriter {bytes:Vec<u8>,calls:usize,interrupt:bool,fail_after:Option<usize>,zero:bool}
    impl std::io::Write for ShortWriter {
        fn write(&mut self,bytes:&[u8])->std::io::Result<usize> {
            self.calls+=1;
            if self.interrupt {self.interrupt=false;return Err(std::io::ErrorKind::Interrupted.into());}
            if self.fail_after.is_some_and(|limit|self.bytes.len()>=limit) {
                return Err(std::io::ErrorKind::PermissionDenied.into());
            }
            let count=if self.zero {0} else {bytes.len().min(2)};
            self.bytes.extend_from_slice(&bytes[..count]);Ok(count)
        }
        fn flush(&mut self)->std::io::Result<()> {Ok(())}
    }
    fn writer()->ShortWriter {ShortWriter {bytes:vec![],calls:0,interrupt:false,fail_after:None,zero:false}}

    #[test]
    fn write_limits_and_create_errors_remain_structured() {
        assert!(validate_write_length(0,"write").is_ok());
        assert!(validate_write_length(64*1024*1024+1,"writeAll").is_ok());
        for length in [usize::MAX].into_iter().filter(|length|i64::try_from(*length).is_err()) {
            let error=failure(validate_write_length(length,"writeAll"));
            assert_eq!((error.code,error.operation,error.os_code),("length-range","writeAll",None));
        }
        let directory=Directory::new();
        let error=failure(create(directory.0.join("missing/child").to_str().unwrap()));
        assert_eq!((error.code,error.operation),("not-found","create"));assert!(error.os_code.is_some());
        let error=failure(create("bad\0path"));
        assert_eq!((error.code,error.operation),("invalid-input","create"));
    }

    #[test]
    fn writes_retry_interruptions_and_preserve_partial_progress() {
        let mut output=writer();output.interrupt=true;
        assert_eq!(write_with(&mut output,b"abcde",false).unwrap(),2);
        assert_eq!(output.bytes,b"ab");assert_eq!(output.calls,2);
        let mut output=writer();output.interrupt=true;
        assert_eq!(write_with(&mut output,b"abcde",true).unwrap(),5);
        assert_eq!(output.bytes,b"abcde");assert_eq!(output.calls,4);
        let mut output=writer();output.fail_after=Some(2);
        let error=failure(write_with(&mut output,b"abcde",true));
        assert_eq!((error.operation,error.code),("writeAll","permission-denied"));
        assert_eq!(output.bytes,b"ab");assert_eq!(output.calls,2);
        let mut output=writer();output.zero=true;
        let error=failure(write_with(&mut output,b"x",true));
        assert_eq!((error.operation,error.code),("writeAll","write-zero"));
        assert_eq!(output.calls,1);
        let mut output=writer();
        assert_eq!(write_with(&mut output,b"",true).unwrap(),0);assert_eq!(output.calls,0);
    }

    #[test]
    fn bounded_reads_advance_cursor_and_validate_before_io() {
        let directory = Directory::new();
        let mut file = open(&directory.input()).unwrap();
        assert!(read(&mut file, 0).unwrap().is_empty());
        for length in [-1] {
            let error = failure(read(&mut file, length));
            assert_eq!((error.code, error.operation, error.os_code), ("length-range", "read", None));
            assert_eq!(file.get_mut().0.stream_position().unwrap(), 0);
        }
        assert_eq!(read(&mut file, 2).unwrap(), [0, 65]);
        assert_eq!(read(&mut file, 9).unwrap(), [255, 66]);
        assert!(read(&mut file, 1).unwrap().is_empty());
        file.close().unwrap();
    }

    #[test]
    fn errors_preserve_operation_and_os_code() {
        let directory = Directory::new();
        let missing = directory.0.join("missing");
        let error = failure(open(missing.to_str().unwrap()));
        assert_eq!((error.code, error.operation), ("not-found", "open"));
        assert!(error.os_code.is_some());
        assert!(!error.message.is_empty());
        let error = failure(open("bad\0path"));
        assert_eq!((error.code, error.operation), ("invalid-input", "open"));
        match open(directory.0.to_str().unwrap()) {
            Ok(mut file)=>{let error=failure(read(&mut file,1));assert_eq!(error.operation,"read");assert!(error.os_code.is_some());file.close().unwrap();},
            Err(error)=>{assert_eq!(error.operation,"open");assert!(error.os_code.is_some());},
        }
    }

    #[test]
    fn whole_file_completion_closes_once_and_preserves_the_first_io_error() {
        struct FailedClose<'a>(&'a std::cell::Cell<usize>);
        impl Backend for FailedClose<'_> {
            type Error=FsFailure;
            fn close(self)->Result<(),FsFailure> {
                self.0.set(self.0.get()+1);
                Err(FsFailure::io("close",std::io::ErrorKind::Interrupted.into()))
            }
        }
        for read_failed in [false,true] {
            let closed=std::cell::Cell::new(0);
            let result=if read_failed {Err(FsFailure::read_error("io","input failed"))} else {Ok(())};
            let error=failure(finish_file(Resource::new(FailedClose(&closed)),result,"readFile"));
            assert_eq!(closed.get(),1);assert_eq!(error.code,if read_failed {"io"} else {"interrupted"});
            assert_eq!(error.operation,if read_failed {"read"} else {"readFile"});
        }
    }

    #[test]
    fn file_owners_release_before_rename_and_remove() {
        let directory=Directory::new();let original=directory.input();
        let moved=directory.0.join("moved.bin");
        for explicit in [false,true] {
            let file=open(&original).unwrap();
            if explicit {file.close().unwrap();} else {drop(file);}
            std::fs::rename(&original,&moved).unwrap();
            std::fs::rename(&moved,&original).unwrap();
        }
        std::fs::remove_file(original).unwrap();
    }

    #[cfg(windows)]
    #[test]
    fn windows_close_and_scope_drop_release_exclusive_open_conflicts() {
        use std::os::windows::fs::OpenOptionsExt;
        let directory=Directory::new();let path=directory.input();
        for explicit in [false,true] {
            let file=open(&path).unwrap();
            assert!(std::fs::OpenOptions::new().read(true).share_mode(0).open(&path).is_err());
            if explicit {file.close().unwrap();} else {drop(file);}
            let exclusive=std::fs::OpenOptions::new().read(true).share_mode(0).open(&path).unwrap();
            drop(exclusive);
        }
    }

    #[test]
    fn whole_file_reads_keep_optional_bound_and_unbounded_short_read_errors() {
        for limit in [None,Some(5)] {
            let mut input=ShortReader {bytes:std::io::Cursor::new(vec![1,2,3,4,5]),interrupted:false,fail:false};
            assert_eq!(read_file_with(&mut input,limit).unwrap(),[1,2,3,4,5]);
            let mut failed=ShortReader {bytes:std::io::Cursor::new(vec![1,2,3,4,5]),interrupted:false,fail:true};
            assert_eq!(failure(read_file_with(&mut failed,limit)).code,"permission-denied");
        }
        assert_eq!(failure(read_file_with(&mut std::io::Cursor::new(vec![1]),Some(0))).code,"length-range");
        assert!(read_file_with(&mut std::io::empty(),Some(0)).unwrap().is_empty());
        assert_eq!(validate_read_length(64*1024*1024+1,"read").unwrap(),64*1024*1024+1);
    }

    #[cfg(unix)]
    #[test]
    fn explicit_and_fallback_close_release_real_descriptors_without_retry() {
        let directory = Directory::new();
        let path = directory.input();
        for explicit in [false, true] {
            let owner = open(&path).unwrap();
            if explicit { owner.close().unwrap(); } else { drop(owner); }
        }
        let file = std::fs::File::open(&path).unwrap();
        let mut calls = 0;
        let error = failure(close_file_with(file, |descriptor| {
            calls += 1;
            close_fd(descriptor).unwrap();
            Err(std::io::Error::from(std::io::ErrorKind::Interrupted))
        }));
        assert_eq!(calls, 1);
        assert_eq!((error.code, error.operation), ("interrupted", "close"));
        let mut replacement = std::fs::File::open(&path).unwrap();
        let mut bytes = Vec::new();
        replacement.read_to_end(&mut bytes).unwrap();
        assert_eq!(bytes, [0, 65, 255, 66]);
    }
}
