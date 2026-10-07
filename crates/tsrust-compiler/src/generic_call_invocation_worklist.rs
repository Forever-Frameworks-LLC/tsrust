//! Inter-call dependencies converge on an explicit finite worklist.
use super::*;
#[derive(Clone, PartialEq, Eq, Hash)]
struct Request {
    declaration: Span,
    producer: Span,
    environment: Option<environment_store::Handle>,
    caller: FlowState,
    arguments: Vec<expression_flow::Value>,
}
struct Entry {
    request: Request,
    outcome: summary_flow::Outcome,
    opaque: bool,
    queued: bool,
    dependents: HashSet<usize>,
}
pub(super) struct Engine<'a> {
    declarations: &'a HashMap<String, Contract>,
    effects: &'a effect_bodies::Index,
    lookup: HashMap<Request, usize>,
    entries: Vec<Entry>,
    work: Vec<usize>,
    current: Option<usize>,
    environments: environment_store::Store,
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
        Self { declarations, effects, lookup: HashMap::new(), entries: Vec::new(), work: Vec::new(), current: None, environments: environment_store::Store::default() }
    }
    fn request(&mut self, request: Request) -> usize {
        let index = if let Some(&index) = self.lookup.get(&request) { index } else {
            let index = self.entries.len();
            self.lookup.insert(request.clone(), index);
            self.entries.push(Entry { request, outcome: summary_flow::Outcome::default(), opaque: false, queued: true, dependents: HashSet::new() });
            self.work.push(index);
            index
        };
        if let Some(parent) = self.current { self.entries[index].dependents.insert(parent); }
        index
    }
    fn for_each(&mut self, expression: &Expr, callee: &expression_flow::Value, arguments: &[expression_flow::Value], state: &FlowState) -> Option<expression_flow::Invocation> {
        let (Some(elements), Some(callback)) = (&callee.elements, arguments.first()) else { return self.opaque_invocation(); };
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
                let index = self.request(Request { declaration, producer: expression.span, environment, caller: incoming.clone(), arguments: callback_arguments });
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
        let root = self.request(Request { declaration, producer: declaration, environment: None, caller: caller.to_vec().into(), arguments: arguments.to_vec() });
        while let Some(index) = self.work.pop() {
            self.entries[index].queued = false;
            let previous_opaque = self.entries[index].opaque;
            self.current = Some(index);
            let request = self.entries[index].request.clone();
            let declarations = self.declarations;
            let effects = self.effects;
            let mut caller = request.caller.clone();
            let available = if let Some(environment) = request.environment {
                self.environments.subscribe(environment,index).is_some() && self.environments.get(environment).is_some_and(|record| {
                    if record.bindings.iter().any(|captured|caller.iter().any(|binding|binding.identity==captured.identity && binding!=captured)) {return false;}
                    for captured in &record.bindings {
                        caller.retain(|binding|binding.identity!=captured.identity);
                        caller.push(captured.clone());
                    }
                    true
                })
            } else { true };
            let captured_before=request.environment.and_then(|handle|self.environments.get(handle).map(|record|record.bindings.clone()));
            let incoming = if available { summary_flow::invoke_with_dispatch(request.declaration, &caller, &request.arguments, declarations, effects, self) } else { None };
            let readers=self.environments.take_changed_readers();
            for reader in readers {self.enqueue(reader);}
            self.current = None;
            let previous = self.entries[index].outcome.clone();
            if let Some(incoming) = incoming {
                if let Some(captured)=&captured_before && [&incoming.normal,&incoming.returning,&incoming.throwing].into_iter().flatten().any(|state|captured.iter().any(|binding|state.iter().find(|outgoing|outgoing.identity==binding.identity)!=Some(binding))) {
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
                for dependent in dependents { self.enqueue(dependent); }
            }
        }
        (!self.entries[root].opaque).then(|| self.entries[root].outcome.clone())
    }
}
impl expression_flow::Dispatch for Engine<'_> {
    fn reject(&mut self) {self.opaque_invocation();}
    fn validate_escape(&mut self, value: &expression_flow::Value, state: &FlowState) -> bool {
        let alternatives=value.alternatives.iter().flatten().chain(value.elements.iter().flatten().flat_map(|element|element.alternatives.iter().flatten()));
        alternatives.into_iter().all(|alternative|match alternative {
            environment_store::Alternative::Body{environment:Some(handle),..}=>self.environments.get(*handle).is_some_and(|record|record.bindings.iter().all(|captured|state.iter().find(|binding|binding.identity==captured.identity).is_none_or(|binding|binding==captured))),
            _=>true,
        })
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
            // Mutable captures still use the existing live-caller path until
            // ownership-aware instance updates are implemented.
            if bindings.iter().any(|binding|binding.mutable) {return;}
            let Some(environment)=self.environments.allocate(expression.span,producer,&bindings) else {return};
            let edges=bindings.iter().filter_map(|binding|binding.alternatives.clone().map(|alternatives|(binding.identity.unwrap(),alternatives))).collect::<Vec<_>>();
            if self.environments.merge(environment,&bindings,&edges,self.declarations).is_none() {return;}
            alternatives.push(environment_store::Alternative::Body {declaration:*declaration,environment:Some(environment)});
            captured_any=true;
        }
        if captured_any {value.alternatives=Some(environment_store::canonical_alternatives(alternatives));}
    }
    fn invoke(&mut self, expression: &Expr, callee: &expression_flow::Value, arguments: &[expression_flow::Value], state: &FlowState) -> Option<expression_flow::Invocation> {
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
        for alternative in alternatives {
            let environment_store::Alternative::Body{declaration,environment}=alternative else {return self.opaque_invocation()};
            let index = self.request(Request { declaration, producer: expression.span, environment, caller: state.clone(), arguments: arguments.to_vec() });
            if self.entries[index].opaque { return self.opaque_invocation(); }
            let mut outcome = self.entries[index].outcome.clone();
            for outgoing in [&mut outcome.normal,&mut outcome.returning,&mut outcome.throwing].into_iter().flatten() {
                outgoing.retain(|binding|state.iter().any(|original|original.identity==binding.identity));
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
            let outcome=engine.solve(outer.name.span,&[],&arguments).unwrap();
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
        for factory in [
            "function factory<A>(action:A){return ()=>{action=two;return action;};}",
            "function factory<A>(action:A){const saved=()=>action;action=two;return saved;}",
        ] {
            let source=crate::format(&format!("function one<T>(value:T):T{{return value;}}function two<T>(value:T,other:T):T{{return other;}}{factory}function root<T>(value:T):T{{const selected=factory(one);const action=selected();return action<T>(value);}}function main(){{}}")).unwrap();
            let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
            let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
            let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();
            assert!(Engine::new(&declarations,&effects).solve(root.name.span,&[],&[expression_flow::Value::default()]).is_none(),"unimplemented mutable capture semantics must not certify stale snapshots: {factory}");
        }
    }
}

#[cfg(test)]
mod mutable_environment_sequence_tests {
    use super::*;
    #[test]
    fn owned_factory_updates_are_point_sensitive_and_instances_stay_separate() {
        let source=crate::format("function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function factory<A>(action:A){return (restore:boolean)=>{const previous=action;if(restore){action=one;}else{action=two;}return previous;};}function root<T>(value:T):T{const first=factory(one);const second=factory(two);const original=first(false);const changed=first(true);const restored=first(false);const independent=second(true);original<T>(value);changed<T>(value,value);restored<T>(value);independent<T>(value,value);return value;}function main(){}").unwrap();
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
