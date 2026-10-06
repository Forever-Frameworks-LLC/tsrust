// Instrumentation only: this wrapper is never emitted by the language compiler.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::Relaxed};
struct CountingAllocator;
static ALLOCS: AtomicUsize = AtomicUsize::new(0);
static REALLOCS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCS: AtomicUsize = AtomicUsize::new(0);
static REQUESTED: AtomicUsize = AtomicUsize::new(0);
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
fn grow(bytes: usize) {
    let current = LIVE.fetch_add(bytes, Relaxed) + bytes;
    PEAK.fetch_max(current, Relaxed);
}
// SAFETY: Every allocation is forwarded unchanged to System; counters never
// allocate, and bookkeeping does not alter layout, alignment, or pointers.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            ALLOCS.fetch_add(1, Relaxed);
            REQUESTED.fetch_add(layout.size(), Relaxed);
            grow(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        DEALLOCS.fetch_add(1, Relaxed);
        LIVE.fetch_sub(layout.size(), Relaxed);
        unsafe { System.dealloc(pointer, layout) };
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        let replacement = unsafe { System.realloc(pointer, layout, size) };
        if !replacement.is_null() {
            REALLOCS.fetch_add(1, Relaxed);
            REQUESTED.fetch_add(size, Relaxed);
            if size >= layout.size() {
                grow(size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - size, Relaxed);
            }
        }
        replacement
    }
}
#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
fn main() {
    let arguments: std::vec::Vec<std::string::String> = std::env::args().collect();
    let base_before_input = LIVE.load(Relaxed);
    let (actual_bits, allocations, reallocations, deallocations, requested, retained, peak,
        input_requested_bytes, after_output_drop) = {
        // Input adaptation injects the borrowed input immediately before n.
        // This scope owns it until the output and operation stats are observed.
        let n: i64 = arguments[1].parse().unwrap();
        let seed: i64 = arguments[2].parse().unwrap();
        let base_live = LIVE.load(Relaxed);
        let input_requested_bytes = base_live - base_before_input;
        ALLOCS.store(0, Relaxed);
        REALLOCS.store(0, Relaxed);
        DEALLOCS.store(0, Relaxed);
        REQUESTED.store(0, Relaxed);
        PEAK.store(base_live, Relaxed);
        let value = std::hint::black_box(function0(std::hint::black_box(n), std::hint::black_box(seed)));
        let allocations = ALLOCS.load(Relaxed);
        let reallocations = REALLOCS.load(Relaxed);
        let deallocations = DEALLOCS.load(Relaxed);
        let requested = REQUESTED.load(Relaxed);
        let retained = LIVE.load(Relaxed) - base_live;
        let peak = PEAK.load(Relaxed) - base_live;
        // Consume the returned value observably after counters have been sampled.
        let actual_bits = value.to_bits();
        drop(value);
        let after_output_drop = LIVE.load(Relaxed) as i64 - base_live as i64;
        (actual_bits, allocations, reallocations, deallocations, requested, retained, peak,
            input_requested_bytes, after_output_drop)
    };
    let after_drop = LIVE.load(Relaxed) as i64 - base_before_input as i64;
    println!(
        "{{\"protocol\":\"mpfr-directed-f64-scalar-v1\",\"kind\":\"native-approximate-allocation-result-v1\",\"iterations\":{},\"seed\":{},\"actualBits\":\"{actual_bits:016x}\",\"allocations\":{allocations},\"reallocations\":{reallocations},\"deallocationsDuringWork\":{deallocations},\"requestedBytes\":{requested},\"retainedRequestedBytes\":{retained},\"peakRequestedBytes\":{peak},\"inputRequestedBytes\":{input_requested_bytes},\"liveBytesAfterOutputDrop\":{after_output_drop},\"liveBytesAfterDrop\":{after_drop}}}"
        , arguments[1].parse::<i64>().unwrap(), arguments[2].parse::<i64>().unwrap()
    );
}
