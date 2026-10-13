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
            loop {
                // Discovery can instantiate new source functions. Admit each
                // once without rescanning every unresolved body after a result.
                while frontier < self.program.functions.len() {
                    if self.callable_result_type(self.signatures[frontier].result)
                        && !self.known_results.contains_key(&frontier)
                    {
                        pending.push_back(frontier);
                        queued.insert(frontier);
                    }
                    frontier += 1;
                }
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
