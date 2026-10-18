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
    self.checked=0;
    let roots=job.roots.iter().filter(|root|root.exists()).cloned().collect::<Vec<_>>();
    let mut files=tsrust_compiler::source_discovery::discover(&roots)?.into_iter()
        .map(|path|SourceId(path.to_string_lossy().into_owned())).collect::<BTreeSet<_>>();
    for id in job.input.open_sources() {
        let path=PathBuf::from(&id.0);
        if path.extension().is_some_and(|extension|extension=="tsr" || extension=="tsrx") && roots.iter().any(|root|path.starts_with(root)) {files.insert(id);}
    }
    self.entries.retain(|id,_|files.contains(id));
    let mut errors=Vec::new();
    for file in files {
        if !self.entries.get(&file).is_some_and(|entry|job.input.can_reuse(&entry.input)) {
            let input=job.input.fresh();
            let errors=input.check_with_diagnostics(&file).err().unwrap_or_default();
            self.entries.insert(file.clone(),Entry{input,errors});
            self.checked+=1;
        }
        let entry=&self.entries[&file];
        job.input.absorb(&entry.input);
        errors.extend(entry.errors.iter().cloned());
    }
    let mut buckets=convert_all(&job.input,&errors)?;
    for diagnostics in buckets.values_mut() {
        diagnostics.sort_by_key(|error|(error.range.start.line,error.range.start.character,error.range.end.line,error.range.end.character));
        diagnostics.dedup();
    }
    Ok(buckets)
}

}

pub fn check(job:&Job)->Result<Buckets,String> {Cache::default().check(job)}
