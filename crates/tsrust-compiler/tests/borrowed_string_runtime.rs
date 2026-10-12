#[allow(dead_code)]
#[path = "../src/runtime/mod.rs"]
mod runtime;
mod support;
#[test]
fn string_readers_accept_local_slices_without_allocating() {
 let directory=support::TempDirectory::new();let path=directory.0.join("borrowed.rs");
 let source=format!("{}\n{}",runtime::STRINGS,r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering};
struct Count;
static ALLOCATIONS:AtomicUsize=AtomicUsize::new(0);
unsafe impl GlobalAlloc for Count {
 unsafe fn alloc(&self,layout:Layout)->*mut u8 {ALLOCATIONS.fetch_add(1,Ordering::Relaxed);unsafe{System.alloc(layout)}}
 unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe{System.dealloc(p,layout)}}
}
#[global_allocator]static ALLOCATOR:Count=Count;
fn main(){
 let owner=std::string::String::from("_é🦀é_");let needle_owner=std::string::String::from("é");
 let text=&owner[1..owner.len()-1];let needle=needle_owner.as_str();let pointer=text.as_ptr();
 let before=ALLOCATIONS.load(Ordering::Relaxed);
 assert_eq!(tsr_length(text),8);
 assert!(tsr_string_starts_with(text,needle,None));
 assert!(tsr_string_ends_with(text,needle,None));
 assert!(tsr_string_includes(text,needle,Some(2)));
 assert_eq!(tsr_string_index_of(text,needle,Some(2)),6);
 assert_eq!(tsr_string_last_index_of(text,needle,Some(i64::MAX)),6);
 assert_eq!(tsr_string_last_index_of(text,needle,Some(i64::MIN)),0);
 assert!(tsr_string_starts_with(text,"",Some(i64::MAX)));
 assert_eq!(tsr_string_index_of(text,"missing",None),-1);
 assert_eq!(tsr_string_last_index_of(text,"",None),8);
 assert_eq!(ALLOCATIONS.load(Ordering::Relaxed),before);
 assert_eq!(pointer,text.as_ptr());assert_eq!(owner,"_é🦀é_");
 assert!(std::panic::catch_unwind(||tsr_string_includes(text,needle,Some(1))).is_err());
}
"#);
 std::fs::write(&path,source).unwrap();let binary=directory.0.join("borrowed");
 support::success(&support::run(support::rustc().arg("--edition=2024").arg("-Awarnings").arg(&path).arg("-o").arg(&binary),30));
 support::success(&support::run(&mut std::process::Command::new(binary),10));
}
