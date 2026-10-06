//! Validated project edits, computed off the protocol loop from pinned source inputs.
use crate::{LineIndex, project_diagnostics::file_uri, workspace::WorkspaceSnapshot};
use lsp_types::Range;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use tsrust_compiler::{
    Span,
    project::{ProjectDiagnostic, ProjectFix, SourceId, SourceProvider, check_graph, load_project},
};

pub struct ProjectActions {
    source: SourceId,
    range: Range,
    actions: Vec<Value>,
}
struct Overlay<'a> {
    snapshot: &'a WorkspaceSnapshot,
    sources: BTreeMap<SourceId, String>,
}
impl SourceProvider for Overlay<'_> {
    fn resolve(&self, importer: Option<&SourceId>, path: &str) -> Result<SourceId, String> {
        self.snapshot.resolve(importer, path)
    }
    fn read(&self, id: &SourceId) -> Result<String, String> {
        self.sources
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Unpinned source {}", id.0))
    }
}
fn original_offset(offset: usize, source: &SourceId, fixes: &[ProjectFix]) -> Option<usize> {
    let mut delta = 0isize;
    for fix in fixes.iter().filter(|fix| &fix.location.source == source) {
        let span = fix.location.span;
        let start = span.start.checked_add_signed(delta)?;
        let end = start.checked_add(fix.replacement.len())?;
        if offset <= start {
            return offset.checked_add_signed(-delta);
        }
        if offset < end {
            return None;
        }
        delta += fix.replacement.len() as isize - (span.end - span.start) as isize;
    }
    offset.checked_add_signed(-delta)
}
impl ProjectActions {
    pub fn build(
        snapshot: &WorkspaceSnapshot,
        entry: &SourceId,
        error: &ProjectDiagnostic,
    ) -> Result<Self, String> {
        let sources = snapshot.sources();
        let source = sources
            .get(&error.location.source)
            .ok_or("Missing diagnostic input")?;
        let range = LineIndex::new(source).range(source, error.location.span)?;
        let mut actions = vec![];
        for first in &error.fixes {
            let clone = first.replacement.starts_with("clone(");
            let mut fixes = vec![first.clone()];
            let accepted = loop {
                fixes.sort_by(|a, b| {
                    (&a.location.source, a.location.span.start)
                        .cmp(&(&b.location.source, b.location.span.start))
                });
                let mut overlay = Overlay {
                    snapshot,
                    sources: sources
                        .iter()
                        .map(|(id, text)| (id.clone(), text.as_ref().clone()))
                        .collect(),
                };
                let mut valid = true;
                for fix in fixes.iter().rev() {
                    // Versioned open buffers protect application; closed-file edits require a later host guard.
                    if snapshot.version(&fix.location.source).is_none() {
                        valid = false;
                        break;
                    }
                    let Some(text) = overlay.sources.get_mut(&fix.location.source) else {
                        valid = false;
                        break;
                    };
                    let span = fix.location.span;
                    if text.get(span.start..span.end).is_none() {
                        valid = false;
                        break;
                    }
                    text.replace_range(span.start..span.end, &fix.replacement);
                }
                if !valid {
                    break false;
                }
                match load_project(&overlay, &entry.0).and_then(check_graph) {
                    Ok(_) => break true,
                    Err(next) if clone && next.code == "transferred" && fixes.len() < 8 => {
                        let Some(mut fix) = next
                            .fixes
                            .into_vec()
                            .into_iter()
                            .find(|fix| fix.replacement.starts_with("clone("))
                        else {
                            break false;
                        };
                        let Some(start) =
                            original_offset(fix.location.span.start, &fix.location.source, &fixes)
                        else {
                            break false;
                        };
                        let Some(end) =
                            original_offset(fix.location.span.end, &fix.location.source, &fixes)
                        else {
                            break false;
                        };
                        if fixes.iter().any(|old| {
                            old.location.source == fix.location.source
                                && start < old.location.span.end
                                && end > old.location.span.start
                        }) {
                            break false;
                        }
                        fix.location.span = Span::new(start, end);
                        fixes.push(fix);
                    }
                    Err(_) => break false,
                }
            };
            if !accepted {
                continue;
            }
            let mut changes: BTreeMap<SourceId, Vec<Value>> = BTreeMap::new();
            for fix in &fixes {
                let text = &sources[&fix.location.source];
                changes.entry(fix.location.source.clone()).or_default().push(json!({"range":LineIndex::new(text).range(text,fix.location.span)?,"newText":fix.replacement}));
            }
            let changes = changes.into_iter().map(|(id,edits)| Ok(json!({"textDocument":{"uri":file_uri(&id)?,"version":snapshot.version(&id)},"edits":edits}))).collect::<Result<Vec<_>,String>>()?;
            let title = if fixes.len() == 1 {
                first.title.clone()
            } else {
                format!(
                    "Clone before {} transfers (copies data; may allocate)",
                    fixes.len()
                )
            };
            actions.push(json!({"title":title,"kind":"quickfix","isPreferred":!clone,"edit":{"documentChanges":changes}}));
        }
        Ok(Self {
            source: error.location.source.clone(),
            range,
            actions,
        })
    }
    pub fn request(&self, source: &SourceId, params: &Value) -> Result<Vec<Value>, String> {
        let range: Range =
            serde_json::from_value(params.get("range").cloned().ok_or("missing action range")?)
                .map_err(|error| error.to_string())?;
        if source != &self.source || range.start > self.range.end || range.end < self.range.start {
            return Ok(vec![]);
        }
        if let Some(only) = params.pointer("/context/only").and_then(Value::as_array)
            && !only.iter().any(|kind| {
                kind.as_str()
                    .is_some_and(|kind| kind.is_empty() || kind == "quickfix")
            })
        {
            return Ok(vec![]);
        }
        Ok(self.actions.clone())
    }
}
