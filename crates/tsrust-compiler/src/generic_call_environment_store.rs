//! Finite source allocation keys stay separate from cyclic environment edges.
use super::*;
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub(crate) struct Handle(usize);
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Alternative {
    Unknown,
    Body { declaration: Span, environment: Option<Handle> },
}
pub(super) fn canonical_alternatives(mut alternatives: Vec<Alternative>) -> Vec<Alternative> {
    alternatives.sort_by_key(|alternative| match alternative {
        Alternative::Unknown => (0,0,0,None),
        Alternative::Body { declaration, environment } => (1,declaration.start,declaration.end,environment.map(|handle|handle.0)),
    });
    alternatives.dedup();
    alternatives
}
pub(super) fn merge_alternatives(left: Option<Vec<Alternative>>, left_origins: &[body_origins::Origin], right: Option<Vec<Alternative>>, right_origins: &[body_origins::Origin]) -> Option<Vec<Alternative>> {
    if left.is_none() && right.is_none() { return None; }
    fn explicit(value: Option<Vec<Alternative>>, origins: &[body_origins::Origin]) -> Vec<Alternative> {
        value.unwrap_or_else(|| origins.iter().map(|origin| match origin {
            body_origins::Origin::Body(declaration) => Alternative::Body { declaration: *declaration, environment: None },
            _ => Alternative::Unknown,
        }).collect())
    }
    Some(canonical_alternatives(explicit(left,left_origins).into_iter().chain(explicit(right,right_origins)).collect()))
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct LeafElement {
    origins: Vec<body_origins::Origin>,
    contracts: Option<Vec<Contract>>,
    selection: Selection,
}
#[derive(Clone, PartialEq, Eq, Hash)]
struct LeafSlot {
    identity: crate::local_capture_plan::BindingId,
    origins: Vec<body_origins::Origin>,
    targets: Vec<Target>,
    selection: Selection,
    elements: Option<Vec<LeafElement>>,
    contract: Option<Contract>,
}
#[derive(PartialEq, Eq, Hash)]
struct Allocation {
    site: Span,
    producer_site: Span,
    context: Vec<(Span, Span)>,
    facts: Vec<LeafSlot>,
}
pub(super) struct Record {
    pub bindings: Vec<Binding>,
    pub edges: HashMap<crate::local_capture_plan::BindingId, Vec<Alternative>>,
    pub version: u64,
}
#[derive(Default)]
pub(super) struct Store {
    allocations: HashMap<Allocation, Handle>,
    records: Vec<Record>,
    readers: HashMap<Handle, HashSet<usize>>,
    pending_readers: HashSet<usize>,
}
fn canonical(bindings: &[Binding]) -> Option<Vec<Binding>> {
    if bindings.iter().any(|binding| binding.identity.is_none()) { return None; }
    let mut bindings = bindings.to_vec();
    bindings.sort_by_key(|binding| binding.identity);
    if bindings.windows(2).any(|pair| pair[0].identity == pair[1].identity) { return None; }
    for binding in &mut bindings {
        binding.origins = body_origins::canonical(std::mem::take(&mut binding.origins));
        let mut seen = HashSet::new();
        binding.targets.retain(|target| seen.insert(target.clone()));
    }
    Some(bindings)
}
impl Store {
    #[cfg(test)]
    pub(super) fn allocate(&mut self, site: Span, producer: Span, bindings: &[Binding]) -> Option<Handle> {
        self.allocate_context(site, producer, &[], bindings)
    }
    pub(super) fn allocate_context(&mut self, site: Span, producer: Span, context: &[(Span, Span)], bindings: &[Binding]) -> Option<Handle> {
        let bindings = canonical(bindings)?;
        let facts = bindings.iter().map(|binding| LeafSlot {
            identity: binding.identity.unwrap(), origins: binding.origins.clone(), targets: binding.targets.clone(), selection: binding.selection,
            elements: binding.elements.as_ref().map(|elements| elements.iter().map(|element| LeafElement { origins: body_origins::canonical(element.origins.clone()), contracts: element.contracts.clone(), selection: element.selection }).collect()),
            contract: binding.contract.clone(),
        }).collect();
        // Site identifies callback creation; producer identifies its producing
        // call site. Correlated environment edges are absent from this key.
        let key = Allocation { site, producer_site: producer, context: context.to_vec(), facts };
        if let Some(handle) = self.allocations.get(&key) { return Some(*handle); }
        let handle = Handle(self.records.len());
        self.records.push(Record { bindings, edges: HashMap::new(), version: 0 });
        self.allocations.insert(key, handle);
        Some(handle)
    }
    pub(super) fn get(&self, handle: Handle) -> Option<&Record> { self.records.get(handle.0) }
    pub(super) fn subscribe(&mut self, handle: Handle, reader: usize) -> Option<()> {
        self.get(handle)?;
        self.readers.entry(handle).or_default().insert(reader);
        Some(())
    }
    pub(super) fn take_changed_readers(&mut self) -> Vec<usize> {
        let mut readers = self.pending_readers.drain().collect::<Vec<_>>();
        readers.sort_unstable();
        readers
    }
    pub(super) fn merge(&mut self, handle: Handle, bindings: &[Binding], edges: &[(crate::local_capture_plan::BindingId, Vec<Alternative>)], declarations: &HashMap<String,Contract>) -> Option<bool> {
        let incoming = canonical(bindings)?;
        let record = self.get(handle)?;
        if record.bindings.len() != incoming.len() || record.bindings.iter().zip(&incoming).any(|(left,right)| left.identity != right.identity) { return None; }
        let identities = incoming.iter().filter_map(|binding| binding.identity).collect::<HashSet<_>>();
        // Reject malformed edges before mutating any record state.
        if edges.iter().any(|(identity, alternatives)| !identities.contains(identity) || alternatives.iter().any(|alternative| matches!(alternative, Alternative::Body { environment: Some(handle), .. } if self.get(*handle).is_none()))) { return None; }
        let record = &mut self.records[handle.0];
        let mut changed = join(&mut record.bindings, &incoming, declarations);
        for (identity,incoming) in edges {
            let previous = record.edges.entry(*identity).or_default();
            let mut alternatives = previous.clone();
            alternatives.extend(incoming.iter().cloned());
            alternatives.sort_by_key(|alternative| match alternative {
                Alternative::Unknown => (0,0,0,None),
                Alternative::Body {declaration,environment} => (1,declaration.start,declaration.end,environment.map(|handle|handle.0)),
            });
            alternatives.dedup();
            if *previous != alternatives { *previous = alternatives; changed = true; }
        }
        if changed {
            record.version += 1;
            if let Some(readers) = self.readers.get(&handle) {
                self.pending_readers.extend(readers.iter().copied());
            }
        }
        Some(changed)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn binding(body: usize) -> Binding {
        Binding { alternatives: None, identity: Some(crate::local_capture_plan::BindingId { owner: 0, declaration_start: 1 }), name: "action".into(), scope: Span::new(0,20), targets: vec![Target::Unknown], origins: vec![body_origins::Origin::Body(Span::new(body,body+1))], selection: Selection::default(), elements: None, mutable: true, contract: None }
    }
    #[test]
    fn changed_records_schedule_only_subscribed_readers_once() {
        let mut store = Store::default();
        let original = binding(5);
        let first = store.allocate(Span::new(30,31), Span::new(40,41), &[original.clone()]).unwrap();
        let second = store.allocate(Span::new(32,33), Span::new(40,41), &[original.clone()]).unwrap();
        assert_eq!(store.subscribe(first, 7), Some(()));
        assert_eq!(store.subscribe(first, 7), Some(()));
        assert_eq!(store.subscribe(second, 9), Some(()));
        assert_eq!(store.subscribe(Handle(999), 11), None);
        assert!(store.take_changed_readers().is_empty());
        assert_eq!(store.merge(first, &[original.clone()], &[], &HashMap::new()), Some(false));
        assert!(store.take_changed_readers().is_empty());
        assert_eq!(store.merge(first, &[binding(6)], &[], &HashMap::new()), Some(true));
        assert_eq!(store.merge(first, &[binding(8)], &[], &HashMap::new()), Some(true));
        assert_eq!(store.take_changed_readers(), vec![7]);
        assert!(store.take_changed_readers().is_empty());
        assert_eq!(store.merge(first, &[], &[], &HashMap::new()), None);
        assert!(store.take_changed_readers().is_empty());
        assert_eq!(store.merge(second, &[binding(6)], &[], &HashMap::new()), Some(true));
        assert_eq!(store.take_changed_readers(), vec![9]);
        assert_eq!(store.allocate(Span::new(30,31), Span::new(40,41), &[original]), Some(first));
    }
    #[test]
    fn allocations_reuse_source_context_and_distinguish_sites_and_capture_facts() {
        let mut store=Store::default();let site=Span::new(30,31);let producer=Span::new(40,41);
        let first=store.allocate(site,producer,&[binding(5)]).unwrap();
        assert_eq!(store.allocate(site,producer,&[binding(5)]),Some(first));
        assert_ne!(store.allocate(Span::new(32,33),producer,&[binding(5)]),Some(first));
        assert_ne!(store.allocate(site,producer,&[binding(7)]),Some(first));
        assert_ne!(store.allocate(site,Span::new(42,43),&[binding(5)]),Some(first));
    }
    #[test]
    fn capture_slot_order_is_canonical_and_updates_preserve_allocation_identity() {
        let mut first=binding(5);let mut second=binding(7);
        second.identity=Some(crate::local_capture_plan::BindingId {owner:0,declaration_start:2});
        let mut store=Store::default();let site=Span::new(30,31);let producer=Span::new(40,41);
        let handle=store.allocate(site,producer,&[first.clone(),second.clone()]).unwrap();
        assert_eq!(store.allocate(site,producer,&[second.clone(),first.clone()]),Some(handle));
        let initial=first.clone();first.origins=vec![body_origins::Origin::Body(Span::new(9,10))];
        assert_eq!(store.merge(handle,&[second.clone(),first.clone()],&[],&HashMap::new()),Some(true));
        assert_eq!(store.merge(handle,&[first,second.clone()],&[],&HashMap::new()),Some(false));
        assert_eq!(store.allocate(site,producer,&[initial,second]),Some(handle));
        let record=store.get(handle).unwrap();assert_eq!(record.version,1);assert_eq!(record.bindings[0].origins.len(),2);
    }
    #[test]
    fn malformed_capture_shapes_and_edges_cannot_partially_mutate_records() {
        let mut store=Store::default();let site=Span::new(30,31);let producer=Span::new(40,41);let initial=binding(5);
        let mut anonymous=initial.clone();anonymous.identity=None;
        assert!(store.allocate(site,producer,&[anonymous]).is_none());
        assert!(store.allocate(site,producer,&[initial.clone(),initial.clone()]).is_none());
        let handle=store.allocate(site,producer,&[initial.clone()]).unwrap();let identity=initial.identity.unwrap();
        let mut changed=initial;changed.origins=vec![body_origins::Origin::Body(Span::new(9,10))];
        assert_eq!(store.merge(handle,&[changed.clone()],&[(identity,vec![Alternative::Body {declaration:site,environment:Some(Handle(999))}])],&HashMap::new()),None);
        assert_eq!(store.merge(handle,&[],&[],&HashMap::new()),None);
        assert_eq!(store.merge(handle,&[changed],&[(crate::local_capture_plan::BindingId {owner:9,declaration_start:9},vec![Alternative::Unknown])],&HashMap::new()),None);
        let record=store.get(handle).unwrap();assert_eq!(record.version,0);assert!(record.edges.is_empty());assert_eq!(record.bindings[0].origins,vec![body_origins::Origin::Body(Span::new(5,6))]);
    }

}

#[cfg(test)]
mod edge_tests {
    use super::*;
    #[test]
    fn correlated_recursive_edges_and_unknowns_join_without_reallocation() {
        let source=crate::format("function main(){let action=():int=>1;}").unwrap();
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let mut bindings=Vec::new();collect(&program.functions[0].body,&mut bindings,&effects);
        let identity=bindings[0].identity.unwrap();let site=Span::new(80,81);let producer=Span::new(90,91);
        let mut store=Store::default();let first=store.allocate(site,producer,&bindings).unwrap();let second=store.allocate(Span::new(82,83),producer,&bindings).unwrap();
        let edges=vec![Alternative::Body {declaration:Span::new(100,101),environment:Some(first)},Alternative::Body {declaration:Span::new(102,103),environment:Some(second)},Alternative::Body {declaration:Span::new(100,101),environment:Some(second)},Alternative::Unknown];
        let declarations=HashMap::new();
        assert_eq!(store.merge(first,&bindings,&[(identity,edges.clone())],&declarations),Some(true));
        assert_eq!(store.merge(first,&bindings,&[(identity,edges.clone())],&declarations),Some(false));
        assert_eq!(store.allocate(site,producer,&bindings),Some(first));
        let record=store.get(first).unwrap();assert_eq!(record.version,1);assert_eq!(record.edges[&identity].len(),4);
        assert!(edges.iter().all(|edge|record.edges[&identity].contains(edge)));
        assert!(!record.edges[&identity].contains(&Alternative::Body {declaration:Span::new(102,103),environment:Some(first)}),"joining edges must not fabricate a body/environment cross product");
        assert!(store.get(second).unwrap().edges.is_empty());
    }
}
