//! Point-sensitive owning storage availability at checked body exits.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct Exits(HashMap<(usize, usize), Vec<Unavailable>>);
#[derive(Clone, Debug)]
struct Unavailable { binding: usize, root: bool, fields: Vec<Vec<String>> }
impl Exits {
    pub(super) fn record(&mut self, body: &[Stmt], bindings: &[Binding]) {
        let (Some(first), Some(last)) = (body.first(), body.last()) else { return; };
        let entries = bindings.iter().enumerate().filter_map(|(binding, value)| {
            (value.transferred.is_some() || !value.moved_fields.is_empty()).then(|| Unavailable {
                binding, root: value.transferred.is_some(), fields: value.moved_fields.keys().cloned().collect(),
            })
        }).collect();
        self.0.insert((first.span.start, last.span.end), entries);
    }
    pub(crate) fn live(&self, body: &[Stmt], binding: usize, path: &[String]) -> bool {
        let (Some(first), Some(last)) = (body.first(), body.last()) else { return true; };
        self.0.get(&(first.span.start, last.span.end)).is_none_or(|entries| {
            !entries.iter().any(|entry| entry.binding == binding && (entry.root || entry.fields.iter().any(|field| path.starts_with(field))))
        })
    }
}
