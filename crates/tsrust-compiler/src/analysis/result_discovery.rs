//! Callable result discovery with dependency-driven retries.
use super::*;
use std::collections::VecDeque;

impl Analyzer {
    pub(super) fn note_result_wait(&mut self, target: usize, span: Span) {
        self.result_waits.insert(target, span);
        if let Some(caller) = self.discovering_function {
            self.result_dependents.entry(target).or_default().insert(caller);
        }
    }

    pub(super) fn discover_results(&mut self) -> Result<(), Diagnostic> {
        let effects = self.effects_only;
        let discovering = self.discovering_results;
        let caller = self.discovering_function;
        self.effects_only = true;
        self.discovering_results = true;
        self.result_dependents.clear();
        let discovered = (|| {
            let mut pending = VecDeque::new();
            let mut queued = HashSet::new();
            let mut frontier = 0;
            let mut waiting_for_captures = Vec::new();
            loop {
                // Discovery can instantiate new source functions. Admit each
                // once without rescanning every unresolved body after a result.
                // Capturing declarations need their defining context before result discovery.
                // Body scheduling retries once module/lexical storage is prepared.
                while frontier < self.program.functions.len() {
                    if self.callable_result_type(self.signatures[frontier].result)
                        && !self.known_results.contains_key(&frontier)
                    {
                        if self.named_captures.needs_context(frontier) && !self.named_captures.ready(frontier) {
                            waiting_for_captures.push(frontier);
                        } else {
                            pending.push_back(frontier);
                            queued.insert(frontier);
                        }
                    }
                    frontier += 1;
                }
                // Discovering one factory can prepare another factory's captures.
                waiting_for_captures.retain(|&index| {
                    if !self.named_captures.ready(index) { return true; }
                    if queued.insert(index) { pending.push_back(index); }
                    false
                });
                let Some(index) = pending.pop_front() else { break; };
                queued.remove(&index);
                if self.known_results.contains_key(&index) { continue; }
                self.discovering_function = Some(index);
                let mut body = self.program.functions[index].body.clone();
                let mut context = self.function_context(index, &mut body)?;
                if let Some(ty) = self.discover_body(&mut context, Self::uninitialized_body(&mut body), self.signatures[index].result)? {
                    self.known_results.insert(index, ty);
                    let mut dependents = self.result_dependents.remove(&index).unwrap_or_default().into_iter().collect::<Vec<_>>();
                    dependents.sort_unstable();
                    for dependent in dependents {
                        if !self.known_results.contains_key(&dependent) && queued.insert(dependent) {
                            pending.push_back(dependent);
                        }
                    }
                }
            }
            Ok::<(), Diagnostic>(())
        })();
        self.discovering_function = caller;
        self.discovering_results = discovering;
        self.effects_only = effects;
        self.result_dependents.clear();
        discovered?;
        for (index, signature) in self.signatures.iter().enumerate() {
            if self.callable_result_type(signature.result)
                && !self.known_results.contains_key(&index)
                && !self.templates.contains_key(&index)
                && (!self.named_captures.needs_context(index) || self.named_captures.ready(index))
            {
                let mut error = Diagnostic::new(
                    "unsupported",
                    "Recursive callable results need one resolvable environment origin; recursive environment storage is unsupported.",
                    self.program.functions[index].name.span,
                );
                if let Some(&span) = self.result_waits.get(&index) {
                    error = error.related(
                        span,
                        "This callable result has no resolved concrete return origin.",
                    );
                }
                return Err(error);
            }
        }
        Ok(())
    }

}

#[cfg(test)]
mod defaulted_alias_dependency_tests {
 use super::*;
 fn recursive_probe(source:&str)->Analyzer {
  let mut analyzer=Analyzer::new(crate::syntax::parse(source).unwrap()).unwrap();
  analyzer.assumed_modes=Some(analyzer.signatures.iter().map(|signature|signature.params.iter().map(|ty|if ty.is_copy(){Mode::Copy}else{Mode::Read}).collect()).collect());
  analyzer.effects_only=true;analyzer.speculative_access_run=true;analyzer
 }
 fn states(analyzer:&Analyzer)->String {
  let states=analyzer.program.functions.iter().enumerate().map(|(id,function)|(id,function.name.text.clone(),analyzer.known_results.contains_key(&id),analyzer.checked[id].is_some(),analyzer.visiting[id],analyzer.templates.contains_key(&id))).collect::<Vec<_>>();
  format!("states(id,name,known,checked,visiting,template)={states:?}; waits={:?}; dependents={:?}",analyzer.result_waits,analyzer.result_dependents)
 }
 #[test]
 fn defaulted_generic_async_alias_discovery_preserves_selected_callee_readiness(){
  for (parameter,arguments) in [("message:string", "7,'explicit'"),("message:int=0", "7,2"),("message?:string", "7,'explicit'"),("message:string='message'", "7,'explicit'"),("message:string='message'", "7")] {
   let source=format!("async function echo<T>(value:T,{parameter}):Task<T>{{return value;}}async function main():Task<void>{{const alias=echo;console.log(await alias({arguments}));}}");
   let source=crate::format(&source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
   let mut ordinary=Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
   let main=ordinary.function_names["main"];
   let first=ordinary.function(main);
   let mut snapshot=format!("parameter={parameter:?}; arguments={arguments:?}; ordinary first pass={first:?}; {}",states(&ordinary));
   // Enter the recursive pass only when the ordinary pass actually requests it,
   // matching analyze/analyze_recursive rather than forcing every control there.
   if first.as_ref().is_err_and(|error|error.code=="recursion-inference") {
    let mut probe=recursive_probe(&source);
    let entry=probe.function_names["async#entry"];
    probe.discovering_results=true;probe.discovering_function=Some(entry);
    let mut body=probe.program.functions[entry].body.clone();
    let mut context=probe.function_context(entry,&mut body).unwrap();
    let result=probe.signatures[entry].result;
    let seed=probe.discover_body(&mut context,Analyzer::uninitialized_body(&mut body),result);
    snapshot.push_str(&format!("; first actual fallback seed={seed:?}; {}",states(&probe)));
    let mut complete=recursive_probe(&source);
    let discovery=complete.discover_results();
    snapshot.push_str(&format!("; full fallback discovery={discovery:?}; {}",states(&complete)));
   }
   let checked=crate::check(&source).unwrap_or_else(|error|panic!("{snapshot}; pipeline error={error:?}"));
   drop(checked.emit_rust());
  }
 }
}

#[cfg(test)]
#[test]
fn selected_callable_defers_while_outer_discovery_temporarily_checks_full_effects(){
    fn probe()->Analyzer{
        let mut analyzer=Analyzer::new(crate::syntax::parse("function make():Function<int>{return ():int=>7;}function main(){}").unwrap()).unwrap();
        analyzer.assumed_modes=Some(analyzer.signatures.iter().map(|signature|signature.params.iter().map(|ty|if ty.is_copy(){Mode::Copy}else{Mode::Read}).collect()).collect());
        analyzer
    }
    let mut pending=probe();let target=pending.function_names["make"];let caller=pending.function_names["main"];let span=Span::new(7,11);
    pending.discovering_function=Some(caller);pending.discovering_results=false;pending.effects_only=false;
    let error=pending.prepare_selected_task_function(target,span,false).expect_err("temporary full mode must not restart the active global discovery queue");
    assert_eq!(error.code,"callable-result-pending");assert_eq!(error.span,span);
    assert_eq!(pending.result_waits.get(&target),Some(&span));assert!(pending.result_dependents[&target].contains(&caller));
    assert!(pending.checked[target].is_none());assert!(!pending.visiting[target]);assert_eq!(pending.discovering_function,Some(caller));assert!(!pending.discovering_results);assert!(!pending.effects_only);
    let mut known=probe();known.discover_results().unwrap();let target=known.function_names["make"];let caller=known.function_names["main"];
    assert!(known.known_results.contains_key(&target));assert!(known.checked[target].is_none());
    known.discovering_function=Some(caller);known.discovering_results=false;known.effects_only=false;
    known.prepare_selected_task_function(target,span,false).unwrap();
    assert!(known.checked[target].is_some(),"a known callable origin still requires its selected error body");
    assert_eq!(known.discovering_function,Some(caller));assert!(!known.discovering_results);assert!(!known.effects_only);
}

#[cfg(test)]
#[test]
fn direct_function_request_preserves_active_discovery_and_checks_known_callable_body(){
    fn probe()->Analyzer{
        let mut analyzer=Analyzer::new(crate::syntax::parse("function make():Function<int>{return ():int=>7;}function main(){}").unwrap()).unwrap();
        analyzer.assumed_modes=Some(analyzer.signatures.iter().map(|signature|signature.params.iter().map(|ty|if ty.is_copy(){Mode::Copy}else{Mode::Read}).collect()).collect());
        analyzer
    }
    let mut pending=probe();let target=pending.function_names["make"];let caller=pending.function_names["main"];let span=pending.program.functions[target].name.span;
    pending.discovering_function=Some(caller);pending.discovering_results=false;pending.effects_only=false;
    let error=pending.function(target).expect_err("direct scheduling must not restart an active discovery queue");
    assert_eq!(error.code,"callable-result-pending");assert_eq!(error.span,span);
    assert_eq!(pending.result_waits.get(&target),Some(&span));assert!(pending.result_dependents[&target].contains(&caller));
    assert!(pending.checked[target].is_none());assert!(!pending.visiting[target]);assert_eq!(pending.discovering_function,Some(caller));assert!(!pending.discovering_results);assert!(!pending.effects_only);
    let mut known=probe();known.discover_results().unwrap();let target=known.function_names["make"];let caller=known.function_names["main"];
    assert!(known.known_results.contains_key(&target));assert!(known.checked[target].is_none());
    known.discovering_function=Some(caller);known.discovering_results=false;known.effects_only=false;
    known.function(target).unwrap();
    assert!(known.checked[target].is_some());assert_eq!(known.discovering_function,Some(caller));assert!(!known.discovering_results);assert!(!known.effects_only);
}
