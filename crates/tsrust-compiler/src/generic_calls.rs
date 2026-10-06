//! Checked generic call preparation. Ordinary call checking owns argument effects.
use super::*;

impl Analyzer {
    fn generic_callable_shapes(
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

    fn inline_generic_callable_shape(&self, argument: &Expr) -> Option<(Vec<Type>, Type)> {
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

    fn generic_argument_hint(
        &self,
        context: &Context,
        argument: &Expr,
        expected: Option<Type>,
    ) -> Option<Type> {
        if let ExprKind::Unary{op:UnaryOp::RestPack,value,..}=&argument.kind {return self.generic_argument_hint(context,value,expected);}
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

    fn generic_call_name(callee: &mut Expr) -> Option<(&mut String, Span)> {
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

    pub(super) fn prepare_generic_call(
        &mut self,
        context: &Context,
        expr: &mut Expr,
        expected: Option<Type>,
    ) -> Result<(), Diagnostic> {
        match &mut expr.kind {
            ExprKind::OptionalChain(value)
            | ExprKind::Unary {
                op: UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected,
                value,
                ..
            } => return self.prepare_generic_call(context, value, expected),
            _ => {}
        }
        if matches!(&expr.kind,ExprKind::Call(_,args) if args.iter().any(|arg|matches!(arg.kind,ExprKind::Unary{op:UnaryOp::Spread,..}))) {
            return self.prepare_generic_spread(context,expr,expected);
        }
        let ExprKind::Call(callee, _) = &mut expr.kind else {
            return Ok(());
        };
        let Some((name, _)) = Self::generic_call_name(callee) else {
            return Ok(());
        };
        if context.names.contains_key(name)
            || context.pending_names.contains(name)
            || self.generic_functions.template(name).is_none()
        {
            return Ok(());
        }
        self.prepare_named_generic_call(context, expr, expected)
    }

    // Specialization state must not occupy the stack for nongeneric calls.
    #[inline(never)]
    fn prepare_named_generic_call(
        &mut self,
        context: &Context,
        expr: &mut Expr,
        expected: Option<Type>,
    ) -> Result<(), Diagnostic> {
        let ExprKind::Call(callee, args) = &mut expr.kind else {
            unreachable!("generic call preparation");
        };
        let (name, callee_span) = Self::generic_call_name(callee)
            .expect("generic call name checked by dispatch");
        let template=self.generic_functions.template(name).expect("generic template");
        if template.rest_parameter {
            let fixed=template.params.len()-1;
            let minimum=crate::parameter_arity::minimum_arguments(&template.parameter_initializers[..fixed],|initializer|initializer.omission().is_omittable());
            if args.len()<minimum {return Err(Diagnostic::new("arity",format!("Expected at least {minimum} arguments, found {}.",args.len()),expr.span));}
            Self::pack_rest_arguments(args,fixed,expr.span);
        }
        let mut inline_shapes = HashMap::new();
        for ((_, pattern, _), argument) in self
            .generic_functions
            .template(name)
            .expect("generic template")
            .params
            .iter()
            .zip(args.iter())
        {
            if let Some(shape) = self.inline_generic_callable_shape(argument) {
                inline_shapes.insert((pattern.span.start, pattern.span.end), shape);
            }
        }
        let mut prepared = vec![false; args.len()];
        loop {
            let callable_shapes = self.generic_callable_shapes(context, args);
            let types = crate::generic_inference::Types {
                callable_shapes: Some(&callable_shapes),
                inline_shapes: Some(&inline_shapes),
                records: &self.program.records,
                names: &self.record_names,
                unions: &self.unions,
            };
            let seed = self
                .generic_functions
                .parameter_contexts(name, expected, &types, None)?;
            let template=self.generic_functions.template(name).expect("generic template");
            let fixed=template.params.len()-usize::from(template.rest_parameter);
            let minimum=crate::parameter_arity::minimum_arguments(&template.parameter_initializers[..fixed],|initializer|initializer.omission().is_omittable());
            if args.len()<minimum||args.len()>seed.len(){return Err(Diagnostic::new("arity",format!("Expected {minimum}..={} arguments, found {}.",seed.len(),args.len()),expr.span));}
            let hints = args
                .iter()
                .zip(&seed)
                .map(|(argument, expected)| {
                    self.generic_argument_hint(context, argument, *expected)
                })
                .collect::<Vec<_>>();
            let contexts =
                self.generic_functions
                    .parameter_contexts(name, expected, &types, Some(&hints))?;
            let mut progress = false;
            let mut deferred = None;
            for (id, (argument, expected)) in args.iter_mut().zip(&contexts).enumerate() {
                if prepared[id] {
                    continue;
                }
                match self.prepare_generic_call(context, argument, *expected) {
                    Ok(()) => {
                        prepared[id] = true;
                        progress = true;
                    }
                    Err(error) if error.code == "generic-inference" => {
                        if deferred.is_none() {
                            deferred = Some(error);
                        }
                    }
                    Err(error) => return Err(error),
                }
            }
            if prepared.iter().all(|prepared| *prepared) {
                break;
            }
            if !progress {
                return Err(deferred.expect("unprepared generic argument"));
            }
        }
        let callable_shapes = self.generic_callable_shapes(context, args);
        let types = crate::generic_inference::Types {
            callable_shapes: Some(&callable_shapes),
            inline_shapes: Some(&inline_shapes),
            records: &self.program.records,
            names: &self.record_names,
            unions: &self.unions,
        };
        let seed = self
            .generic_functions
            .parameter_contexts(name, expected, &types, None)?;
        let hints = args
            .iter()
            .zip(&seed)
            .map(|(argument, expected)| self.generic_argument_hint(context, argument, *expected))
            .collect::<Vec<_>>();
        let contexts =
            self.generic_functions
                .parameter_contexts(name, expected, &types, Some(&hints))?;
        let hints = args
            .iter()
            .zip(&contexts)
            .map(|(argument, expected)| self.generic_argument_hint(context, argument, *expected))
            .collect::<Vec<_>>();
        let arguments =
            self.generic_functions
                .infer(name, &hints, expected, &types, callee_span)?;
        let concrete = self.generic_functions.request_checked(
            name,
            arguments,
            &types,
            expr.span,
            self.function_stack.len(),
        )?;
        if self.function_names.contains_key(&concrete) {
            *name = concrete;
            return Ok(());
        }
        let mut output = self.generic_functions.drain(true)?;
        for function in &mut output {
            let mut materialize = |annotation: &mut crate::type_syntax::ValueAnnotation| {
                self.generic_records.materialize(
                    annotation,
                    &mut self.program.records,
                    &mut self.record_names,
                    &self.unions,
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
                &self.unions,
            )?;
            resolve_value_type(&argument, false, &self.record_names, &self.unions)?;
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
            self.signatures.push(Signature { params, body_params, result });
            self.checked.push(None);
            self.visiting.push(false);
            self.program.functions.push(function);
        }
        self.generic_functions.seed_functions(
            &self.program.functions[first_new_function..],
            &self.record_names,
            &self.unions,
        )?;
        *name = concrete;
        Ok(())
    }
}
