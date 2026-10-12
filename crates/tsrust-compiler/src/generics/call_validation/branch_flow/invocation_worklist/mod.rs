//! Inter-call dependencies converge on an explicit finite worklist.
use super::*;
mod array_transfer;
mod recursive_inputs;
mod terminal_returns;
#[derive(Clone, PartialEq, Eq, Hash)]
struct Request {
    declaration: Span,
    producer: Span,
    context: Vec<(Span, Span)>,
    environment: Option<environment_store::Handle>,
    caller: FlowState,
    arguments: Vec<expression_flow::Value>,
}
#[derive(Clone,Copy,PartialEq,Eq,Hash)]
struct Dependency {parent:usize,node:Option<usize>}
struct Entry<'a> {
    request: Request,
    outcome: summary_flow::Outcome,
    opaque: bool,
    queued: bool,
    dependents: HashSet<Dependency>,
    session:Option<summary_flow::invocation_state::InvocationState<'a>>,
    session_input:Option<FlowState>,
    pending_nodes:HashSet<usize>,
    pending_entry:bool,
    epoch:usize,
}
pub(super) struct Engine<'a> {
    declarations: &'a HashMap<String, Contract>,
    effects: &'a effect_bodies::Index,
    lookup: HashMap<Request, usize>,
    entries: Vec<Entry<'a>>,
    active_nodes:Vec<(usize,usize)>,
    eager_wrapper_active:bool,
    work: Vec<usize>,
    current: Option<usize>,
    environments: environment_store::Store,
    #[cfg(test)]
    evaluations: usize,
    #[cfg(test)]
    invocations: usize,
    #[cfg(test)]
    visits_by_site: HashMap<(usize,usize),usize>,
    #[cfg(test)]
    trace: bool,
}
fn merge_state(current:&mut Option<FlowState>,incoming:Option<FlowState>,declarations:&HashMap<String,Contract>)->bool {
    if let Some(incoming)=incoming {
        if let Some(current)=current {return current.join(&incoming,declarations).is_some();}
        *current=Some(incoming);
    }
    true
}
fn merge_facts(current: &mut HashMap<(usize, usize), Option<Vec<Contract>>>, incoming: HashMap<(usize, usize), Option<Vec<Contract>>>, declarations: &HashMap<String, Contract>) {
    for (key, incoming) in incoming {
        let merged = if let Some(previous) = current.get(&key) {
            match (previous, incoming) {
                (Some(previous), Some(incoming)) => resolve_all(previous.iter().cloned().chain(incoming).map(Target::Contract).collect(), &[], declarations),
                _ => None,
            }
        } else { incoming };
        current.insert(key, merged);
    }
}
impl<'a> Engine<'a> {
    pub(super) fn new(declarations: &'a HashMap<String, Contract>, effects: &'a effect_bodies::Index) -> Self {
        Self { declarations, effects, lookup: HashMap::new(), entries: Vec::new(), active_nodes:Vec::new(), eager_wrapper_active:false, work: Vec::new(), current: None, environments: environment_store::Store::default(), #[cfg(test)] evaluations: 0, #[cfg(test)] invocations: 0, #[cfg(test)] visits_by_site:HashMap::new(), #[cfg(test)] trace:std::env::var_os("TSRUST_TRACE_WORKLIST").is_some() }
    }
    // Keep a simple declaration path. Recursive edges reuse the earlier frame,
    // so allocation identity cannot grow with runtime recursion depth.
    fn call_context(&self, declaration: Span, site: Span) -> Vec<(Span, Span)> {
        let mut context=self.current.map(|current|self.entries[current].request.context.clone()).unwrap_or_default();
        if let Some(position)=context.iter().position(|(body,_)|*body==declaration) {
            context.truncate(position+1);
        } else {context.push((declaration,site));}
        context
    }
    fn request(&mut self, mut request: Request) -> usize {
        if self.current.is_some_and(|current|self.entries[current].request.context.iter().any(|(declaration,_)|*declaration==request.declaration)) {
            recursive_inputs::bound(&mut request,self.declarations);
        }
        let index = if let Some(&index) = self.lookup.get(&request) { index } else {
            let index = self.entries.len();
            self.lookup.insert(request.clone(), index);
            self.entries.push(Entry { request, outcome: summary_flow::Outcome::default(), opaque: false, queued: true, dependents: HashSet::new(),session:None,session_input:None,pending_nodes:HashSet::new(),pending_entry:false,epoch:0 });
            self.work.push(index);
            index
        };
        if let Some(parent) = self.current { let node=self.active_nodes.last().filter(|(owner,_)|*owner==parent).map(|(_,node)|*node);self.entries[index].dependents.insert(Dependency {parent,node}); }
        index
    }
    fn for_each(&mut self, expression: &Expr, callee: &expression_flow::Value, arguments: &[expression_flow::Value], state: &FlowState) -> Option<expression_flow::Invocation> {
        let (Some(elements), Some(callback)) = (&callee.elements, arguments.first()) else { return self.opaque_invocation(); };
        let Some(elements)=elements.exact_values() else {return self.opaque_invocation();};
        let mut normal = Some(state.clone());
        let mut throwing = None;
        let mut facts = HashMap::new();
        for (position, element) in elements.iter().enumerate() {
            let Some(incoming) = normal.take() else { break; };
            if callback.origins.iter().any(|origin| !matches!(origin, body_origins::Origin::Body(_))) { return self.opaque_invocation(); }
            let alternatives=callback.alternatives.clone().unwrap_or_else(||callback.origins.iter().map(|origin|match origin {
                body_origins::Origin::Body(declaration)=>environment_store::Alternative::Body{declaration:*declaration,environment:None},
                _=>environment_store::Alternative::Unknown,
            }).collect());
            for alternative in alternatives {
                let environment_store::Alternative::Body{declaration,environment}=alternative else {return self.opaque_invocation()};
                let callback_arguments = vec![expression_flow::Value::element(element), expression_flow::Value::index(position), expression_flow::Value::array(&elements.iter().map(expression_flow::Value::element).collect::<Vec<_>>())];
                let index = self.request(Request { declaration, producer: expression.span, context: self.call_context(declaration, expression.span), environment, caller: incoming.clone(), arguments: callback_arguments });
                if self.entries[index].opaque { return self.opaque_invocation(); }
                let mut outcome = self.entries[index].outcome.clone();
                for outgoing in [&mut outcome.normal,&mut outcome.returning,&mut outcome.throwing].into_iter().flatten() {
                    outgoing.retain(|binding|incoming.iter().any(|original|original.identity==binding.identity));
                }
                if !merge_state(&mut normal, outcome.normal, self.declarations) {return self.opaque_invocation();}
                if !merge_state(&mut normal, outcome.returning, self.declarations) {return self.opaque_invocation();}
                if !merge_state(&mut throwing, outcome.throwing, self.declarations) {return self.opaque_invocation();}
                merge_facts(&mut facts, outcome.facts, self.declarations);
            }
        }
        Some(expression_flow::Invocation { normal, throwing: throwing.into_iter().collect(), result: expression_flow::Value::undefined(), facts })
    }
    fn opaque_invocation(&mut self) -> Option<expression_flow::Invocation> {
        // An unknown effect invalidates this contextual proof and propagates
        // through reverse dependencies when the current request completes.
        if let Some(current) = self.current { self.entries[current].opaque = true; }
        None
    }
    fn enqueue(&mut self, index: usize) {
        if !self.entries[index].queued { self.entries[index].queued = true; self.work.push(index); }
    }
    pub(super) fn solve(&mut self, declaration: Span, caller: &[Binding], arguments: &[expression_flow::Value]) -> Option<summary_flow::Outcome> {
        #[cfg(test)]
        let trace=self.trace;
        #[cfg(test)]
        if trace {eprintln!("worklist-start span={}..{} bindings={}",declaration.start,declaration.end,caller.len());}
        let root = self.request(Request { declaration, producer: declaration, context: vec![(declaration, declaration)], environment: None, caller: caller.to_vec().into(), arguments: arguments.to_vec() });
        while let Some(index) = self.work.pop() {
            self.evaluate(index);
        }
        #[cfg(test)]
        if trace {eprintln!("worklist-finish span={}..{} requests={} evaluations={} invocationVisits={} opaque={}",declaration.start,declaration.end,self.entries.len(),self.evaluations,self.invocations,self.entries[root].opaque);}
        (!self.entries[root].opaque).then(|| self.entries[root].outcome.clone())
    }
    fn evaluate(&mut self, index: usize) {
        #[cfg(test)]
        { self.evaluations += 1; }
        self.entries[index].queued = false;
        let previous_opaque = self.entries[index].opaque;
        let parent = self.current;
        self.current = Some(index);
        let request = self.entries[index].request.clone();
        let declarations = self.declarations;
        let effects = self.effects;
        let mut caller = request.caller.clone();
        let captured_before=request.environment.and_then(|handle|match caller.captures.get(&handle) {
            Some(environment_state::Capture::Live {bindings,instance:environment_state::Instance::Merged})=> {
                bindings.iter().map(|captured|caller.iter().find(|binding|binding.identity==captured.identity).cloned()).collect::<Option<Vec<_>>>().or_else(||Some(bindings.clone()))
            },
            Some(environment_state::Capture::Live {bindings,..})=>Some(bindings.clone()),
            Some(_)=>None,
            None=>self.environments.get(handle).map(|record|record.bindings.clone()),
        });
        let available = if let Some(environment)=request.environment {
            self.environments.subscribe(environment,index).is_some() && captured_before.as_ref().is_some_and(|bindings| {
                if bindings.iter().any(|captured|caller.iter().any(|binding|binding.identity==captured.identity && binding!=captured)) {return false;}
                for captured in bindings {caller.retain(|binding|binding.identity!=captured.identity);caller.push(captured.clone());}
                true
            })
        } else {true};
        let epoch=self.entries[index].epoch;
        let mut session=if self.entries[index].session_input.as_ref()==Some(&caller) {self.entries[index].session.take()}else{None};
        if std::mem::take(&mut self.entries[index].pending_entry) && session.as_mut().is_some_and(|session|!session.requeue_entry()) {session=None;}
        if session.is_none() && available {session=summary_flow::invocation_state::InvocationState::new(request.declaration,&caller,&request.arguments,declarations,effects,self);}
        let pending=std::mem::take(&mut self.entries[index].pending_nodes);
        if let Some(session)=&mut session {for node in pending {if session.requeue(node).is_none() {self.entries[index].opaque=true;}}}
        let mut incoming=if available {session.as_mut().and_then(|session|session.drive(declarations,effects,self))}else{None};
        if self.entries[index].epoch==epoch {self.entries[index].session=session;self.entries[index].session_input=Some(caller.clone());}

        if let (Some(handle),Some(before),Some(outcome))=(request.environment,&captured_before,&mut incoming) {
            for state in [&mut outcome.normal,&mut outcome.returning,&mut outcome.throwing].into_iter().flatten() {
                if state.captures.contains_key(&handle) {
                    let updated=before.iter().map(|binding|state.iter().find(|next|next.identity==binding.identity).cloned()).collect::<Option<Vec<_>>>();
                    if updated.is_none_or(|bindings|state.update(handle,&bindings,declarations).is_none()) {self.entries[index].opaque=true;}
                }
            }
        }
        let readers=self.environments.take_changed_readers();
        for reader in readers {self.entries[reader].session=None;self.entries[reader].session_input=None;self.entries[reader].epoch+=1;self.enqueue(reader);}
        self.current = parent;
        let previous = self.entries[index].outcome.clone();
        if let Some(incoming) = incoming {
            if request.environment.is_none_or(|handle| !request.caller.captures.contains_key(&handle)) && let Some(captured)=&captured_before && [&incoming.normal,&incoming.returning,&incoming.throwing].into_iter().flatten().any(|state|captured.iter().any(|binding|state.iter().find(|outgoing|outgoing.identity==binding.identity)!=Some(binding))) {
                self.entries[index].opaque=true;
            }
            let entry=&mut self.entries[index];
            let outcome = &mut entry.outcome;
            if !merge_state(&mut outcome.normal, incoming.normal, declarations) {entry.opaque=true;}
            if !merge_state(&mut outcome.returning, incoming.returning, declarations) {entry.opaque=true;}
            if !merge_state(&mut outcome.throwing, incoming.throwing, declarations) {entry.opaque=true;}
            if let Some(incoming) = incoming.returned_value {
                outcome.returned_value = Some(if let Some(previous) = outcome.returned_value.take() { expression_flow::merge_value(previous, incoming, declarations) } else { incoming });
            }
            merge_facts(&mut outcome.facts, incoming.facts, declarations);
        } else { self.entries[index].opaque = true; }
        if previous != self.entries[index].outcome || previous_opaque != self.entries[index].opaque {
            let dependents = self.entries[index].dependents.iter().copied().collect::<Vec<_>>();
            for dependent in dependents {
                let active=self.current==Some(dependent.parent) && self.active_nodes.last().filter(|(owner,_)|*owner==dependent.parent).map(|(_,node)|*node)==dependent.node;
                if active {continue;}
                if let Some(node)=dependent.node {self.entries[dependent.parent].pending_nodes.insert(node);}else{self.entries[dependent.parent].pending_entry=true;}
                self.enqueue(dependent.parent);
            }
        }
    }
    fn terminal_leaf_return(&self, declaration: Span) -> bool {
        terminal_returns::eligible(declaration,self.effects)
    }

}
// Preserve outgoing order and unknown identities while indexing caller membership once.
fn retain_caller_bindings(outgoing:&mut FlowState,identities:&HashSet<Option<crate::local_capture_plan::BindingId>>) {
    outgoing.retain(|binding|identities.contains(&binding.identity));
}
impl expression_flow::Dispatch for Engine<'_> {
    fn begin_node(&mut self,node:usize){if let Some(owner)=self.current {self.active_nodes.push((owner,node));}}
    fn end_node(&mut self){if self.current.is_some() {self.active_nodes.pop().expect("balanced node dispatch");}}
    fn reject(&mut self) {self.opaque_invocation();}
    fn transfer_array_result(&mut self,expression:&Expr,value:&mut expression_flow::Value,state:&mut FlowState)->bool {
        array_transfer::transfer(self,expression,value,state)
    }

    fn validate_escape(&mut self, value: &mut expression_flow::Value, state: &mut FlowState) -> bool {
        if value.alternatives.is_none() {
            let Some(current)=self.current else {return false;};
            let request=&self.entries[current].request;
            let mut alternatives=Vec::new();
            for origin in &value.origins {
                let body_origins::Origin::Body(declaration)=origin else {return true;};
                let identities=self.effects.captures(*declaration).cloned().unwrap_or_default();
                if identities.is_empty() {alternatives.push(environment_store::Alternative::Body {declaration:*declaration,environment:None});continue;}
                let Some(bindings)=identities.iter().map(|id|state.iter().find(|binding|binding.identity==Some(*id)).cloned()).collect::<Option<Vec<_>>>() else {return false;};
                if bindings.iter().any(|captured|request.caller.iter().any(|binding|binding.identity==captured.identity)) {return false;}
                let Some(handle)=self.environments.allocate_context(*declaration,request.producer,&request.context,&bindings) else {return false;};
                if !state.captures.contains_key(&handle) && state.install(handle,&bindings,environment_state::Instance::OwnedSingle).is_none() {return false;}
                alternatives.push(environment_store::Alternative::Body {declaration:*declaration,environment:Some(handle)});
            }
            value.alternatives=Some(environment_store::canonical_alternatives(alternatives));
        }
        let alternatives=value.alternatives.iter().flatten().chain(value.elements.iter().flat_map(|shape|shape.iter()).flat_map(|element|element.alternatives.iter().flatten()));
        for alternative in alternatives {
            let environment_store::Alternative::Body{environment:Some(handle),..}=alternative else {continue;};
            let Some(record)=self.environments.get(*handle) else {return false;};
            if let Some(environment_state::Capture::Live {bindings,instance})=state.captures.get_mut(handle) {
                // Only captures local to this producing frame can become owned
                // instances on escape. Caller captures retain conservative state.
                let Some(current)=self.current else {return false;};
                if !self.entries[current].request.caller.captures.contains_key(handle) && bindings.iter().all(|captured|!self.entries[current].request.caller.iter().any(|binding|binding.identity==captured.identity)) {
                    *instance=environment_state::Instance::OwnedSingle;
                }
            }
            let bindings=match state.captures.get(handle) {
                Some(environment_state::Capture::Live {bindings,..})=>bindings,
                Some(_)=>return false,
                None=>&record.bindings,
            };
            if !bindings.iter().all(|captured|state.iter().find(|binding|binding.identity==captured.identity).is_none_or(|binding|binding==captured)) {return false;}
        }
        true
    }
    fn capture(&mut self, expression: &Expr, value: &mut expression_flow::Value, state: &mut FlowState) {
        if value.alternatives.is_some() { return; }
        let Some(current)=self.current else {return};
        let producer=self.entries[current].request.producer;
        let mut alternatives=Vec::new();
        let mut captured_any=false;
        for origin in &value.origins {
            let body_origins::Origin::Body(declaration)=origin else {alternatives.push(environment_store::Alternative::Unknown);continue};
            let captures=self.effects.captures(*declaration).cloned().unwrap_or_default();
            if captures.is_empty() {alternatives.push(environment_store::Alternative::Body {declaration:*declaration,environment:None});continue;}
            let bindings=captures.iter().map(|identity|state.iter().find(|binding|binding.identity==Some(*identity)).cloned()).collect::<Option<Vec<_>>>();
            let Some(bindings)=bindings else {return};
            // Mutable local captures remain live until an owned escape.
            if bindings.iter().any(|binding|binding.mutable) {return;}
            let Some(environment)=self.environments.allocate_context(expression.span,producer,&self.entries[current].request.context,&bindings) else {return};
            let edges=bindings.iter().filter_map(|binding|binding.alternatives.clone().map(|alternatives|(binding.identity.unwrap(),alternatives))).collect::<Vec<_>>();
            if self.environments.merge(environment,&bindings,&edges,self.declarations).is_none() {return;}
            if bindings.iter().any(|binding|binding.mutable || binding.identity.is_some_and(|identity|self.effects.may_write(identity))) && !state.captures.contains_key(&environment) {
                if state.install(environment,&bindings,environment_state::Instance::Merged).is_none() {return;}
            }
            alternatives.push(environment_store::Alternative::Body {declaration:*declaration,environment:Some(environment)});
            captured_any=true;
        }
        if captured_any {value.alternatives=Some(environment_store::canonical_alternatives(alternatives));}
    }
    fn invoke(&mut self, expression: &Expr, callee: &expression_flow::Value, arguments: &[expression_flow::Value], state: &FlowState) -> Option<expression_flow::Invocation> {
        #[cfg(test)]
        {self.invocations+=1;*self.visits_by_site.entry((expression.span.start,expression.span.end)).or_default()+=1;if self.trace && self.invocations%128==0 {eprintln!("worklist-progress requests={} evaluations={} invocationVisits={} bindings={}",self.entries.len(),self.evaluations,self.invocations,state.len());}}
        if leaf_effects::observer_call(expression, self.effects)
            || !callee.origins.is_empty() && callee.origins.iter().all(|origin|
                matches!(origin,body_origins::Origin::Name(symbol,_) if leaf_effects::native_observer(symbol))) {
            return Some(expression_flow::Invocation { normal: Some(state.clone()), throwing: vec![state.clone()], result: expression_flow::Value::default(), facts: HashMap::new() });
        }
        if callee.console {
            return Some(expression_flow::Invocation { normal: Some(state.clone()), throwing: vec![state.clone()], result: expression_flow::Value::undefined(), facts: HashMap::new() });
        }
        if callee.for_each { return self.for_each(expression, callee, arguments, state); }
        if callee.origins.is_empty() || callee.origins.iter().any(|origin| !matches!(origin, body_origins::Origin::Body(_))) { return self.opaque_invocation(); }
        let mut normal = None;
        let mut throwing = None;
        let mut facts = HashMap::new();
        let mut result = None;
        let alternatives=callee.alternatives.clone().unwrap_or_else(||callee.origins.iter().map(|origin|match origin {
            body_origins::Origin::Body(declaration)=>environment_store::Alternative::Body{declaration:*declaration,environment:None},
            _=>environment_store::Alternative::Unknown,
        }).collect());
        let caller_identities=state.iter().map(|binding|binding.identity).collect::<HashSet<_>>();
        for alternative in alternatives {
            let environment_store::Alternative::Body{declaration,environment}=alternative else {return self.opaque_invocation()};
            let index = self.request(Request { declaration, producer: expression.span, context: self.call_context(declaration, expression.span), environment, caller: state.clone(), arguments: arguments.to_vec() });
            // Eager dispatch has at most one wrapper followed by an invocation-free
            // leaf. Arbitrary nested bodies and recursive calls remain scheduled.
            let leaf=self.terminal_leaf_return(declaration);
            let wrapper=!self.eager_wrapper_active && terminal_returns::wrapper(declaration,self.declarations,self.effects);
            if environment.is_none() && self.entries[index].queued && (leaf || wrapper) {
                self.work.retain(|pending| *pending != index);
                let previous=self.eager_wrapper_active;
                self.eager_wrapper_active|=wrapper;
                self.evaluate(index);
                self.eager_wrapper_active=previous;
            }
            if self.entries[index].opaque { return self.opaque_invocation(); }
            let mut outcome = self.entries[index].outcome.clone();
            for outgoing in [&mut outcome.normal,&mut outcome.returning,&mut outcome.throwing].into_iter().flatten() {
                retain_caller_bindings(outgoing,&caller_identities);
            }
            let returned = if outcome.normal.is_some() {
                Some(outcome.returned_value.clone().map_or_else(expression_flow::Value::undefined, |value| expression_flow::merge_value(value, expression_flow::Value::undefined(), self.declarations)))
            } else { outcome.returned_value.clone() };
            if let Some(value) = returned { result = Some(result.map_or_else(|| value.clone(), |previous| expression_flow::merge_value(previous, value.clone(), self.declarations))); }
            if !merge_state(&mut normal, outcome.normal, self.declarations) {return self.opaque_invocation();}
            if !merge_state(&mut normal, outcome.returning, self.declarations) {return self.opaque_invocation();}
            if !merge_state(&mut throwing, outcome.throwing, self.declarations) {return self.opaque_invocation();}
            merge_facts(&mut facts, outcome.facts, self.declarations);
        }
        Some(expression_flow::Invocation { normal, throwing: throwing.into_iter().collect(), result: result.unwrap_or_default(), facts })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recursive_mutable_array_growth_has_bounded_requests() {
        let source=crate::format("function walk(values:Mutable<int[]>,n:int):int{if(n===0){return values.length;}values.push(n);return walk(values,n-1);}function root(){const values:int[]=[];walk(values,3);}function main(){}").unwrap();
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();let mut bindings=Vec::new();collect(&root.body,&mut bindings,&effects);
        let mut engine=Engine::new(&declarations,&effects);assert!(engine.solve(root.name.span,&bindings,&[]).is_some());
        assert!(engine.entries.len()<=128,"{} requests",engine.entries.len());assert!(engine.evaluations<=2048,"{} evaluations",engine.evaluations);
    }
    #[test]
    fn pending_callees_finish_before_revisiting_their_common_caller() {
        let calls = 32;
        let body = "identity(1);".repeat(calls);
        let source = crate::format(&format!("function identity<T>(value:T):T{{return value;}}function outer(){{{body}}}function main(){{}}")).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let effects = effect_bodies::Index::new(&program).unwrap();
        let declarations = program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let outer = program.functions.iter().find(|function|function.name.text=="outer").unwrap();
        let mut engine = Engine::new(&declarations, &effects);
        let outcome = engine.solve(outer.name.span, &[], &[]).unwrap();
        assert_eq!(outcome.facts.len(), calls);
        assert!(outcome.facts.values().all(|contracts|contracts.as_ref().is_some_and(|contracts|contracts.len()==1 && contracts[0].required==1)));
        assert!(engine.evaluations <= calls + 2, "{} evaluations for {calls} independent calls", engine.evaluations);
        let evaluations = engine.evaluations;
        assert!(engine.solve(outer.name.span, &[], &[]).unwrap() == outcome);
        assert_eq!(engine.evaluations, evaluations);
    }
    #[test]
    fn independent_operator_calls_do_not_replay_completed_prefixes() {
        for calls in [8,16,32,64] {
            let body="increment(1);".repeat(calls);
            let source=crate::format(&format!("function increment(value:int):int{{return value+1;}}function outer(){{{body}}}function main(){{}}")).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();
            let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
            let mut engine=Engine::new(&declarations,&effects);
            let outcome=engine.solve(outer.name.span,&[],&[]).unwrap();
            assert_eq!(outcome.facts.len(),calls);
            assert!(engine.entries.len()<=calls+1);
            assert!(engine.evaluations<=calls+2,"{} evaluations for {calls} independent operator calls",engine.evaluations);
            assert!(engine.invocations<=3*calls,"{} invocation visits for {calls} independent operator calls",engine.invocations);
            println!("calls={calls} requests={} evaluations={} invocationVisits={}",engine.entries.len(),engine.evaluations,engine.invocations);
            let evaluations=engine.evaluations;
            assert!(engine.solve(outer.name.span,&[],&[]).unwrap()==outcome);
            assert_eq!(engine.evaluations,evaluations);
        }
    }
    #[test]
    #[ignore = "Opt-in whole-source scheduling diagnostic; run alone with an external bound"]
    fn trace_adapter_source_worklist_cost() {
        let source=include_str!("../../../../../../../benchmarks/runtime/workloads/generic_value_adapters.tsr");
        let mut program=crate::syntax::parse(source).unwrap();
        for function in &program.functions {eprintln!("source-function span={}..{} name={}",function.name.span.start,function.name.span.end,function.name.text);}
        validate(&mut program).unwrap();
    }
    #[test]
    fn independent_aggregate_returns_do_not_replay_completed_prefixes() {
        for (shape,result,expression) in [("array","int[]","[value]"),("record","Box","{value:value}"),("literal","int","1")] {
            for calls in [8,32,64] {
                let body="wrap(1);".repeat(calls);
                let source=crate::format(&format!("type Box={{value:int;}};function wrap(value:int):{result}{{return {expression};}}function outer(){{{body}}}function main(){{}}")).unwrap();
                assert_eq!(crate::format(&source).unwrap(),source);
                let program=crate::syntax::parse(&source).unwrap();
                let effects=effect_bodies::Index::new(&program).unwrap();
                let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
                let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
                let mut engine=Engine::new(&declarations,&effects);
                let outcome=engine.solve(outer.name.span,&[],&[]).unwrap();
                assert_eq!(outcome.facts.len(),calls);
                assert!(engine.entries.len()<=calls+1);
                println!("shape={shape} calls={calls} requests={} evaluations={} invocationVisits={}",engine.entries.len(),engine.evaluations,engine.invocations);
                assert!(engine.evaluations<=calls+2,"{shape}: {} evaluations for {calls} calls",engine.evaluations);
                assert!(engine.invocations<=3*calls,"{shape}: {} invocation visits for {calls} calls",engine.invocations);
            }
        }
    }
    #[test]
    fn nonleaf_resumption_preserves_completed_prefixes_and_finally_calls() {
        for (expression,cleanup_paths) in [("value",1),("value+1",2)] {
        for calls in [8,32,64] {
            let source=crate::format(&format!("function before():int{{return 1;}}function after():int{{return 2;}}function cleanup():int{{return 3;}}function increment(value:int):int{{return {expression};}}function wrap(value:int):int{{try{{const next=increment(value);return next;}}finally{{cleanup();}}}}function outer(){{before();{}after();}}function main(){{}}","wrap(1);".repeat(calls))).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
            let mut engine=Engine::new(&declarations,&effects);
            let outcome=engine.solve(outer.name.span,&[],&[]).unwrap();assert!(outcome.normal.is_some());
            for (text,expected) in [("before()",1),("after()",1),("cleanup()",cleanup_paths*calls)] {
                let position=source.rfind(text).unwrap();
                assert_eq!(engine.visits_by_site.get(&(position,position+text.len())),Some(&expected),"{text}: {calls} calls");
            }
            assert!(engine.invocations<=(3+cleanup_paths)*calls+2,"{} invocation visits for {calls} calls",engine.invocations);
        }
        }
    }
    #[test]
    fn nested_independent_calls_resume_without_replaying_prefixes() {
        for calls in [8,32,64] {
            let body="wrap(1);".repeat(calls);
            let source=crate::format(&format!("function increment(value:int):int{{return value+1;}}function wrap(value:int):int{{return increment(value);}}function outer(){{{body}}}function main(){{}}")).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();
            let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
            let mut engine=Engine::new(&declarations,&effects);
            let outcome=engine.solve(outer.name.span,&[],&[]).unwrap();
            assert_eq!(outcome.facts.len(),calls+1);
            println!("nested calls={calls} requests={} evaluations={} invocationVisits={}",engine.entries.len(),engine.evaluations,engine.invocations);
            assert!(engine.evaluations<=2*calls+2,"{} evaluations for {calls} nested calls",engine.evaluations);
            assert!(engine.invocations<=4*calls,"{} invocation visits for {calls} nested calls",engine.invocations);
        }
    }
    #[test]
    fn producing_context_distinguishes_helpers_and_bounds_recursive_cycles() {
        let declarations=HashMap::new();let program=crate::syntax::parse("function main(){}").unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();let mut engine=Engine::new(&declarations,&effects);
        let root=Span::new(1,2);let helper=Span::new(3,4);let factory=Span::new(5,6);
        let index=engine.request(Request {declaration:helper,producer:Span::new(10,11),context:vec![(root,root),(helper,Span::new(10,11))],environment:None,caller:FlowState::default(),arguments:vec![]});
        engine.current=Some(index);
        let first=engine.call_context(factory,Span::new(20,21));
        assert_eq!(first.len(),3);
        assert_eq!(engine.call_context(helper,Span::new(30,31)),engine.entries[index].request.context);
        engine.entries[index].request.context[1].1=Span::new(12,13);
        assert_ne!(first,engine.call_context(factory,Span::new(20,21)));
        let cycle=engine.call_context(root,Span::new(40,41));assert_eq!(cycle,vec![(root,root)]);
    }

    #[test]
    fn loop_array_join_keeps_prefix_and_repeated_tail_call_contracts() {
        let source=crate::format("function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function root(){const values=[one];let index=0;while(index<2){values.push(two);index=index+1;}const first=values.shift()!;const second=values.shift()!;first<int>(3);second<int>(4,7);}function main(){}").unwrap();
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();
        let outcome=Engine::new(&declarations,&effects).solve(root.name.span,&[],&[]).expect("bounded loop summary");
        for (statement,required) in root.body[root.body.len()-2..].iter().zip([1,2]) {
            let StmtKind::Expr(call)=&statement.kind else {panic!("call")};
            let candidates=outcome.facts[&(call.span.start,call.span.end)].as_ref().expect("retained callable contract");
            assert_eq!(candidates.len(),1);assert_eq!(candidates[0].required,required);
        }
    }
    #[test]
    fn terminal_return_requires_parameter_identity_and_no_entry_effects() {
        let source = crate::format("type Box={value:int;};function identity<T>(value:T):T{return value;}function increment(value:int):int{return value+1;}function divided(value:int,divisor:int):int{return value/divisor;}function dispatched(value:int):int{return identity(value);}function mutated(value:int):int{return value++;}function indexed(values:int[]):int{return values[0];}function defaulted(value:int=1):int{return value;}function packed(...values:int[]):int[]{return values;}function array(value:int):int[]{return [value];}function record(value:int):Box{return {value:value};}function literal(value:int):int{return 1;}function outer(value:int){function captured():int{return value;}function written(other:int):int{value=other;return other;}}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let effects = effect_bodies::Index::new(&program).unwrap();
        let declarations = HashMap::new();
        let engine = Engine::new(&declarations, &effects);
        for function in &program.functions {
            assert_eq!(engine.terminal_leaf_return(function.name.span), matches!(function.name.text.as_str(),"identity"|"increment"|"divided"|"array"|"record"|"literal"), "{}", function.name.text);
        }
    }
    #[test]
    fn caller_projection_preserves_unknown_identities_aliases_and_binding_order() {
        let known=crate::local_capture_plan::BindingId{owner:1,declaration_start:7};
        let foreign=crate::local_capture_plan::BindingId{owner:2,declaration_start:7};
        let binding=|name:&str,identity| Binding { alternatives:None,identity,name:name.into(),scope:Span::default(),targets:vec![Target::Unknown],origins:vec![body_origins::Origin::Unknown],selection:Selection::default(),elements:None,mutable:false,contract:None };
        let caller:FlowState=vec![binding("caller",Some(known)),binding("unknown",None)].into();
        let identities=caller.iter().map(|binding|binding.identity).collect::<HashSet<_>>();
        let mut outgoing:FlowState=vec![binding("first_unknown",None),binding("foreign",Some(foreign)),binding("first_alias",Some(known)),binding("second_alias",Some(known)),binding("second_unknown",None)].into();
        retain_caller_bindings(&mut outgoing,&identities);
        assert_eq!(outgoing.iter().map(|binding|binding.name.as_str()).collect::<Vec<_>>(),vec!["first_unknown","first_alias","second_alias","second_unknown"]);
        let identities=HashSet::from([Some(known)]);
        retain_caller_bindings(&mut outgoing,&identities);
        assert_eq!(outgoing.iter().map(|binding|binding.name.as_str()).collect::<Vec<_>>(),vec!["first_alias","second_alias"]);
    }
    #[test]
    fn direct_transitive_recursive_and_throwing_writes_reach_caller_call_facts() {
        for body in [
            "let action=one;function change(){action=two;}change();return action<T>(value);",
            "let action=one;const change=()=>{action=two;};change();return action<T>(value);",
            "let action=one;function change(){action=two;}if(flag){change();}return action<T>(value);",
            "let action=one;function change():boolean{action=two;return flag;}one<T>((change(),value));return action<T>(value);",
            "let action=one;function change(){action=two;}function relay(){change();}relay();return action<T>(value);",
            "let action=one;function change(remaining:int){if(remaining===0){action=two;return;}change(remaining-1);}change(2);return action<T>(value);",
            "let action=one;function change(){action=two;throw 1;}try{change();}catch(error){return action<T>(value);}return value;",
        ] {
            let source=crate::format(&format!("function one<T>(value:T):T{{return value;}}function two<T>(value:T,other:T):T{{return other;}}function outer<T>(flag:boolean,value:T):T{{{body}}}function main(){{}}")).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();
            let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
            let mut engine=Engine::new(&declarations,&effects);
            let arguments=vec![expression_flow::Value::default();2];
            let outcome=engine.solve(outer.name.span,&[],&arguments).unwrap_or_else(||panic!("live capture body: {body}"));
            let count=engine.entries.len();
            let repeated=engine.solve(outer.name.span,&[],&arguments).unwrap();
            assert!(repeated==outcome);assert_eq!(engine.entries.len(),count);
            let mut probe=outer.clone();let mut call=None;
            crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
                if let ExprKind::Call(callee,_) =&expression.kind && name(callee).is_some_and(|(name,_)|name=="action") {call=Some((expression.span.start,expression.span.end));} Ok(())
            }).unwrap();
            assert!(outcome.facts.get(&call.unwrap()).and_then(|contracts|contracts.as_ref()).is_some_and(|contracts|contracts.iter().any(|contract|contract.required==2)),"{body}");
        }
    }
}

#[cfg(test)]
mod guard_tests {
    use super::*;
    #[test]
    fn supplied_false_parameter_excludes_guarded_capture_write() {
        let source=crate::format("function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function outer<T>(value:T):T{let action=one;function change(flag:boolean){if(flag){action=two;}}change(false);return action<T>(value);}function main(){}").unwrap();
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
        let outcome=Engine::new(&declarations,&effects).solve(outer.name.span,&[],&[expression_flow::Value::default()]).unwrap();
        let mut probe=outer.clone();let mut call=None;
        crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
            if let ExprKind::Call(callee,_) =&expression.kind && name(callee).is_some_and(|(name,_)|name=="action") {call=Some((expression.span.start,expression.span.end));} Ok(())
        }).unwrap();
        assert!(outcome.facts.get(&call.unwrap()).and_then(|contracts|contracts.as_ref()).is_some_and(|contracts|contracts.iter().all(|contract|contract.required==1)));
    }
}

#[cfg(test)]
mod opaque_tests {
    use super::*;
    #[test]
    fn missing_callback_body_invalidates_transitive_parent_summary() {
        let source = crate::format("function relay(callback:()=>void){callback();}function outer(callback:()=>void){relay(callback);}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let effects = effect_bodies::Index::new(&program).unwrap();
        let declarations = program.functions.iter().map(|function| (function.name.text.clone(), contract(function.type_parameters.len(), &function.parameter_initializers, function.rest_parameter, function.name.span))).collect::<HashMap<_,_>>();
        let outer = program.functions.iter().find(|function| function.name.text == "outer").unwrap();
        let mut argument = expression_flow::Value::default();
        argument.origins = vec![body_origins::Origin::Body(Span { start: source.len()+1, end: source.len()+2 })];
        let mut engine = Engine::new(&declarations, &effects);
        assert!(engine.solve(outer.name.span, &[], &[argument.clone()]).is_none(), "opaque callback must not certify its caller");
        assert!(engine.solve(outer.name.span, &[], &[argument]).is_none(), "cached parents must remain opaque");
        assert!(engine.solve(outer.name.span, &[], &[expression_flow::Value::default()]).is_none(), "unknown callback must not certify its caller");
    }
}

#[cfg(test)]
mod default_tests {
    use super::*;
    #[test]
    fn omitted_defaults_run_in_parameter_order_before_the_body() {
        for (call, required) in [("change()",2), ("change(undefined)",2), ("change(false)",1), ("throwing",2)] {
        let source=crate::format(&format!("function one<T>(value:T):T{{return value;}}function two<T>(value:T,other:T):T{{return other;}}function outer<T>(value:T):T{{let action=one;function change(first:boolean=(action=two,false),second:boolean=first){{if(second){{action=one;}}}}{call};return action<T>(value);}}function main(){{}}")).unwrap();
        let source = if call == "throwing" { crate::format(&source.replace("(action = two, false)", "(action = two, fail())").replace("throwing;", "try { change(); } catch(error) { return action<T>(value); }").replace("function main()", "function fail():boolean { throw 1; } function main()")).unwrap() } else { source };
        if call == "throwing" { assert!(source.contains("fail())"), "throwing default witness must call fail"); }
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
        let outcome=Engine::new(&declarations,&effects).solve(outer.name.span,&[],&[expression_flow::Value::default()]).unwrap();
        let mut probe=outer.clone();let mut call=None;
        crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
            if let ExprKind::Call(callee,_) =&expression.kind && name(callee).is_some_and(|(name,_)|name=="action") {call=Some((expression.span.start,expression.span.end));} Ok(())
        }).unwrap();
        let contracts=outcome.facts.get(&call.unwrap()).and_then(|contracts|contracts.as_ref()).unwrap();
        assert!(contracts.iter().all(|contract|contract.required==required),"{call:?}: expected {required}, found {:?}", contracts.iter().map(|contract|contract.required).collect::<Vec<_>>());
        }
    }
}

#[cfg(test)]
mod rest_tests {
    use super::*;
    #[test]
    fn indexed_rest_callback_projects_its_capture_write() {
        for (invocation, required) in [("relay(change)",Some(2)), ("const callbacks=[change];callbacks[0]()",Some(2)), ("const callbacks=[change];const saved=callbacks;saved[0]()",Some(2)), ("const callbacks=[noop,change];callbacks[0]()",Some(1)), ("const callbacks=[noop,change];callbacks[1]()",Some(2)), ("const callbacks=[change];const saved=callbacks;callbacks[0]=noop;saved[0]()",None)] {
        let source=crate::format("function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function noop(){}function relay(...callbacks:Function<void>[]){callbacks[0]();}function outer<T>(value:T):T{let action=one;function change(){action=two;}INVOCATION;return action<T>(value);}function main(){}".replace("INVOCATION", invocation).as_str()).unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let outer=program.functions.iter().find(|function|function.name.text=="outer").unwrap();
        let outcome=Engine::new(&declarations,&effects).solve(outer.name.span,&[],&[expression_flow::Value::default()]);
        if required.is_none() { assert!(outcome.is_none(), "alias mutation must invalidate stale element provenance"); continue; }
        assert!(outcome.is_some(),"rest callback body must be traceable");
        let outcome=outcome.unwrap();
        let mut probe=outer.clone();let mut call=None;
        crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
            if let ExprKind::Call(callee,_) =&expression.kind && name(callee).is_some_and(|(name,_)|name=="action") {call=Some((expression.span.start,expression.span.end));} Ok(())
        }).unwrap();
        assert!(outcome.facts.get(&call.unwrap()).and_then(|contracts|contracts.as_ref()).is_some_and(|contracts|contracts.iter().all(|contract|contract.required==required.unwrap())));
        }
    }
}

#[cfg(test)]
mod returned_value_tests {
    use super::*;
    #[test]
    fn pending_returns_survive_cleanup_and_are_replaced_by_abrupt_completion() {
        for (body, expected) in [
            ("let action=one;try{return action;}finally{action=two;}",Some("one")),
            ("try{return one;}finally{return two;}",Some("two")),
            ("try{return one;}finally{throw 1;}",None),
            ("try{try{return one;}finally{throw 1;}}catch(error){return two;}",Some("two")),
        ] {
            let source=crate::format(&format!("function one(){{}}function two(){{}}function pick(){{{body}}}function main(){{}}")).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let pick=program.functions.iter().find(|function|function.name.text=="pick").unwrap();
            let outcome=Engine::new(&declarations,&effects).solve(pick.name.span,&[],&[]).unwrap();
            if let Some(expected)=expected {
                let declaration=program.functions.iter().find(|function|function.name.text==expected).unwrap().name.span;
                assert_eq!(outcome.returned_value.unwrap().origins,vec![body_origins::Origin::Body(declaration)],"{body}");
                assert!(outcome.returning.is_some());
            } else {assert!(outcome.returned_value.is_none());assert!(outcome.returning.is_none());assert!(outcome.throwing.is_some());}
        }
    }
}

#[cfg(test)]
mod factory_environment_tests {
    use super::*;
    #[test]
    fn separate_factory_instances_preserve_captured_callable_contracts() {
        let mut failures=Vec::new();
        for factory in [
            "function factory<A>(action:A){function select(){return action;}return select;}",
            "function factory<A>(action:A){return ()=>action;}",
        ] {
            let source=crate::format(&format!("function one<T>(value:T):T{{return value;}}function two<T>(value:T,other:T):T{{return other;}}{factory}function root<T>(value:T):T{{const first=factory(one);const second=factory(two);const saved=first();const wrong=second();saved<T>(value);wrong<T>(value);return value;}}function main(){{}}")).unwrap();
            assert_eq!(crate::format(&source).unwrap(),source);
            let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();
            let mut engine=Engine::new(&declarations,&effects);
            let Some(outcome)=engine.solve(root.name.span,&[],&[expression_flow::Value::default()]) else { failures.push(format!("{factory}: producer environment lost")); continue; };
            let count=engine.entries.len();
            assert!(engine.solve(root.name.span,&[],&[expression_flow::Value::default()])==Some(outcome.clone()));
            assert_eq!(engine.entries.len(),count,"repeated requests reuse stable environments");
            let mut probe=root.clone();
            crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
                if let ExprKind::Call(callee,_) = &expression.kind && let Some((name,_))=name(callee) && matches!(name,"saved"|"wrong") {
                    let expected=if name=="saved" {1} else {2};
                    let contracts=outcome.facts.get(&(expression.span.start,expression.span.end)).and_then(|contracts|contracts.as_ref()).expect("captured returned callable contract");
                    assert!(contracts.iter().all(|contract|contract.required==expected),"{name}: independent producer contracts must not merge");
                }
                Ok(())
            }).unwrap();
        }
        assert!(failures.is_empty(),"{}",failures.join("\n"));
    }
}

#[cfg(test)]
mod environment_mutation_guard_tests {
    use super::*;
    #[test]
    fn mutable_environment_updates_and_live_capture_conflicts_remain_opaque() {
        for (factory,expected_required) in [
            ("function factory<A>(initial:A){let action=initial;return ()=>{action=two;return action;};}",Some(2)),
            ("function factory<A>(action:A){const saved=()=>action;action=two;return saved;}",None),
        ] {
            let source=crate::format(&format!("function one<T>(value:T):T{{return value;}}function two<T>(value:T,other:T):T{{return other;}}{factory}function root<T>(value:T):T{{const selected=factory(one);const action=selected();return action<T>(value);}}function main(){{}}")).unwrap();
            let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();
            let outcome=Engine::new(&declarations,&effects).solve(root.name.span,&[],&[expression_flow::Value::default()]);
            if let Some(required)=expected_required {
                let outcome=outcome.expect("owned replacement returns its updated declaration");
                let mut probe=root.clone();let mut checked=false;
                crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
                    if let ExprKind::Call(callee,_)=&expression.kind && name(callee).is_some_and(|(name,_)|name=="action") {
                        let contracts=outcome.facts[&(expression.span.start,expression.span.end)].as_ref().expect("selected contract");
                        assert!(!contracts.is_empty() && contracts.iter().all(|contract|contract.required==required));checked=true;
                    }
                    Ok(())
                }).unwrap();assert!(checked);
            } else {assert!(outcome.is_none(),"live capture conflicts must remain opaque: {factory}");}
        }
    }
}

#[cfg(test)]
mod mutable_environment_sequence_tests {
    use super::*;
    #[test]
    fn owned_factory_updates_are_point_sensitive_and_instances_stay_separate() {
        let source=crate::format("function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function factory<A>(initial:A){let action=initial;return (restore:boolean)=>{const previous=action;if(restore){action=one;}else{action=two;}return previous;};}function root<T>(value:T):T{const first=factory(one);const second=factory(two);const original=first(false);const changed=first(true);const restored=first(false);const independent=second(true);original<T>(value);changed<T>(value,value);restored<T>(value);independent<T>(value,value);return value;}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();
        let mut engine=Engine::new(&declarations,&effects);
        let outcome=engine.solve(root.name.span,&[],&[expression_flow::Value::default()]).expect("owned mutable factory state must survive sequential invocations");
        let count=engine.entries.len();
        assert!(engine.solve(root.name.span,&[],&[expression_flow::Value::default()])==Some(outcome.clone()));
        assert_eq!(engine.entries.len(),count,"stable requests must converge without growing environment identities");
        let mut probe=root.clone();
        let mut observed=0;
        crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression|{
            if let ExprKind::Call(callee,_)=&expression.kind && let Some((name,_))=name(callee) && matches!(name,"original"|"changed"|"restored"|"independent") {
                let expected=if matches!(name,"changed"|"independent") {2}else{1};
                let contracts=outcome.facts.get(&(expression.span.start,expression.span.end)).and_then(|contracts|contracts.as_ref()).expect("point-sensitive captured contract");
                assert!(!contracts.is_empty());
                assert!(contracts.iter().all(|contract|contract.required==expected),"{name}: stale or merged instance contract");
                observed+=1;
            }
            Ok(())
        }).unwrap();
        assert_eq!(observed,4);
    }
}
