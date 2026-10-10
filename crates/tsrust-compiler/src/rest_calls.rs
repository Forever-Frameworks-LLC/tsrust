//! Checked rest-to-fixed adapters preserve concrete environments and native ABI.
use super::*;

impl Analyzer {
    pub(super) fn rest_callable_adapter(
        &mut self,
        context: &Context,
        expression: &mut Expr,
        actual: Type,
        expected: Option<Type>,
        storage: Mode,
    ) -> Result<Type, Diagnostic> {
        let Some(expected) = expected else {
            return Ok(actual);
        };
        let expected_payload = self.unions.payload(expected).unwrap_or(expected);
        let expected_parameters = match expected_payload {
            Type::Callback(id) => self.callables.callbacks[id].params.clone(),
            Type::Callable(id) => self.callables.results[id].params.clone(),
            _ => return Ok(actual),
        };
        if expected_parameters
            .last()
            .is_some_and(|parameter| parameter.rest)
        {
            return Ok(actual);
        }
        let contracts = crate::callable_compatibility::Contracts {
            storage: &self.callables.storage,
            signatures: &self.callables.shapes,
            callbacks: &self.callables.callbacks,
            results: &self.callables.results,
            closures: &self.closures,
            effects_only: self.effects_only,
        };
        let Some(matched) = contracts.matching(actual, expected_payload, expression.span)? else {
            return Ok(actual);
        };
        let parameters = matched.actual.iter().collect::<Vec<_>>();
        if !parameters.last().is_some_and(|parameter| parameter.rest) {
            return Ok(actual);
        }
        let fixed = parameters.len() - 1;
        let minimum =
            crate::parameter_arity::minimum_arguments(&parameters[..fixed], |parameter| {
                parameter.omission.is_omittable()
            });
        let wanted_minimum =
            crate::parameter_arity::minimum_arguments(&expected_parameters, |parameter| {
                parameter.omission.is_omittable()
            });
        if minimum > wanted_minimum {
            return Err(Diagnostic::new(
                "callback-contract",
                "A rest callable requires a fixed position the destination may omit.",
                expression.span,
            ));
        }
        let Type::Array(element) = parameters[fixed].ty else {
            unreachable!("rest array contract")
        };
        for (position, wanted) in expected_parameters.iter().enumerate() {
            let accepted = if position < fixed {
                parameters[position].ty
            } else {
                element.ty()
            };
            self.require(wanted.ty, Some(accepted), expression.span)?;
            let access = if position < fixed {
                parameters[position].mode
            } else if element.ty().is_copy() {
                Mode::Copy
            } else {
                Mode::Consume
            };
            if !self.effects_only && wanted.mode != access {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Rest adaptation must preserve fixed argument access and transfer owned tail elements into its pack.",
                    expression.span,
                ));
            }
        }
        self.require(
            matched.result,
            Some(matched.expected_result),
            expression.span,
        )?;
        let result = matched.result;
        let counted = expected_parameters
            .iter()
            .skip(fixed)
            .any(|parameter| parameter.omission.is_omittable());
        let key = (actual, expected_payload, storage);
        let id = if let Some(&id) = self.rest_adapters.get(&key) {
            id
        } else {
            let (invocation, nested, borrowed) = match actual {
                Type::Closure(id) => (
                    self.closures[id].invocation,
                    self.closures[id].lifetime_count,
                    self.closures[id].borrowed,
                ),
                Type::Callback(id) => (self.callables.callbacks[id].kind.invocation(), 0, false),
                Type::StoredCallable(family) => {
                    let id = self
                        .callables
                        .storage
                        .declaration(family)
                        .expect("stored declaration");
                    (self.callables.callbacks[id].kind.invocation(), 0, false)
                }
                _ => return Ok(actual),
            };
            let span = expression.span;
            let capture = Binding {
                rest: false,
                name: "rest_source".into(),
                ty: actual,
                parameter: false,
                input_ty: None,
                omission: crate::ParameterOmission::Required,
                declaration_span: None,
                initializer: None,
                rebindable: false,
                capture_reference: matches!(storage, Mode::Read | Mode::Mutate),
                borrowed_element: None,
                borrowed_place: None,
                mutable_place: false,
                view_roots: Vec::new(),
                native_view_loans: None,
                callback_invocation: Some(invocation),
                callback_reference: false,
                callback_kind: None,
                mode: storage,
                mutable: invocation == Mode::Mutate,
                transferred: None,
                moved_fields: BTreeMap::new(),
                used: Some(span),
            };
            let mut bindings = vec![capture.clone()];
            bindings.extend(
                expected_parameters
                    .iter()
                    .enumerate()
                    .map(|(index, parameter)| Binding {
                        name: format!("argument{index}"),
                        ty: parameter.ty,
                        parameter: true,
                        input_ty: Some(parameter.ty),
                        omission: parameter.omission,
                        mode: parameter.mode,
                        callback_invocation: None,
                        capture_reference: false,
                        used: None,
                        ..capture.clone()
                    }),
            );
            let mut arguments = expected_parameters
                .iter()
                .enumerate()
                .map(|(index, parameter)| {
                    let mut value = Expr::new(ExprKind::Name(format!("argument{index}")), span);
                    value.binding = Some(index + 1);
                    value.ty = Some(parameter.ty);
                    value
                })
                .collect::<Vec<_>>();
            Self::pack_rest_arguments(&mut arguments, fixed, span);
            for (position, argument) in arguments.iter_mut().enumerate() {
                if let ExprKind::Unary {
                    op: UnaryOp::RestPack,
                    value,
                    ..
                } = &mut argument.kind
                {
                    value.ty = Some(parameters[fixed].ty);
                    argument.ty = value.ty;
                } else if argument.ty.is_none() {
                    argument.ty = Some(Type::Undefined);
                    if let Some(union) = parameters[position].ty.union() {
                        argument.inject(union);
                        argument.ty = Some(parameters[position].ty);
                    }
                }
                if position < fixed
                    && argument.ty != Some(parameters[position].ty)
                    && let Some(union) = parameters[position].ty.union()
                {
                    argument.inject(union);
                    argument.ty = Some(parameters[position].ty);
                }
            }
            let mut callee = Expr::new(ExprKind::Name("rest_source".into()), span);
            callee.binding = Some(0);
            callee.ty = Some(actual);
            let mut call = Expr::new(ExprKind::Call(Box::new(callee), arguments), span);
            call.ty = Some(result);
            call.callable_origin=Some(match actual {
                Type::Closure(id)=>CallableOrigin::Closure(id),
                Type::Callback(id)=>CallableOrigin::Callback(id),
                Type::StoredCallable(id)=>CallableOrigin::Storage(id),
                _=>unreachable!("checked rest source"),
            });
            call.target = Some(match actual {
                Type::Closure(_) => CallTarget::Closure(0),
                Type::Callback(_) => CallTarget::Callback(0),
                Type::StoredCallable(_) => CallTarget::StoredCallable(0),
                _ => unreachable!(),
            });
            let id = self.closures.len();
            let lifetime_count = nested + usize::from(matches!(storage, Mode::Read | Mode::Mutate));
            if lifetime_count > 128 {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Closure environment lifetime complexity exceeds the prototype limit of 128.",
                    span,
                ));
            }
            let polymorphic = expected_parameters.iter().any(|parameter| {
                matches!(
                    self.unions.payload(parameter.ty).unwrap_or(parameter.ty),
                    Type::Callback(_)
                )
            }) || matches!(actual, Type::Closure(source) if self.closures[source].polymorphic);
            let body = vec![Stmt { labels:Vec::new(),control_target:None,
                kind: StmtKind::Return(Some(call)),
                span,
            }];
            if polymorphic {
                let mut source = Context::new(context.function_name.clone());
                source.closure_depth = context.closure_depth + 1;
                source.capture_boundary = 1;
                source.bindings = bindings.clone();
                source.names = source
                    .bindings
                    .iter()
                    .enumerate()
                    .map(|(index, binding)| (binding.name.clone(), index))
                    .collect();
                for binding in &mut source.bindings {
                    binding.used = None;
                }
                // Creation storage may own the environment; method checking
                // starts with observed capture access, as ordinary arrows do.
                source.bindings[0].mode = Mode::Read;
                self.closure_method_sources.insert(
                    id,
                    ClosureMethodSource {
                        context: source,
                        parameters: (1..bindings.len()).collect(),
                        result,
                        body: body.clone(),
                    },
                );
            }
            self.closures.push(CheckedClosure {
                polymorphic,
                methods: Vec::new(),
                forward: Some(ClosureForward::RestAdapter { fixed, counted }),
                borrowed: borrowed || lifetime_count != 0,
                lifetime_count,
                depth: context.closure_depth + 1,
                parameters: (1..bindings.len()).collect(),
                function: CheckedFunction {
                storage_exits: Default::default(),
                    template: matches!(actual, Type::Closure(source) if self.closures[source].function.template)
                        || (polymorphic
                            && matches!(actual, Type::Callback(source) if self.callback_needs_context(source))),
                    specialization: false,
                    name: context.function_name.clone(),
                    bindings,
                    parameter_count: 0,
                    result,
                    body,
                },
                captures: vec![Capture {
                    restored:false,
                    outer: expression.binding.unwrap_or(0),
                    inner: 0,
                    storage,
                    access: invocation,
                    span,
                }],
                span,
                escapes: storage == Mode::Consume,
                invocation,
            });
            self.rest_adapters.insert(key, id);
            id
        };
        if !self.discovering_results {
            self.validated_closures.insert(id);
        }
        let span = expression.span;
        let placeholder = Expr::new(ExprKind::Undefined, span);
        let mut source = std::mem::replace(expression, placeholder);
        source.ty = Some(actual);
        *expression = Expr::new(
            ExprKind::Unary {
                op: UnaryOp::CallableRestAdapter,
                value: Box::new(source),
                operator: span,
            },
            span,
        );
        expression.ty = Some(Type::Closure(id));
        Ok(Type::Closure(id))
    }
}
