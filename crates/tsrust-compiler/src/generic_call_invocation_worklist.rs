//! Inter-call dependencies converge on an explicit finite worklist.
use super::*;
#[derive(Clone, PartialEq, Eq, Hash)]
struct Request {
    declaration: Span,
    caller: Vec<Binding>,
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
}
fn merge_state(current: &mut Option<Vec<Binding>>, incoming: Option<Vec<Binding>>, declarations: &HashMap<String, Contract>) {
    if let Some(incoming) = incoming {
        if let Some(current) = current { join(current, &incoming, declarations); }
        else { *current = Some(incoming); }
    }
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
        Self { declarations, effects, lookup: HashMap::new(), entries: Vec::new(), work: Vec::new(), current: None }
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
        let root = self.request(Request { declaration, caller: caller.to_vec(), arguments: arguments.to_vec() });
        while let Some(index) = self.work.pop() {
            self.entries[index].queued = false;
            let previous_opaque = self.entries[index].opaque;
            self.current = Some(index);
            let request = self.entries[index].request.clone();
            let declarations = self.declarations;
            let effects = self.effects;
            let incoming = summary_flow::invoke_with_dispatch(request.declaration, &request.caller, &request.arguments, declarations, effects, self);
            self.current = None;
            let previous = self.entries[index].outcome.clone();
            if let Some(incoming) = incoming {
                let outcome = &mut self.entries[index].outcome;
                merge_state(&mut outcome.normal, incoming.normal, declarations);
                merge_state(&mut outcome.returning, incoming.returning, declarations);
                merge_state(&mut outcome.throwing, incoming.throwing, declarations);
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
    fn invoke(&mut self, _: &Expr, callee: &expression_flow::Value, arguments: &[expression_flow::Value], state: &[Binding]) -> Option<expression_flow::Invocation> {
        if callee.origins.is_empty() || callee.origins.iter().any(|origin| !matches!(origin, body_origins::Origin::Body(_))) { return self.opaque_invocation(); }
        let mut normal = None;
        let mut throwing = None;
        let mut facts = HashMap::new();
        for origin in &callee.origins {
            let body_origins::Origin::Body(declaration) = origin else { unreachable!() };
            let index = self.request(Request { declaration: *declaration, caller: state.to_vec(), arguments: arguments.to_vec() });
            if self.entries[index].opaque { return self.opaque_invocation(); }
            let outcome = self.entries[index].outcome.clone();
            merge_state(&mut normal, outcome.normal, self.declarations);
            merge_state(&mut normal, outcome.returning, self.declarations);
            merge_state(&mut throwing, outcome.throwing, self.declarations);
            merge_facts(&mut facts, outcome.facts, self.declarations);
        }
        Some(expression_flow::Invocation { normal, throwing: throwing.into_iter().collect(), result: expression_flow::Value::default(), facts })
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
