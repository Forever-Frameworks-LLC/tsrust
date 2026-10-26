use std::{fs, path::PathBuf};
use tsrust_compiler::project::SourceId;
use tsrust_lsp::{project_diagnostics::{file_uri, source_id}, workspace::Workspace};

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.0); }
}

#[test]
fn host_file_uris_round_trip_real_paths_and_imported_diagnostics() {
    let directory = Scratch(std::env::temp_dir().join(format!("tsrust-host-paths-{}", std::process::id())));
    fs::create_dir(&directory.0).unwrap();
    let main = directory.0.join("main café %.tsr");
    let dependency = directory.0.join("dependency #.tsr");
    let main_source = "import { value } from './dependency #.tsr';function main(){value();}";
    let dependency_source = "export function value(){console.log(missing);}";
    fs::write(&main, main_source).unwrap();
    fs::write(&dependency, dependency_source).unwrap();
    let identity = |path: &std::path::Path| SourceId(fs::canonicalize(path).unwrap().to_str().unwrap().into());
    let main_id = identity(&main);
    let dependency_id = identity(&dependency);
    for id in [&main_id, &dependency_id] {
        let uri = file_uri(id).unwrap();
        assert_eq!(source_id(uri.as_str()).unwrap(), *id);
        assert!(source_id(&format!("{uri}?query")).is_err());
        assert!(source_id(&format!("{uri}#fragment")).is_err());
    }
    let mut workspace = Workspace::default();
    workspace.open(main_id.clone(), 1, main_source.into());
    workspace.open(dependency_id.clone(), 1, dependency_source.into());
    let error = workspace.snapshot().check(&main_id).err().unwrap();
    assert_eq!(error.location.source, dependency_id);
    for uri in ["file://remote/share/main.tsr", "file:///bad%", "file:///bad%GG", "file:///bad%FF", "file:///bad%00"] {
        assert!(source_id(uri).is_err(), "{uri}");
    }
}

#[cfg(windows)]
#[test]
fn windows_drive_uris_keep_drive_letters_and_reject_network_paths() {
    let id = SourceId(r"C:\tsrust unsaved\café %.tsr".into());
    let uri = file_uri(&id).unwrap();
    assert_eq!(uri.as_str(), "file:///C:/tsrust%20unsaved/caf%C3%A9%20%25.tsr");
    assert_eq!(source_id(uri.as_str()).unwrap(), id);
    assert_eq!(source_id("file://localhost/C:/tsrust%20unsaved/caf%C3%A9%20%25.tsr").unwrap(), id);
    assert!(file_uri(&SourceId(r"\\server\share\main.tsr".into())).is_err());
    assert!(source_id("file:///no-drive/main.tsr").is_err());
}
