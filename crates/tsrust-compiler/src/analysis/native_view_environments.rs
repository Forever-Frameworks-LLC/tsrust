//! Native view loans carried by concrete callable capture fields.
use super::*;

impl Analyzer {
    pub(super) fn remember_native_environment(&self, context: &mut Context, origin: usize) {
        let captures = self.closures[origin]
            .captures
            .iter()
            .filter(|capture| matches!(capture.storage,Mode::Read|Mode::Mutate) || context.bindings[capture.outer].native_view_loans.is_some())
            .map(|capture| (capture.inner, capture.outer, capture.storage))
            .collect::<Vec<_>>();
        if !captures.is_empty() {
            context
                .native_view_environments
                .get_or_insert_with(Default::default)
                .insert(origin, captures);
            context.native_view_calls = true;
        }
    }
}
