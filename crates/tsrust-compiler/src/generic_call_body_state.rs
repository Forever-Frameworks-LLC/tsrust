//! Persistent intraprocedural state, independent of invocation scheduling.
use super::*;
pub(super) struct BodyState<'a> {
 graph:flow_graph::SummaryGraph<'a>,
 states:Vec<Partitions>,
 work:Vec<usize>,
 queued:Vec<bool>,
 facts:HashMap<(usize,usize),Option<Vec<Contract>>>,
}
impl<'a> BodyState<'a> {
 pub(super) fn new(body:&'a [Stmt],initial:&FlowState,declarations:&HashMap<String,Contract>,effects:&effect_bodies::Index)->Option<Self> {
    let graph = flow_graph::summary_graph(body)?;
    let mut seed = initial.clone();
    let mut locals = Vec::new();
    collect(body, &mut locals, effects);
    for mut local in locals {
        if seed.iter().any(|binding| binding.identity == local.identity && binding.name == local.name && binding.scope == local.scope) { continue; }
        local.alternatives = None;
        local.origins = vec![body_origins::Origin::Unknown];
        local.selection = Selection::default();
        local.elements = None;
        local.targets = vec![Target::Unknown];
        seed.push(local);
    }
    let mut states = vec![Partitions::default(); graph.nodes.len()];
    let entry = graph.nodes[0].next[0];
    states[entry].insert(seed, None, declarations)?;
    let mut queued=vec![false;graph.nodes.len()];queued[entry]=true;
    Some(Self {graph,states,work:vec![entry],queued,facts:HashMap::new()})
 }
 pub(super) fn drive(&mut self,declarations:&HashMap<String,Contract>,effects:&effect_bodies::Index,dispatch:&mut dyn expression_flow::Dispatch)->Option<Outcome> {
    while let Some(index) = self.work.pop() {
        self.queued[index]=false;
        if index == self.graph.exits.normal || index == self.graph.exits.returning || index == self.graph.exits.throwing { continue; }
        let node = &self.graph.nodes[index];
        for (mut state, previous_pending) in self.states[index].entries.clone() {
            let transfer = node_flow::apply_with_dispatch(node, &mut state, &mut self.facts, declarations, effects, dispatch);
            let pending = match node.statement.map(|statement| &statement.kind) {
                Some(StmtKind::Return(value)) if transfer.normal => Some(if value.is_some() { transfer.value.clone() } else { expression_flow::Value::undefined() }),
                Some(StmtKind::Throw(_)) => None,
                _ => previous_pending,
            };
            for next in transfer.normal_edges(node) {
                let mut outgoing = state.clone();
                guard_partitions::refine(node, next, &mut outgoing, effects);
                if self.states[next].insert(outgoing, pending.clone(), declarations)? { if !self.queued[next] {self.queued[next]=true;self.work.push(next);} }
            }
            if let Some(next) = node.exceptional {
                for outgoing in transfer.failures {
                    if self.states[next].insert(outgoing, None, declarations)? { if !self.queued[next] {self.queued[next]=true;self.work.push(next);} }
                }
            }
        }
    }
    let normal = self.states[self.graph.exits.normal].joined(declarations)?.map(|(state, _)| state);
    let returning = self.states[self.graph.exits.returning].joined(declarations)?;
    let throwing = self.states[self.graph.exits.throwing].joined(declarations)?.map(|(state, _)| state);
    let (returning, returned_value) = returning.map_or((None, None), |(state, value)| (Some(state), value));
    Some(Outcome { normal, returning, throwing, returned_value, facts:self.facts.clone() })
}
}
