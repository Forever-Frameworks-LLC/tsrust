//! Point-sensitive owning storage availability at checked body exits.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct Exits(HashMap<(usize, usize), Vec<Unavailable>>, HashMap<usize, Vec<Unavailable>>, HashMap<(usize, usize), Vec<Unavailable>>, Option<Box<HashMap<(usize,usize),MapEntrySurvivors>>>, Option<Box<HashMap<(usize,usize),Vec<MapEntryEpoch>>>>, Option<Box<HashMap<(usize,usize),Type>>>, Option<Box<HashMap<(usize,usize),Vec<MapEntrySurvivors>>>>, Option<Box<HashMap<(usize,usize,bool),Vec<MapEntryEpoch>>>>, Option<Box<HashMap<(usize,usize,MapLoopEdge),std::collections::BTreeMap<crate::capture_loan_owners::Place,HashSet<Type>>>>>, Option<Box<HashMap<(usize,usize),Vec<MapIterationEpoch>>>>, Option<Box<HashMap<(usize,usize),HashMap<usize,Type>>>>, Option<Box<HashMap<(usize,usize),(usize,bool,Vec<MapEntryEpoch>)>>>);
#[derive(Clone,Debug)]
pub(crate) struct MapIterationEpoch {
    pub(crate) entry:MapEntryEpoch,
    pub(crate) back:MapEntryEpoch,
    pub(crate) body:MapEntryEpoch,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,Hash)]
pub(crate) enum MapLoopEdge { Entry, Back, Break }
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
    pub(crate) released: HashSet<usize>,
}

#[cfg(test)]
mod map_entry_tests {
    use super::*;
    #[test]
    fn loop_entry_backedge_and_break_payloads_remain_distinct() {
        use crate::capture_loan_owners::{Owners,Place};
        let span=Span::new(30,40);
        let root=Place{binding:2,fields:vec!["handlers".into()]};
        let first=Place{binding:2,fields:vec!["handlers".into(),"\0map-entry:string:first".into()]};
        let second=Place{binding:2,fields:vec!["handlers".into(),"\0map-entry:string:second".into()]};
        let mut owners=Owners::default();
        owners.attach_map_payload(first.clone(),Type::Record(10));
        owners.attach_map_payload(second.clone(),Type::Record(11));
        let mut exits=Exits::default();
        exits.record_map_loop_edge(span,MapLoopEdge::Entry,&owners);
        exits.record_map_loop_edge(span,MapLoopEdge::Break,&owners);
        owners.discard(&first);
        exits.record_map_loop_edge(span,MapLoopEdge::Back,&owners);
        assert_eq!(exits.map_loop_edge(span,MapLoopEdge::Entry,&root),Some(&HashSet::from([Type::Record(10),Type::Record(11)])));
        assert_eq!(exits.map_loop_edge(span,MapLoopEdge::Break,&root),exits.map_loop_edge(span,MapLoopEdge::Entry,&root));
        assert_eq!(exits.map_loop_edge(span,MapLoopEdge::Back,&root),Some(&HashSet::from([Type::Record(11)])));
        owners.attach_map_payload(first,Type::Record(12));
        exits.record_map_loop_edge(span,MapLoopEdge::Back,&owners);
        assert_eq!(exits.map_loop_edge(span,MapLoopEdge::Back,&root),Some(&HashSet::from([Type::Record(11),Type::Record(12)])));
        assert!(!exits.map_loop_edge(span,MapLoopEdge::Entry,&root).unwrap().contains(&Type::Record(12)));
        owners.discard(&root);
        let empty_span=Span::new(50,60);
        exits.record_map_loop_edge(empty_span,MapLoopEdge::Back,&owners);
        assert!(exits.has_map_loop_edge(empty_span,MapLoopEdge::Back));
        assert!(!exits.has_map_loop_edge(empty_span,MapLoopEdge::Break));
        assert!(exits.map_loop_edge(empty_span,MapLoopEdge::Back,&root).is_none());
    }
    #[test]
    fn replayed_entry_release_unions_survivors_and_invalidates_changed_input() {
        let mut exits=Exits::default();
        let span=Span::new(10,20);
        let place=crate::capture_loan_owners::Place{binding:1,fields:vec![]};
        exits.record_map_entry_survivors(span,place.clone(),Type::Map(3),HashSet::from([Type::Record(4)]),HashSet::from([7]));
        exits.record_map_entry_survivors(span,place.clone(),Type::Map(3),HashSet::from([Type::Record(5)]),HashSet::from([8]));
        let state=exits.map_entry_survivors(span).unwrap();
        assert!(state.consistent);
        assert_eq!(state.released,HashSet::from([7,8]));
        assert_eq!(state.payloads,HashSet::from([Type::Record(4),Type::Record(5)]));
        exits.record_map_entry_survivors(span,place,Type::Map(6),HashSet::new(),HashSet::new());
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
    pub(super) fn record_map_entry_survivors(&mut self, span: Span, place: crate::capture_loan_owners::Place, input: Type, payloads: HashSet<Type>, released: HashSet<usize>) {
        let points=self.3.get_or_insert_with(Default::default);
        points.entry((span.start,span.end)).and_modify(|previous| {
            previous.consistent &= previous.place==place && previous.input==input;
            previous.payloads.extend(payloads.iter().copied());
            previous.released.extend(released.iter().copied());
        }).or_insert(MapEntrySurvivors{place,input,payloads,consistent:true,released});
    }
    pub(super) fn record_map_completion_survivors(&mut self,span:Span,state:MapEntrySurvivors) {
        let states=self.6.get_or_insert_with(Default::default).entry((span.start,span.end)).or_default();
        if let Some(previous)=states.iter_mut().find(|previous|previous.place==state.place) {
            previous.consistent &= previous.input==state.input;
            previous.payloads.extend(state.payloads);
            previous.released.extend(state.released);
        } else {states.push(state);}
    }
    pub(crate) fn map_completion_survivors(&self,span:Span)->&[MapEntrySurvivors] {
        self.6.as_ref().and_then(|points|points.get(&(span.start,span.end))).map(Vec::as_slice).unwrap_or(&[])
    }
    pub(crate) fn map_entry_survivors(&self, span: Span) -> Option<&MapEntrySurvivors> {
        self.3.as_ref()?.get(&(span.start,span.end))
    }
    pub(super) fn record_map_control_inputs(&mut self,span:Span,inputs:&HashMap<usize,Type>) {
        self.10.get_or_insert_with(Default::default).insert((span.start,span.end),inputs.clone());
    }
    pub(super) fn map_control_inputs(&self,span:Span)->Option<&HashMap<usize,Type>> {
        self.10.as_ref()?.get(&(span.start,span.end))
    }
    pub(super) fn record_map_iteration_control(&mut self,span:Span,label:usize,continuing:bool,epochs:Vec<MapEntryEpoch>) {
        self.11.get_or_insert_with(Default::default).insert((span.start,span.end),(label,continuing,epochs));
    }
    pub(crate) fn has_map_iteration_controls(&self,label:usize)->bool {
        self.11.as_ref().is_some_and(|points|points.values().any(|(target,_,_)|*target==label))
    }
    pub(crate) fn map_iteration_control(&self,span:Span)->Option<&(usize,bool,Vec<MapEntryEpoch>)> {
        self.11.as_ref()?.get(&(span.start,span.end))
    }
    pub(super) fn record_map_iteration_epochs(&mut self,span:Span,epochs:Vec<MapIterationEpoch>) {
        self.9.get_or_insert_with(Default::default).insert((span.start,span.end),epochs);
    }
    pub(crate) fn map_iteration_epochs(&self,span:Span)->&[MapIterationEpoch] {
        self.9.as_ref().and_then(|points|points.get(&(span.start,span.end))).map(Vec::as_slice).unwrap_or(&[])
    }
    pub(super) fn record_map_loop_edge(&mut self,span:Span,edge:MapLoopEdge,owners:&crate::capture_loan_owners::Owners) {
        let snapshot=owners.map_payload_snapshot();
        if snapshot.is_empty() && self.8.is_none() && !self.has_map_entry_survivors(){return;}
        let roots=self.8.get_or_insert_with(Default::default).entry((span.start,span.end,edge)).or_default();
        for (place,payloads) in snapshot {roots.entry(place).or_default().extend(payloads);}
    }
    pub(crate) fn has_map_loop_edge(&self,span:Span,edge:MapLoopEdge)->bool {
        self.8.as_ref().is_some_and(|points|points.contains_key(&(span.start,span.end,edge)))
    }
    pub(crate) fn map_loop_edge(&self,span:Span,edge:MapLoopEdge,place:&crate::capture_loan_owners::Place)->Option<&HashSet<Type>> {
        self.8.as_ref()?.get(&(span.start,span.end,edge))?.get(place)
    }
    pub(super) fn record_map_branch_restore(&mut self,span:Span,right:bool,epochs:Vec<MapEntryEpoch>) {
        self.7.get_or_insert_with(Default::default).insert((span.start,span.end,right),epochs);
    }
    pub(crate) fn map_branch_restore(&self,span:Span,right:bool)->&[MapEntryEpoch] {
        self.7.as_ref().and_then(|points|points.get(&(span.start,span.end,right))).map(Vec::as_slice).unwrap_or(&[])
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
