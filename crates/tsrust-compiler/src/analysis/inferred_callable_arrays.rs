//! Infer owned callable storage from a checked first array element.
use super::*;
impl Analyzer {
    #[inline(never)]
    pub(super) fn inferred_callable_array_element(
        &mut self, context: &Context, value: &mut Expr, actual: Type,
    ) -> Result<Type, Diagnostic> {
        let Type::Closure(origin) = actual else { return Ok(actual); };
        let closure = &self.closures[origin];
        let params: Vec<_> = closure.parameters.iter()
            .map(|&slot| ArgumentContract::from(&closure.function.bindings[slot])).collect();
        let parameter_names = closure.parameters.iter()
            .map(|&slot| closure.function.bindings[slot].name.clone()).collect();
        let result = closure.function.result;
        let kind = if self.is_task_type(actual) { CallbackKind::Task } else if closure.invocation == Mode::Consume { CallbackKind::OwnedOnce } else { CallbackKind::Owned };
        let structural = self.callable_resolver().intern_callable_shape(&params, result, Some(kind));
        let contract = CallbackContract { structural, parameter_names, params, result, kind };
        let declaration = self.callables.callbacks.len();
        let ty = self.callables.storage.intern(&self.callables.shapes, &contract, declaration).expect("owned inferred contract");
        let Type::StoredCallable(family) = ty else { unreachable!("owned storage type"); };
        if self.callables.storage.declaration(family) == Some(declaration) {
            self.callables.callbacks.push(contract);
        }
        if !context.template && !self.discovering_results && !self.effects_only {
            self.callables.storage.register_origin(family, origin, &self.callables.callbacks, &self.callables.shapes, &self.closures, value.span)?;
        } else if self.speculative_access_run && !context.template && !self.discovering_results {
            self.callables.storage.note_speculative_origin(family, origin);
        }
        value.wrap_storage(family);
        Ok(ty)
    }
}
