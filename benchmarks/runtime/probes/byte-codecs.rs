// Direct crate/std profile, independent of compiler runtime and ABI bridges.
use base64::engine::{general_purpose, GeneralPurposeConfig};
use std::hint::black_box;
#[cfg(codec_allocations)]
include!("counting-allocator.rs");
#[cfg(not(codec_allocations))]
use std::time::Instant;

#[cfg(not(codec_allocations))]
fn run<E: base64::Engine>(engine: &E, input: &[u8], rounds: usize) -> (Vec<u128>, Vec<u128>) {
    let encoded = engine.encode(input);
    assert_eq!(engine.decode(&encoded).unwrap(), input);
    let mut samples = (Vec::new(), Vec::new());
    for sample in 0..39 {
        let began = Instant::now();
        for _ in 0..rounds { black_box(engine.encode(black_box(input))); }
        let encode = began.elapsed().as_nanos();
        let began = Instant::now();
        for _ in 0..rounds { black_box(engine.decode(black_box(&encoded)).unwrap()); }
        if sample >= 8 { samples.0.push(encode); samples.1.push(began.elapsed().as_nanos()); }
    }
    samples
}
#[cfg(codec_allocations)]
fn run<E: base64::Engine>(engine: &E, input: &[u8], _: usize) -> (Vec<u128>, Vec<u128>) {
    let encoded = engine.encode(input);
    for encode in [true, false] {
        let baseline = LIVE.load(Relaxed);
        ALLOCS.store(0, Relaxed); REALLOCS.store(0, Relaxed); REQUESTED.store(0, Relaxed); PEAK.store(baseline, Relaxed);
        let output = if encode { engine.encode(black_box(input)).into_bytes() } else { engine.decode(black_box(&encoded)).unwrap() };
        let allocations = ALLOCS.load(Relaxed); let reallocations = REALLOCS.load(Relaxed);
        let requested = REQUESTED.load(Relaxed); let retained = LIVE.load(Relaxed) - baseline; let peak = PEAK.load(Relaxed) - baseline;
        drop(output); let after_drop = LIVE.load(Relaxed) as i64 - baseline as i64;
        assert_eq!(after_drop, 0);
        println!("allocation encode={encode} allocations={allocations} reallocations={reallocations} requested={requested} retained={retained} peak={peak} after_drop={after_drop}");
    }
    (vec![], vec![])
}
fn main() {
    let config = GeneralPurposeConfig::new();
    for size in [32usize, 4096, 1048576] {
        let input: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let rounds = (4 * 1048576 / size).clamp(4, 2048);
        let result = run(&general_purpose::STANDARD, &input, rounds);
        println!("{{\"engine\":\"scalar\",\"size\":{size},\"rounds\":{rounds},\"encode_ns\":{:?},\"decode_ns\":{:?}}}", result.0, result.1);
        #[cfg(any(target_arch = "x86_64", all(target_arch = "aarch64", target_feature = "neon")))] {
            let simd = base64::engine::Simd::standard(config);
            let result = run(&simd, &input, rounds);
            println!("{{\"engine\":\"simd\",\"size\":{size},\"rounds\":{rounds},\"encode_ns\":{:?},\"decode_ns\":{:?}}}", result.0, result.1);
        }
    }
}
