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
 #[cfg_attr(not(test),expect(dead_code,reason="Node resumption is verified before inter-call dependency wiring."))]
 pub(super) fn requeue(&mut self,node:usize)->Option<bool> {
    if node>=self.graph.nodes.len() || node==self.graph.exits.normal || node==self.graph.exits.returning || node==self.graph.exits.throwing {return None;}
    if self.queued[node] {return Some(false);}
    self.queued[node]=true;self.work.push(node);Some(true)
 }
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
            dispatch.begin_node(index);
            let transfer = node_flow::apply_with_dispatch(node, &mut state, &mut self.facts, declarations, effects, dispatch);
            dispatch.end_node();
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

#[cfg(test)]
mod tests {
 use super::*;
 struct Delayed {ready:bool,throw:bool,nodes:Vec<usize>,calls:Vec<String>,target_node:Option<usize>}
 impl expression_flow::Dispatch for Delayed {
  fn begin_node(&mut self,node:usize){self.nodes.push(node);}
  fn end_node(&mut self){self.nodes.pop().unwrap();}
  fn invoke(&mut self,expression:&Expr,_callee:&expression_flow::Value,_arguments:&[expression_flow::Value],state:&FlowState)->Option<expression_flow::Invocation>{
   let ExprKind::Call(callee,_)=&expression.kind else {panic!("call")};
   let ExprKind::Name(name)=&callee.kind else {panic!("name")};
   self.calls.push(name.clone());
   if name=="target" {self.target_node=self.nodes.last().copied();}
   Some(expression_flow::Invocation {normal:if name=="target" && (!self.ready || self.throw) {None}else{Some(state.clone())},throwing:if name=="target" && self.ready && self.throw {vec![state.clone()]}else{vec![]},result:expression_flow::Value::index(7),facts:HashMap::new()})
  }
 }
 #[test] fn a_resolved_call_requeues_its_node_without_replaying_prior_statements(){
  let source=crate::format("function before(){}function target():int{return 7;}function after(){}function root(){before();const value=target();after();return value;}function main(){}").unwrap();
  let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
  let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();let declarations=HashMap::new();
  let mut body=BodyState::new(&root.body,&FlowState::default(),&declarations,&effects).unwrap();
  let mut dispatch=Delayed {ready:false,throw:false,nodes:vec![],calls:vec![],target_node:None};
  let waiting=body.drive(&declarations,&effects,&mut dispatch).unwrap();assert!(waiting.returning.is_none());assert_eq!(dispatch.calls,["before","target"]);
  dispatch.ready=true;let node=dispatch.target_node.unwrap();assert_eq!(body.requeue(node),Some(true));assert_eq!(body.requeue(node),Some(false));
  let completed=body.drive(&declarations,&effects,&mut dispatch).unwrap();assert!(completed.returning.is_some());assert_eq!(dispatch.calls,["before","target","target","after"]);assert!(dispatch.nodes.is_empty());
  assert!(body.drive(&declarations,&effects,&mut dispatch).unwrap()==completed);assert_eq!(dispatch.calls.len(),4);
 }
 #[test] fn a_resolved_failure_preserves_finally_without_replaying_prefix(){
  let source=crate::format("function before(){}function target(){}function after(){}function cleanup(){}function root(){before();try{target();after();}finally{cleanup();}}function main(){}").unwrap();
  let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
  let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();let declarations=HashMap::new();
  let mut body=BodyState::new(&root.body,&FlowState::default(),&declarations,&effects).unwrap();
  let mut dispatch=Delayed {ready:false,throw:true,nodes:vec![],calls:vec![],target_node:None};
  let waiting=body.drive(&declarations,&effects,&mut dispatch).unwrap();assert!(waiting.normal.is_none() && waiting.throwing.is_none());
  dispatch.ready=true;assert_eq!(body.requeue(dispatch.target_node.unwrap()),Some(true));
  let completed=body.drive(&declarations,&effects,&mut dispatch).unwrap();assert!(completed.normal.is_none() && completed.throwing.is_some());
  assert_eq!(dispatch.calls,["before","target","target","cleanup"]);assert!(dispatch.nodes.is_empty());
  assert!(body.drive(&declarations,&effects,&mut dispatch).unwrap()==completed);assert_eq!(dispatch.calls.len(),4);
 }

}
