//! Identify the compiler's own source, so caches written by one build (the
//! language server, `tsrust dev`) are reused only by a build of the same code.
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src");
    let mut files = Vec::new();
    collect(Path::new("src"), &mut files);
    files.sort();
    // FNV-1a over paths and contents, two independent 64-bit lanes.
    let mut lanes = [0xcbf2_9ce4_8422_2325u64, 0x8422_2325_cbf2_9ce4u64];
    for file in files {
        let bytes = std::fs::read(&file).unwrap_or_default();
        for part in [file.to_string_lossy().as_bytes(), &(bytes.len() as u64).to_le_bytes(), &bytes] {
            for (lane, seed) in lanes.iter_mut().zip([0x100_0000_01b3u64, 0x100_0000_0233u64]) {
                for &byte in part {
                    *lane = (*lane ^ u64::from(byte)).wrapping_mul(seed);
                }
            }
        }
    }
    println!("cargo:rustc-env=TSRUST_SOURCE_IDENTITY={:016x}{:016x}", lanes[0], lanes[1]);
}

fn collect(directory: &Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else {
            files.push(path);
        }
    }
}
