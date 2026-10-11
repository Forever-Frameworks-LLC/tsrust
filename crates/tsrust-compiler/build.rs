//! Embed locked dependency sources so installed native builds work offline.
use std::{env, fs, path::{Path, PathBuf}, process::Command};

fn collect(root: &Path, path: &Path, files: &mut Vec<(String, PathBuf)>) {
    for entry in fs::read_dir(path).expect("read bundled dependency directory") {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        assert!(!kind.is_symlink(), "dependency bundle cannot contain symlinks");
        if kind.is_dir() { collect(root, &entry.path(), files); }
        else if kind.is_file() {
            files.push((entry.path().strip_prefix(root).unwrap().to_str().unwrap().replace('\\', "/"), entry.path()));
        }
    }
}
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=native-dependencies");
    let source = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("native-dependencies");
    let output = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let vendor = output.join("native-vendor");
    let result = Command::new(env::var_os("CARGO").unwrap())
        .args(["vendor", "--offline", "--locked", "--versioned-dirs", "--manifest-path"])
        .arg(source.join("Cargo.toml")).arg(&vendor).output().expect("invoke Cargo vendor");
    assert!(result.status.success(), "Cannot bundle locked native dependencies. Run cargo fetch --locked --manifest-path crates/tsrust-compiler/native-dependencies/Cargo.toml before an offline compiler build: {}", String::from_utf8_lossy(&result.stderr));
    let mut files = Vec::new();
    collect(&vendor, &vendor, &mut files);
    for (name, _) in &mut files { *name = format!("vendor/{name}"); }
    for name in ["Cargo.toml", "Cargo.lock", "src/lib.rs"] { files.push((name.into(), source.join(name))); }
    files.sort_by(|a,b| a.0.cmp(&b.0));
    let mut generated = String::from("static FILES: &[(&str, &[u8])] = &[\n");
    for (name, path) in files {
        generated.push_str(&format!("({name:?}, include_bytes!({:?})),\n", path.to_str().unwrap()));
    }
    generated.push_str("];\n");
    fs::write(output.join("native_dependency_files.rs"), generated).unwrap();
}
