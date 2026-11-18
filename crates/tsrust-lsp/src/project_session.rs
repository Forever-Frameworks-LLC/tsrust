//! Protocol-side project lifecycle and publication ownership.
use crate::{
    Snapshot,
    project_diagnostics::{file_uri, source_id, source_path_id},
    workspace::{Workspace, WorkspaceSnapshot},
};
use lsp_types::Diagnostic;
use std::collections::{BTreeMap, BTreeSet};
use tsrust_compiler::project::SourceId;

pub type Buckets = BTreeMap<SourceId, Vec<Diagnostic>>;
pub struct Publication {
    pub uri: String,
    pub version: Option<i32>,
    pub diagnostics: Vec<Diagnostic>,
}
#[derive(Default)]
pub struct ProjectSession {
    pub workspace: Workspace,
    uris: BTreeMap<SourceId, String>,
    contributions: BTreeMap<SourceId, Buckets>,
    indices: BTreeMap<SourceId, (WorkspaceSnapshot, crate::project_intelligence::ProjectIndex)>,
    actions: BTreeMap<SourceId, (WorkspaceSnapshot, crate::project_actions::ProjectActions)>,
    buffers: BTreeMap<String, (SourceId, Snapshot, u64)>,
    sequence: u64,
    /// Indices built by a full check. Declaration indices answer early but cannot
    /// know types inferred from expressions, such as members of `await connect()`.
    checked_indices: std::collections::BTreeSet<SourceId>,
}
impl ProjectSession {
    // Invalid results cannot answer requests; release their graphs before a replacement check.
    fn release_stale(&mut self) {
        self.indices.retain(|_, (input, _)| self.workspace.current(input));
        self.actions.retain(|_, (input, _)| self.workspace.current(input));
    }
    pub fn publication(&self, id: &SourceId, version: Option<i32>, diagnostics: Vec<Diagnostic>) -> Option<Publication> {
        let uri=self.uris.get(id).cloned().or_else(||file_uri(id).ok().map(|uri|uri.to_string()))?;
        Some(Publication {uri,version,diagnostics})
    }
    pub fn cache_actions(
        &mut self,
        uri: &str,
        input: &WorkspaceSnapshot,
        actions: Option<crate::project_actions::ProjectActions>,
    ) {
        let Ok(id) = source_id(uri) else {
            return;
        };
        self.release_stale();
        self.actions.remove(&id);
        if self.workspace.current(input)
            && let Some(actions) = actions
        {
            self.actions.insert(id, (input.retained(), actions));
        }
    }
    pub fn code_actions(
        &self,
        uri: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let id = source_id(uri)?;
        let mut output = vec![];
        for (_, actions) in self
            .actions
            .values()
            .filter(|(input, _)| self.workspace.current(input))
        {
            for mut action in actions.request(&id, params)? {
                if let Some(changes) = action
                    .pointer_mut("/edit/documentChanges")
                    .and_then(serde_json::Value::as_array_mut)
                {
                    for change in changes {
                        if let Some(uri) = change
                            .pointer("/textDocument/uri")
                            .and_then(serde_json::Value::as_str)
                            && let Ok(id) = source_id(uri)
                            && let Some(open_uri) = self.uris.get(&id)
                        {
                            change["textDocument"]["uri"] = serde_json::json!(open_uri);
                        }
                    }
                }
                if !output.contains(&action) {
                    output.push(action);
                }
            }
        }
        Ok(serde_json::json!(output))
    }
    pub fn cache_index(
        &mut self,
        uri: &str,
        input: &WorkspaceSnapshot,
        index: Option<crate::project_intelligence::ProjectIndex>,
    ) {
        let Ok(id) = source_id(uri) else {
            return;
        };
        self.release_stale();
        self.indices.remove(&id);
        self.checked_indices.remove(&id);
        if self.workspace.current(input)
            && let Some(mut index) = index
        {
            use tsrust_compiler::project::SourceProvider;
            if let Ok(logical)=input.resolve(None,&id.0){index.alias_entry(id.clone(),logical);}
            self.indices.insert(id, (input.retained(), index));
        }
    }
    /// `cache_index` for an index built by a full check of the project.
    pub fn cache_checked_index(&mut self, uri: &str, input: &WorkspaceSnapshot, index: Option<crate::project_intelligence::ProjectIndex>) {
        self.cache_index(uri, input, index);
        if let Ok(id) = source_id(uri)
            && self.indices.contains_key(&id)
        {
            self.checked_indices.insert(id);
        }
    }
    /// Like `covers`, counting only indices built by a full check.
    pub fn covers_checked(&self, uri: &str) -> bool {
        source_id(uri).is_ok_and(|id| {
            self.indices.iter().any(|(entry, (input, index))| {
                self.checked_indices.contains(entry) && self.workspace.current(input) && index.contains(&id)
            })
        })
    }
    /// Whether the current indices can answer this request. Completion of a member
    /// needs the receiver's inferred type, which a declaration index does not
    /// have, so while this document's own full check is running a completion
    /// waits for it. A check of another document never delays the answer.
    pub fn covers_for(&self, method: &str, uri: &str, own_check_running: bool) -> bool {
        if method == "textDocument/completion" && own_check_running { self.covers_checked(uri) } else { self.covers(uri) }
    }
    /// A project index built from the current text of every source it read
    /// includes this document.
    pub fn covers(&self, uri: &str) -> bool {
        source_id(uri).is_ok_and(|id| {
            self.indices
                .values()
                .any(|(input, index)| self.workspace.current(input) && index.contains(&id))
        })
    }
    pub fn request(
        &self,
        method: &str,
        params: &serde_json::Value,
        uri: &str,
    ) -> Option<Result<serde_json::Value, String>> {
        if !matches!(
            method,
            "textDocument/hover"
                | "textDocument/completion"
                | "textDocument/signatureHelp"
                | "textDocument/definition"
                | "textDocument/references"
        ) {
            return None;
        }
        self.indices
            .values()
            .filter(|(input, _)| self.workspace.current(input))
            .find_map(|(_, index)| index.request(method, params, uri))
            .map(|result| {
                result.map(|mut value| {
                    if matches!(
                        method,
                        "textDocument/definition" | "textDocument/references"
                    ) {
                        self.open_location_uris(&mut value);
                    }
                    value
                })
            })
    }
    fn open_location_uris(&self, value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Array(values) => {
                for value in values {
                    self.open_location_uris(value);
                }
            }
            serde_json::Value::Object(object) => {
                if let Some(uri) = object.get("uri").and_then(serde_json::Value::as_str)
                    && let Ok(id) = source_id(uri)
                    && let Some(open_uri) = self.uris.get(&id)
                {
                    object.insert("uri".into(), serde_json::json!(open_uri));
                }
            }
            _ => {}
        }
    }
    pub fn changed_file(&mut self, uri: &str) -> Vec<String> {
        let Ok(id) = source_id(uri) else {
            return vec![];
        };
        let mut affected = BTreeSet::new();
        let mut ids = BTreeSet::from([id]);
        if let Ok(path) = source_path_id(uri) {
            ids.insert(path);
        }
        for id in ids {
            self.workspace.invalidate_file(&id);
            affected.extend(self.affected(&id));
        }
        self.release_stale();
        affected.into_iter().collect()
    }
    pub fn forget_project(&mut self, uri: &str) -> Vec<Publication> {
        let Ok(id) = source_id(uri) else {
            return vec![];
        };
        self.workspace.remove_entry(&id);
        self.indices.remove(&id);
        let old = self.contributions.remove(&id).unwrap_or_default();
        self.actions.remove(&id);
        self.publications(old.into_keys().collect())
    }
    pub fn open(&mut self, snapshot: &Snapshot) -> Vec<String> {
        let Ok(id) = source_id(&snapshot.uri) else {
            return vec![];
        };
        self.uris.insert(id.clone(), snapshot.uri.clone());
        self.sequence += 1;
        self.buffers.insert(
            snapshot.uri.clone(),
            (id.clone(), snapshot.clone(), self.sequence),
        );
        self.workspace
            .open(id.clone(), snapshot.version, snapshot.text.clone());
        self.release_stale();
        self.affected(&id)
    }
    pub fn change(&mut self, snapshot: &Snapshot) -> Vec<String> {
        let Some((id, previous, _)) = self.buffers.get(&snapshot.uri) else {
            return vec![];
        };
        if snapshot.version <= previous.version {
            return vec![];
        }
        let id = id.clone();
        self.sequence += 1;
        self.buffers.insert(
            snapshot.uri.clone(),
            (id.clone(), snapshot.clone(), self.sequence),
        );
        self.uris.insert(id.clone(), snapshot.uri.clone());
        // Versions belong to URI buffers, not to the shared canonical file identity.
        self.workspace
            .open(id.clone(), snapshot.version, snapshot.text.clone());
        self.release_stale();
        self.affected(&id)
    }
    pub fn close(&mut self, uri: &str) -> (Vec<String>, Vec<Publication>) {
        let Some((id, _, _)) = self.buffers.remove(uri) else {
            return (vec![], vec![]);
        };
        if let Some((_, snapshot, _)) = self
            .buffers
            .values()
            .filter(|(other, _, _)| other == &id)
            .max_by_key(|(_, _, sequence)| sequence)
        {
            self.uris.insert(id.clone(), snapshot.uri.clone());
            self.workspace
                .open(id.clone(), snapshot.version, snapshot.text.clone());
            self.release_stale();
            return (self.affected(&id), vec![]);
        }
        self.workspace.close(&id);
        self.uris.remove(&id);
        self.workspace.remove_entry(&id);
        self.indices.remove(&id);
        let old = self.contributions.remove(&id).unwrap_or_default();
        self.actions.remove(&id);
        self.release_stale();
        (
            self.affected(&id),
            self.publications(old.keys().cloned().collect()),
        )
    }
    fn affected(&self, id: &SourceId) -> Vec<String> {
        self.workspace
            .affected(id)
            .iter()
            .filter_map(|entry| self.uris.get(entry).cloned())
            .collect()
    }
    pub fn prepare(&self, snapshot: &Snapshot) -> Option<WorkspaceSnapshot> {
        if source_id(&snapshot.uri).is_ok()
            && tsrust_compiler::project_cli::uses_modules(&snapshot.text).unwrap_or(false)
        {
            Some(self.workspace.snapshot())
        } else {
            None
        }
    }
    pub fn accept(
        &mut self,
        uri: &str,
        snapshot: &WorkspaceSnapshot,
        buckets: Buckets,
    ) -> Result<Vec<Publication>, String> {
        let entry = source_id(uri)?;
        if !self.workspace.register(entry.clone(), snapshot) {
            return Ok(vec![]);
        }
        let mut affected = buckets.keys().cloned().collect::<BTreeSet<_>>();
        if let Some(old) = self.contributions.insert(entry, buckets) {
            affected.extend(old.into_keys());
        }
        Ok(self.publications(affected))
    }
    fn publications(&self, affected: BTreeSet<SourceId>) -> Vec<Publication> {
        let snapshot = self.workspace.snapshot();
        affected
            .into_iter()
            .filter_map(|id| {
                let uri = self
                    .uris
                    .get(&id)
                    .cloned()
                    .or_else(|| file_uri(&id).ok().map(|uri| uri.to_string()))?;
                let mut diagnostics = vec![];
                for bucket in self.contributions.values() {
                    if let Some(values) = bucket.get(&id) {
                        for value in values {
                            if !diagnostics.contains(value) {
                                diagnostics.push(value.clone());
                            }
                        }
                    }
                }
                Some(Publication {
                    uri,
                    version: snapshot.version(&id),
                    diagnostics,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod retention_tests {
    use super::*;
    use crate::{project_actions::ProjectActions, project_intelligence::ProjectIndex};
    use tsrust_compiler::project::load_project_headers;

    #[test]
    fn invalidation_releases_old_graphs_and_actions_without_evicting_current_projects() {
        let root = std::env::temp_dir().canonicalize().unwrap().join("tsrust-session-retention");
        let make = |name: &str, text: &str| Snapshot {
            uri: file_uri(&SourceId(root.join(name).to_string_lossy().into_owned())).unwrap().to_string(),
            version: 1, generation: 1, text: text.into(),
        };
        let mut session = ProjectSession::default();
        let mut dep = make("dep.tsr", "export function value():int{return 1;}");
        let mut entry = make("entry.tsr", "import {value} from './dep.tsr';function main(){console.log(value());}");
        let other = make("other.tsr", "export function other():int{return 2;}");
        for document in [&dep, &entry, &other] { session.open(document); }
        let other_id = source_id(&other.uri).unwrap();
        let input = session.workspace.snapshot();
        let graph = load_project_headers(&input, &other_id.0).unwrap();
        session.cache_index(&other.uri, &input, Some(ProjectIndex::new(&graph)));
        drop(input);
        let id = source_id(&entry.uri).unwrap();
        let dep_id = source_id(&dep.uri).unwrap();
        for version in 2..22 {
            let old_text = {
                let input = session.prepare(&entry).unwrap();
                let graph = load_project_headers(&input, &id.0).unwrap();
                let old_text = std::sync::Arc::downgrade(&input.sources()[&dep_id]);
                // A diagnostic with no fix candidates still owns its observation snapshot.
                let error = tsrust_compiler::project::ProjectDiagnostic {
                    code: "test", message: "retention witness".into(),
                    location: tsrust_compiler::project::SourceLocation { source: id.clone(), span: tsrust_compiler::Span::new(0, 1) },
                    related: vec![], fixes: Box::new([]),
                };
                let actions = ProjectActions::build(&input, &id, &error).unwrap();
                session.cache_index(&entry.uri, &input, Some(ProjectIndex::new(&graph)));
                session.cache_actions(&entry.uri, &input, Some(actions));
                assert_eq!(session.indices.len(), 2);
                assert_eq!(session.actions.len(), 1);
                old_text
            };
            assert!(old_text.upgrade().is_some());
            dep.version = version;
            dep.text = format!("export function value():int{{return {version};}}");
            session.change(&dep);
            assert_eq!(session.indices.len(), 1);
            assert!(session.indices.contains_key(&other_id));
            assert!(session.actions.is_empty());
            assert!(old_text.upgrade().is_none(), "old dependency snapshot survives edit {version}");
        }
        // A watched-file change must release the same caches before analysis finishes.
        let input = session.prepare(&entry).unwrap();
        let graph = load_project_headers(&input, &id.0).unwrap();
        session.cache_index(&entry.uri, &input, Some(ProjectIndex::new(&graph)));
        session.changed_file(&dep.uri);
        assert_eq!(session.indices.len(), 1);
        entry.version += 1;
        session.change(&entry);
        session.close(&other.uri);
        assert!(session.indices.is_empty());
    }
}
