//! Point-sensitive owning storage availability at checked body exits.
use super::*;

#[derive(Clone, Debug, Default)]
pub(crate) struct Exits(HashMap<(usize, usize), Vec<Unavailable>>, HashMap<usize, Vec<Unavailable>>, HashMap<(usize, usize), Vec<Unavailable>>, Option<Box<HashMap<(usize,usize),MapEntrySurvivors>>>, Option<Box<HashMap<(usize,usize),Vec<MapEntryEpoch>>>>, Option<Box<HashMap<(usize,usize),Type>>>, Option<Box<HashMap<(usize,usize),Vec<MapEntrySurvivors>>>>, Option<Box<HashMap<(usize,usize,bool),Vec<MapEntryEpoch>>>>, Option<Box<HashMap<(usize,usize,MapLoopEdge),std::collections::BTreeMap<crate::capture_loan_owners::Place,HashSet<Type>>>>>, Option<Box<HashMap<(usize,usize),Vec<MapIterationEpoch>>>>, Option<Box<HashMap<(usize,usize),HashMap<usize,Type>>>>, Option<Box<HashMap<(usize,usize),(usize,bool,Vec<MapEntryEpoch>)>>>, Option<Box<map_completion_payloads::Snapshots>>, Option<Box<HashMap<usize,Vec<MapEntryEpoch>>>>, Option<std::sync::Arc<map_cleanup_phase_plan::Plans>>, Option<Box<HashMap<(usize,usize),Vec<MapEntryEpoch>>>>, Option<Box<HashMap<usize,Vec<MapEntryEpoch>>>>, Option<Box<HashMap<(usize,usize),Type>>>, Option<Box<discarded_sequence::Sites>>, Option<Box<HashMap<usize,Vec<CompletionOwners>>>>);
#[derive(Clone,Debug)]
pub(crate) struct MapIterationEpoch {
    /// Working family includes every reaching iteration origin, even when the
    /// incoming owner was narrowed before the loop.
    pub(crate) working: Type,
    pub carrier_empty: bool,
    pub(crate) break_empty: bool,
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
#[derive(Clone,Debug)]
struct CompletionOwners {
    phase:map_completion_payloads::Phase,
    kind:Option<crate::completion::CompletionKind>,
    bindings:usize,
    unavailable:Vec<Unavailable>,
}
impl Exits {
    pub(super) fn record_completion_owners(&mut self,boundary:usize,phase:map_completion_payloads::Phase,kind:Option<crate::completion::CompletionKind>,owners:&[Availability]) {
        let unavailable=owners.iter().enumerate().filter_map(|(binding,owner)|(owner.transferred.is_some() || !owner.fields.is_empty()).then(||Unavailable{binding,root:owner.transferred,fields:owner.fields.iter().map(|(path,span)|(path.clone(),*span)).collect()})).collect::<Vec<_>>();
        let states=self.19.get_or_insert_with(Default::default).entry(boundary).or_default();
        if let Some(previous)=states.iter_mut().find(|state|state.phase==phase && state.kind==kind) {
            previous.bindings=previous.bindings.min(owners.len());
            for entry in unavailable {
                if let Some(old)=previous.unavailable.iter_mut().find(|old|old.binding==entry.binding) {
                    old.root=old.root.or(entry.root);
                    for field in entry.fields {if !old.fields.iter().any(|(path,_)|*path==field.0){old.fields.push(field);}}
                }else{previous.unavailable.push(entry);}
            }
        }else{states.push(CompletionOwners{phase,kind,bindings:owners.len(),unavailable});}
    }
    pub(crate) fn completion_field_available(&self,boundary:usize,phase:map_completion_payloads::Phase,kind:Option<crate::completion::CompletionKind>,place:&crate::capture_loan_owners::Place)->Option<bool> {
        let state=self.19.as_ref()?.get(&boundary)?.iter().find(|state|state.phase==phase && state.kind==kind)?;
        Some(place.binding<state.bindings && !state.unavailable.iter().any(|owner|owner.binding==place.binding
            && (owner.root.is_some() || owner.fields.iter().any(|(field,_)|place.fields.starts_with(field) || field.starts_with(&place.fields)))))
    }

    pub(super) fn record_bound_sequence(&mut self, span: Span) { self.18.get_or_insert_with(Default::default).bound_sequences.insert((span.start,span.end)); }
    pub(super) fn bound_sequence(&self, span: Span) -> bool { self.18.as_ref().is_some_and(|sites|sites.bound_sequences.contains(&(span.start,span.end))) }
    pub(super) fn record_sequence_export(&mut self, start: usize, binding: usize) { self.18.get_or_insert_with(Default::default).exports.insert(start,binding); }
    pub(super) fn sequence_export(&self, start: usize) -> Option<usize> { self.18.as_ref()?.exports.get(&start).copied() }
    pub(super) fn record_discarded_sequence(&mut self, span: Span) {
        self.18.get_or_insert_with(Default::default).sequences.insert((span.start,span.end));
    }
    pub(super) fn discarded_sequence(&self, span: Span) -> bool {
        self.18.as_ref().is_some_and(|sites| sites.sequences.contains(&(span.start,span.end)))
    }
    pub(super) fn record_comma_operand(&mut self, span: Span) {
        self.18.get_or_insert_with(Default::default).operands.insert((span.start,span.end));
    }
    pub(crate) fn comma_operand(&self, span: Span) -> bool {
        self.18.as_ref().is_some_and(|sites| sites.operands.contains(&(span.start,span.end)))
    }

    pub(super) fn record_array_result_write(&mut self, span: Span, ty: Type) {
        self.17.get_or_insert_with(Default::default).insert((span.start, span.end), ty);
    }
    pub(crate) fn array_result_write(&self, span: Span) -> Option<Type> {
        self.17.as_ref()?.get(&(span.start, span.end)).copied()
    }

    pub(super) fn record_array_block_epochs(&mut self, start: usize, epochs: Vec<MapEntryEpoch>) {
        self.16.get_or_insert_with(Default::default).insert(start, epochs);
    }
    pub(crate) fn array_block_epochs(&self, start: usize) -> &[MapEntryEpoch] {
        self.16.as_ref().and_then(|points| points.get(&start)).map(Vec::as_slice).unwrap_or(&[])
    }
    pub(super) fn record_array_epoch(&mut self, span: Span, epoch: MapEntryEpoch) {
        let epochs = self.15.get_or_insert_with(Default::default).entry((span.start,span.end)).or_default();
        if let Some(previous) = epochs.iter_mut().find(|previous| previous.place == epoch.place && previous.before == epoch.before) {
            *previous = epoch;
        } else { epochs.push(epoch); }
    }
    pub(crate) fn array_epochs(&self, span: Span) -> &[MapEntryEpoch] {
        self.15.as_ref().and_then(|points|points.get(&(span.start,span.end))).map(Vec::as_slice).unwrap_or(&[])
    }

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
    pub(super) fn record_map_native_exit(&mut self,boundary:usize,types:std::collections::BTreeMap<usize,Type>) {
        self.12.get_or_insert_with(Default::default).record_native_exit(boundary,types);
    }
    pub(crate) fn map_native_exit(&self,boundary:usize,binding:usize)->Option<Type> {self.12.as_ref()?.native_exit(boundary,binding)}
    pub(super) fn record_map_native_inputs(&mut self,start:usize,inputs:std::collections::BTreeMap<usize,Type>) {
        if !inputs.is_empty(){self.12.get_or_insert_with(Default::default).record_native_inputs(start,inputs);}
    }
    pub(crate) fn map_native_input(&self,start:usize,binding:usize)->Option<Type> {
        self.12.as_ref()?.native_input(start,binding)
    }
    pub(crate) fn map_cleanup_plan(&self,boundary:usize,kind:Option<crate::completion::CompletionKind>)->Option<&map_cleanup_phase_plan::Plan> {
        self.14.as_ref()?.get(boundary,kind)
    }
    pub(super) fn record_map_cleanup_plan(&mut self,boundary:usize,kind:Option<crate::completion::CompletionKind>,plan:map_cleanup_phase_plan::Plan) {
        std::sync::Arc::make_mut(self.14.get_or_insert_with(Default::default)).insert(boundary,kind,plan);
    }
    pub(super) fn reset_map_cleanup_native_plans(&mut self) {
        self.4=None;self.7=None;self.9=None;self.10=None;self.11=None;self.13=None;self.14=None;
        if let Some(states)=&mut self.12{states.clear_native();}
    }
    pub(super) fn record_map_completion_phase(&mut self,boundary:usize,phase:map_completion_payloads::Phase,kind:Option<crate::completion::CompletionKind>,payloads:std::collections::BTreeMap<crate::capture_loan_owners::Place,HashSet<Type>>) {
        self.12.get_or_insert_with(Default::default).record(boundary,phase,kind,payloads);
    }
    pub(super) fn has_map_completion_layouts(&self)->bool {self.12.as_ref().is_some_and(|states|states.has_layouts())}
    pub(super) fn map_completion_snapshots(&self,boundary:usize)->Vec<map_completion_payloads::Snapshot> {
        self.12.as_ref().map_or_else(Vec::new,|states|states.snapshots(boundary))
    }
    pub(super) fn record_map_completion_layout(&mut self,boundary:usize,phase:map_completion_payloads::Phase,kind:Option<crate::completion::CompletionKind>,types:std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>) {
        self.12.get_or_insert_with(Default::default).record_layout(boundary,phase,kind,types);
    }
    pub(crate) fn map_completion_layout(&self,boundary:usize,phase:map_completion_payloads::Phase,kind:Option<crate::completion::CompletionKind>)->Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,Type>> {
        self.12.as_ref()?.layout(boundary,phase,kind)
    }
    #[cfg(test)]
    pub(crate) fn map_completion_payloads(&self,boundary:usize,phase:map_completion_payloads::Phase,kind:Option<crate::completion::CompletionKind>)->Option<&std::collections::BTreeMap<crate::capture_loan_owners::Place,HashSet<Type>>> {
        self.12.as_ref()?.at(boundary,phase,kind)
    }
    pub(super) fn record_map_control_inputs(&mut self,span:Span,inputs:&HashMap<usize,Type>) {
        self.10.get_or_insert_with(Default::default).insert((span.start,span.end),inputs.clone());
    }
    pub(super) fn map_control_inputs(&self,span:Span)->Option<&HashMap<usize,Type>> {
        self.10.as_ref()?.get(&(span.start,span.end))
    }
    pub(super) fn record_map_block_epochs(&mut self,label:usize,epochs:Vec<MapEntryEpoch>) {
        self.13.get_or_insert_with(Default::default).insert(label,epochs);
    }
    pub(crate) fn map_block_epochs(&self,label:usize)->&[MapEntryEpoch] {
        self.13.as_ref().and_then(|points|points.get(&label)).map(Vec::as_slice).unwrap_or(&[])
    }
    pub(super) fn record_map_iteration_control(&mut self,span:Span,label:usize,continuing:bool,epochs:Vec<MapEntryEpoch>) {
        self.11.get_or_insert_with(Default::default).insert((span.start,span.end),(label,continuing,epochs));
    }
    pub(crate) fn has_map_iteration_controls(&self,label:usize)->bool {
        self.11.as_ref().is_some_and(|points|points.values().any(|(target,_,_)|*target==label)) || self.12.as_ref().is_some_and(|states|states.cleanup_loops.contains(&label))
    }
    pub(super) fn record_map_cleanup_loop(&mut self,label:usize) {self.12.get_or_insert_with(Default::default).cleanup_loops.insert(label);}
    pub(crate) fn map_iteration_epochs_at_label(&self,label:usize)->&[MapIterationEpoch] {
        self.9.as_ref().and_then(|points|points.iter().find(|((start,_),_)|*start==label)).map(|(_,epochs)|epochs.as_slice()).unwrap_or(&[])
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
    pub(crate) fn map_loop_has_break(&self,span:Span)->bool {
        self.8.as_ref().is_some_and(|points|points.contains_key(&(span.start,span.end,MapLoopEdge::Break)))
    }
    #[cfg(test)]
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
    /// Actual field transfers performed while evaluating this return, excluding
    /// prior moves and whole-root transfers already handled by initialized slots.
    pub(crate) fn partial_transfer_within(&self, point: (usize,usize), value: Span) -> bool {
        self.2.get(&point).is_some_and(|entries|entries.iter().any(|entry|entry.root.is_none()
            && entry.fields.iter().any(|(_,span)|span.start>=value.start && span.end<=value.end)))
    }
    pub(crate) fn partial_places_within(&self, point:(usize,usize), value:Span) -> Vec<crate::capture_loan_owners::Place> {
        self.2.get(&point).into_iter().flatten().filter(|entry|entry.root.is_none()).flat_map(|entry|
            entry.fields.iter().filter(move |(_,span)|span.start>=value.start && span.end<=value.end)
                .map(move |(fields,_)|crate::capture_loan_owners::Place{binding:entry.binding,fields:fields.clone()})).collect()
    }
    pub(crate) fn transfer_at_exit(&self, body: &[Stmt], binding: usize, path: &[String]) -> Option<Span> {
        let (Some(first),Some(last))=(body.first(),body.last()) else {return None;};
        Self::transfer(self.0.get(&(first.span.start,last.span.end)),binding,path)
    }
}
