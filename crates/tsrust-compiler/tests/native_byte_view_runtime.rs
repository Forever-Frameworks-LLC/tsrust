mod support;

fn backend() -> String {
    format!(
        "#![allow(dead_code)]\nmod bytes {{\n{}\n{}\n{}\n}}\n",
        include_str!("../src/stdlib/bytes/runtime.rs").replace("//!", "//"),
        include_str!("../src/stdlib/bytes/views_runtime.rs"),
        include_str!("../src/stdlib/bytes/utf8_runtime.rs")
    )
}

#[test]
fn native_slice_helpers_do_not_allocate_and_preserve_storage_at_both_optimization_levels() {
    let directory = support::TempDirectory::new();
    let source = format!(
        "{}{}",
        backend(),
        r#"
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};
struct Observer;
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
unsafe impl GlobalAlloc for Observer {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(pointer, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: Observer = Observer;
fn forward(source: &[u8]) -> Result<&[u8], bytes::ByteFailure> { bytes::view(source, 1, 3) }
fn main() {
    let mut owner = vec![0, 65, 66, 255];
    let pointer = owner.as_ptr();
    let capacity = owner.capacity();
    let before = ALLOCATIONS.load(Ordering::Relaxed);
    for _ in 0..32 {
        let part = forward(&owner).unwrap_or_else(|_| panic!("valid view"));
        assert_eq!(part.as_ptr(), owner[1..].as_ptr());
        let nested = bytes::view(part, 1, 2).unwrap_or_else(|_| panic!("valid reborrow"));
        assert_eq!(nested.as_ptr(), owner[2..].as_ptr());
        assert!(bytes::view(&owner, 4, 4).is_ok_and(|part| part.is_empty()));
        {
            let part = bytes::view_mut(&mut owner, 1, 3).unwrap_or_else(|_| panic!("valid mutable view"));
            let nested = bytes::view_mut(part, 1, 2).unwrap_or_else(|_| panic!("valid mutable reborrow"));
            assert!(bytes::set(nested, 0, 7).is_ok());
            assert!(bytes::set(part, 0, 8).is_ok());
        }
        assert_eq!(owner, [0, 8, 7, 255]);
        for (start, end, offset) in [(-1, 99, -1), (5, 0, 5), (0, 5, 5), (2, 1, 1)] {
            let error = bytes::view_mut(&mut owner, start, end).err().expect("invalid range");
            assert_eq!((error.code, error.offset), ("range", Some(offset)));
        }
    }
    assert_eq!(ALLOCATIONS.load(Ordering::Relaxed), before);
    assert_eq!(owner.as_ptr(), pointer);
    assert_eq!(owner.capacity(), capacity);
    let unicode = [0, 195, 169, 240, 159, 152, 128, 255];
    let text = bytes::view(&unicode, 1, 7).unwrap_or_else(|_| panic!("valid UTF-8 range"));
    assert_eq!(bytes::to_utf8(text).ok().as_deref(), Some("é😀"));
    let truncated = bytes::view(&unicode, 1, 6).unwrap_or_else(|_| panic!("valid byte range"));
    let error = bytes::to_utf8(truncated).err().expect("truncated UTF-8");
    assert_eq!((error.code, error.offset), ("utf8", Some(2)));
}
"#
    );
    let path = directory.0.join("views.rs");
    std::fs::write(&path, source).unwrap();
    for level in [0, 3] {
        let binary = directory.0.join(format!("views-{level}"));
        support::success(&support::run(
            support::rustc()
                .arg("--edition=2024")
                .arg(format!("-Copt-level={level}"))
                .arg(&path)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        support::success(&support::run(&mut std::process::Command::new(&binary), 30));
    }
}
