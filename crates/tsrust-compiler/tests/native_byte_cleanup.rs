mod support;

#[test]
fn owned_bytes_release_exactly_once_and_restore_allocations_at_o0_o3() {
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip(include_str!("fixtures/native-owned-bytes/cleanup.tsr")));
    let emitted = support::run(support::cli().arg("emit").arg(path), 60);
    support::success(&emitted);
    let rust = String::from_utf8(emitted.stdout).unwrap();
    assert_eq!(rust.matches("fn main()").count(), 1);
    let entry = rust.split("fn main()").nth(1).unwrap();
    let call = entry.split("tsr_entry_result(").nth(1).expect("fallible cleanup entry");
    let callee = call.split('(').next().unwrap();
    assert!(callee.starts_with("tsr_result_function"));
    let call = format!("assert!({callee}().is_ok());");
    let rust = rust.replace("fn main()", "fn source_main()") + r#"
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
static LIVE: AtomicIsize = AtomicIsize::new(0);
static CREATED: AtomicUsize = AtomicUsize::new(0);
static BYTE_DROPS: AtomicUsize = AtomicUsize::new(0);
struct AllocationObserver;
unsafe impl GlobalAlloc for AllocationObserver {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            LIVE.fetch_add(layout.size() as isize, Ordering::SeqCst);
            CREATED.fetch_add(1, Ordering::SeqCst);
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        LIVE.fetch_sub(layout.size() as isize, Ordering::SeqCst);
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let pointer = unsafe { System.realloc(pointer, layout, size) };
        if !pointer.is_null() {
            LIVE.fetch_add(size as isize - layout.size() as isize, Ordering::SeqCst);
            CREATED.fetch_add(1, Ordering::SeqCst);
        }
        pointer
    }
}
#[global_allocator] static ALLOCATOR: AllocationObserver = AllocationObserver;
fn main() {
    assert_eq!(source_main(), std::process::ExitCode::SUCCESS);
    let baseline = LIVE.load(Ordering::SeqCst);
    for _ in 0..32 {
        let created = CREATED.load(Ordering::SeqCst);
        let drops = BYTE_DROPS.load(Ordering::SeqCst);
        SOURCE_BODY_CALL
        assert_eq!(BYTE_DROPS.load(Ordering::SeqCst) - drops, 16, "every byte owner must drop once");
        assert!(CREATED.load(Ordering::SeqCst) > created, "cleanup witness must allocate owned payloads");
        assert_eq!(LIVE.load(Ordering::SeqCst), baseline, "Byte owner, aggregate or callback environment leaked");
    }
}
"#;
    let bytes_record = rust.lines().find_map(|line| line.strip_prefix("#[derive(Clone)] struct ").and_then(|tail| tail.strip_suffix(" { bytes: Vec<u8> }"))).expect("opaque byte record");
    let drop = format!("\nimpl Drop for {bytes_record} {{ fn drop(&mut self) {{ BYTE_DROPS.fetch_add(1, Ordering::SeqCst); }} }}\n");
    let rust = rust.replace("SOURCE_BODY_CALL", &call) + &drop;
    let path = directory.0.join("byte-cleanup.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("byte-cleanup-{level}"));
        support::success(&support::run(support::rustc().arg("--edition=2024").arg(format!("-Copt-level={level}")).arg(&path).arg("-o").arg(&binary), 60));
        support::success(&support::run(&mut std::process::Command::new(binary), 20));
    }
}

