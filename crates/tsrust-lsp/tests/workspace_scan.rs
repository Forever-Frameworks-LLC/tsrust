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

#[test]
fn cached_scans_recheck_only_changed_inputs_and_importers() {
    use tsrust_lsp::workspace_scan::Cache;
    let dir=Directory::new();let now=Instant::now();
    std::fs::write(dir.0.join("library.tsr"),"export function value():int{return 7;}").unwrap();
    std::fs::write(dir.0.join("entry.tsr"),"import {value} from './library.tsr';function use():int{return value();}").unwrap();
    std::fs::write(dir.0.join("other.tsr"),"function other(){const first:int=\"bad\";const second:boolean=42;}").unwrap();
    let mut projects=ProjectSession::default();let mut scanner=Scanner::default();let mut cache=Cache::default();
    scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let first=cache.check(&job).unwrap();
    assert_eq!(cache.checked_entries(),3);assert_eq!(first.values().map(Vec::len).sum::<usize>(),2);
    scanner.accept(Checked{job,result:Ok(first.clone())},&projects,now,Duration::ZERO).unwrap();
    scanner.dirty(now,Duration::ZERO);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();assert_eq!(cache.check(&job).unwrap(),first);
    assert_eq!(cache.checked_entries(),0,"unchanged scan must reuse successful and failed analyses");
    projects.open(&Snapshot{uri:dir.uri("library.tsr"),version:1,generation:1,text:"export function value():int{return \"bad\";}".into()});
    scanner.dirty(now,Duration::ZERO);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let changed=cache.check(&job).unwrap();
    assert_eq!(cache.checked_entries(),2);assert_eq!(changed.values().map(Vec::len).sum::<usize>(),3);
    assert_eq!(job.input.version(&SourceId(dir.0.join("library.tsr").to_str().unwrap().into())),Some(1));
    projects.close(&dir.uri("library.tsr"));
    std::fs::remove_file(dir.0.join("other.tsr")).unwrap();
    scanner.dirty(now,Duration::ZERO);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=cache.check(&job);
    assert_eq!(cache.checked_entries(),2);
    let cleared=scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap();
    assert!(cleared.iter().all(|item|item.diagnostics.is_empty()));
    assert!(cleared.iter().any(|item|item.uri==dir.uri("other.tsr")));
}

#[test]
fn cached_missing_imports_and_disk_edits_are_invalidated_without_watch_events() {
    use tsrust_lsp::workspace_scan::Cache;
    let dir=Directory::new();let now=Instant::now();
    std::fs::write(dir.0.join("entry.tsr"),"import {value} from './missing.tsr';function use():int{return value();}").unwrap();
    let projects=ProjectSession::default();let mut scanner=Scanner::default();let mut cache=Cache::default();
    scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();
    assert_eq!(cache.check(&job).unwrap().values().map(Vec::len).sum::<usize>(),1);
    std::fs::write(dir.0.join("missing.tsr"),"export function value():int{return 7;}").unwrap();
    scanner.dirty(now,Duration::ZERO);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();
    assert!(cache.check(&job).unwrap().values().all(Vec::is_empty));assert_eq!(cache.checked_entries(),2);
    std::fs::write(dir.0.join("missing.tsr"),"export function value():int{return \"bad\";}").unwrap();
    scanner.dirty(now,Duration::ZERO);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=cache.check(&job);
    assert_eq!(result.as_ref().unwrap().values().map(Vec::len).sum::<usize>(),1);
    assert_eq!(cache.checked_entries(),2);
    // Cached dependencies still participate in stale-publication rejection.
    scanner.dirty(now,Duration::ZERO);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=cache.check(&job);
    assert_eq!(cache.checked_entries(),0);
    std::fs::write(dir.0.join("missing.tsr"),"export function value():int{return 7;}").unwrap();
    assert!(scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap().is_empty());
    assert!(scanner.due_in(now).is_some());
}

#[test]
fn interrupted_scan_resumes_completed_entries_and_keeps_stale_input_guards() {
    use tsrust_lsp::workspace_scan::Cache;
    let dir=Directory::new();let projects=ProjectSession::default();let mut scanner=Scanner::default();let mut cache=Cache::default();let now=Instant::now();
    for name in ["a","b","c"] {std::fs::write(dir.0.join(format!("{name}.tsr")),format!("function {name}(){{const value:int=\"bad\";}}")).unwrap();}
    scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let mut visits=0;
    assert!(cache.check_cancellable(&job,||{visits+=1;visits>1}).unwrap().is_none());assert_eq!(cache.checked_entries(),1);
    scanner.dirty(now,Duration::ZERO);let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=cache.check(&job);assert_eq!(cache.checked_entries(),2);
    assert_eq!(result.as_ref().unwrap().values().map(Vec::len).sum::<usize>(),3);
    std::fs::write(dir.0.join("a.tsr"),"function a(){}").unwrap();assert!(scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap().is_empty());
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=cache.check(&job).unwrap();assert_eq!(cache.checked_entries(),1);assert_eq!(result.values().map(Vec::len).sum::<usize>(),2);
}

#[test]
fn configured_graph_checks_dependencies_once_without_hiding_orphans_tests_or_failures() {
    use tsrust_lsp::workspace_scan::Cache;
    let dir=Directory::new();let now=Instant::now();
    std::fs::write(dir.0.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"scan","build":{"entry":"entry.tsr","output":"app"}}"#).unwrap();
    std::fs::write(dir.0.join("entry.tsr"),"import {value} from './library.tsr';function main(){console.log(value());}").unwrap();
    std::fs::write(dir.0.join("library.tsr"),"export function value():int{return 7;}").unwrap();
    std::fs::write(dir.0.join("orphan.tsr"),"function unused(){const value:int='bad';}").unwrap();
    let projects=ProjectSession::default();let mut scanner=Scanner::default();let mut cache=Cache::default();
    scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let first=cache.check(&job).unwrap();
    assert_eq!(cache.checked_entries(),2);assert_eq!(first.values().map(Vec::len).sum::<usize>(),1);
    scanner.dirty(now,Duration::ZERO);let job=scanner.next(now,projects.workspace.snapshot()).unwrap();assert_eq!(cache.check(&job).unwrap(),first);assert_eq!(cache.checked_entries(),0);
    std::fs::write(dir.0.join("library.tsr"),"export function value():int{return 'bad';}").unwrap();
    scanner.dirty(now,Duration::ZERO);let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let failed=cache.check(&job).unwrap();assert_eq!(cache.checked_entries(),2);assert_eq!(failed.values().map(Vec::len).sum::<usize>(),2);
    std::fs::write(dir.0.join("library.tsr"),"export function value():int{return 7;}").unwrap();
    std::fs::write(dir.0.join("entry.tsr"),"function main(){}").unwrap();
    scanner.dirty(now,Duration::ZERO);let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=cache.check(&job);assert_eq!(cache.checked_entries(),2);
    std::fs::write(dir.0.join("library.tsr"),"export function value():int{return 'bad';}").unwrap();assert!(scanner.accept(Checked{job,result},&projects,now,Duration::ZERO).unwrap().is_empty());
}

#[test]
fn configured_graph_keeps_imported_test_and_nested_project_entries() {
    use tsrust_lsp::workspace_scan::Cache;
    let dir=Directory::new();let now=Instant::now();std::fs::create_dir(dir.0.join("nested")).unwrap();
    for (root,entry) in [(dir.0.clone(),"entry.tsr"),(dir.0.join("nested"),"entry.tsr")] {
        std::fs::write(root.join("tsrust.config.json"),format!(r#"{{"schemaVersion":1,"name":"scan","build":{{"entry":"{entry}","output":"app"}}}}"# )).unwrap();
    }
    std::fs::write(dir.0.join("entry.tsr"),"import {value} from './library.test.tsr';import {nested} from './nested/entry.tsr';function main(){console.log(value()+nested());}").unwrap();
    std::fs::write(dir.0.join("library.test.tsr"),"export function value():int{return 7;}").unwrap();
    std::fs::write(dir.0.join("nested/entry.tsr"),"export function nested():int{return 8;}").unwrap();
    let projects=ProjectSession::default();let mut scanner=Scanner::default();let mut cache=Cache::default();scanner.initialize(&json!({"rootUri":dir.uri("")}),now);
    let job=scanner.next(now,projects.workspace.snapshot()).unwrap();let result=cache.check(&job).unwrap();assert!(result.values().all(Vec::is_empty));assert_eq!(cache.checked_entries(),3);
}
