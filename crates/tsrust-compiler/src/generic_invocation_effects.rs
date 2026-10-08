//! Join effects of point-specific checked alternatives, independently of storage.
use super::*;

impl ErrorEffects {
    #[inline(never)]
    pub(super) fn refresh_invocation_errors(&mut self, registry: &crate::polymorphic_invocation_plans::Registry) -> bool {
        let mut changed = false;
        for (id, pairs) in registry.entries() {
            let mut found = FailureSet::default();
            for pair in pairs {
                found |= self.method_errors.get(&(pair.origin, pair.method)).cloned().unwrap_or_else(FailureSet::unknown);
            }
            changed |= self.invocation_errors[id].join(found);
        }
        changed
    }
}
