//! Workspace diagnostics use immutable inputs and one replaceable pending scan.
use crate::{project_diagnostics::{convert_all, source_id}, project_session::{Buckets, ProjectSession, Publication}, workspace::WorkspaceSnapshot};
use std::{collections::{BTreeMap,BTreeSet}, path::PathBuf, time::{Duration, Instant}};
use tsrust_compiler::project::{SourceId,ProjectDiagnostic};

pub struct Job {
    pub revision: u64,
    pub roots: Vec<PathBuf>,
    pub input: WorkspaceSnapshot,
}
pub struct Checked {
    pub job: Job,
    pub result: Result<Buckets, String>,
}
#[derive(Default)]
pub struct Scanner {
    roots: BTreeSet<PathBuf>,
    revision: u64,
    due: Option<Instant>,
    published: BTreeSet<SourceId>,
}
impl Scanner {
    pub fn initialize(&mut self, params: &serde_json::Value, now: Instant) {
        let uris = params.get("workspaceFolders").and_then(|value|value.as_array())
            .map(|folders|folders.iter().filter_map(|folder|folder.get("uri").and_then(|uri|uri.as_str())).collect::<Vec<_>>())
            .unwrap_or_else(||params.get("rootUri").and_then(|uri|uri.as_str()).into_iter().collect());
        self.roots = uris.into_iter().filter_map(|uri|source_id(uri).ok().map(|id|PathBuf::from(id.0))).collect();
        self.dirty(now, Duration::ZERO);
    }
    pub fn change_folders(&mut self, params: &serde_json::Value, now: Instant) {
        for (field, add) in [("removed",false),("added",true)] {
            if let Some(folders)=params.pointer(&format!("/event/{field}")).and_then(|value|value.as_array()) {
                for folder in folders {
                    if let Some(uri)=folder.get("uri").and_then(|uri|uri.as_str()) && let Ok(id)=source_id(uri) {
                        let path=PathBuf::from(id.0);
                        if add {self.roots.insert(path);} else {self.roots.remove(&path);}
                    }
                }
            }
        }
        self.revision += 1;
        self.due=Some(now);
    }
    pub fn covers(&self, uri: &str) -> bool {
        source_id(uri).is_ok_and(|id|self.roots.iter().any(|root|PathBuf::from(&id.0).starts_with(root)) || self.published.contains(&id))
    }
    pub fn enabled(&self) -> bool { !self.roots.is_empty() || !self.published.is_empty() }
    pub fn dirty(&mut self, now: Instant, debounce: Duration) {
        if self.enabled() { self.revision += 1; self.due=Some(now+debounce); }
    }
    pub fn due_in(&self, now: Instant) -> Option<Duration> { self.due.map(|due|due.saturating_duration_since(now)) }
    pub fn next(&mut self, now: Instant, input: WorkspaceSnapshot) -> Option<Job> {
        if self.due.is_none_or(|due|due>now) {return None;}
        self.due=None;
        Some(Job {revision:self.revision, roots:self.roots.iter().cloned().collect(), input})
    }
    pub fn accept(&mut self, checked: Checked, projects: &ProjectSession, now: Instant, debounce: Duration) -> Result<Vec<Publication>,String> {
        if checked.job.revision != self.revision {return Ok(vec![]);}
        if !projects.workspace.current(&checked.job.input) {
            self.dirty(now,debounce);
            return Ok(vec![]);
        }
        let mut buckets=checked.result?;
        let next=buckets.keys().cloned().collect();
        for removed in self.published.difference(&next) {buckets.insert(removed.clone(),vec![]);}
        self.published=next;
        Ok(buckets.into_iter().filter_map(|(id,diagnostics)|projects.publication(&id,checked.job.input.version(&id),diagnostics)).collect())
    }
}

struct Entry {
    input: WorkspaceSnapshot,
    errors: Vec<ProjectDiagnostic>,
    modules: BTreeSet<SourceId>,
}
/// Worker-local entry results. Each entry retains its own dependency observations.
#[derive(Default)]
pub struct Cache {
    entries: BTreeMap<SourceId,Entry>,
    checked: usize,
}
impl Cache {
    /// Number of entry analyses performed by the most recent scan.
    pub fn checked_entries(&self)->usize {self.checked}
    pub fn check(&mut self, job: &Job) -> Result<Buckets,String> {
        self.check_cancellable(job,||false).map(|result|result.expect("uninterrupted scan"))
    }
    /// Completed entries stay reusable when foreground editor work interrupts a scan.
    pub fn check_cancellable(&mut self, job:&Job,mut interrupted:impl FnMut()->bool)->Result<Option<Buckets>,String> {
    self.checked=0;
    let roots=job.roots.iter().filter(|root|root.exists()).cloned().collect::<Vec<_>>();
    let mut files=tsrust_compiler::source_discovery::discover(&roots)?.into_iter()
        .map(|path|SourceId(path.to_string_lossy().into_owned())).collect::<BTreeSet<_>>();
    for id in job.input.open_sources() {
        let path=PathBuf::from(&id.0);
        if path.extension().is_some_and(|extension|extension=="tsr" || extension=="tsrx") && roots.iter().any(|root|path.starts_with(root)) {files.insert(id);}
    }
    self.entries.retain(|id,_|files.contains(id));
    let configured=files.iter().map(|id|(id.clone(),job.input.configured_entry(id))).collect::<BTreeMap<_,_>>();
    let preferred=configured.values().flatten().filter(|id|files.contains(*id)).cloned().collect::<BTreeSet<_>>();
    let ordered=preferred.iter().cloned().chain(files.iter().filter(|id|!preferred.contains(*id)).cloned());
    let mut covered=BTreeSet::new();
    let mut used=BTreeSet::new();
    let mut errors=Vec::new();
    for file in ordered {
        if interrupted(){return Ok(None);}
        if covered.contains(&file) && !tsrust_compiler::project::is_test_entry(&file.0) {continue;}
        used.insert(file.clone());
        if !self.entries.get(&file).is_some_and(|entry|job.input.can_reuse(&entry.input)) {
            let input=job.input.fresh();
            let (errors,modules)=match input.check_with_diagnostics(&file) {
                Ok(analysis)=>(vec![],analysis.graph.modules.keys().cloned().collect()),
                Err(errors)=>(errors,BTreeSet::new()),
            };
            self.entries.insert(file.clone(),Entry{input:input.retained(),errors,modules});
            self.checked+=1;
        }
        let entry=&self.entries[&file];
        // A successful configured graph checks every imported body. Failed graphs,
        // test entries and files outside that project still receive independent checks.
        covered.extend(entry.modules.iter().filter(|id|configured.get(*id).is_some_and(|owner|owner.as_ref()==Some(&file))).cloned());
        job.input.absorb(&entry.input);
        errors.extend(entry.errors.iter().cloned());
    }
    if interrupted(){return Ok(None);}
    let mut buckets=convert_all(&job.input,&errors)?;
    for diagnostics in buckets.values_mut() {
        diagnostics.sort_by_key(|error|(error.range.start.line,error.range.start.character,error.range.end.line,error.range.end.character));
        diagnostics.dedup();
    }
    // A configured graph can supersede formerly independent entries after a
    // repair. Interrupted scans keep completed work until a full scan succeeds.
    self.entries.retain(|id,_|used.contains(id));
    Ok(Some(buckets))
}

}

pub fn check(job:&Job)->Result<Buckets,String> {Cache::default().check(job)}

#[cfg(test)]
mod retention_tests {
    use super::*;
    #[test]
    fn repaired_configured_graph_releases_superseded_independent_entries() {
        let root = std::env::temp_dir().join(format!("tsrust-scan-retention-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir(&root).unwrap();
        struct Scratch(PathBuf);
        impl Drop for Scratch {fn drop(&mut self){std::fs::remove_dir_all(&self.0).unwrap();}}
        let scratch = Scratch(root.canonicalize().unwrap());
        let write = |name: &str, source: &str| {
            let formatted = tsrust_compiler::format(source).unwrap();
            assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
            std::fs::write(scratch.0.join(name),formatted).unwrap();
        };
        std::fs::write(scratch.0.join("tsrust.config.json"),r#"{"schemaVersion":1,"name":"retention","build":{"entry":"entry.tsr","output":"app"}}"#).unwrap();
        write("entry.tsr","import {value} from './library.tsr';function main(){console.log(value());}");
        write("library.tsr","export function value():int{return 'bad';}");
        let workspace = crate::workspace::Workspace::default();
        let mut cache = Cache::default();
        let job = || Job {revision:1,roots:vec![scratch.0.clone()],input:workspace.snapshot()};
        assert!(cache.check(&job()).unwrap().values().any(|errors|!errors.is_empty()));
        assert_eq!(cache.entries.len(),2);
        write("library.tsr","export function value():int{return 7;}");
        assert!(cache.check(&job()).unwrap().values().all(Vec::is_empty));
        assert_eq!(cache.entries.len(),1,"successful graph replaces covered independent results");
        assert_eq!(cache.checked_entries(),1);
        assert!(cache.check(&job()).unwrap().values().all(Vec::is_empty));
        assert_eq!(cache.checked_entries(),0);
    }
}
