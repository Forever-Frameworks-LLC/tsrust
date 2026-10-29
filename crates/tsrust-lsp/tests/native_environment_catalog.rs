use tsrust_compiler::project::SourceId;
use tsrust_lsp::workspace::Workspace;

struct Directory(std::path::PathBuf);
impl Drop for Directory {
    fn drop(&mut self) { std::fs::remove_dir_all(&self.0).unwrap(); }
}

#[test]
fn every_environment_catalog_example_checks_and_executes() {
    let path = std::env::temp_dir().join(format!("tsrust-env-catalog-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(std::fs::canonicalize(path).unwrap());
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entries = catalog["std:env"].as_array().unwrap();
    assert_eq!(entries.len(), 3);
    for entry in entries {
        for example in entry["examples"].as_array().unwrap() {
            let source = tsrust_compiler::format(example["source"].as_str().unwrap()).unwrap();
            assert_eq!(tsrust_compiler::format(&source).unwrap(), source);
            let id = SourceId(directory.0.join("example.tsr").to_str().unwrap().into());
            let mut workspace = Workspace::default();
            workspace.open(id.clone(), 1, source);
            let checked = workspace.snapshot().check(&id).unwrap();
            let rust = directory.0.join("example.rs");
            std::fs::write(&rust, checked.emit_rust()).unwrap();
            for level in ["0", "3"] {
                let binary = directory.0.join("example");
                let compiled = std::process::Command::new("rustup")
                    .args(["run", tsrust_compiler::RUST_BACKEND_VERSION, "rustc", "--edition=2024"])
                    .arg(format!("-Copt-level={level}")).arg(&rust).arg("-o").arg(&binary).output().unwrap();
                assert!(compiled.status.success(), "{}: {}", entry["name"], String::from_utf8_lossy(&compiled.stderr));
                let output = std::process::Command::new(&binary).current_dir(&directory.0).output().unwrap();
                assert!(output.status.success(), "{}: {}", entry["name"], String::from_utf8_lossy(&output.stderr));
                assert_eq!(String::from_utf8(output.stdout).unwrap(), example["output"].as_str().unwrap(), "{} O{level}", entry["name"]);
            }
        }
    }
}
