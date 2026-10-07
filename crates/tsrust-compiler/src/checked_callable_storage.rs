//! Register already checked environments without repeating source access effects.
use super::*;
impl Analyzer {
    // Conversion consumes only checked metadata; the source expression was
    // already evaluated and its ownership effects must not be repeated.
    pub(super) fn checked_stored_callable_value(&mut self,context:&mut Context,value:&mut Expr,family:usize,actual:Type)->Result<Type,Diagnostic> {
        let declaration=self.callables.storage.declaration(family).ok_or_else(||Diagnostic::new("callable-layout","Unknown callable storage signature.",value.span))?;
        let expected=Type::Callback(declaration);
        if matches!(actual, Type::StoredCallable(_)) {
            self.require(actual, Some(Type::StoredCallable(family)), value.span)?;
            return Ok(actual);
        }
        self.require(actual, Some(expected), value.span)?;
        let Type::Closure(mut origin) = actual else {
            return Err(Diagnostic::new(
                "callable-layout",
                "Stored callable construction requires a concrete owned environment.",
                value.span,
            ));
        };
        if self.callables.callbacks[declaration].kind.is_once()
            && self.closures[origin].invocation != Mode::Consume
        {
            origin = self.once_return(origin, context, value.span);
            value.wrap_once(origin);
        }
        if !context.template && !self.discovering_results && !self.effects_only {
            self.callables.storage.register_origin(
                family,
                origin,
                &self.callables.callbacks,
                &self.callables.shapes,
                &self.closures,
                value.span,
            )?;
        } else if self.speculative_access_run && !context.template && !self.discovering_results {
            self.callables.storage.note_speculative_origin(family, origin);
        }
        value.wrap_storage(family);
        Ok(Type::StoredCallable(family))
    }

}
