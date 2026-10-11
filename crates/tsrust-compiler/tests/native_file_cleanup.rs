mod support;

#[test]
fn generated_files_release_once_in_lexical_order_at_o0_o3() {
    let directory = support::TempDirectory::new();
    for name in ["normal", "early", "replace-old", "replace-new", "result", "unused", "capture", "callback-local", "return", "finally", "partial-first", "partial-second", "fail-close", "close-error"] {
        std::fs::write(directory.0.join(format!("{name}.bin")), [1, 2]).unwrap();
    }
    let source = support::format_round_trip(include_str!("fixtures/native-files/cleanup.tsr"));
    let path = directory.source(&source);
    let emitted = support::run(support::cli().arg("emit").arg(path), 60);
    support::success(&emitted);
    let mut rust = String::from_utf8(emitted.stdout).unwrap();
    let original = "Resource::new(FileBackend(file))";
    assert_eq!(rust.matches(original).count(), 2); // open and create share resource ownership.
    rust = rust.replace(original, "{ crate::opened(path, std::os::fd::AsRawFd::as_raw_fd(&file)); Resource::new(FileBackend(file)) }");
    let original = "release(file.into_raw_fd()).map_err(|error| FsFailure::io(\"close\", error))";
    assert_eq!(rust.matches(original).count(), 1);
    rust = rust.replace(original, "{ let fd=file.into_raw_fd(); let fail=crate::released(fd); let result=release(fd); if fail && result.is_ok() { Err(FsFailure::io(\"close\",std::io::Error::from(std::io::ErrorKind::Interrupted))) } else { result.map_err(|error|FsFailure::io(\"close\",error)) } }");
    assert_eq!(rust.matches("fn main()").count(), 1);
    rust = rust.replace("fn main()", "fn source_main()");
    rust.push_str(r#"
static ACTIVE: std::sync::Mutex<Vec<(i32,std::string::String)>>=std::sync::Mutex::new(Vec::new());
static EVENTS: std::sync::Mutex<Vec<std::string::String>>=std::sync::Mutex::new(Vec::new());
static FAILED: std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
fn opened(path:&str,fd:i32){
    let mut active=ACTIVE.lock().unwrap();
    assert!(!active.iter().any(|entry|entry.0==fd),"live descriptor reused");
    active.push((fd,path.into()));
    EVENTS.lock().unwrap().push(format!("open {path}"));
}
fn released(fd:i32)->bool{
    let mut active=ACTIVE.lock().unwrap();
    let index=active.iter().position(|entry|entry.0==fd).expect("release must have one live owner");
    let (_,path)=active.remove(index);
    EVENTS.lock().unwrap().push(format!("close {path}"));
    if path=="fail-close.bin" {FAILED.fetch_add(1,std::sync::atomic::Ordering::SeqCst);true} else {false}
}
fn main(){
    for _ in 0..8 {
        assert_eq!(source_main(),std::process::ExitCode::SUCCESS);
        assert!(ACTIVE.lock().unwrap().is_empty(),"leaked file owners");
        let expected=[
            "open normal.bin","close normal.bin","open early.bin","close early.bin",
            "open replace-old.bin","open replace-new.bin","close replace-old.bin","close replace-new.bin",
            "open result.bin","close result.bin","open unused.bin","close unused.bin",
            "open capture.bin","open callback-local.bin","close capture.bin","close callback-local.bin",
            "open return.bin","open finally.bin","close finally.bin","close return.bin",
            "open return.bin","open finally.bin","close finally.bin","close return.bin",
            "open partial-first.bin","open partial-second.bin","close partial-second.bin","close partial-first.bin",
            "open partial-first.bin","open partial-second.bin","close partial-second.bin","close partial-first.bin",
            "open fail-close.bin","close fail-close.bin","open close-error.bin","close close-error.bin"
        ];
        assert_eq!(*EVENTS.lock().unwrap(),expected);
        EVENTS.lock().unwrap().clear();
    }
    assert_eq!(FAILED.load(std::sync::atomic::Ordering::SeqCst),8);
}
"#);
    let path = directory.0.join("cleanup.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("cleanup-{level}"));
        support::success(&support::run(support::rustc().arg("--edition=2024").arg(format!("-Copt-level={level}")).arg(&path).arg("-o").arg(&binary), 60));
        support::success(&support::run(std::process::Command::new(binary).current_dir(&directory.0), 20));
    }
}
