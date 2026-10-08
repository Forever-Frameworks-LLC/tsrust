//! Point-sensitive owning storage availability at checked body exits.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct Exits(HashMap<(usize, usize), Vec<Unavailable>>, HashMap<usize, Vec<Unavailable>>, HashMap<(usize, usize), Vec<Unavailable>>, Option<Box<HashMap<(usize,usize),MapEntrySurvivors>>>, Option<Box<HashMap<(usize,usize),Vec<MapEntryEpoch>>>>, Option<Box<HashMap<(usize,usize),Type>>>);
#[derive(Clone, Debug)]
pub(crate) struct MapEntryEpoch {
    pub(crate) place: crate::capture_loan_owners::Place,
    pub(crate) input: Type,
    pub(crate) output: Type,
    pub(crate) empty: bool,
    pub(crate) before: bool,
}
#[derive(Clone, Debug)]
pub(crate) struct MapEntrySurvivors {
    pub(crate) place: crate::capture_loan_owners::Place,
    pub(crate) input: Type,
    pub(crate) payloads: HashSet<Type>,
    pub(crate) consistent: bool,
}

#[cfg(test)]
mod map_entry_tests {
    use super::*;
    #[test]
    fn replayed_entry_release_unions_survivors_and_invalidates_changed_input() {
        let mut exits=Exits::default();
        let span=Span::new(10,20);
        let place=crate::capture_loan_owners::Place{binding:1,fields:vec![]};
        exits.record_map_entry_survivors(span,place.clone(),Type::Map(3),HashSet::from([Type::Record(4)]));
        exits.record_map_entry_survivors(span,place.clone(),Type::Map(3),HashSet::from([Type::Record(5)]));
        let state=exits.map_entry_survivors(span).unwrap();
        assert!(state.consistent);
        assert_eq!(state.payloads,HashSet::from([Type::Record(4),Type::Record(5)]));
        exits.record_map_entry_survivors(span,place,Type::Map(6),HashSet::new());
        assert!(!exits.map_entry_survivors(span).unwrap().consistent);
    }
}
#[derive(Clone, Debug)]
struct Unavailable { binding: usize, root: Option<Span>, fields: Vec<(Vec<String>, Span)> }
impl Exits {
    pub(super) fn record_map_entry_write(&mut self, span: Span, ty:Type) {
        self.5.get_or_insert_with(Default::default).insert((span.start,span.end),ty);
    }
    pub(crate) fn map_entry_write(&self, span:Span)->Option<Type> {
        self.5.as_ref()?.get(&(span.start,span.end)).copied()
    }
    pub(super) fn has_map_entry_survivors(&self) -> bool { self.3.is_some() }
    pub(super) fn record_map_entry_survivors(&mut self, span: Span, place: crate::capture_loan_owners::Place, input: Type, payloads: HashSet<Type>) {
        let points=self.3.get_or_insert_with(Default::default);
        points.entry((span.start,span.end)).and_modify(|previous| {
            previous.consistent &= previous.place==place && previous.input==input;
            previous.payloads.extend(payloads.iter().copied());
        }).or_insert(MapEntrySurvivors{place,input,payloads,consistent:true});
    }
    pub(crate) fn map_entry_survivors(&self, span: Span) -> Option<&MapEntrySurvivors> {
        self.3.as_ref()?.get(&(span.start,span.end))
    }
    pub(super) fn record_map_entry_epoch(&mut self, span: Span, epoch: MapEntryEpoch) {
        let epochs=self.4.get_or_insert_with(Default::default).entry((span.start,span.end)).or_default();
        if let Some(previous)=epochs.iter_mut().find(|previous|previous.place==epoch.place && previous.before==epoch.before) {*previous=epoch;}
        else {epochs.push(epoch);}
    }
    pub(crate) fn map_entry_epochs(&self, span: Span) -> &[MapEntryEpoch] {
        self.4.as_ref().and_then(|points|points.get(&(span.start,span.end))).map(Vec::as_slice).unwrap_or(&[])
    }
    pub(crate) fn map_entry_epoch_root(&self, binding:usize)->bool {
        self.4.as_ref().is_some_and(|points|points.values().flatten().any(|epoch|epoch.place.binding==binding))
    }
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
