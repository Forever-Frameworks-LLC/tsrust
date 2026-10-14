//! Checked generic call preparation. Ordinary call checking owns argument effects.
use super::*;

impl Analyzer {
    pub(super) fn generic_callable_shapes(
        &self,
        context: &Context,
        args: &[Expr],
    ) -> crate::generic_inference::CallableShapes {
        args.iter()
            .filter_map(|argument| {
                let ty = self.expression_type_hint(context, argument)?;
                let payload = self.unions.payload(ty).unwrap_or(ty);
                self.checked_generic_callable_shape(payload)
                    .map(|shape| (payload, shape))
            })
            .collect()
    }

    pub(super) fn inline_generic_callable_shape(&self, argument: &Expr) -> Option<(Vec<Type>, Type)> {
        let ExprKind::Closure {
            params,
            result: ResultAnnotation::Named(result),
            ..
        } = &argument.kind
        else {
            return None;
        };
        let parameters = params
            .iter()
            .map(|parameter| {
                let ParameterAnnotation::Named(annotation) = &parameter.ty else {
                    return None;
                };
                resolve_value_type(annotation, false, &self.record_names, &self.unions).ok()
            })
            .collect::<Option<Vec<_>>>()?;
        let result = resolve_value_type(result, false, &self.record_names, &self.unions).ok()?;
        Some((parameters, result))
    }

    pub(super) fn checked_generic_callable_shape(&self, ty: Type) -> Option<(Vec<Type>, Type)> {
        match ty {
            Type::Closure(id) if self.deferred_generic_values.contains_key(&id) => None,
            Type::Closure(id) => {
                let closure = &self.closures[id];
                Some((
                    closure
                        .parameters
                        .iter()
                        .map(|id| closure.function.bindings[*id].ty)
                        .collect(),
                    closure.function.result,
                ))
            }
            Type::Callable(id) => Some((
                self.callables.results[id]
                    .params
                    .iter()
                    .map(|parameter| parameter.ty)
                    .collect(),
                self.callables.results[id].result,
            )),
            Type::StoredCallable(family) => {
                let id = self.callables.storage.declaration(family)?;
                let contract = &self.callables.callbacks[id];
                Some((
                    contract.params.iter().map(|param| param.ty).collect(),
                    contract.result,
                ))
            }
            Type::Callback(id) => Some((
                self.callables.callbacks[id]
                    .params
                    .iter()
                    .map(|parameter| parameter.ty)
                    .collect(),
                self.callables.callbacks[id].result,
            )),
            _ => None,
        }
    }

    pub(super) fn generic_argument_hint(
        &self,
        context: &Context,
        argument: &Expr,
        expected: Option<Type>,
    ) -> Option<Type> {
        if let Some(Type::Closure(origin))=argument.ty && self.deferred_generic_values.contains_key(&origin) {return argument.ty;}
        if let ExprKind::Unary{op:UnaryOp::RestPack,value,..}=&argument.kind {return self.generic_argument_hint(context,value,expected);}
        if let ExprKind::Call(callee, _) = &argument.kind
            && let ExprKind::Name(name) = &callee.kind
            && !context.names.contains_key(name)
            && (self.function_names.get(name).is_some_and(|&index| self.signatures[index].inferred_result && self.checked[index].is_none())
                || self.generic_functions.template(name).is_some_and(|function| function.result.is_none())) {
            // An unchecked omitted result is unknown, not a void constraint.
            return None;
        }
        if let ExprKind::Call(callee, _) = &argument.kind
            && let ExprKind::Name(name) = &callee.kind
            && let Some(&binding) = context.names.get(name)
            && let Type::Closure(origin) = context.bindings[binding].ty
            && self.deferred_generic_values.contains_key(&origin) {
            // An unsolved invocation has no result hint. Its empty environment
            // prototype must never contribute a void constraint to its caller.
            return None;
        }
        match (expected, &argument.kind) {
            (Some(Type::Number), _) if number_literal_context(argument) => Some(Type::Number),
            (Some(Type::Array(element)), ExprKind::Array(values))
                if values.iter().all(|value| {
                    self.generic_argument_hint(context, value, Some(element.ty()))
                        == Some(element.ty())
                }) =>
            {
                expected
            }
            (Some(Type::Record(_)), ExprKind::Record(_)) => expected,
            _ => self
                .expression_type_hint(context, argument)
                .map(Type::owning),
        }
    }

    pub(super) fn generic_call_name(callee: &mut Expr) -> Option<(&mut String, Span)> {
        match &mut callee.kind {
            ExprKind::Name(name) => Some((name, callee.span)),
            ExprKind::Unary {
                op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected,
                value,
                ..
            } => Self::generic_call_name(value),
            _ => None,
        }
    }

    pub(super) fn prepare_generic_call(&mut self, context: &Context, expr: &mut Expr, expected: Option<Type>) -> Result<(), Diagnostic> {
        continuations::driver::check_generic_preparation(self, context, expr, expected)
    }

    #[inline(never)]
    pub(super) fn publish_generic_call(&mut self, context: &Context, concrete: String) -> Result<String, Diagnostic> {
        let concrete = self.publish_generic_call_work(concrete)?;
        self.prepare_named_result_hint(context, self.function_names[&concrete])?;
        Ok(concrete)
    }

    pub(super) fn publish_generic_call_work(&mut self, concrete: String) -> Result<String, Diagnostic> {
        if self.function_names.contains_key(&concrete) { return Ok(concrete); }
        let mut output = self.generic_functions.drain(true)?;
        for function in &mut output {
            let mut materialize = |annotation: &mut crate::type_syntax::ValueAnnotation| {
                self.generic_records.materialize(
                    annotation,
                    &mut self.program.records,
                    &mut self.record_names,
                    &self.unions, &mut self.callables,
                )
            };
            crate::source_walk::function_annotations(function, &mut materialize)?;
            crate::source_walk::body(&mut function.body, &mut materialize, &mut |_| Ok(()))?;
            if let Some(application) = &mut function.application {
                for argument in &mut application.arguments {
                    materialize(argument)?;
                }
            }
        }
        for mut argument in self.generic_functions.take_argument_checks() {
            self.generic_records.materialize(
                &mut argument,
                &mut self.program.records,
                &mut self.record_names,
                &self.unions, &mut self.callables,
            )?;
            self.callable_resolver().resolve_value_annotation(&argument, false)?;
        }
        append_checked_record_fields(
            &mut self.program.records,
            &self.record_names,
            &self.unions,
            &mut self.fields,
            &mut self.callables,
        )?;
        let first_new_function = self.program.functions.len();
        for mut function in output {
            let id = self.program.functions.len();
            Self::escape_origins(&function.body, &HashMap::new(), &mut self.escaping);
            if function.params.iter().any(|(_, _, callback)| {
                callback.as_ref().is_some_and(|callback| {
                    callback.kind.is_owned()
                        || callback.params.iter().any(|parameter| {
                            matches!(parameter.ty, ParameterAnnotation::Callback(_))
                        })
                        || matches!(*callback.result, ResultAnnotation::Callable { .. })
                })
            }) {
                self.templates.insert(id, function.clone());
            }
            let mut params = function
                .params
                .iter()
                .map(|(_, annotation, callback)| {
                    if let Some(callback) = callback {
                        self.resolve_callback(callback)
                    } else {
                        self.resolve_value_annotation(annotation, false)
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            let inferred_result = function.result.is_none() && function.name.text != "main" && return_type_inference::has_value_return(&function.body);
            let result = function
                .result
                .take()
                .as_ref()
                .map(|annotation| self.resolve_result(annotation))
                .transpose()?
                .unwrap_or(Type::Void);
            if let Some(modes) = &mut self.assumed_modes
                && modes.len() <= id
            {
                modes.push(
                    params
                        .iter()
                        .map(|ty| if ty.is_copy() { Mode::Copy } else { Mode::Read })
                        .collect(),
                );
            }
            self.function_names.insert(function.name.text.clone(), id);
            if let Some(application) = &function.application {
                self.named_captures.instance(id, &application.declaration.text);
            }
            let body_params = params.clone();
            for (ty, initializer) in params.iter_mut().zip(&function.parameter_initializers) {
                if initializer.omission().is_omittable() { *ty = self.unions.intern([*ty,Type::Undefined], function.name.span)?; }
            }
            self.signatures.push(Signature { params, body_params, result, inferred_result });
            self.checked.push(None);
            self.visiting.push(false);
            self.program.functions.push(function);
        }
        self.generic_functions.seed_functions(
            &self.program.functions[first_new_function..],
            &self.record_names,
            &self.unions,
        )?;
        Ok(concrete)
    }
}
