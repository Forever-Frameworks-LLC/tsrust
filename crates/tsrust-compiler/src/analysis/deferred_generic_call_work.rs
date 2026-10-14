//! Deferred invocation preparation is separate from body scheduling and access.
use super::*;
pub(super) struct Request { pub binding: Option<usize>, pub origin: usize, pub once: bool, pub expected: Option<Type>, pub checked_receiver: bool }
pub(super) struct Selection { pub origins: Vec<usize>, pub primary: usize }
impl Analyzer {
    pub(super) fn deferred_call_selection(&self, context: &Context, callee: &Expr, origin: usize, span: Span) -> Result<Selection, Diagnostic> {
        let declaration = self.deferred_generic_values[&origin].declaration.clone();
        let reachable_origins = if !context.template
            && let Some(reachable) = self.generic_functions.reachable_declarations(&context.function_name, span, callee.span)
        {
            self.polymorphic_storage.reachable_origins(origin, reachable, |member| {
                self.deferred_generic_values.get(&member)
                    .and_then(|source| self.generic_functions.template(&source.declaration))
                    .map(|template| template.name.span)
            }, span)?
        } else if self.polymorphic_storage.contains(origin) {
            // Generated contextual adapters have no original source call proof.
            // Retain every matching environment instead of guessing the first.
            let source = self.generic_functions.template(&declaration).expect("deferred source declaration").name.span;
            self.polymorphic_storage.reachable_origins(origin, &[source], |member| {
                self.deferred_generic_values.get(&member)
                    .and_then(|source| self.generic_functions.template(&source.declaration))
                    .map(|template| template.name.span)
            }, span)?
        } else { vec![origin] };
        let preparation_origin=if reachable_origins.contains(&origin) {origin} else {
            *reachable_origins.first().ok_or_else(||Diagnostic::new("callable-layout","Generic invocation has no reachable environment.",span))?
        };
        Ok(Selection { origins: reachable_origins, primary: preparation_origin })
    }
    pub(super) fn join_prepared_deferred_contracts(&self, contracts: &mut [ArgumentContract], other: &[ArgumentContract], span: Span) -> Result<(), Diagnostic> {
        if contracts.len() != other.len() || contracts.iter().zip(other).any(|(left,right)| !self.same_deferred_argument_type(left.ty,right.ty) || left.omission != right.omission || left.rest != right.rest) {
            return Err(Diagnostic::new("callable-layout", "Reachable generic declarations require compatible prepared argument contracts.", span));
        }
        for (contract, other) in contracts.iter_mut().zip(other) { contract.mode = contract.mode.max(other.mode); }
        Ok(())
    }
    pub(super) fn deferred_alternative_needs_body(&self, primary: usize, origin: usize) -> bool {
        if primary == origin { return false; }
        let previous = &self.deferred_generic_values[&primary];
        let next = &self.deferred_generic_values[&origin];
        previous.declaration != next.declaration || !previous.identities.is_empty() || !next.identities.is_empty()
            || !self.closures[primary].captures.is_empty() || !self.closures[origin].captures.is_empty()
    }
}
