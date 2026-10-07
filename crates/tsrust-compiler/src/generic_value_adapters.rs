//! Concrete callable views retain an existing deferred generic environment.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn adapt_generic_value(
        &mut self, context: &mut Context, expression: &mut Expr,
        binding: usize, origin: usize, expected: Type, mode: Mode, loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let (parameters, result) = self.checked_generic_callable_shape(expected).expect("concrete adapter contract");
        let hints = parameters.into_iter().map(Some).collect::<Vec<_>>();
        self.check_generic_adapter(context, expression, binding, origin, hints, Some(result), Some(expected), mode, loans)
    }

    #[inline(never)]
    pub(super) fn adapt_generic_traversal(
        &mut self, context: &mut Context, expression: &mut Expr, supplied: &[Type], loans: &Loans,
    ) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Name(name) = &expression.kind else {
            return self.stage_generic_traversal(context, expression, supplied, loans);
        };
        let Some(&binding) = context.names.get(name) else { return Ok(None); };
        let Type::Closure(origin) = context.bindings[binding].ty else { return Ok(None); };
        let Some(source) = self.deferred_generic_values.get(&origin) else { return Ok(None); };
        let count = self.generic_functions.template(&source.declaration).expect("traversal template").params.len();
        let hints = supplied.iter().take(count).copied().map(Some).collect();
        self.check_generic_adapter(context, expression, binding, origin, hints, None, None, Mode::Mutate, loans).map(Some)
    }

    #[inline(never)]
    fn check_generic_adapter(
        &mut self, context: &mut Context, expression: &mut Expr, binding: usize, origin: usize,
        hints: Vec<Option<Type>>, result: Option<Type>, expected: Option<Type>, mode: Mode, loans: &Loans,
    ) -> Result<Type, Diagnostic> {
        let span = expression.span;
        let declaration = self.deferred_generic_values[&origin].declaration.clone();
        let template = self.generic_functions.template(&declaration).expect("generic adapter template").clone();
        let types = crate::generic_inference::Types {
            callable_shapes: None, inline_shapes: None, records: &self.program.records,
            names: &self.record_names, unions: &self.unions,
        };
        let contract = self.generic_functions.value_signature(&declaration, &hints, result, &types, span)?;
        let id = usize::MAX - span.start;
        let mut wrapper = crate::local_function_values::value(&template, id, span);
        let ExprKind::Closure { params, result, body, .. } = &mut wrapper.kind else { unreachable!() };
        if let Some(solved) = contract.result { *result = solved; }
        for (parameter, solved) in params.iter_mut().zip(contract.parameters) {
            if matches!(parameter.ty, ParameterAnnotation::Inferred(_)) {
                parameter.ty = ParameterAnnotation::Named(Box::new(solved.annotation));
            }
        }
        // Solving substitutes type arguments; nominal applications still need
        // concrete record identities before the wrapper signature is checked.
        let mut materialize = |annotation: &mut crate::type_syntax::ValueAnnotation| {
            self.generic_records.materialize(annotation, &mut self.program.records,
                &mut self.record_names, &self.unions)
        };
        crate::source_walk::result_annotation(result, &mut materialize)?;
        for parameter in params.iter_mut() {
            if let ParameterAnnotation::Named(annotation) = &mut parameter.ty {
                materialize(annotation)?;
            }
        }
        append_checked_record_fields(&mut self.program.records, &self.record_names,
            &self.unions, &mut self.fields, &mut self.callables)?;
        // The source parameter may have the same name as the outer alias. A
        // private lexical name makes the call target the existing environment.
        let hidden = format!("#generic-adapter-{id}");
        let previous = context.names.insert(hidden.clone(), binding);
        let StmtKind::Return(Some(call)) = &mut body[0].kind else { unreachable!() };
        let ExprKind::Call(callee, arguments) = &mut call.kind else { unreachable!() };
        callee.kind = ExprKind::Name(hidden.clone());
        // The wrapper already owns the checked rest pack. Forward that pack
        // directly rather than planning a spread against an unsolved prototype.
        if template.rest_parameter
            && let Some(argument) = arguments.last_mut()
            && let ExprKind::Unary { op, .. } = &mut argument.kind
            && *op == UnaryOp::Spread
        {
            *op = UnaryOp::RestPack;
        }
        let owned = match expected {
            Some(Type::Callback(id)) => self.callables.callbacks[id].kind.is_owned(),
            Some(Type::StoredCallable(family)) => self.callables.storage.declaration(family)
                .is_some_and(|id| self.callables.callbacks[id].kind.is_owned()),
            Some(Type::Callable(_)) => true,
            _ => false,
        };
        let inserted_escape = owned && self.escaping.insert(id);
        let checked = self.expression(context, &mut wrapper, mode, loans, expected);
        if inserted_escape { self.escaping.remove(&id); }
        match previous { Some(previous) => { context.names.insert(hidden, previous); }, None => { context.names.remove(&hidden); } }
        let ty = checked?;
        *expression = wrapper;
        Ok(ty)
    }
}
