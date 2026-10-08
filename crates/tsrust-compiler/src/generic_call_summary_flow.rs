//! Callable summaries preserve distinct normal, return and exceptional states.
use super::*;
#[derive(Default, Clone, PartialEq, Eq)]
pub(super) struct Outcome {
    pub normal: Option<FlowState>,
    pub returning: Option<FlowState>,
    pub throwing: Option<FlowState>,
    pub returned_value: Option<expression_flow::Value>,
    pub facts: HashMap<(usize, usize), Option<Vec<Contract>>>,
}
/// Evaluate a callee body without merging normal and exceptional completion.
/// Invocation dispatch and contextual parameters are supplied by the next layer.
#[cfg(test)]
pub(super) fn run(body: &[Stmt], initial: &[Binding], declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index) -> Option<Outcome> {
    run_with_dispatch(body, &initial.to_vec().into(), declarations, effects, &mut expression_flow::UnknownDispatch)
}
pub(super) fn run_with_dispatch(body: &[Stmt], initial: &FlowState, declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index, dispatch: &mut dyn expression_flow::Dispatch) -> Option<Outcome> {
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
    let mut states = vec![None; graph.nodes.len()];
    let mut pending_returns = vec![None; graph.nodes.len()];
    let entry = graph.nodes[0].next[0];
    states[entry] = Some(seed);
    let mut work = vec![entry];
    let mut facts = HashMap::new();
    while let Some(index) = work.pop() {
        if index == graph.exits.normal || index == graph.exits.returning || index == graph.exits.throwing { continue; }
        let node = &graph.nodes[index];
        let mut state = states[index].as_ref().unwrap().clone();
        let transfer = node_flow::apply_with_dispatch(node, &mut state, &mut facts, declarations, effects, dispatch);
        let normal_edges = transfer.normal_edges(node);
        let pending = match node.statement.map(|statement| &statement.kind) {
            Some(StmtKind::Return(value)) if transfer.normal => Some(if value.is_some() { transfer.value.clone() } else { expression_flow::Value::undefined() }),
            Some(StmtKind::Throw(_)) => None,
            _ => pending_returns[index].clone(),
        };
        let successors = normal_edges.iter().map(|&next| (next, &state, pending.clone()))
            .chain(node.exceptional.into_iter().flat_map(|next| transfer.failures.iter().map(move |state| (next, state, None))));
        for (successor, outgoing, pending) in successors {
            let existed = states[successor].is_some();
            let mut changed = if let Some(previous) = &mut states[successor] { previous.join(outgoing,declarations)? }
                else { states[successor] = Some(outgoing.clone()); true };
            let joined = if existed {
                match (&pending_returns[successor], pending) {
                    (Some(previous), Some(incoming)) => Some(expression_flow::merge_value(previous.clone(), incoming, declarations)),
                    (None, None) => None,
                    (Some(previous), None) => Some(expression_flow::merge_value(previous.clone(), expression_flow::Value::undefined(), declarations)),
                    (None, Some(incoming)) => Some(expression_flow::merge_value(expression_flow::Value::undefined(), incoming, declarations)),
                }
            } else { pending };
            if joined != pending_returns[successor] { pending_returns[successor] = joined; changed = true; }
            if changed { work.push(successor); }
        }
    }
    Some(Outcome { normal: states[graph.exits.normal].take(), returning: states[graph.exits.returning].take(), throwing: states[graph.exits.throwing].take(), returned_value: pending_returns[graph.exits.returning].take(), facts })
}

/// One contextual body evaluation projects captured state back to its caller.
/// Parameter entry executes defaults and packs rest arguments before the body.
/// Nested invocation dispatch remains the responsibility of the worklist engine.
#[cfg(test)]
pub(super) fn invoke(declaration: Span, caller: &[Binding], arguments: &[expression_flow::Value], declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index) -> Option<Outcome> {
    invoke_with_dispatch(declaration, &caller.to_vec().into(), arguments, declarations, effects, &mut expression_flow::UnknownDispatch)
}
pub(super) fn invoke_with_dispatch(declaration: Span, caller: &FlowState, arguments: &[expression_flow::Value], declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index, dispatch: &mut dyn expression_flow::Dispatch) -> Option<Outcome> {
    if effects.aggregate_writes(declaration) { return None; }
    let mut frame = invocation_frame::Frame::build_with_state(declaration, caller, arguments, effects)?;
    let entry = parameter_entry::prepare(declaration, arguments, &mut frame.state, declarations, effects, dispatch)?;
    let mut outcome = if entry.normal { run_with_dispatch(effects.body(declaration)?, &frame.state, declarations, effects, dispatch)? } else { Outcome::default() };
    for failure in entry.failures {
        if let Some(previous) = &mut outcome.throwing { previous.join(&failure,declarations)?; }
        else { outcome.throwing = Some(failure); }
    }
    for (key, incoming) in entry.facts {
        let value = match outcome.facts.remove(&key) {
            Some(Some(previous)) => incoming.and_then(|incoming| resolve_all(previous.into_iter().chain(incoming).map(Target::Contract).collect(), &[], declarations)),
            Some(None) => None,
            None => incoming,
        };
        outcome.facts.insert(key, value);
    }
    if let (Some(value),Some(state))=(&outcome.returned_value,&outcome.returning) && !dispatch.validate_escape(value,state) {return None;}
    outcome.normal = outcome.normal.map(|state| frame.project(&state));
    outcome.returning = outcome.returning.map(|state| frame.project(&state));
    outcome.throwing = outcome.throwing.map(|state| frame.project(&state));
    Some(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn outcomes(body: &str) -> (Option<usize>, Option<usize>, Option<usize>) {
        outcomes_with_call_facts(body, None)
    }
    fn outcomes_with_call_facts(body: &str, expected: Option<&[usize]>) -> (Option<usize>, Option<usize>, Option<usize>) {
        let source=crate::format(&format!("function one<T>(value:T):T{{return value;}}function two<T>(value:T,other:T):T{{return other;}}function outer<T>(){{let action=one;function change(){{{body}}}}}function main(){{}}")).unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
        let change=program.functions.iter().find(|function|function.name.text.ends_with("#change")).unwrap();
        let mut bindings=Vec::new();collect(&outer.body,&mut bindings,&effects);
        let outcome=run(&change.body,&bindings,&declarations,&effects).unwrap();
        if let Some(expected) = expected {
            let mut facts=outcome.facts.iter().map(|(span,contracts)|(*span,contracts.as_ref().unwrap()[0].required)).collect::<Vec<_>>();
            facts.sort_by_key(|(span,_)|*span);
            assert_eq!(facts.iter().map(|(_,required)|*required).collect::<Vec<_>>(),expected);
        }
        let arity=|state:Option<FlowState>| state.map(|state| {
            let binding=state.iter().find(|binding|binding.name=="action").unwrap();
            { let contracts=resolve_indexed(binding.targets.clone(),&state,&declarations,&effects).unwrap(); assert_eq!(contracts.len(),1); contracts[0].required }
        });
        (arity(outcome.normal),arity(outcome.returning),arity(outcome.throwing))
    }
    #[test]
    fn writes_and_restoration_are_ordered_on_normal_and_throwing_exits() {
        assert_eq!(outcomes("action=two;"),(Some(2),None,None));
        assert_eq!(outcomes("let shadow=two;action=shadow;"),(Some(2),None,None));
        assert_eq!(outcomes("let action=two;action=one;"),(Some(1),None,None));
        assert_eq!(outcomes("action=two;action=one;return;"),(None,Some(1),None));
        assert_eq!(outcomes("action=two;throw 1;"),(None,None,Some(2)));
    }
    #[test]
    fn saved_callees_and_argument_writes_keep_ordered_call_facts() {
        assert_eq!(outcomes_with_call_facts("action<T>((action=two,1));action<T>(1,1);",Some(&[1,2])),(Some(2),None,Some(2)));
    }
    #[test]
    fn constant_guards_do_not_execute_dead_writes() {
        assert_eq!(outcomes("if(true){action=two;}else{action=one;}"),(Some(2),None,None));
        assert_eq!(outcomes("if(false){action=two;}else{action=one;}"),(Some(1),None,None));
    }
    #[test]
    fn cleanup_and_catch_overrides_preserve_final_binding_state() {
        assert_eq!(outcomes("try{action=two;throw 1;}finally{action=one;}"),(None,None,Some(1)));
        assert_eq!(outcomes("try{action=two;throw 1;}catch(error){return;}"),(None,Some(2),None));
        assert_eq!(outcomes("try{action=two;return;}finally{action=one;throw 2;}"),(None,None,Some(1)));
        assert_eq!(outcomes("try{action=two;throw 1;}finally{action=one;return;}"),(None,Some(1),None));
    }
}

#[cfg(test)]
mod contextual_tests {
    use super::*;
    #[test]
    fn supplied_parameter_body_reaches_captured_assignment_and_projects_only_caller_slots() {
        let source=crate::format("function one():int{return 1;}function two():int{return 2;}function main(){let shared=one;function replace(next:Function<int>){shared=next;}}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let main=program.functions.iter().find(|function|function.name.text=="main").unwrap();
        let replace=program.functions.iter().find(|function|function.name.text.ends_with("#replace")).unwrap();
        let two=program.functions.iter().find(|function|function.name.text=="two").unwrap().name.span;
        let mut caller=Vec::new();collect(&main.body,&mut caller,&effects);
        let outcome=invoke(replace.name.span,&caller,&[expression_flow::Value::from_origins(vec![body_origins::Origin::Body(two)])],&HashMap::new(),&effects).unwrap();
        let normal=outcome.normal.unwrap();
        assert_eq!(normal.len(),caller.len());
        assert_eq!(normal[0].identity,caller[0].identity);
        assert_eq!(normal[0].origins,vec![body_origins::Origin::Body(two)]);
        assert!(outcome.returning.is_none());assert!(outcome.throwing.is_none());
    }
}

#[cfg(test)]
mod environment_transport_tests {
 use super::*;
 struct Writer {handle:environment_store::Handle}
 impl expression_flow::Dispatch for Writer {
  fn invoke(&mut self,expression:&Expr,_:&expression_flow::Value,_:&[expression_flow::Value],state:&FlowState)->Option<expression_flow::Invocation> {
   let ExprKind::Call(callee,_)=&expression.kind else {return None;};let (name,_)=name(callee)?;
   let mut outgoing=state.clone();
   let mut throwing=Vec::new();
   if name=="step" {
    let mut failed=state.clone();failed.consume(self.handle)?;throwing.push(failed);
   }
   if let Some(environment_state::Capture::Live {bindings,..})=outgoing.captures.get(&self.handle) {
    let mut bindings=bindings.clone();bindings[0].origins=vec![body_origins::Origin::Body(Span::new(if name=="step" {9}else{11},12))];
    outgoing.update(self.handle,&bindings,&HashMap::new())?;
   }
   Some(expression_flow::Invocation {normal:Some(outgoing),throwing,result:expression_flow::Value::undefined(),facts:HashMap::new()})
  }
 }
 #[test] fn finally_preserves_distinct_returning_and_consumed_throwing_environments() {
  let source=crate::format("function step():int{return 7;}function cleanup(){}function root(){try{return step();}finally{cleanup();}}function main(){}").unwrap();assert_eq!(crate::format(&source).unwrap(),source);
  let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();let declarations=HashMap::new();
  let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();
  let binding=Binding {alternatives:None,identity:Some(crate::local_capture_plan::BindingId {owner:0,declaration_start:1}),name:"captured".into(),scope:Span::new(0,100),targets:vec![Target::Unknown],origins:vec![body_origins::Origin::Body(Span::new(5,6))],selection:Selection::default(),elements:None,mutable:true,contract:None};
  let mut store=environment_store::Store::default();let handle=store.allocate(Span::new(20,21),Span::new(30,31),&[binding.clone()]).unwrap();let mut state=FlowState::default();state.install(handle,&[binding],environment_state::Instance::OwnedSingle).unwrap();
  let outcome=run_with_dispatch(&root.body,&state,&declarations,&effects,&mut Writer {handle}).unwrap();
  assert!(outcome.normal.is_none());
  let returning=outcome.returning.unwrap();let throwing=outcome.throwing.unwrap();
  let environment_state::Capture::Live {bindings,..}=&returning.captures[&handle] else {panic!("normal return retains live instance");};
  assert_eq!(bindings[0].origins,vec![body_origins::Origin::Body(Span::new(11,12))]);assert!(matches!(throwing.captures[&handle],environment_state::Capture::Consumed));
  assert!(matches!(state.captures[&handle],environment_state::Capture::Live {..}),"evaluation does not mutate incoming request state");
 }
 #[test] fn conditional_invocations_join_live_updates_and_keep_consumed_failures_separate() {
  let source=crate::format("function step():int{return 7;}function cleanup():int{return 9;}function root(flag:boolean):int{return flag?step():cleanup();}function main(){}").unwrap();assert_eq!(crate::format(&source).unwrap(),source);
  let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();
  let binding=Binding {alternatives:None,identity:Some(crate::local_capture_plan::BindingId {owner:0,declaration_start:1}),name:"captured".into(),scope:Span::new(0,100),targets:vec![Target::Unknown],origins:vec![body_origins::Origin::Body(Span::new(5,6))],selection:Selection::default(),elements:None,mutable:true,contract:None};
  let mut store=environment_store::Store::default();let handle=store.allocate(Span::new(20,21),Span::new(30,31),&[binding.clone()]).unwrap();let mut initial=FlowState::default();initial.install(handle,&[binding],environment_state::Instance::OwnedSingle).unwrap();
  let outcome=invoke_with_dispatch(root.name.span,&initial,&[expression_flow::Value::default()],&HashMap::new(),&effects,&mut Writer {handle}).unwrap();
  let returning=outcome.returning.unwrap();let environment_state::Capture::Live {bindings,..}=&returning.captures[&handle] else {panic!("live joined state");};
  assert_eq!(bindings[0].origins,vec![body_origins::Origin::Body(Span::new(9,12)),body_origins::Origin::Body(Span::new(11,12))]);
  assert!(matches!(outcome.throwing.unwrap().captures[&handle],environment_state::Capture::Consumed));assert!(returning.bindings.is_empty(),"frame projects formal slots out without dropping environments");
 }

}
