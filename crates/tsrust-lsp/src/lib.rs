//! Single-file editor diagnostics. The compiler owns all language semantics.
pub mod intelligence;
pub mod project_actions;
pub mod project_diagnostics;
pub mod project_intelligence;
pub mod project_session;
pub mod workspace;

use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
    Position, Range, Uri,
};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tsrust_compiler::Span;

/// Indexes the exact immutable compiler input; LSP positions are UTF-16.
pub struct LineIndex {
    starts: Vec<usize>,
}
impl LineIndex {
    pub fn new(source: &str) -> Self {
        let mut starts = vec![0];
        starts.extend(source.match_indices('\n').map(|(offset, _)| offset + 1));
        Self { starts }
    }
    fn position(&self, source: &str, offset: usize) -> Result<Position, String> {
        if offset > source.len() || !source.is_char_boundary(offset) {
            return Err(format!("invalid compiler byte offset {offset}"));
        }
        let line = self.starts.partition_point(|&start| start <= offset) - 1;
        let prefix = &source[self.starts[line]..offset];
        // CRLF line endings occupy bytes but are not part of the line's text.
        let prefix = prefix.strip_suffix('\r').unwrap_or(prefix);
        Ok(Position::new(
            u32::try_from(line).map_err(|_| "line count exceeds LSP limit")?,
            u32::try_from(prefix.encode_utf16().count()).map_err(|_| "line exceeds LSP limit")?,
        ))
    }
    pub fn range(&self, source: &str, span: Span) -> Result<Range, String> {
        if span.start > span.end {
            return Err("reversed compiler source span".into());
        }
        Ok(Range::new(
            self.position(source, span.start)?,
            self.position(source, span.end)?,
        ))
    }
}

pub fn diagnostics(source: &str, uri: &str) -> Result<Vec<Diagnostic>, String> {
    match tsrust_compiler::check(source) {
        Ok(_) => Ok(vec![]),
        Err(error) => {
            let index = LineIndex::new(source);
            let uri: Uri = uri.parse().map_err(|_| "invalid document URI")?;
            let related = error
                .related
                .into_iter()
                .map(|(span, message)| {
                    Ok(DiagnosticRelatedInformation {
                        location: Location::new(uri.clone(), index.range(source, span)?),
                        message,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok(vec![Diagnostic {
                range: index.range(source, error.span)?,
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String(error.code.into())),
                source: Some("tsrust".into()),
                message: error.message,
                related_information: (!related.is_empty()).then_some(related),
                ..Diagnostic::default()
            }])
        }
    }
}

#[derive(Clone, Debug)]
pub struct Snapshot {
    pub uri: String,
    pub version: i32,
    pub generation: u64,
    pub text: String,
}
struct Document {
    snapshot: Snapshot,
    pending: Option<(Instant, u64)>,
    index: Option<intelligence::SourceIndex>,
}
/// At most one latest pending snapshot per document. A sequence ensures fairness.
#[derive(Default)]
pub struct Documents {
    open: HashMap<String, Document>,
    sequence: u64,
    generation: u64,
}
impl Documents {
    pub fn open(&mut self, uri: String, version: i32, text: String, now: Instant) {
        self.generation += 1;
        self.sequence += 1;
        self.open.insert(
            uri.clone(),
            Document {
                snapshot: Snapshot {
                    uri,
                    version,
                    generation: self.generation,
                    text,
                },
                pending: Some((now, self.sequence)),
                index: None,
            },
        );
    }
    pub fn change(
        &mut self,
        uri: &str,
        version: i32,
        text: String,
        now: Instant,
        debounce: Duration,
    ) -> bool {
        let Some(document) = self.open.get_mut(uri) else {
            return false;
        };
        if version <= document.snapshot.version {
            return false;
        }
        document.snapshot.version = version;
        document.snapshot.text = text;
        document.index = None;
        // Preserve queue order while replacing the due time and latest buffer.
        self.sequence += 1;
        let order = document.pending.map_or(self.sequence, |(_, order)| order);
        document.pending = Some((now + debounce, order));
        true
    }
    pub fn index(&mut self, uri: &str) -> Option<&intelligence::SourceIndex> {
        let document = self.open.get_mut(uri)?;
        if document.index.is_none() {
            document.index = Some(intelligence::SourceIndex::new(&document.snapshot.text));
        }
        document.index.as_ref()
    }
    /// Queue a dependency-triggered recheck without pretending the document was edited.
    /// Project result freshness also requires Workspace::current on its input snapshot.
    pub fn reschedule(&mut self, uri: &str, now: Instant, debounce: Duration) -> bool {
        let Some(document) = self.open.get_mut(uri) else {
            return false;
        };
        self.sequence += 1;
        let order = document.pending.map_or(self.sequence, |(_, order)| order);
        document.pending = Some((now + debounce, order));
        document.index = None;
        true
    }
    pub fn close(&mut self, uri: &str) {
        self.open.remove(uri);
    }
    pub fn snapshot(&self, uri: &str) -> Option<&Snapshot> {
        self.open.get(uri).map(|document| &document.snapshot)
    }
    pub fn current(&self, snapshot: &Snapshot) -> bool {
        self.open.get(&snapshot.uri).is_some_and(|document| {
            document.snapshot.version == snapshot.version
                && document.snapshot.generation == snapshot.generation
        })
    }
    pub fn next(&mut self, now: Instant) -> Option<Snapshot> {
        let uri = self
            .open
            .iter()
            .filter_map(|(uri, document)| {
                document
                    .pending
                    .filter(|(due, _)| *due <= now)
                    .map(|(_, order)| (uri.clone(), order))
            })
            .min_by_key(|(_, order)| *order)?
            .0;
        let document = self.open.get_mut(&uri)?;
        document.pending = None;
        Some(document.snapshot.clone())
    }
    pub fn due_in(&self, now: Instant) -> Option<Duration> {
        self.open
            .values()
            .filter_map(|document| {
                document
                    .pending
                    .map(|(due, _)| due.saturating_duration_since(now))
            })
            .min()
    }
    pub fn pending_count(&self) -> usize {
        self.open
            .values()
            .filter(|document| document.pending.is_some())
            .count()
    }
}

/// Recheck the current snapshot rather than trusting stale published diagnostics.
pub fn code_actions(
    snapshot: &Snapshot,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use serde_json::{Value, json};
    let range: Range =
        serde_json::from_value(params.get("range").cloned().ok_or("missing action range")?)
            .map_err(|error| error.to_string())?;
    if let Some(only) = params.pointer("/context/only").and_then(Value::as_array)
        && !only.iter().any(|kind| {
            kind.as_str()
                .is_some_and(|kind| kind.is_empty() || kind == "quickfix")
        })
    {
        return Ok(json!([]));
    }
    let Err(error) = tsrust_compiler::check(&snapshot.text) else {
        return Ok(json!([]));
    };
    let index = LineIndex::new(&snapshot.text);
    let diagnostic_range = index.range(&snapshot.text, error.span)?;
    if range.start > diagnostic_range.end || range.end < diagnostic_range.start {
        return Ok(json!([]));
    }
    let diagnostic = diagnostics(&snapshot.text, &snapshot.uri)?;
    let mut actions = Vec::new();
    for fix in error.fixes {
        if snapshot.text.get(fix.span.start..fix.span.end).is_none() {
            continue;
        }
        let clone = fix.replacement.starts_with("clone(");
        let mut fixes = vec![fix];
        let accepted = loop {
            let mut edited = snapshot.text.clone();
            fixes.sort_by_key(|fix| fix.span.start);
            for fix in fixes.iter().rev() {
                edited.replace_range(fix.span.start..fix.span.end, &fix.replacement);
            }
            match tsrust_compiler::check(&edited) {
                Ok(_) => break true,
                Err(next) if clone && next.code == "transferred" && fixes.len() < 8 => {
                    let Some(mut next) = next
                        .fixes
                        .into_iter()
                        .find(|fix| fix.replacement.starts_with("clone("))
                    else {
                        break false;
                    };
                    let Some(start) = original_offset(next.span.start, &fixes) else {
                        break false;
                    };
                    let Some(end) = original_offset(next.span.end, &fixes) else {
                        break false;
                    };
                    if fixes
                        .iter()
                        .any(|fix| start < fix.span.end && end > fix.span.start)
                    {
                        break false;
                    }
                    next.span = Span::new(start, end);
                    fixes.push(next);
                }
                Err(_) => break false,
            }
        };
        if !accepted {
            continue;
        }
        let title = if fixes.len() == 1 {
            fixes[0].title.clone()
        } else {
            format!(
                "Clone before {} transfers (copies data; may allocate)",
                fixes.len()
            )
        };
        let edits = fixes.iter().map(|fix| Ok(json!({"range":index.range(&snapshot.text, fix.span)?,"newText":fix.replacement}))).collect::<Result<Vec<_>,String>>()?;
        actions.push(json!({"title":title,"kind":"quickfix","isPreferred":!clone,
        "diagnostics":diagnostic,"edit":{"documentChanges":[{
            "textDocument":{"uri":snapshot.uri,"version":snapshot.version}, "edits":edits
        }]}}));
    }
    Ok(json!(actions))
}

// Translate follow-up compiler spans back through previously applied clone edits.
fn original_offset(offset: usize, fixes: &[tsrust_compiler::SuggestedFix]) -> Option<usize> {
    let mut delta = 0;
    for fix in fixes {
        let start = fix.span.start + delta;
        let end = start + fix.replacement.len();
        if offset <= start {
            return offset.checked_sub(delta);
        }
        if offset < end {
            return None;
        }
        delta += fix
            .replacement
            .len()
            .checked_sub(fix.span.end - fix.span.start)?;
    }
    offset.checked_sub(delta)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn utf16_ranges_and_invalid_spans() {
        let text = "aé😀e\u{301}\r\n二\n";
        let index = LineIndex::new(text);
        let range = index.range(text, Span::new(3, 10)).unwrap();
        assert_eq!(range, Range::new(Position::new(0, 2), Position::new(0, 6)));
        assert_eq!(index.position(text, 12).unwrap(), Position::new(1, 0));
        assert_eq!(
            index.position(text, text.len()).unwrap(),
            Position::new(2, 0)
        );
        assert_eq!(
            index
                .range(text, Span::new(text.len(), text.len()))
                .unwrap()
                .start,
            Position::new(2, 0)
        );
        assert!(index.range(text, Span::new(4, 6)).is_err());
        assert!(index.range(text, Span::new(0, text.len() + 1)).is_err());
        assert!(index.range(text, Span::new(3, 1)).is_err());
        assert_eq!(
            index.range(text, Span::new(0, 15)).unwrap().end,
            Position::new(1, 1)
        );
    }
    #[test]
    fn pending_work_is_bounded_fair_and_versioned() {
        let mut documents = Documents::default();
        let now = Instant::now();
        documents.open("a".into(), 1, "old".into(), now);
        let running = documents.next(now).unwrap();
        documents.open("b".into(), 1, "other".into(), now);
        for version in 2..1000 {
            assert!(documents.change("a", version, "new".into(), now, Duration::from_millis(200)));
        }
        assert_eq!(documents.pending_count(), 2);
        assert!(!documents.current(&running));
        assert!(!documents.change("a", 998, "stale".into(), now, Duration::ZERO));
        assert_eq!(documents.next(now).unwrap().uri, "b");
        assert!(documents.next(now).is_none());
        let latest = documents.next(now + Duration::from_millis(200)).unwrap();
        assert_eq!(latest.version, 999);
        assert!(documents.current(&latest));
        documents.close("a");
        documents.open("a".into(), 999, "reopened".into(), now);
        assert!(!documents.current(&latest));
        documents.close("a");
        assert!(documents.next(now).is_none());
    }
    #[test]
    fn diagnostics_match_compiler_including_related_locations() {
        let source =
            include_str!("../../tsrust-compiler/tests/fixtures/ownership/use_after_call.tsr");
        let error = tsrust_compiler::check(source).err().unwrap();
        let output = diagnostics(source, "file:///example.tsr").unwrap();
        assert_eq!(output[0].message, error.message);
        assert_eq!(
            output[0].code,
            Some(NumberOrString::String(error.code.into()))
        );
        assert_eq!(
            output[0].range,
            LineIndex::new(source).range(source, error.span).unwrap()
        );
        assert!(!error.related.is_empty());
        assert_eq!(
            output[0].related_information.as_ref().unwrap().len(),
            error.related.len()
        );
        assert!(
            diagnostics("function main() {}", "file:///example.tsr")
                .unwrap()
                .is_empty()
        );
        for source in [
            "function main( {",
            "function main() { const x: int = true; }",
        ] {
            let error = tsrust_compiler::check(source).err().unwrap();
            assert_eq!(
                diagnostics(source, "file:///example.tsr").unwrap()[0].message,
                error.message
            );
        }
    }
}
