//! Source staging gives fresh generic environments one ordinary lexical owner.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn stage_generic_traversal(&mut self, context: &mut Context, expression: &mut Expr, supplied: &[Type], loans: &Loans) -> Result<Option<Type>, Diagnostic> {
        let selected = self.lower_selected_generic_factory(context, expression)?;
        let source = if selected {
            let ExprKind::Conditional { consequent, .. } = &expression.kind else { unreachable!() };
            let mut source = &**consequent;
            while let ExprKind::Conditional { consequent, .. } = &source.kind {
                source = consequent;
            }
            source
        } else { &*expression };
        let Some(origin) = self.fresh_generic_origin(context, source)? else { return Ok(None); };
        let declaration = self.deferred_generic_values[&origin].declaration.clone();
        let template = self.generic_functions.template(&declaration).expect("fresh traversal template").clone();
        let types = crate::generic_inference::Types { callable_shapes: None, inline_shapes: None, records: &self.program.records, names: &self.record_names, unions: &self.unions };
        let hints = supplied.iter().take(template.params.len()).copied().map(Some).collect::<Vec<_>>();
        let contract = self.generic_functions.value_signature(&declaration, &hints, None, &types, expression.span)?;
        let mut result = if let Some(result) = contract.result { result } else {
            let result = self.deferred_generic_result_hint(context, origin, &hints, expression.span)?;
            ResultAnnotation::Named(Box::new(self.checked_value_annotation(result, expression.span)?))
        };
        let mut parameters = crate::local_function_values::value(&template, usize::MAX - expression.span.start, expression.span);
        let ExprKind::Closure { params, .. } = &mut parameters.kind else { unreachable!() };
        for (parameter, solved) in params.iter_mut().zip(contract.parameters) {
            parameter.ty = ParameterAnnotation::Named(Box::new(solved.annotation));
        }
        let mut materialize = |annotation: &mut crate::type_syntax::ValueAnnotation| self.generic_records.materialize(annotation, &mut self.program.records, &mut self.record_names, &self.unions);
        crate::source_walk::result_annotation(&mut result, &mut materialize)?;
        for parameter in params.iter_mut() {
            if let ParameterAnnotation::Named(annotation) = &mut parameter.ty { materialize(annotation)?; }
        }
        append_checked_record_fields(&mut self.program.records, &self.record_names, &self.unions, &mut self.fields, &mut self.callables)?;
        let expected = self.resolve_callback(&CallbackAnnotation { kind: CallbackKind::Owned, params: params.clone(), result: Box::new(result), span: expression.span })?;
        if selected {
            self.stage_selected_generic_expression(context, expression, expected, loans).map(Some)
        } else {
            self.stage_generic_expression(context, expression, expected, Mode::Consume, loans).map(Some)
        }
    }

    #[inline(never)]
    pub(super) fn stage_selected_generic_expression(&mut self, context: &mut Context, expression: &mut Expr, expected: Type, loans: &Loans) -> Result<Type, Diagnostic> {
        let span = expression.span;
        let callback = match expected {
            Type::Callback(id) if self.callables.callbacks[id].kind.is_owned() => id,
            Type::StoredCallable(family) => self.callables.storage.declaration(family).expect("selected storage declaration"),
            Type::Callable(id) => {
                let contract = &self.callables.results[id];
                let kind = self.callables.shapes.get(contract.structural).kind.unwrap_or(CallbackKind::Owned);
                let params = contract.params.iter().enumerate().map(|(index, parameter)| Ok(CallableParameter {
                    rest: parameter.rest,
                    name: Token { text: format!("argument{index}"), span, kind: TokenKind::Name },
                    ty: ParameterAnnotation::Named(Box::new(self.checked_value_annotation(parameter.ty, span)?)),
                    access: (parameter.mode != Mode::Copy && self.checked_generic_callable_shape(self.unions.payload(parameter.ty).unwrap_or(parameter.ty)).is_none()).then(|| Token { text: parameter.mode.as_str().into(), span, kind: TokenKind::Name }),
                    initializer: if parameter.omission.is_omittable() { ParameterInitializer::Optional(span) } else { ParameterInitializer::Required },
                })).collect::<Result<Vec<_>, Diagnostic>>()?;
                let result = ResultAnnotation::Named(Box::new(self.checked_value_annotation(contract.result, span)?));
                let Type::Callback(id) = self.resolve_callback(&CallbackAnnotation { kind, params, result: Box::new(result), span })? else { unreachable!() };
                id
            }
            _ => return Err(Diagnostic::new("closure-escape", "Selected fresh generic environments require an owning callable context.", span)),
        };
        let Type::StoredCallable(family) = self.callables.storage.intern(&self.callables.shapes, &self.callables.callbacks[callback], callback).expect("selected owning contract") else { unreachable!() };
        self.stored_callable_value(context, expression, family, loans)
    }

    // Distributing a call over its selected callee preserves evaluation order:
    // predicate first, then only the selected callee and its arguments.
    // Concrete adaptation happens in each branch before callable origins join.
    #[inline(never)]
    pub(super) fn lower_selected_generic_factory(&mut self, context: &Context, expression: &mut Expr) -> Result<bool, Diagnostic> {
        let ExprKind::Call(callee, _) = &expression.kind else { return Ok(false); };
        if !matches!(callee.kind, ExprKind::Conditional { .. }) { return Ok(false); }
        let mut distributed = expression.clone();
        let mut pending = vec![&mut distributed];
        while let Some(call) = pending.pop() {
            if let ExprKind::Call(callee, arguments) = &call.kind
                && let ExprKind::Conditional { predicate, consequent, alternate } = &callee.kind
            {
                call.kind = ExprKind::Conditional {
                    predicate: predicate.clone(),
                    consequent: Box::new(Expr::new(ExprKind::Call(consequent.clone(), arguments.clone()), call.span)),
                    alternate: Box::new(Expr::new(ExprKind::Call(alternate.clone(), arguments.clone()), call.span)),
                };
            }
            if !matches!(call.kind, ExprKind::Conditional { .. }) {
                if self.fresh_generic_origin(context, call)?.is_none() { return Ok(false); }
                continue;
            }
            let ExprKind::Conditional { consequent, alternate, .. } = &mut call.kind else { unreachable!() };
            pending.push(alternate);
            pending.push(consequent);
        }
        *expression = distributed;
        Ok(true)
    }

    #[inline(never)]
    pub(super) fn fresh_generic_origin(&mut self, context: &Context, expression: &Expr) -> Result<Option<usize>, Diagnostic> {
        let ExprKind::Call(callee, _) = &expression.kind else { return Ok(None); };
        if let ExprKind::Name(name) = &callee.kind
            && !context.names.contains_key(name)
            && self.generic_functions.template(name).is_some()
        {
            // Specialization preparation reads type hints only. The staged
            // source retains the real arguments and evaluates them once.
            let mut prepared = expression.clone();
            self.prepare_generic_call(context, &mut prepared, None)?;
            let Some(Type::Closure(origin)) = self.expression_type_hint(context, &prepared) else { return Ok(None); };
            return Ok(self.deferred_generic_values.contains_key(&origin).then_some(origin));
        }
        if let ExprKind::Name(name) = &callee.kind
            && !context.names.contains_key(name)
            && let Some(&function) = self.function_names.get(name)
        {
            self.prepare_named_result_hint(context, function)?;
        }
        let Some(Type::Closure(origin)) = self.expression_type_hint(context, expression) else { return Ok(None); };
        Ok(self.deferred_generic_values.contains_key(&origin).then_some(origin))
    }

    #[inline(never)]
    pub(super) fn stage_generic_expression(&mut self, context: &mut Context, expression: &mut Expr, expected: Type, mode: Mode, loans: &Loans) -> Result<Type, Diagnostic> {
        let span = expression.span;
        let payload = self.unions.payload(expected).unwrap_or(expected);
        let (parameters, result, kind) = match payload {
            Type::Callable(id) => (self.callables.results[id].params.clone(), self.callables.results[id].result, CallbackKind::Owned),
            Type::Callback(id) => (self.callables.callbacks[id].params.clone(), self.callables.callbacks[id].result, self.callables.callbacks[id].kind),
            Type::StoredCallable(family) => {
                let id = self.callables.storage.declaration(family).expect("fresh callable storage declaration");
                let contract = &self.callables.callbacks[id];
                (contract.params.clone(), contract.result, contract.kind)
            }
            _ => return Err(Diagnostic::new("unsupported", "Fresh generic adaptation needs a concrete callable context.", span)),
        };
        let params = parameters.iter().enumerate().map(|(index, parameter)| Ok(CallableParameter {
            rest: parameter.rest,
            name: Token { text: format!("argument{index}"), span, kind: TokenKind::Name },
            ty: ParameterAnnotation::Named(Box::new(self.checked_value_annotation(parameter.ty, span)?)),
            access: (parameter.mode != Mode::Copy && self.checked_generic_callable_shape(self.unions.payload(parameter.ty).unwrap_or(parameter.ty)).is_none()).then(|| Token { text: parameter.mode.as_str().into(), span, kind: TokenKind::Name }),
            initializer: if parameter.omission.is_omittable() { ParameterInitializer::Optional(span) } else { ParameterInitializer::Required },
        })).collect::<Result<Vec<_>, Diagnostic>>()?;
        let result_annotation = ResultAnnotation::Callable { kind: None, params: params.clone(), result: Box::new(ResultAnnotation::Named(Box::new(self.checked_value_annotation(result, span)?))), span };
        let annotation = CallbackAnnotation { kind, params, result: Box::new(ResultAnnotation::Named(Box::new(self.checked_value_annotation(result, span)?))), span }.value_annotation();
        let name = Token { text: format!("#generic-source-{}", span.start), span, kind: TokenKind::Name };
        let adapted = Token { text: format!("#generic-result-{}", span.start), span, kind: TokenKind::Name };
        let statement = |kind| Stmt { labels: Vec::new(), control_target: None, kind, span };
        let source = expression.clone();
        let body = vec![
            statement(StmtKind::Let { rebindable: false, name: name.clone(), annotation: None, value: source, binding: None }),
            statement(StmtKind::Let { rebindable: false, name: adapted.clone(), annotation: Some(annotation), value: Expr::new(ExprKind::Name(name.text), span), binding: None }),
            statement(StmtKind::Return(Some(Expr::new(ExprKind::Name(adapted.text), span)))),
        ];
        let closure = Expr::new(ExprKind::Closure { id: usize::MAX / 2 + span.start, params: Vec::new(), result: result_annotation, body }, span);
        *expression = Expr::new(ExprKind::Call(Box::new(closure), Vec::new()), span);
        let first_capture = context.capture_loans.len();
        let checked = self.expression(context, expression, mode, loans, Some(expected));
        // The staging closure is invoked immediately. Its argument-evaluation
        // borrows end here; the returned owned adapter retains its own environment.
        context.capture_loans.truncate(first_capture);
        checked
    }
}
