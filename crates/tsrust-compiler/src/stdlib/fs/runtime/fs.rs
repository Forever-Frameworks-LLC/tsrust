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
    pub fn io(operation: &'static str, error: std::io::Error) -> Self {
        use std::io::ErrorKind;
        let code = match error.kind() {
            ErrorKind::NotFound => "not-found",
            ErrorKind::AlreadyExists => "already-exists",
            ErrorKind::NotADirectory => "not-directory",
            ErrorKind::DirectoryNotEmpty => "directory-not-empty",
            ErrorKind::CrossesDevices => "cross-device",
            ErrorKind::Unsupported => "unsupported",
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

/// File-open switches retain std defaults: all switches start false.
#[derive(Default)]
pub struct OpenOptions {
    pub read: bool, pub write: bool, pub append: bool,
    pub create: bool, pub create_new: bool, pub truncate: bool,
}
pub fn open_file(path:&str,options:&OpenOptions)->Result<FileHandle,FsFailure> {
    std::fs::OpenOptions::new().read(options.read).write(options.write).append(options.append)
        .create(options.create).create_new(options.create_new).truncate(options.truncate)
        .open(path).map(|file|Resource::new(FileBackend(file))).map_err(|error|FsFailure::io("openFile",error))
}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct FileTime { pub unix_seconds:i64, pub nanoseconds:i64 }
#[derive(Clone,Debug)]
pub struct Metadata {
    pub is_file:bool, pub is_directory:bool, pub is_symlink:bool,
    pub size:i64, pub read_only:bool,
    pub modified:Option<FileTime>, pub accessed:Option<FileTime>, pub created:Option<FileTime>,
}
#[derive(Clone,Debug)]
pub struct DirEntry { pub name:String, pub path:String, pub is_file:bool, pub is_directory:bool, pub is_symlink:bool }
fn failure(code:&'static str,operation:&'static str,message:&str)->FsFailure {
    FsFailure {code,operation,message:message.into(),os_code:None}
}
fn integer(value:u64,operation:&'static str)->Result<i64,FsFailure> {
    i64::try_from(value).map_err(|_|failure("integer-range",operation,"Filesystem value does not fit int."))
}
fn file_time(time:std::time::SystemTime,operation:&'static str)->Result<FileTime,FsFailure> {
    let (negative,duration)=match time.duration_since(std::time::UNIX_EPOCH) {
        Ok(duration)=>(false,duration),Err(error)=>(true,error.duration()),
    };
    let nanos=i128::from(duration.as_secs())*1_000_000_000+i128::from(duration.subsec_nanos());
    let nanos=if negative {-nanos} else {nanos};
    let unix_seconds=i64::try_from(nanos.div_euclid(1_000_000_000))
        .map_err(|_|failure("integer-range",operation,"Filesystem timestamp seconds do not fit int."))?;
    Ok(FileTime {unix_seconds,nanoseconds:nanos.rem_euclid(1_000_000_000) as i64})
}
fn optional_time(time:std::io::Result<std::time::SystemTime>,operation:&'static str)->Result<Option<FileTime>,FsFailure> {
    match time {Ok(time)=>file_time(time,operation).map(Some),Err(error) if error.kind()==std::io::ErrorKind::Unsupported=>Ok(None),Err(error)=>Err(FsFailure::io(operation,error))}
}
fn metadata(value:std::fs::Metadata,operation:&'static str)->Result<Metadata,FsFailure> {
    Ok(Metadata {is_file:value.is_file(),is_directory:value.is_dir(),is_symlink:value.is_symlink(),
        size:integer(value.len(),operation)?,read_only:value.permissions().readonly(),
        modified:optional_time(value.modified(),operation)?,accessed:optional_time(value.accessed(),operation)?,created:optional_time(value.created(),operation)?})
}
pub fn stat(path:&str)->Result<Metadata,FsFailure> {
    metadata(std::fs::metadata(path).map_err(|error|FsFailure::io("stat",error))?,"stat")
}
pub fn lstat(path:&str)->Result<Metadata,FsFailure> {
    metadata(std::fs::symlink_metadata(path).map_err(|error|FsFailure::io("lstat",error))?,"lstat")
}
pub fn file_stat(file:&FileHandle)->Result<Metadata,FsFailure> {
    metadata(file.get().0.metadata().map_err(|error|FsFailure::io("fileStat",error))?,"fileStat")
}
fn unicode(path:std::ffi::OsString,operation:&'static str)->Result<String,FsFailure> {
    path.into_string().map_err(|_|failure("not-unicode",operation,"Filesystem path is not valid Unicode."))
}
pub fn read_dir(path:&str,max_entries:Option<i64>)->Result<Vec<DirEntry>,FsFailure> {
    let limit=max_entries.map(|limit|usize::try_from(limit).map_err(|_|failure("length-range","readDir","Entry limit must be nonnegative and fit native storage."))).transpose()?;
    let mut entries=Vec::new();
    for entry in std::fs::read_dir(path).map_err(|error|FsFailure::io("readDir",error))? {
        let entry=entry.map_err(|error|FsFailure::io("readDir",error))?;
        if limit.is_some_and(|limit|entries.len()>=limit) {return Err(failure("length-range","readDir","Directory exceeds the requested entry limit."));}
        let kind=entry.file_type().map_err(|error|FsFailure::io("readDir",error))?;
        entries.push(DirEntry {name:unicode(entry.file_name(),"readDir")?,path:unicode(entry.path().into_os_string(),"readDir")?,is_file:kind.is_file(),is_directory:kind.is_dir(),is_symlink:kind.is_symlink()});
    }
    Ok(entries)
}
pub fn mkdir(path:&str)->Result<(),FsFailure> {std::fs::create_dir(path).map_err(|error|FsFailure::io("mkdir",error))}
pub fn mkdir_all(path:&str)->Result<(),FsFailure> {std::fs::create_dir_all(path).map_err(|error|FsFailure::io("mkdirAll",error))}
pub fn rename(path:&str,destination:&str)->Result<(),FsFailure> {std::fs::rename(path,destination).map_err(|error|FsFailure::io("rename",error))}
pub fn copy_file(path:&str,destination:&str)->Result<i64,FsFailure> {
    integer(std::fs::copy(path,destination).map_err(|error|FsFailure::io("copyFile",error))?,"copyFile")
}
pub fn remove_file(path:&str)->Result<(),FsFailure> {std::fs::remove_file(path).map_err(|error|FsFailure::io("removeFile",error))}
pub fn remove_dir(path:&str)->Result<(),FsFailure> {std::fs::remove_dir(path).map_err(|error|FsFailure::io("removeDir",error))}
pub fn remove_dir_all(path:&str)->Result<(),FsFailure> {std::fs::remove_dir_all(path).map_err(|error|FsFailure::io("removeDirAll",error))}
pub fn canonicalize(path:&str)->Result<String,FsFailure> {
    unicode(std::fs::canonicalize(path).map_err(|error|FsFailure::io("canonicalize",error))?.into_os_string(),"canonicalize")
}
pub fn read_link(path:&str)->Result<String,FsFailure> {
    unicode(std::fs::read_link(path).map_err(|error|FsFailure::io("readLink",error))?.into_os_string(),"readLink")
}
pub fn hard_link(path:&str,destination:&str)->Result<(),FsFailure> {std::fs::hard_link(path,destination).map_err(|error|FsFailure::io("hardLink",error))}
pub fn append_file(path:&str,bytes:&[u8])->Result<(),FsFailure> {
    validate_write_length(bytes.len(),"appendFile")?;
    let mut file=open_file(path,&OpenOptions {append:true,create:true,..OpenOptions::default()}).map_err(|mut error|{error.operation="appendFile";error})?;
    let result=write_all(&mut file,bytes).map_err(|mut error|{error.operation="appendFile";error});
    finish_file(file,result,"appendFile")
}
pub fn read_text_file(path:&str,limit:Option<i64>)->Result<String,FsFailure> {
    let limit=limit.map(|length|validate_read_length(length,"readTextFile")).transpose()?;
    let mut file=open(path).map_err(|mut error|{error.operation="readTextFile";error})?;
    let result=read_file_with(&mut file.get_mut().0,limit).map_err(|mut error|{error.operation="readTextFile";error})
        .and_then(|bytes|String::from_utf8(bytes).map_err(|error|failure("invalid-utf8","readTextFile",&error.to_string())));
    finish_file(file,result,"readTextFile")
}
pub fn write_text_file(path:&str,text:&str)->Result<(),FsFailure> {
    write_file(path,text.as_bytes()).map_err(|mut error|{error.operation="writeTextFile";error})
}
pub fn truncate(file:&mut FileHandle,length:i64)->Result<(),FsFailure> {
    let length=u64::try_from(length).map_err(|_|failure("length-range","truncate","File length must be nonnegative."))?;
    file.get_mut().0.set_len(length).map_err(|error|FsFailure::io("truncate",error))
}
pub fn sync_all(file:&FileHandle)->Result<(),FsFailure> {file.get().0.sync_all().map_err(|error|FsFailure::io("syncAll",error))}
pub fn sync_data(file:&FileHandle)->Result<(),FsFailure> {file.get().0.sync_data().map_err(|error|FsFailure::io("syncData",error))}

pub fn seek(file:&mut FileHandle,offset:i64,origin:&str)->Result<i64,FsFailure> {
    use std::io::{Seek,SeekFrom};
    let origin=match origin {
        "start"=>SeekFrom::Start(u64::try_from(offset).map_err(|_|failure("offset-range","seek","Start offset must be nonnegative."))?),
        "current"=>SeekFrom::Current(offset),"end"=>SeekFrom::End(offset),
        _=>return Err(failure("invalid-input","seek","Origin must be start, current or end.")),
    };
    integer(file.get_mut().0.seek(origin).map_err(|error|FsFailure::io("seek",error))?,"seek")
}
pub fn position(file:&mut FileHandle)->Result<i64,FsFailure> {
    use std::io::Seek;
    integer(file.get_mut().0.stream_position().map_err(|error|FsFailure::io("position",error))?,"position")
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
    #[test]
    fn options_preserve_default_validation_create_new_and_handle_borrows() {
        let directory=Directory::new();let path=directory.input();
        assert_eq!(failure(open_file(&path,&OpenOptions::default())).code,"invalid-input");
        assert_eq!(failure(open_file(&path,&OpenOptions {truncate:true,..OpenOptions::default()})).code,"invalid-input");
        assert_eq!(read_file(&path,None).unwrap(),[0,65,255,66]);
        assert_eq!(failure(open_file(&path,&OpenOptions {write:true,create_new:true,truncate:true,..OpenOptions::default()})).code,"already-exists");
        let mut file=open_file(&path,&OpenOptions {read:true,write:true,..OpenOptions::default()}).unwrap();
        assert_eq!(file_stat(&file).unwrap().size,4);assert_eq!(read(&mut file,2).unwrap(),[0,65]);
        assert_eq!(failure(truncate(&mut file,-1)).code,"length-range");truncate(&mut file,1).unwrap();
        assert_eq!(file.get_mut().0.stream_position().unwrap(),2,"truncate leaves cursor unchanged");
        write_all(&mut file,&[9]).unwrap();sync_all(&file).unwrap();sync_data(&file).unwrap();file.close().unwrap();
        assert_eq!(read_file(&path,None).unwrap(),[0,0,9]);
    }
    #[test]
    fn metadata_and_directory_snapshots_are_owned_and_bounds_are_opt_in() {
        let directory=Directory::new();let path=directory.input();let root=directory.0.to_str().unwrap();
        std::fs::create_dir(directory.0.join("nested")).unwrap();std::fs::write(directory.0.join("é-🦀"),b"x").unwrap();
        let value=stat(&path).unwrap();assert!(value.is_file);assert!(!value.is_directory);assert!(!value.is_symlink);assert_eq!(value.size,4);
        assert_eq!(value.read_only,std::fs::metadata(&path).unwrap().permissions().readonly());
        for time in [&value.modified,&value.accessed,&value.created].into_iter().flatten() {assert!((0..1_000_000_000).contains(&time.nanoseconds));}
        assert_eq!(failure(read_dir("missing",Some(-1))).code,"length-range");
        assert_eq!(failure(read_dir(root,Some(2))).code,"length-range");assert_eq!(failure(read_dir(root,Some(0))).code,"length-range");
        assert_eq!(read_dir(root,Some(3)).unwrap().len(),3);
        let entries=read_dir(root,None).unwrap();remove_file(&path).unwrap();
        let input=entries.iter().find(|entry|entry.name=="input.bin").unwrap();assert!(input.is_file);assert_eq!(input.path,path);
        assert!(entries.iter().find(|entry|entry.name=="nested").unwrap().is_directory);
        assert_eq!(entries.iter().filter(|entry|entry.name=="é-🦀").count(),1);
        let empty=directory.0.join("empty");mkdir(empty.to_str().unwrap()).unwrap();assert!(read_dir(empty.to_str().unwrap(),Some(0)).unwrap().is_empty());
    }
    #[test]
    fn filesystem_utilities_preserve_contents_and_host_error_categories() {
        let directory=Directory::new();let path=directory.input();let target=directory.0.join("target");let target=target.to_str().unwrap();
        assert_eq!(copy_file(&path,target).unwrap(),4);assert_eq!(read_file(target,None).unwrap(),[0,65,255,66]);
        let moved=directory.0.join("moved");let moved=moved.to_str().unwrap();rename(target,moved).unwrap();assert!(!std::path::Path::new(target).exists());
        hard_link(moved,target).unwrap();remove_file(moved).unwrap();assert_eq!(read_file(target,None).unwrap(),[0,65,255,66]);
        assert_eq!(canonicalize(target).unwrap(),std::fs::canonicalize(target).unwrap().to_str().unwrap());
        assert_eq!(failure(read_link(target)).operation,"readLink");remove_file(target).unwrap();
        let nested=directory.0.join("a/b");mkdir_all(nested.to_str().unwrap()).unwrap();mkdir_all(nested.to_str().unwrap()).unwrap();
        assert_eq!(failure(mkdir(nested.to_str().unwrap())).code,"already-exists");
        let parent=directory.0.join("a");assert_eq!(failure(remove_dir(parent.to_str().unwrap())).code,"directory-not-empty");
        remove_dir_all(parent.to_str().unwrap()).unwrap();assert!(!parent.exists());
    }
    #[test]
    fn text_is_strict_utf8_preserves_bom_and_counts_byte_limits() {
        let directory=Directory::new();let path=directory.input();
        write_text_file(&path,"\u{feff}hé🦀").unwrap();assert_eq!(read_text_file(&path,None).unwrap(),"\u{feff}hé🦀");
        assert_eq!(read_text_file(&path,Some(10)).unwrap(),"\u{feff}hé🦀");
        assert_eq!(failure(read_text_file(&path,Some(9))).code,"length-range");
        assert_eq!(failure(read_text_file("missing",Some(-1))).code,"length-range");
        append_file(&path,b"\n").unwrap();assert_eq!(read_text_file(&path,None).unwrap(),"\u{feff}hé🦀\n");
        write_file(&path,&[0xff]).unwrap();let error=failure(read_text_file(&path,None));assert_eq!((error.code,error.operation),("invalid-utf8","readTextFile"));
        let created=directory.0.join("append");append_file(created.to_str().unwrap(),&[]).unwrap();assert!(created.is_file());
    }
    #[test]
    fn timestamp_normalization_and_unsupported_fields_are_explicit() {
        use std::time::{UNIX_EPOCH,Duration};
        let epoch=file_time(UNIX_EPOCH,"stat").unwrap();assert_eq!((epoch.unix_seconds,epoch.nanoseconds),(0,0));
        assert_eq!(file_time(UNIX_EPOCH-Duration::from_nanos(1),"stat").unwrap(),FileTime {unix_seconds:-1,nanoseconds:999999999});
        assert_eq!(file_time(UNIX_EPOCH-Duration::new(1,500000000),"stat").unwrap(),FileTime {unix_seconds:-2,nanoseconds:500000000});
        assert_eq!(optional_time(Err(std::io::ErrorKind::Unsupported.into()),"stat").unwrap(),None);
        assert_eq!(failure(optional_time(Err(std::io::ErrorKind::PermissionDenied.into()),"stat")).code,"permission-denied");
        assert_eq!(failure(integer(u64::MAX,"stat")).code,"integer-range");
    }
    #[cfg(unix)]
    #[test]
    fn non_unicode_conversion_rejects_raw_bytes_without_loss() {
        use std::os::unix::ffi::OsStringExt;
        for operation in ["readDir","readLink","createTempDir"] {
            let error=failure(unicode(std::ffi::OsString::from_vec(vec![0xff]),operation));
            assert_eq!((error.code,error.operation,error.os_code),("not-unicode",operation,None));
        }
    }
    #[cfg(unix)]
    #[test]
    fn link_metadata_and_raw_symlink_targets_preserve_host_values() {
        use std::os::unix::{ffi::OsStringExt,fs::symlink};
        let directory=Directory::new();let input=directory.input();let link=directory.0.join("link");symlink("input.bin",&link).unwrap();
        assert!(lstat(link.to_str().unwrap()).unwrap().is_symlink);assert!(stat(link.to_str().unwrap()).unwrap().is_file);
        assert_eq!(read_link(link.to_str().unwrap()).unwrap(),"input.bin");
        let entry=read_dir(directory.0.to_str().unwrap(),None).unwrap().into_iter().find(|entry|entry.name=="link").unwrap();assert!(entry.is_symlink);assert!(!entry.is_file);
        std::fs::remove_file(&link).unwrap();let odd=std::ffi::OsString::from_vec(vec![0xff]);symlink(&odd,&link).unwrap();
        assert_eq!(std::fs::read_link(&link).unwrap(),std::path::PathBuf::from(odd));
        assert_eq!(failure(read_link(link.to_str().unwrap())).code,"not-unicode");
        remove_file(&input).unwrap();remove_file(link.to_str().unwrap()).unwrap();
    }
    // macOS rejects this filename at creation with EILSEQ; Linux admits raw names.
    #[cfg(target_os="linux")]
    #[test]
    fn raw_directory_entry_names_reject_non_unicode_without_loss() {
        use std::os::unix::ffi::OsStringExt;
        let directory=Directory::new();let odd=std::ffi::OsString::from_vec(vec![0xff]);let path=directory.0.join(odd);
        std::fs::write(&path,b"x").unwrap();assert_eq!(failure(read_dir(directory.0.to_str().unwrap(),None)).code,"not-unicode");
        assert_eq!(std::fs::read(path).unwrap(),b"x");
    }


    #[test]
    fn cursor_origins_eof_validation_and_truncation_use_exact_positions() {
        let directory=Directory::new();let path=directory.input();
        let mut file=open_file(&path,&OpenOptions {read:true,write:true,..OpenOptions::default()}).unwrap();
        assert_eq!(position(&mut file).unwrap(),0);assert_eq!(seek(&mut file,2,"start").unwrap(),2);
        assert_eq!(read(&mut file,1).unwrap(),[255]);assert_eq!(position(&mut file).unwrap(),3);
        assert_eq!(seek(&mut file,-2,"current").unwrap(),1);assert_eq!(read(&mut file,3).unwrap(),[65,255,66]);
        assert_eq!(seek(&mut file,-1,"end").unwrap(),3);assert_eq!(read(&mut file,1).unwrap(),[66]);
        assert_eq!(seek(&mut file,7,"start").unwrap(),7);assert!(read(&mut file,64).unwrap().is_empty());assert_eq!(position(&mut file).unwrap(),7);
        let error=failure(seek(&mut file,-1,"start"));assert_eq!((error.code,error.operation,error.os_code),("offset-range","seek",None));assert_eq!(position(&mut file).unwrap(),7);
        assert_eq!(failure(seek(&mut file,-10,"current")).operation,"seek");assert_eq!(position(&mut file).unwrap(),7);
        assert_eq!(failure(seek(&mut file,0,"wrong")).code,"invalid-input");truncate(&mut file,1).unwrap();assert_eq!(position(&mut file).unwrap(),7);
        file.close().unwrap();
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
        let closed=std::cell::Cell::new(0);
        let result=Err::<String,_>(super::failure("invalid-utf8","readTextFile","invalid text"));
        let error=failure(finish_file(Resource::new(FailedClose(&closed)),result,"readTextFile"));
        assert_eq!(closed.get(),1);assert_eq!((error.code,error.operation),("invalid-utf8","readTextFile"));
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
