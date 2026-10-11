//! Bounded synchronous file operations and consuming native close.

use super::resource::{Backend, Resource};
use std::io::Read;
use std::os::fd::IntoRawFd;

#[cfg(not(any(target_os = "macos", target_os = "linux")))]
compile_error!("std:fs consuming close is currently supported on macOS and Linux");

pub const MAX_READ_BYTES: usize = 64 * 1024 * 1024;

#[derive(Debug)]
pub struct FsFailure {
    pub code: &'static str,
    pub operation: &'static str,
    pub message: String,
    pub os_code: Option<i64>,
}
impl FsFailure {
    fn io(operation: &'static str, error: std::io::Error) -> Self {
        use std::io::ErrorKind;
        let code = match error.kind() {
            ErrorKind::NotFound => "not-found",
            ErrorKind::PermissionDenied => "permission-denied",
            ErrorKind::InvalidInput => "invalid-input",
            ErrorKind::Interrupted => "interrupted",
            ErrorKind::IsADirectory => "is-directory",
            ErrorKind::WouldBlock => "would-block",
            _ => "io",
        };
        Self { code, operation, message: error.to_string(), os_code: error.raw_os_error().map(i64::from) }
    }
    fn read_error(code: &'static str, message: &str) -> Self {
        Self { code, operation: "read", message: message.into(), os_code: None }
    }
}

pub struct FileBackend(std::fs::File);
pub type FileHandle = Resource<FileBackend>;
impl Backend for FileBackend {
    type Error = FsFailure;
    fn close(self) -> Result<(), FsFailure> { close_file_with(self.0, close_fd) }
}

pub fn open(path: &str) -> Result<FileHandle, FsFailure> {
    std::fs::File::open(path).map(|file| Resource::new(FileBackend(file))).map_err(|error| FsFailure::io("open", error))
}

/// Read at most length bytes once, retrying only interrupted reads. Short reads and
/// EOF are successful. Validation and reservation precede any cursor change.
pub fn read(file: &mut FileHandle, length: i64) -> Result<Vec<u8>, FsFailure> {
    let length = usize::try_from(length).ok().filter(|length| *length <= MAX_READ_BYTES)
        .ok_or_else(|| FsFailure::read_error("length-range", "Read length must be between 0 and 67108864 bytes."))?;
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

fn close_file_with(file: std::fs::File, release: impl FnOnce(i32) -> std::io::Result<()>) -> Result<(), FsFailure> {
    // Transfer ownership out of File before releasing. Neither an error nor the
    // Resource destructor may drop/retry the same descriptor after this call.
    release(file.into_raw_fd()).map_err(|error| FsFailure::io("close", error))
}
fn close_fd(descriptor: i32) -> std::io::Result<()> {
    unsafe extern "C" { #[link_name = "close"] fn native_close(descriptor: i32) -> i32; }
    // SAFETY: close accepts an integer descriptor. Production callers transfer a
    // uniquely owned File and invoke this exactly once. On macOS/Linux the handle
    // is released before a backend close error; retry could close a reused handle.
    if unsafe { native_close(descriptor) } == 0 { Ok(()) } else { Err(std::io::Error::last_os_error()) }
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
    fn bounded_reads_advance_cursor_and_validate_before_io() {
        let directory = Directory::new();
        let mut file = open(&directory.input()).unwrap();
        assert!(read(&mut file, 0).unwrap().is_empty());
        for length in [-1, MAX_READ_BYTES as i64 + 1, i64::MAX] {
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
        let mut directory_handle = open(directory.0.to_str().unwrap()).unwrap();
        let error = failure(read(&mut directory_handle, 1));
        assert_eq!(error.operation, "read");
        assert!(error.os_code.is_some());
        directory_handle.close().unwrap();
    }

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
