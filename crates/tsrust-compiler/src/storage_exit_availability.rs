//! Point-sensitive owning storage availability at checked body exits.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct Exits(HashMap<(usize, usize), Vec<Unavailable>>, HashMap<usize, Vec<Unavailable>>, HashMap<(usize, usize), Vec<Unavailable>>);
#[derive(Clone, Debug)]
struct Unavailable { binding: usize, root: Option<Span>, fields: Vec<(Vec<String>, Span)> }
impl Exits {
    #[inline(never)]
    pub(super) fn record_before(&mut self, span: Span, bindings: &[Binding]) {
        let entries = bindings.iter().enumerate().filter_map(|(binding, value)| {
            (value.transferred.is_some() || !value.moved_fields.is_empty()).then(|| Unavailable {
                binding, root: value.transferred, fields: value.moved_fields.iter().map(|(path,span)|(path.clone(),*span)).collect(),
            })
        }).collect();
        self.1.insert(span.start, entries);
    }
    #[inline(never)]
    pub(super) fn record_expression(&mut self, span: Span, bindings: &[Binding]) {
        let entries = bindings.iter().enumerate().filter_map(|(binding, value)| {
            (value.transferred.is_some() || !value.moved_fields.is_empty()).then(|| Unavailable {
                binding, root: value.transferred, fields: value.moved_fields.iter().map(|(path,span)|(path.clone(),*span)).collect(),
            })
        }).collect();
        self.2.insert((span.start, span.end), entries);
    }
    pub(crate) fn expression_points(&self) -> impl Iterator<Item=(usize,usize)> + '_ { self.2.keys().copied() }
    pub(crate) fn starts(&self) -> impl Iterator<Item=usize> + '_ { self.1.keys().copied() }
    pub(super) fn record(&mut self, body: &[Stmt], bindings: &[Binding]) {
        let (Some(first), Some(last)) = (body.first(), body.last()) else { return; };
        let entries = bindings.iter().enumerate().filter_map(|(binding, value)| {
            (value.transferred.is_some() || !value.moved_fields.is_empty()).then(|| Unavailable {
                binding, root: value.transferred, fields: value.moved_fields.iter().map(|(path,span)|(path.clone(),*span)).collect(),
            })
        }).collect();
        self.0.insert((first.span.start, last.span.end), entries);
    }
    fn transfer(entries: Option<&Vec<Unavailable>>, binding: usize, path: &[String]) -> Option<Span> {
        let entry = entries?.iter().find(|entry|entry.binding==binding)?;
        entry.root.or_else(||entry.fields.iter().find(|(field,_)|path.starts_with(field)).map(|(_,span)|*span))
    }
    pub(crate) fn transfer_before(&self, start: usize, binding: usize, path: &[String]) -> Option<Span> {
        Self::transfer(self.1.get(&start),binding,path)
    }
    pub(crate) fn transfer_at_expression(&self, point: (usize,usize), binding: usize, path: &[String]) -> Option<Span> {
        Self::transfer(self.2.get(&point),binding,path)
    }
    pub(crate) fn transfer_at_exit(&self, body: &[Stmt], binding: usize, path: &[String]) -> Option<Span> {
        let (Some(first),Some(last))=(body.first(),body.last()) else {return None;};
        Self::transfer(self.0.get(&(first.span.start,last.span.end)),binding,path)
    }
}
