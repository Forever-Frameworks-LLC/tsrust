use tsrust_compiler::project::SourceId;
use tsrust_lsp::workspace::Workspace;

struct Directory(std::path::PathBuf);
impl Drop for Directory {
    fn drop(&mut self) { std::fs::remove_dir_all(&self.0).unwrap(); }
}

#[test]
fn every_file_catalog_example_checks_and_executes() {
    let path = std::env::temp_dir().join(format!("tsrust-file-catalog-{}-{}", std::process::id(), std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    std::fs::create_dir(&path).unwrap();
    let directory = Directory(std::fs::canonicalize(path).unwrap());
    std::fs::write(directory.0.join("input.bin"), [0, 65, 255, 66]).unwrap();
    let catalog: serde_json::Value = serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entries = catalog["std:fs"].as_array().unwrap();
    assert_eq!(entries.len(), 41);
    for entry in entries {
        for example in entry["examples"].as_array().unwrap() {
            let source = tsrust_compiler::format(example["source"].as_str().unwrap()).unwrap();
            assert_eq!(tsrust_compiler::format(&source).unwrap(), source);
            let id = SourceId(directory.0.join("example.tsr").to_str().unwrap().into());
            let mut workspace = Workspace::default();
            workspace.open(id.clone(), 1, source.clone());
            std::fs::write(&id.0, source).unwrap();
            let checked = workspace.snapshot().check(&id).unwrap();
            let rust = directory.0.join("example.rs");
            std::fs::write(&rust, checked.emit_rust()).unwrap();
            for level in ["0", "3"] {
                let binary = directory.0.join("example");
                // The normal build path supplies native file dependencies at both levels.
                let cli=std::env::var_os("TSRUST").map(std::path::PathBuf::from).unwrap_or_else(||std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join(format!("tsrust{}",std::env::consts::EXE_SUFFIX)));
                let mut command=std::process::Command::new(cli);
                command.arg("build").arg(&id.0).arg("-o").arg(&binary);
                if level=="3" {command.arg("--release");}
                let compiled=command.output().unwrap();
                assert!(compiled.status.success(), "{}: {}", entry["name"], String::from_utf8_lossy(&compiled.stderr));
                let output = std::process::Command::new(&binary).current_dir(&directory.0).output().unwrap();
                assert!(output.status.success(), "{} O{level}: {}\nstdout: {}\nstderr: {}", entry["name"], output.status, String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
                assert_eq!(String::from_utf8(output.stdout).unwrap(), example["output"].as_str().unwrap(), "{} O{level}", entry["name"]);
            }
        }
    }
}

#[test]
fn file_help_retains_optional_bounds_and_platform_close_contract() {
    let catalog:serde_json::Value=serde_json::from_str(include_str!("../src/builtins.json")).unwrap();
    let entries=catalog["std:fs"].as_array().unwrap();
    for entry in entries {
        let text=entry["documentation"].as_str().unwrap();
        assert!(!text.contains("67108864"),"{} retains obsolete ceiling",entry["name"]);
        assert!(!text.contains("other targets produce a native compilation diagnostic"));
    }
    let read=entries.iter().find(|entry|entry["name"]=="readFile").unwrap();
    assert!(read["documentation"].as_str().unwrap().contains("Omitting maxBytes or passing undefined"));
    assert!(read["parameters"][1]["documentation"].as_str().unwrap().contains("Invalid explicit limits fail before opening"));
    let close=entries.iter().find(|entry|entry["name"]=="close").unwrap();
    assert!(close["documentation"].as_str().unwrap().contains("does not expose OS close failures"));
}
