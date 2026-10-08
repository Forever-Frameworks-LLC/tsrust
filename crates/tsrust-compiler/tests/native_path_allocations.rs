use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
thread_local! {
    static TRACK: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
struct Counting;
fn count() {
    if TRACK.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|counter| counter.set(counter.get() + 1));
    }
}
unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        count();
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        count();
        unsafe { System.realloc(pointer, layout, size) }
    }
}
#[global_allocator]
static ALLOCATOR: Counting = Counting;
fn observe(operation: impl FnOnce() -> String) -> (String, usize) {
    ALLOCATIONS.set(0);
    TRACK.set(true);
    let result = operation();
    TRACK.set(false);
    (result, ALLOCATIONS.get())
}
#[test]
fn extension_allocates_only_owned_result() {
    let (result, allocations) = observe(|| tsrust_compiler::native_path::extname("/tmp/report.txt/", false));
    assert_eq!(result, ".txt");
    assert_eq!(allocations, 1, "no temporary basename allocation");
}
#[test]
fn ordinary_normalization_allocates_only_owned_result() {
    let (result, allocations) = observe(|| tsrust_compiler::native_path::normalize("/tmp//reports/../report.txt", false));
    assert_eq!(result, "/tmp/report.txt");
    assert_eq!(allocations, 1, "no temporary component vector or copied tail");
}
