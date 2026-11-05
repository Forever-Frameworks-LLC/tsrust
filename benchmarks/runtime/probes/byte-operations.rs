// Direct standard-library floor; no compiler runtime, bridge, or proposed facade.
use std::{hint::black_box, time::Instant};
include!("counting-allocator.rs");
fn facts(bytes: &[u8]) -> u64 { bytes.len() as u64 + u64::from(*bytes.first().unwrap_or(&0)) + u64::from(*bytes.last().unwrap_or(&0)) }
fn profile(name: &str, size: usize, rounds: usize, mut operation: impl FnMut() -> u64) {
    let baseline = LIVE.load(Relaxed);
    ALLOCS.store(0, Relaxed); REALLOCS.store(0, Relaxed); REQUESTED.store(0, Relaxed); PEAK.store(baseline, Relaxed);
    black_box(operation());
    let allocations = ALLOCS.load(Relaxed); let reallocations = REALLOCS.load(Relaxed); let requested = REQUESTED.load(Relaxed);
    let retained = LIVE.load(Relaxed) as i64 - baseline as i64; let peak = PEAK.load(Relaxed) - baseline;
    assert_eq!(retained, 0);
    let mut samples = Vec::with_capacity(31);
    for sample in 0..39 {
        let started = Instant::now();
        for _ in 0..rounds { black_box(operation()); }
        if sample >= 8 { samples.push(started.elapsed().as_nanos()); }
    }
    println!("{{\"operation\":\"{name}\",\"size\":{size},\"rounds\":{rounds},\"allocations\":{allocations},\"reallocations\":{reallocations},\"requested\":{requested},\"retained\":{retained},\"peak\":{peak},\"durations_ns\":{samples:?}}}");
}
fn packet_write(packet: &mut [u8], value: i64) {
    packet[1..3].copy_from_slice(&(value as i16).to_le_bytes());
    packet[3..7].copy_from_slice(&(value as u32).to_be_bytes());
    packet[7..15].copy_from_slice(&value.to_le_bytes());
    packet[15..19].copy_from_slice(&(value as f32).to_be_bytes());
    packet[19..27].copy_from_slice(&(value as f64).to_le_bytes());
}
fn packet_read(packet: &[u8]) -> u64 {
    let signed = i16::from_le_bytes(packet[1..3].try_into().unwrap()) as i64;
    let unsigned = u32::from_be_bytes(packet[3..7].try_into().unwrap());
    let wide = i64::from_le_bytes(packet[7..15].try_into().unwrap());
    let single = f32::from_be_bytes(packet[15..19].try_into().unwrap());
    let double = f64::from_le_bytes(packet[19..27].try_into().unwrap());
    (signed as u64).wrapping_add(u64::from(unsigned)).wrapping_add(wide as u64).wrapping_add(single as u64).wrapping_add(double as u64)
}
#[cfg(not(memchr_candidate))]
fn main() {
    for size in [32usize, 4096, 1048576] {
        let input = vec![97u8; size]; let mut different = input.clone(); different[size-1] = 98;
        let mut target = vec![0u8; size+1]; let mut overlapping = input.clone();
        let rounds = (4*1048576/size).clamp(4,2048);
        assert_eq!([input.as_slice(),different.as_slice()].concat().len(),size*2);
        assert_eq!(different.windows(4).position(|window|window==b"aaab"),Some(size-4));
        profile("concat", size, rounds, || { let output = [black_box(input.as_slice()), black_box(different.as_slice())].concat(); facts(&output) });
        profile("copy_into", size, rounds, || { target[1..].copy_from_slice(black_box(&input)); facts(black_box(&target)) });
        profile("copy_within_right", size, rounds, || { black_box(&mut overlapping).copy_within(0..size-1,1); facts(&overlapping) });
        profile("equals_equal", size, rounds, || u64::from(black_box(input.as_slice())==black_box(input.as_slice())));
        profile("equals_last_difference", size, rounds, || u64::from(black_box(input.as_slice())==black_box(different.as_slice())));
        profile("compare_last_difference", size, rounds, || (black_box(input.as_slice()).cmp(black_box(different.as_slice())) as i8 as i64) as u64);
        profile("index_of_tail", size, rounds, || { let position = black_box(&different).windows(4).position(|window|window==black_box(b"aaab")).unwrap(); position as u64 });
    }
    // Repetitive adversarial miss: report the std windows baseline before choosing a search backend.
    for size in [256usize,4096,65536] {
        let input=vec![97u8;size]; let mut needle=vec![97u8;size.min(512)/2]; *needle.last_mut().unwrap()=98;
        assert!(input.windows(needle.len()).position(|part|part==needle.as_slice()).is_none());
        profile("index_of_repetitive_miss",size,4,|| black_box(&input).windows(needle.len()).position(|part|part==black_box(needle.as_slice())).map_or(u64::MAX,|index|index as u64));
    }
    let mut packet=[0u8;33]; packet_write(&mut packet,12345);
    assert_eq!(packet_read(&packet),61725);
    profile("packet_read_unaligned",packet.len(),65536,||packet_read(black_box(&packet)));
    profile("packet_write_reuse",packet.len(),65536,|| {packet_write(black_box(&mut packet),black_box(12345)); facts(&packet)});
    // Independent exact endian and IEEE bit controls before any facade exists.
    assert_eq!(i64::from_le_bytes([0,0,0,0,0,0,0,128]),i64::MIN);
    assert_eq!(u32::from_be_bytes([255;4]),4294967295);
    assert_eq!(((-0.0f64) as f32).to_bits(),0x80000000);
    assert_eq!((f64::INFINITY as f32).to_bits(),0x7f800000);
    assert!((f64::NAN as f32).is_nan());
}

#[cfg(memchr_candidate)]
fn main() {
    for size in [32usize,4096,1048576] {
        let mut input=vec![97u8;size];input[size-1]=98;
        let rounds=(4*1048576/size).clamp(4,2048);
        assert_eq!(memchr::memmem::find(&input,b"aaab"),Some(size-4));
        profile("index_of_tail",size,rounds,|| memchr::memmem::find(black_box(&input),black_box(b"aaab")).unwrap() as u64);
    }
    for size in [256usize,4096,65536] {
        let input=vec![97u8;size];let mut needle=vec![97u8;size.min(512)/2];*needle.last_mut().unwrap()=98;
        assert!(memchr::memmem::find(&input,&needle).is_none());
        profile("index_of_repetitive_miss",size,4,||memchr::memmem::find(black_box(&input),black_box(&needle)).map_or(u64::MAX,|index|index as u64));
    }
}
