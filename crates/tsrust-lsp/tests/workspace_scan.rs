use std::{path::PathBuf,time::{Duration,Instant}};
use serde_json::json;
use tsrust_lsp::{Snapshot,project_diagnostics::file_uri,project_session::ProjectSession,workspace_scan::{Scanner,Checked,check}};
use tsrust_compiler::project::SourceId;
struct Directory(PathBuf);
impl Directory {
    fn new()->Self {
        static NEXT:std::sync::atomic::AtomicU64=std::sync::atomic::AtomicU64::new(0);
        let path=std::env::temp_dir().join(format!("tsrust-scan-unit-{}-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),NEXT.fetch_add(1,std::sync::atomic::Ordering::Relaxed)));
        std::fs::create_dir(&path).unwrap();Self(path.canonicalize().unwrap())
    }
    fn uri(&self,name:&str)->String {file_uri(&SourceId(self.0.join(name).to_str().unwrap().into())).unwrap().to_string()}
}
impl Drop for Directory {fn drop(&mut self){std::fs::remove_dir_all(&self.0).unwrap();}}

#[test]
fn scan_excludes_generated_files_deduplicates_roots_and_includes_unsaved_new_files() {
    let dir=Directory::new();let now=Instant::now();
    std::fs::write(dir.0.join("bad.tsr"),"function bad(){const value:int=\"bad\";}").unwrap();
    for excluded in ["target","node_modules",".git"] {
        std::fs::create_dir(dir.0.join(excluded)).unwrap();
        std::fs::write(dir.0.join(excluded).join("bad.tsr"),"invalid").unwrap();
    }
    #[cfg(unix)] std::os::unix::fs::symlink(&dir.0,dir.0.join("cycle")).unwrap();
    let mut projects=ProjectSession::default();
    projects.open(&Snapshot{uri:dir.uri("new.tsr"),version:1,generation:1,text:"function bad(){const first=;const second=;}".into()});
    let mut scanner=Scanner::default();
    scanner.initialize(&json!({"workspaceFolders":[{"uri":dir.uri("")},{"uri":dir.uri("")}]}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();
    let buckets=check(&job).unwrap();
    assert_eq!(buckets.len(),2);
    assert_eq!(buckets.values().map(Vec::len).sum::<usize>(),3);
    let publications=scanner.accept(Checked{job,result:Ok(buckets)},&projects,now,Duration::ZERO).unwrap();
    assert_eq!(publications.len(),2);
}

#[test]
fn scans_discard_stale_edits_disk_changes_and_removed_roots() {
    let dir=Directory::new();let now=Instant::now();
    std::fs::write(dir.0.join("bad.tsr"),"function bad(){const value:int=\"bad\";}").unwrap();
    let mut projects=ProjectSession::default();let mut scanner=Scanner::default();
    scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=check(&job);
    projects.open(&Snapshot{uri:dir.uri("bad.tsr"),version:1,generation:1,text:"function bad(){}".into()});
    scanner.dirty(now,Duration::ZERO);
    assert!(scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap().is_empty());
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=check(&job);
    scanner.change_folders(&json!({"event":{"removed":[{"uri":dir.uri("")}],"added":[]}}),now);
    assert!(scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap().is_empty());
    assert!(!scanner.covers(&dir.uri("bad.tsr")));
    scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    projects.close(&dir.uri("bad.tsr"));
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=check(&job);
    std::fs::write(dir.0.join("bad.tsr"),"function repaired(){}").unwrap();
    assert!(scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap().is_empty());
    assert!(scanner.due_in(now).is_some());
}

#[test]
fn removing_last_root_clears_previous_publications() {
    let dir=Directory::new();let now=Instant::now();
    std::fs::write(dir.0.join("bad.tsr"),"function bad(){const value=;}").unwrap();
    let projects=ProjectSession::default();let mut scanner=Scanner::default();
    scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=check(&job);
    assert_eq!(scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap()[0].diagnostics.len(),1);
    scanner.change_folders(&json!({"event":{"removed":[{"uri":dir.uri("")}],"added":[]}}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=check(&job);
    let cleared=scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap();
    assert_eq!(cleared.len(),1);assert!(cleared[0].diagnostics.is_empty());assert!(!scanner.enabled());
}
