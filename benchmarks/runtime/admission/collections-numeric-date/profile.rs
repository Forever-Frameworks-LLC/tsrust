#![allow(dead_code)]
include!("reference.rs");
#[cfg(feature="allocation-probe")]
include!("../../probes/counting-allocator.rs");
fn main(){
    if std::env::args().nth(1).as_deref()==Some("--check"){verify();return;}
    for name in ["endpoints","set-snapshot","deque-snapshot","heap-sorted-vec","heap-sort-snapshot","heap-sorted-vec-seeded","heap-sort-snapshot-seeded","heap-sorted-vec-reverse","heap-sort-snapshot-reverse","heap-sorted-vec-duplicates","heap-sort-snapshot-duplicates","float-parse","radix-parse","checked","date-time"] {
        for size in [0,1,8,1024] {
            let mut operation=kernel(name,size); let expected=operation();
            for _ in 0..8 {assert_eq!(black_box(operation()),expected);}
            #[cfg(not(feature="allocation-probe"))]
            for sample in 0..31 {let start=std::time::Instant::now();for _ in 0..1024 {assert_eq!(black_box(operation()),expected);}let ns=start.elapsed().as_nanos();println!("{{\"kind\":\"timing\",\"name\":\"{name}\",\"size\":{size},\"sample\":{sample},\"operations\":1024,\"ns\":{ns},\"checksum\":\"{expected}\"}}");}
            #[cfg(feature="allocation-probe")]
            {let live=LIVE.load(std::sync::atomic::Ordering::Relaxed); ALLOCS.store(0,std::sync::atomic::Ordering::Relaxed);REALLOCS.store(0,std::sync::atomic::Ordering::Relaxed);REQUESTED.store(0,std::sync::atomic::Ordering::Relaxed);PEAK.store(live,std::sync::atomic::Ordering::Relaxed);assert_eq!(black_box(operation()),expected);let allocations=ALLOCS.load(std::sync::atomic::Ordering::Relaxed);let reallocations=REALLOCS.load(std::sync::atomic::Ordering::Relaxed);let bytes=REQUESTED.load(std::sync::atomic::Ordering::Relaxed);let peak=PEAK.load(std::sync::atomic::Ordering::Relaxed)-live;assert_eq!(LIVE.load(std::sync::atomic::Ordering::Relaxed),live);println!("{{\"kind\":\"allocation\",\"name\":\"{name}\",\"size\":{size},\"allocations\":{allocations},\"reallocations\":{reallocations},\"bytes\":{bytes},\"peak\":{peak},\"checksum\":\"{expected}\"}}");}
        }
    }
}
