//! Native generic slots preserve nested scoped record environments.
use super::*;

impl Emitter<'_> {
    pub(super) fn scoped_record_native_parameter(&self, ty: Type) -> bool {
        if matches!(
            self.callback_payload(ty).unwrap_or(ty),
            Type::Callback(_) | Type::Closure(_) | Type::BorrowedCallable(_)
        ) {
            return true;
        }
        crate::aggregate_callable_facts::inspect(
            ty,
            &self.program.records,
            |id| self.program.unions[id.index()].members.clone(),
            |ty| match ty {
                Type::BorrowedCallable(_) => true,
                Type::Closure(id) => self.program.closures[id].borrowed,
                Type::Callback(id) => !self.program.callback_contracts[id].kind.is_owned(),
                _ => false,
            },
        )
        .scoped
    }
}
