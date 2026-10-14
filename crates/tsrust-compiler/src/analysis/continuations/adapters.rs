//! Generated generic adapters retain lexical cleanup until child checking ends.
use super::super::*;
pub(in crate::analysis) enum Cleanup {
    None,
    Named { hidden: String, previous: Option<usize>, escape: Option<usize> },
    Source(crate::capture_loan_identity::Boundary),
    Stored(super::StoredValueFrame),
}
pub(in crate::analysis) struct Request {
    pub(in crate::analysis) value: Box<Expr>,
    pub(in crate::analysis) mode: Mode,
    pub(in crate::analysis) expected: Option<Type>,
    pub(in crate::analysis) cleanup: Cleanup,
    pub(in crate::analysis) restore_on_error: bool,
}
pub(in crate::analysis) enum Other { Ready(Type), Adapter(Request) }
impl Analyzer {
    pub(in crate::analysis) fn finish_adapter(&mut self, context: &mut Context, value: &mut Expr, cleanup: Cleanup, checked: Result<Type, Diagnostic>) -> Result<Type, Diagnostic> {
        match cleanup {
            Cleanup::None => checked,
            Cleanup::Named { hidden, previous, escape } => {
                if let Some(id) = escape { self.escaping.remove(&id); }
                match previous { Some(previous) => { context.names.insert(hidden, previous); }, None => { context.names.remove(&hidden); } }
                checked
            }
            Cleanup::Source(boundary) => { boundary.unwind(&mut context.capture_loans); checked }
            Cleanup::Stored(frame) => checked.and_then(|actual| self.finish_stored_value(context, value, frame, actual)),
        }
    }
    pub(in crate::analysis) fn prepare_named_adapter(&mut self, context: &mut Context, expression: &mut Expr, mode: Mode, expected: Option<Type>) -> Result<Option<Request>, Diagnostic> {
        let ExprKind::Name(name) = &expression.kind else { return Ok(None); };
        if crate::native_bindings::by_symbol(name).is_some() || self.discovering_results && context.pending_names.contains(name) { return Ok(None); }
        if !context.names.contains_key(name) && self.generic_functions.template(name).is_some() {
            if let Some(expected) = expected && self.checked_generic_callable_shape(self.unions.payload(expected).unwrap_or(expected)).is_some() {
                return self.prepare_contextual_generic_value(expression, mode, expected).map(Some);
            }
        }
        if let Some(&binding) = context.names.get(name) && let Type::Closure(origin) = context.bindings[binding].ty
            && self.deferred_generic_values.contains_key(&origin) && let Some(expected) = expected
            && self.checked_generic_callable_shape(expected).is_some() {
            return self.prepare_generic_value(context, expression, binding, origin, expected, mode).map(Some);
        }
        Ok(None)
    }
}
