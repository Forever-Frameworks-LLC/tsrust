//! Point-sensitive owning storage availability at checked body exits.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct Exits(HashMap<(usize, usize), Vec<Unavailable>>, HashMap<usize, Vec<Unavailable>>, HashMap<(usize, usize), Vec<Unavailable>>);
#[derive(Clone, Debug)]
struct Unavailable { binding: usize, root: bool, fields: Vec<Vec<String>> }
impl Exits {
    #[inline(never)]
    pub(super) fn record_before(&mut self, span: Span, bindings: &[Binding]) {
        let entries = bindings.iter().enumerate().filter_map(|(binding, value)| {
            (value.transferred.is_some() || !value.moved_fields.is_empty()).then(|| Unavailable {
                binding, root: value.transferred.is_some(), fields: value.moved_fields.keys().cloned().collect(),
            })
        }).collect();
        self.1.insert(span.start, entries);
    }
    #[inline(never)]
    pub(super) fn record_expression(&mut self, span: Span, bindings: &[Binding]) {
        let entries = bindings.iter().enumerate().filter_map(|(binding, value)| {
            (value.transferred.is_some() || !value.moved_fields.is_empty()).then(|| Unavailable {
                binding, root: value.transferred.is_some(), fields: value.moved_fields.keys().cloned().collect(),
            })
        }).collect();
        self.2.insert((span.start, span.end), entries);
    }
    pub(crate) fn expression_points(&self) -> impl Iterator<Item=(usize,usize)> + '_ { self.2.keys().copied() }
    pub(crate) fn live_at_expression(&self, point: (usize,usize), binding: usize, path: &[String]) -> bool {
        self.2.get(&point).is_none_or(|entries| !entries.iter().any(|entry| entry.binding == binding && (entry.root || entry.fields.iter().any(|field| path.starts_with(field)))))
    }
    pub(crate) fn starts(&self) -> impl Iterator<Item=usize> + '_ { self.1.keys().copied() }
    pub(crate) fn live_before(&self, start: usize, binding: usize, path: &[String]) -> bool {
        self.1.get(&start).is_none_or(|entries| !entries.iter().any(|entry| entry.binding == binding && (entry.root || entry.fields.iter().any(|field| path.starts_with(field)))))
    }
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
