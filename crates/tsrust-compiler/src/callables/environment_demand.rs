//! Environment retention is independent of callback invocation cardinality.
use crate::local_capture_plan::BindingId;
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Demand { Scoped, Owned, Unknown }

#[derive(Default)]
pub(crate) struct Graph {
    nodes: BTreeMap<BindingId, Demand>,
    forwarding: BTreeSet<(BindingId, BindingId)>,
}
impl Graph {
    /// Scoped admission requires a complete accounting of the binding's uses.
    pub(crate) fn checked(&mut self, binding: BindingId) { self.nodes.entry(binding).or_insert(Demand::Scoped); }
    pub(crate) fn retains(&mut self, binding: BindingId) { self.nodes.entry(binding).and_modify(|demand|*demand=(*demand).max(Demand::Owned)).or_insert(Demand::Owned); }
    pub(crate) fn unknown(&mut self, binding: BindingId) { self.nodes.insert(binding, Demand::Unknown); }
    /// A move, capture or forwarded argument carries its source environment demand.
    pub(crate) fn forwards(&mut self, source: BindingId, destination: BindingId) { self.forwarding.insert((source,destination)); }
    pub(crate) fn solve(&self) -> BTreeMap<BindingId, Demand> {
        let mut demands=self.nodes.clone();
        let mut dependents:BTreeMap<BindingId,Vec<BindingId>>=BTreeMap::new();
        for &(source,destination) in &self.forwarding {
            demands.entry(source).or_insert(Demand::Unknown);
            demands.entry(destination).or_insert(Demand::Unknown);
            dependents.entry(destination).or_default().push(source);
        }
        let mut pending=demands.iter().filter_map(|(&binding,&demand)|(demand!=Demand::Scoped).then_some(binding)).collect::<VecDeque<_>>();
        while let Some(destination)=pending.pop_front() {
            let incoming=demands[&destination];
            for source in dependents.get(&destination).into_iter().flatten() {
                let demand=demands.get_mut(source).expect("forwarding source indexed");
                if incoming>*demand {*demand=incoming;pending.push_back(*source);}
            }
        }
        demands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn binding(owner:usize,declaration_start:usize)->BindingId {BindingId{owner,declaration_start}}
    #[test]
    fn returned_capture_and_alias_moves_require_ownership_of_the_original_parameter() {
        let (parameter,moved,closure)=(binding(0,10),binding(0,20),binding(1,30));
        let mut graph=Graph::default();for id in [parameter,moved,closure] {graph.checked(id);}
        graph.forwards(parameter,moved);graph.forwards(moved,closure);graph.retains(closure);
        let result=graph.solve();assert_eq!(result[&parameter],Demand::Owned);assert_eq!(result[&moved],Demand::Owned);
    }
    #[test]
    fn recursive_forwarding_is_scoped_until_one_path_retains_the_environment() {
        let (first,second,third)=(binding(0,1),binding(1,2),binding(2,3));
        let mut graph=Graph::default();for id in [first,second,third] {graph.checked(id);}
        graph.forwards(first,second);graph.forwards(second,third);graph.forwards(third,first);
        assert!(graph.solve().values().all(|demand|*demand==Demand::Scoped));
        graph.retains(third);assert!(graph.solve().values().all(|demand|*demand==Demand::Owned));
    }
    #[test]
    fn distinct_shadowed_bindings_and_independent_factory_parameters_do_not_merge() {
        let (outer,inner,independent,sink)=(binding(0,10),binding(1,10),binding(2,10),binding(3,10));
        let mut graph=Graph::default();for id in [outer,inner,independent,sink] {graph.checked(id);}
        graph.forwards(inner,sink);graph.retains(sink);
        let result=graph.solve();assert_eq!(result[&outer],Demand::Scoped);assert_eq!(result[&inner],Demand::Owned);assert_eq!(result[&independent],Demand::Scoped);
    }
    #[test]
    fn missing_destinations_and_unknown_branches_cannot_certify_scoped_admission() {
        let (source,known,missing)=(binding(0,1),binding(1,2),binding(2,3));
        let mut graph=Graph::default();graph.checked(source);graph.checked(known);
        graph.forwards(source,known);graph.forwards(source,missing);
        assert_eq!(graph.solve()[&source],Demand::Unknown);
        graph.checked(missing);graph.unknown(known);
        assert_eq!(graph.solve()[&source],Demand::Unknown);
    }
    #[test]
    fn incomplete_evidence_cannot_be_overridden_by_later_retention_or_checked_facts() {
        let source=binding(0,1);let mut graph=Graph::default();graph.unknown(source);
        graph.retains(source);graph.checked(source);assert_eq!(graph.solve()[&source],Demand::Unknown);
    }
    #[test]
    fn long_forwarding_chain_converges_without_recursive_dispatch() {
        let mut graph=Graph::default();
        for owner in 0..10_000 {graph.checked(binding(owner,1));}
        for owner in 0..9_999 {graph.forwards(binding(owner,1),binding(owner+1,1));}
        graph.retains(binding(9_999,1));
        let result=graph.solve();assert_eq!(result.len(),10_000);
        assert!(result.values().all(|demand|*demand==Demand::Owned));
    }
}
