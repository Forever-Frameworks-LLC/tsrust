//! Direct Tsrust import references from an exact workspace snapshot, without body checking.
use crate::{LineIndex, project_diagnostics::file_uri, workspace::WorkspaceSnapshot};
use lsp_types::Location;

use std::{collections::BTreeSet, path::PathBuf};
use tsrust_compiler::project::{SourceFile, SourceId, SourceProvider};

pub struct Issue { pub uri: String, pub message: String }
pub struct References {
    pub locations: Vec<Location>,
    pub issues: Vec<Issue>,
    pub scanned: usize,
}

pub fn find(input: &WorkspaceSnapshot, roots: &[PathBuf], target: &SourceId, mut cancelled: impl FnMut() -> bool) -> Result<Option<References>, String> {
    if roots.is_empty() || !roots.iter().any(|root|std::path::Path::new(&target.0).starts_with(root)) {
        return Err("File references require a target inside an initialized workspace folder.".into());
    }
    let Some(files)=tsrust_compiler::source_discovery::discover_cancellable(roots,false,&mut cancelled)? else {return Ok(None)};
    let mut files=files.into_iter().map(|path|SourceId(path.to_string_lossy().into_owned())).collect::<BTreeSet<_>>();
    for id in input.open_sources() {
        let path=std::path::Path::new(&id.0);
        if roots.iter().any(|root|path.starts_with(root)) && path.extension().is_some_and(|ext|ext=="tsr" || ext=="tsrx") {files.insert(id);}
    }
    let mut result=References { locations:vec![],issues:vec![],scanned:0 };
    for id in files {
        if cancelled() {return Ok(None)}
        let uri=file_uri(&id)?.to_string();
        let source=match input.read(&id) {
            Ok(text)=>text,
            Err(message)=>{result.issues.push(Issue {uri,message});continue;}
        };
        let imports=match tsrust_compiler::project::module_imports(SourceFile {id:id.clone(),text:source.clone()}) {
            Ok(imports)=>imports,
            Err(error)=>{result.issues.push(Issue {uri,message:error.message});continue;}
        };
        result.scanned+=1;
        let mut lines=None;
        for import in imports {
            if cancelled() {return Ok(None)}
            if import.path.starts_with("std:") || import.path.starts_with("api:") {continue;}
            match input.resolve(Some(&id),&import.path) {
                Ok(resolved) if resolved==*target=>{
                    let range=lines.get_or_insert_with(||LineIndex::new(&source)).range(&source,import.span)?;
                    result.locations.push(Location {uri:file_uri(&id)?,range});
                },
                Ok(_)=>{},
                Err(message)=>result.issues.push(Issue {uri:uri.clone(),message}),
            }
        }
    }
    if cancelled() {return Ok(None)}
    result.locations.sort_by(|a,b|a.uri.as_str().cmp(b.uri.as_str()).then_with(||(a.range.start.line,a.range.start.character,a.range.end.line,a.range.end.character).cmp(&(b.range.start.line,b.range.start.character,b.range.end.line,b.range.end.character))));
    result.locations.dedup();
    Ok(Some(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn imports_use_current_aliases_unicode_buffers_ignores_and_report_incomplete_input() {
        let directory=std::env::temp_dir().join(format!("tsrust-file-refs-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&directory).unwrap();
        struct Cleanup(PathBuf);impl Drop for Cleanup {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let _cleanup=Cleanup(directory.clone());let root=directory.canonicalize().unwrap();
        let write=|name:&str,text:&str| {let path=root.join(name);std::fs::write(&path,text).unwrap();path};
        write("tsrust.config.json",r#"{"schemaVersion":1,"name":"refs","build":{"entry":"main.tsr","output":"target/test"},"paths":{"@lib/*":"./*"}}"#);
        let dep=write("dep.tsr","export function value():int{return 8;}");
        let main=write("main.tsr","/*😀*/ import {value} from './dep.tsr';function main(){ invalid body ! }");
        write("other.tsrx","import {value} from '@lib/dep.tsr'; function App(){return <View />;}");
        write("broken.tsr","import {missing} from './missing.tsr';");
        write(".gitignore","ignored.tsr\n");write("ignored.tsr","import {value} from './dep.tsr';");
        let mut workspace=crate::workspace::Workspace::default();
        let id=SourceId(main.to_string_lossy().into_owned());
        workspace.open(id.clone(),1,"/*🦀*/ import {value} from './dep.tsr';".into());
        let input=workspace.snapshot();let target=SourceId(dep.to_string_lossy().into_owned());
        let result=find(&input,&[root.clone()],&target,||false).unwrap().unwrap();
        assert_eq!(result.locations.len(),2);
        assert_eq!(result.issues.len(),1);
        let main_location=result.locations.iter().find(|item|crate::project_diagnostics::source_id(item.uri.as_str()).unwrap()==id).unwrap();
        assert_eq!(main_location.range.start.character,27);
        assert!(workspace.current(&input));
        workspace.change(&id,2,"import {value} from './other.tsrx';".into());
        assert!(!workspace.current(&input));
        assert!(find(&input,&[root],&target,||true).unwrap().is_none());
    }
}

// One scan at a time; separate from body analysis and latency-sensitive intelligence.
pub struct Job {
    pub id: lsp_server::RequestId,
    pub roots: Vec<PathBuf>,
    pub revision: u64,
    pub input: WorkspaceSnapshot,
    pub target: SourceId,
    pub cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
}
pub struct Checked { pub job: Job, pub result: Result<Option<References>,String> }
pub fn worker() -> Result<(crossbeam_channel::Sender<Job>,crossbeam_channel::Receiver<Checked>),String> {
    let (send, jobs)=crossbeam_channel::bounded::<Job>(1);
    let (done, receive)=crossbeam_channel::bounded(1);
    std::thread::Builder::new().name("tsrust-file-references".into()).stack_size(tsrust_compiler::COMPILER_STACK_BYTES).spawn(move || {
        while let Ok(job)=jobs.recv() {
            let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||find(&job.input,&job.roots,&job.target,||job.cancel.load(std::sync::atomic::Ordering::Relaxed))))
                .unwrap_or_else(|_|Err("File reference scan failed; retry or report the source case.".into()));
            if done.send(Checked {job,result}).is_err(){break;}
        }
    }).map_err(|error|error.to_string())?;
    Ok((send,receive))
}
impl References {
    pub fn json(self) -> serde_json::Value {
        serde_json::json!({"locations":self.locations,"scanned":self.scanned,"issues":self.issues.into_iter().map(|issue|serde_json::json!({"uri":issue.uri,"message":issue.message})).collect::<Vec<_>>()})
    }
}
