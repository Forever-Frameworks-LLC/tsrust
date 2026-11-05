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


#[test]
fn decimal_digit_text_reuses_ascii_storage_and_converts_to_owned_native_strings() {
 let directory=support::TempDirectory::new();let path=directory.0.join("digits.rs");
 let source=format!("{}\n{}",runtime::STRINGS,r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering};
struct Count;static ALLOCATIONS:AtomicUsize=AtomicUsize::new(0);
unsafe impl GlobalAlloc for Count {
 unsafe fn alloc(&self,layout:Layout)->*mut u8 {ALLOCATIONS.fetch_add(1,Ordering::Relaxed);unsafe{System.alloc(layout)}}
 unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe{System.dealloc(p,layout)}}
}
#[global_allocator]static ALLOCATOR:Count=Count;
fn main(){
 for value in 0..=9 {
  let before=ALLOCATIONS.load(Ordering::Relaxed);
  let text=tsr_int_text(std::hint::black_box(value));assert!(matches!(text,String::Borrowed(_)));
  assert_eq!(text.as_bytes(),&[b'0'+value as u8]);assert_eq!(ALLOCATIONS.load(Ordering::Relaxed),before);
  let mut copy=text.clone();copy.to_mut().push('x');assert_eq!(text.len(),1);assert_eq!(copy.len(),2);
  let native:std::string::String=text.into_owned();assert_eq!(native.as_bytes(),&[b'0'+value as u8]);
 }
 for value in [i64::MIN,-10,-1,10,11,i64::MAX] {
  let text=tsr_int_text(value);assert!(matches!(text,String::Owned(_)));assert_eq!(text,value.to_string());
 }
}
"#);
 std::fs::write(&path,source).unwrap();
 for level in ["0","3"] {
  let binary=directory.0.join(format!("digits-{level}"));
  support::success(&support::run(support::rustc().arg("--edition=2024").arg("-Awarnings").arg("-C").arg(format!("opt-level={level}")).arg(&path).arg("-o").arg(&binary),30));
  support::success(&support::run(&mut std::process::Command::new(binary),10));
 }
}
