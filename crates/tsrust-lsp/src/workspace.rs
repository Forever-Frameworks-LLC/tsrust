//! Immutable project inputs and dependency invalidation, independent of protocol scheduling.
use std::cell::{Cell,RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tsrust_compiler::project::{
    FileSourceProvider, ProjectAnalysis, ProjectDiagnostic, SourceId, SourceProvider, check_graph,
    load_project,
};

#[derive(Clone)]
struct Buffer {
    version: i32,
    generation: u64,
    text: Arc<String>,
}
#[derive(Default)]
pub struct Workspace {
    open: BTreeMap<SourceId, Buffer>,
    revisions: BTreeMap<SourceId, u64>,
    generation: u64,
    dependencies: BTreeMap<SourceId, BTreeSet<SourceId>>,
}
impl Workspace {
    /// Called for watched-file writes, removal or rename; open buffers remain authoritative.
    pub fn invalidate_file(&mut self, id: &SourceId) {
        self.generation += 1;
        self.revisions.insert(id.clone(), self.generation);
    }
    /// Input identities must be canonical absolute paths (or normalized absolute new-file paths).
    pub fn open(&mut self, id: SourceId, version: i32, text: String) {
        self.generation += 1;
        self.revisions.insert(id.clone(), self.generation);
        self.open.insert(
            id,
            Buffer {
                version,
                generation: self.generation,
                text: Arc::new(text),
            },
        );
    }
    pub fn change(&mut self, id: &SourceId, version: i32, text: String) -> bool {
        let Some(buffer) = self.open.get(id) else {
            return false;
        };
        if version <= buffer.version {
            return false;
        }
        self.open(id.clone(), version, text);
        true
    }
    pub fn close(&mut self, id: &SourceId) -> bool {
        if self.open.remove(id).is_none() {
            return false;
        }
        self.generation += 1;
        self.revisions.insert(id.clone(), self.generation);
        true
    }
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        WorkspaceSnapshot {
            consistent:Cell::new(true),
            open: self.open.clone(),
            revisions: self.revisions.clone(),
            read: RefCell::new(BTreeMap::new()),
            configs: RefCell::new(BTreeMap::new()),
            observed: RefCell::new(BTreeSet::new()),
            resolved: RefCell::new(BTreeMap::new()),
            disk: RefCell::new(BTreeMap::new()),
        }
    }
    pub fn current(&self, snapshot: &WorkspaceSnapshot) -> bool {
        snapshot.disk_current()
            && snapshot
                .observed
                .borrow()
                .iter()
                .all(|id| self.revisions.get(id) == snapshot.revisions.get(id))
    }
    pub fn register(&mut self, entry: SourceId, snapshot: &WorkspaceSnapshot) -> bool {
        if !self.current(snapshot) {
            return false;
        }
        self.dependencies
            .insert(entry, snapshot.observed.borrow().clone());
        true
    }
    pub fn affected(&self, id: &SourceId) -> Vec<SourceId> {
        self.dependencies
            .iter()
            .filter(|(entry, dependencies)| *entry == id || dependencies.contains(id))
            .map(|(entry, _)| entry.clone())
            .collect()
    }
    pub fn remove_entry(&mut self, entry: &SourceId) {
        self.dependencies.remove(entry);
    }
}

type Resolutions = BTreeMap<(Option<SourceId>, String), Result<SourceId, String>>;

#[derive(Clone)]
pub struct WorkspaceSnapshot {
    consistent:Cell<bool>,
    open: BTreeMap<SourceId, Buffer>,
    revisions: BTreeMap<SourceId, u64>,
    read: RefCell<BTreeMap<SourceId, Arc<String>>>,
    configs: RefCell<BTreeMap<SourceId,Result<Option<String>,String>>>,
    observed: RefCell<BTreeSet<SourceId>>,
    resolved: RefCell<Resolutions>,
    disk: RefCell<BTreeMap<SourceId, Option<DiskStamp>>>,
}
#[derive(Clone, PartialEq, Eq)]
struct DiskStamp {
    len: u64,
    modified: Option<std::time::SystemTime>,
    #[cfg(unix)]
    identity: (u64, u64, i64, i64),
}
fn disk_stamp(id: &SourceId) -> Option<DiskStamp> {
    let metadata = std::fs::metadata(&id.0).ok()?;
    Some(DiskStamp {
        len: metadata.len(),
        modified: metadata.modified().ok(),
        #[cfg(unix)]
        identity: {
            use std::os::unix::fs::MetadataExt;
            (
                metadata.dev(),
                metadata.ino(),
                metadata.ctime(),
                metadata.ctime_nsec(),
            )
        },
    })
}
impl WorkspaceSnapshot {
    fn disk_current(&self) -> bool {
        self.consistent.get() && self.disk
            .borrow()
            .iter()
            .all(|(id, stamp)| disk_stamp(id) == *stamp)
    }
    /// Start an independent entry analysis against the same immutable buffers.
    pub(crate) fn fresh(&self) -> Self {
        Self {consistent:Cell::new(true),open:self.open.clone(), revisions:self.revisions.clone(),
            read:RefCell::default(), configs:RefCell::default(), observed:RefCell::default(),
            resolved:RefCell::default(), disk:RefCell::default()}
    }
    pub(crate) fn can_reuse(&self, previous:&Self) -> bool {
        previous.disk_current() && previous.observed.borrow().iter()
            .all(|id|self.revisions.get(id)==previous.revisions.get(id))
    }
    /// Carry all reused dependencies into final stale-input validation and ranges.
    pub(crate) fn absorb(&self, input:&Self) {
        // Separate entries may observe an edit at different times. Never publish
        // a combined result whose dependency versions disagree.
        if !input.consistent.get()
            || input.read.borrow().iter().any(|(id,text)|self.read.borrow().get(id).is_some_and(|old|old!=text))
            || input.configs.borrow().iter().any(|(id,text)|self.configs.borrow().get(id).is_some_and(|old|old!=text))
            || input.disk.borrow().iter().any(|(id,stamp)|self.disk.borrow().get(id).is_some_and(|old|old!=stamp)) {
            self.consistent.set(false);
        }
        self.read.borrow_mut().extend(input.read.borrow().clone());
        self.configs.borrow_mut().extend(input.configs.borrow().clone());
        self.observed.borrow_mut().extend(input.observed.borrow().iter().cloned());
        self.disk.borrow_mut().extend(input.disk.borrow().clone());
    }
    pub fn open_sources(&self) -> Vec<SourceId> { self.open.keys().cloned().collect() }
    pub fn check(&self, entry: &SourceId) -> Result<ProjectAnalysis, ProjectDiagnostic> {
        check_graph(load_project(self, &entry.0)?)
    }
    pub fn check_with_diagnostics(&self, entry: &SourceId) -> Result<ProjectAnalysis, Vec<ProjectDiagnostic>> {
        tsrust_compiler::project::check_with_diagnostics(self, &entry.0)
    }
    /// Exact loaded inputs, including disk fallback, for coordinate conversion after checking.
    pub fn sources(&self) -> BTreeMap<SourceId, Arc<String>> {
        self.read.borrow().clone()
    }
    pub fn version(&self, id: &SourceId) -> Option<i32> {
        self.open.get(id).map(|buffer| buffer.version)
    }
    pub fn generation(&self, id: &SourceId) -> Option<u64> {
        self.open.get(id).map(|buffer| buffer.generation)
    }
}
fn normalize(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err("Workspace source identities must be absolute paths.".into());
    }
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                result.pop();
            }
            Component::CurDir => {}
            other => result.push(other.as_os_str()),
        }
    }
    Ok(result)
}
impl SourceProvider for WorkspaceSnapshot {
    fn resolve(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        let key = (importer.cloned(), path.to_owned());
        if let Some(result) = self.resolved.borrow().get(&key) {
            return result.clone();
        }
        let result = self.resolve_path(importer, path);
        self.resolved.borrow_mut().insert(key, result.clone());
        result
    }
    fn read(&self, id: &SourceId) -> Result<String, String> {
        self.observed.borrow_mut().insert(id.clone());
        if let Some(source) = self.read.borrow().get(id) {
            return Ok(source.as_ref().clone());
        }
        let source = if let Some(buffer) = self.open.get(id) {
            Arc::clone(&buffer.text)
        } else {
            let before = disk_stamp(id);
            let source = FileSourceProvider.read(id)?;
            if disk_stamp(id) != before {
                return Err(format!("Source changed while reading {}", id.0));
            }
            self.disk.borrow_mut().insert(id.clone(), before);
            Arc::new(source)
        };
        self.read
            .borrow_mut()
            .insert(id.clone(), Arc::clone(&source));
        Ok(source.as_ref().clone())
    }
}

impl WorkspaceSnapshot {
    fn resolve_path(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        let candidates=if let Some(importer)=importer {
            let aliases=tsrust_compiler::project_resolution::discover(Path::new(&importer.0),|path|self.read_config(path))?;
            tsrust_compiler::project_resolution::candidates(Path::new(&importer.0),path,aliases.as_ref())?
        } else {vec![PathBuf::from(path)]};
        for candidate in &candidates {
            let id=self.observe_path(candidate)?;
            if self.open.contains_key(&id) {return Ok(id);}
            match std::fs::metadata(&id.0) {
                Ok(metadata) if metadata.is_file()=>return Ok(id),
                Ok(_)=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
                Err(error)=>return Err(format!("{}: {error}",id.0)),
            }
        }
        Err(format!("Cannot resolve {path}; tried {}",candidates.iter().map(|path|path.display().to_string()).collect::<Vec<_>>().join(", ")))
    }
    // Observe lexical and canonical identities, including absent candidates, so
    // file creation and symlink retargeting invalidate previously chosen paths.
    fn observe_path(&self,path:&Path)->Result<SourceId,String> {
        let normalized=normalize(path)?;
        let lexical=SourceId(normalized.to_str().ok_or("Source path must be valid UTF-8.")?.into());
        self.observed.borrow_mut().insert(lexical.clone());
        let canonical=std::fs::canonicalize(&normalized).unwrap_or(normalized);
        let id=SourceId(canonical.to_str().ok_or("Source path must be valid UTF-8.")?.into());
        self.observed.borrow_mut().insert(id.clone());
        // A changed disk file does not override an open buffer. A lexical
        // symlink's identity still matters even when its target is open.
        if !self.open.contains_key(&id)||lexical!=id {
            let stamp=disk_stamp(&lexical);
            let mut disk=self.disk.borrow_mut();
            if disk.get(&lexical).is_some_and(|old|*old!=stamp) {self.consistent.set(false);}
            disk.insert(lexical,stamp);
        }
        Ok(id)
    }
    fn read_config(&self,path:&Path)->Result<Option<String>,String> {
        let id=self.observe_path(path)?;
        if let Some(value)=self.configs.borrow().get(&id) {return value.clone();}
        let value=if let Some(buffer)=self.open.get(&id) {Ok(Some((*buffer.text).clone()))} else {
            let before=disk_stamp(&id);
            let result=match std::fs::read_to_string(&id.0) {
                Ok(text)=>Ok(Some(text)),Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(None),
                Err(error)=>Err(format!("{}: {error}",id.0)),
            };
            if disk_stamp(&id)!=before {self.consistent.set(false);return Err(format!("Configuration changed while reading {}",id.0));}
            result
        };
        self.configs.borrow_mut().insert(id,value.clone());value
    }

}

#[cfg(test)]
mod scan_input_tests {
    use super::*;
    #[test]
    fn disk_config_creation_and_rewrite_invalidate_without_watch_notifications() {
        struct Scratch(PathBuf);
        impl Drop for Scratch {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}
        let scratch=Scratch(std::env::temp_dir().join(format!("tsrust-alias-snapshot-{}-{}",std::process::id(),std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos())));
        std::fs::create_dir(&scratch.0).unwrap();
        let root=std::fs::canonicalize(&scratch.0).unwrap();
        let entry=SourceId(root.join("main.tsr").to_str().unwrap().into());
        let dependency=SourceId(root.join("target.tsr").to_str().unwrap().into());
        let config=root.join("tsrust.config.json");
        let mut workspace=Workspace::default();workspace.open(dependency.clone(),1,"export function greet(){}".into());
        let missing=workspace.snapshot();assert!(missing.resolve(Some(&entry),"@/target").is_err());
        std::fs::write(&config,r#"{"paths":{"@/*":"./*"}}"#).unwrap();
        assert!(!workspace.current(&missing));
        let snapshot=workspace.snapshot();assert_eq!(snapshot.resolve(Some(&entry),"@/target").unwrap(),dependency);
        std::fs::write(&config,r#"{"paths":{"@/*":"./different/*"}}"#).unwrap();
        assert!(!workspace.current(&snapshot));assert!(!workspace.snapshot().can_reuse(&snapshot));
        assert!(workspace.snapshot().resolve(Some(&entry),"@/target").is_err());
    }
    #[test]
    fn aliases_use_unsaved_config_and_source_and_invalidate_on_edits() {
        let mut workspace=Workspace::default();
        let entry=SourceId("/tsrust-alias-editor-test/src/main.tsr".into());
        let config=SourceId("/tsrust-alias-editor-test/tsrust.config.json".into());
        let dependency=SourceId("/tsrust-alias-editor-test/src/components/index.tsr".into());
        workspace.open(config.clone(),1,r#"{"paths":{"@/*":"./src/*"}}"#.into());
        workspace.open(entry.clone(),1,"import {greet} from '@/components';function main(){console.log(greet());}".into());
        workspace.open(dependency.clone(),1,"export function greet():string{return 'hello';}".into());
        let snapshot=workspace.snapshot();snapshot.check(&entry).unwrap();
        assert_eq!(snapshot.resolve(Some(&entry),"@/components").unwrap(),dependency);
        assert!(!snapshot.sources().contains_key(&config));
        assert!(workspace.register(entry.clone(),&snapshot));
        assert!(workspace.affected(&config).contains(&entry));
        workspace.change(&config,2,r#"{"paths":{"@/*":"./missing/*"}}"#.into());
        assert!(!workspace.current(&snapshot));
        assert!(workspace.snapshot().check(&entry).is_err());
    }
    #[test]
    fn opening_preferred_extensionless_candidate_invalidates_index_resolution() {
        let mut workspace=Workspace::default();
        let entry=SourceId("/tsrust-extension-editor-test/main.tsr".into());
        let index=SourceId("/tsrust-extension-editor-test/component/index.tsr".into());
        let file=SourceId("/tsrust-extension-editor-test/component.tsr".into());
        workspace.open(entry.clone(),1,"function main(){}".into());
        workspace.open(index.clone(),1,"export function greet(){}".into());
        let snapshot=workspace.snapshot();
        assert_eq!(snapshot.resolve(Some(&entry),"./component").unwrap(),index);
        assert!(workspace.register(entry.clone(),&snapshot));
        workspace.open(file.clone(),1,"export function greet(){}".into());
        assert!(!workspace.current(&snapshot));assert!(workspace.affected(&file).contains(&entry));
        assert_eq!(workspace.snapshot().resolve(Some(&entry),"./component").unwrap(),file);
    }
    #[test]
    fn combining_different_dependency_contents_never_certifies_a_current_scan() {
        let id=SourceId("/tmp/tsrust-scan-consistency.tsr".into());
        let mut workspace=Workspace::default();
        workspace.open(id.clone(),1,"function old(){}".into());
        let old=workspace.snapshot();old.read(&id).unwrap();
        workspace.change(&id,2,"function changed(){}".into());
        let new=workspace.snapshot();new.read(&id).unwrap();
        let combined=workspace.snapshot();combined.absorb(&old);combined.absorb(&new);
        assert!(!workspace.current(&combined));
        let stable=workspace.snapshot();stable.absorb(&new);
        assert!(workspace.current(&stable));
    }
}
