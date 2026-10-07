//! Source staging gives fresh generic environments one ordinary lexical owner.
use super::*;

impl Analyzer {
    #[inline(never)]
    pub(super) fn stage_generic_traversal(&mut self, context: &mut Context, expression: &mut Expr, supplied: &[Type], loans: &Loans) -> Result<Option<Type>, Diagnostic> {
        let Some(origin) = self.fresh_generic_origin(context, expression)? else { return Ok(None); };
        let declaration = self.deferred_generic_values[&origin].declaration.clone();
        let template = self.generic_functions.template(&declaration).expect("fresh traversal template").clone();
        let types = crate::generic_inference::Types { callable_shapes: None, inline_shapes: None, records: &self.program.records, names: &self.record_names, unions: &self.unions };
        let hints = supplied.iter().take(template.params.len()).copied().map(Some).collect::<Vec<_>>();
        let contract = self.generic_functions.value_signature(&declaration, &hints, None, &types, expression.span)?;
        let Some(mut result) = contract.result else { return Ok(None); };
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
        self.stage_generic_expression(context, expression, expected, Mode::Consume, loans).map(Some)
    }

    #[inline(never)]
    pub(super) fn fresh_generic_origin(&mut self, context: &Context, expression: &Expr) -> Result<Option<usize>, Diagnostic> {
        let ExprKind::Call(callee, _) = &expression.kind else { return Ok(None); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        if context.names.contains_key(name) { return Ok(None); }
        let Some(&function) = self.function_names.get(name) else { return Ok(None); };
        self.prepare_named_result_hint(context, function)?;
        let Type::Closure(origin) = self.signatures[function].result else { return Ok(None); };
        Ok(self.deferred_generic_values.contains_key(&origin).then_some(origin))
    }

    #[inline(never)]
    pub(super) fn stage_generic_expression(&mut self, context: &mut Context, expression: &mut Expr, expected: Type, mode: Mode, loans: &Loans) -> Result<Type, Diagnostic> {
        let span = expression.span;
        let payload = self.unions.payload(expected).unwrap_or(expected);
        let (parameters, result, kind) = match payload {
            Type::Callable(id) => (self.callables.results[id].params.clone(), self.callables.results[id].result, CallbackKind::Owned),
            Type::Callback(id) => (self.callables.callbacks[id].params.clone(), self.callables.callbacks[id].result, self.callables.callbacks[id].kind),
            _ => return Err(Diagnostic::new("unsupported", "Fresh generic adaptation needs a concrete callable context.", span)),
        };
        let types = crate::generic_inference::Types { callable_shapes: None, inline_shapes: None, records: &self.program.records, names: &self.record_names, unions: &self.unions };
        let params = parameters.iter().enumerate().map(|(index, parameter)| Ok(CallableParameter {
            rest: parameter.rest,
            name: Token { text: format!("argument{index}"), span, kind: TokenKind::Name },
            ty: ParameterAnnotation::Named(Box::new(types.annotation(parameter.ty, span)?)),
            access: (parameter.mode != Mode::Copy).then(|| Token { text: parameter.mode.as_str().into(), span, kind: TokenKind::Name }),
            initializer: if parameter.omission.is_omittable() { ParameterInitializer::Optional(span) } else { ParameterInitializer::Required },
        })).collect::<Result<Vec<_>, Diagnostic>>()?;
        let result_annotation = ResultAnnotation::Callable { kind: None, params: params.clone(), result: Box::new(ResultAnnotation::Named(Box::new(types.annotation(result, span)?))), span };
        let annotation = CallbackAnnotation { kind, params, result: Box::new(ResultAnnotation::Named(Box::new(types.annotation(result, span)?))), span }.value_annotation();
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
        self.expression(context, expression, mode, loans, Some(expected))
    }
}
