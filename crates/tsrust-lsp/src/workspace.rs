//! Immutable project inputs and dependency invalidation, independent of protocol scheduling.
use std::cell::RefCell;
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
            open: self.open.clone(),
            revisions: self.revisions.clone(),
            read: RefCell::new(BTreeMap::new()),
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
    open: BTreeMap<SourceId, Buffer>,
    revisions: BTreeMap<SourceId, u64>,
    read: RefCell<BTreeMap<SourceId, Arc<String>>>,
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
        self.disk
            .borrow()
            .iter()
            .all(|(id, stamp)| disk_stamp(id) == *stamp)
    }
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
        let path = match importer {
            Some(id) => Path::new(&id.0)
                .parent()
                .unwrap_or(Path::new("/"))
                .join(path),
            None => PathBuf::from(path),
        };
        let normalized = normalize(&path)?;
        // A symlink removal/retarget event names its lexical path, not the old target.
        self.observed.borrow_mut().insert(SourceId(
            normalized
                .to_str()
                .ok_or("Source path must be valid UTF-8.")?
                .into(),
        ));
        let canonical = std::fs::canonicalize(&normalized).unwrap_or(normalized);
        let id = SourceId(
            canonical
                .to_str()
                .ok_or("Source path must be valid UTF-8.")?
                .into(),
        );
        // Recheck closed paths even when the editor's watch notification is delayed.
        // Tracking the lexical symlink path detects replacement/retarget as well.
        if !self.open.contains_key(&id) {
            let lexical = SourceId(
                path.to_str()
                    .ok_or("Source path must be valid UTF-8.")?
                    .into(),
            );
            let stamp = disk_stamp(&lexical);
            self.disk.borrow_mut().insert(lexical, stamp);
        }
        // Observe attempted missing imports too: opening a new buffer must invalidate failures.
        self.observed.borrow_mut().insert(id.clone());
        if self.open.contains_key(&id) || Path::new(&id.0).is_file() {
            Ok(id)
        } else {
            Err(format!("Missing source {}", id.0))
        }
    }
}
