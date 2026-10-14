//! Reduction stages preserve callback-before-initial checking and receiver loans.
use super::super::*;
pub(in crate::analysis) struct Start {
    pub(in crate::analysis) receiver_hint: Option<Type>,
    accumulator_hint: Option<Type>, value_hint: Option<Type>, callback_hint: Option<Type>,
}
pub(in crate::analysis) struct Callback {
    pub(in crate::analysis) active: Loans,
    pub(in crate::analysis) checked_hint: Option<Type>,
    pub(in crate::analysis) supplied: [Type; 4],
    receiver: Type, element: ArrayElement, lends_callable: bool,
    boundary: crate::capture_loan_identity::Boundary,
}
pub(in crate::analysis) struct Initial {
    pub(in crate::analysis) active: Loans,
    pub(in crate::analysis) result: Type,
    receiver: Type, element: ArrayElement, lends_callable: bool,
    boundary: crate::capture_loan_identity::Boundary, callback: Type, parameters: Vec<ArgumentContract>,
}
pub(in crate::analysis) struct CompletionFrame {
    result: Type, lends_callable: bool, boundary: crate::capture_loan_identity::Boundary,
    pub(in crate::analysis) method: Option<super::transform::MethodRequest>,
}
impl Analyzer {
    pub(in crate::analysis) fn prepare_reduction(&mut self, context: &Context, base: &Expr, args: &mut [Expr], span: Span) -> Result<Start, Diagnostic> {
        if !(1..=2).contains(&args.len()) {
            return Err(Diagnostic::new(
                "arity",
                "reduce() and reduceRight() expect a callback and optional initial value.",
                span,
            ));
        }
        args[0].traversal_method = None;
        let callback_hint = self.callback_value_hint(context, &args[0])?;
        let (parameter_hints, result_hint) = match &args[0].kind {
            ExprKind::Closure {
                params,
                result: annotation,
                ..
            } => (
                params.iter().map(|parameter| match &parameter.ty {
                    ParameterAnnotation::Named(token) => self.callable_resolver().resolve_value_annotation(token, false).map(Some),
                    ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation).map(Some),
                    ParameterAnnotation::Inferred(_) => Ok(match &parameter.initializer {
                        ParameterInitializer::Default { value, .. } => self.expression_type_hint(context, value),
                        _ => None,
                    }),
                }).collect::<Result<Vec<_>, Diagnostic>>()?,
                if matches!(annotation, ResultAnnotation::Inferred(_)) { None } else { Some(self.resolve_result(annotation)?) },
            ),
            _ => match callback_hint {
                Some(Type::Closure(id)) if !self.deferred_generic_values.contains_key(&id) => (
                    self.closures[id]
                        .parameters
                        .iter()
                        .map(|&parameter| Some(self.closures[id].function.bindings[parameter].ty))
                        .collect(),
                    Some(self.closures[id].function.result),
                ),
                Some(Type::Callback(id)) => (
                    self.callables.callbacks[id]
                        .params
                        .iter()
                        .map(|parameter| Some(parameter.ty))
                        .collect(),
                    Some(self.callables.callbacks[id].result),
                ),
                Some(Type::Callable(id)) => (
                    self.callables.results[id]
                        .params
                        .iter()
                        .map(|parameter| Some(parameter.ty))
                        .collect(),
                    Some(self.callables.results[id].result),
                ),
                _ => (Vec::new(), None),
            },
        };
        let rest_position = self.array_callback_rest_position(&args[0], callback_hint);
        let parameter_hints = parameter_hints.into_iter().enumerate().map(|(index, ty)| ty.map(|ty| Self::array_callback_hint_type(ty, rest_position == Some(index)))).collect::<Vec<_>>();
        let accumulator_hint = parameter_hints.first().copied().flatten().or(result_hint);
        let value_hint = parameter_hints
            .get(1)
            .copied()
            .flatten()
            .or_else(|| (args.len() == 1 || rest_position == Some(0)).then_some(accumulator_hint).flatten());
        let value_hint = value_hint.map(|ty| self.unions.payload(ty).unwrap_or(ty));
        let absent_elements = matches!(self.expression_type_hint(context, base), Some(Type::Array(element)) if matches!(element.ty(), Type::Null | Type::Undefined)) || matches!(&base.kind, ExprKind::Array(values) if !values.is_empty() && values.iter().all(|value| matches!(value.kind, ExprKind::Null | ExprKind::Undefined)));
        let receiver_hint = value_hint
            .filter(|_| !absent_elements)
            .filter(|value| !matches!(value, Type::Callback(_)))
            .map(|value| Self::transform_array_type(value, base.span))
            .transpose()?;
        Ok(Start { receiver_hint, accumulator_hint, value_hint, callback_hint })
    }
    pub(in crate::analysis) fn receive_reduction(&mut self, context: &mut Context, base: &Expr, args: &mut [Expr], loans: &Loans, start: Start, receiver: Type) -> Result<Callback, Diagnostic> {
        let Start { accumulator_hint, value_hint, callback_hint, .. } = start;
        let Type::Array(element) = receiver else {
            return Err(Diagnostic::new(
                "type",
                "Array reduction requires an array.",
                base.span,
            ));
        };
        if let ExprKind::Closure { params, .. } = &args[0].kind
            && params.iter().any(|param| matches!(param.ty, ParameterAnnotation::Inferred(_))) {
            let accumulator = accumulator_hint.or_else(|| args.get(1).and_then(|initial| self.expression_type_hint(context, initial))).unwrap_or(element.ty());
            self.contextual_arrow_parameters(&mut args[0], &[accumulator, element.ty(), Type::Int, receiver])?;
        }
        let lends_callable = matches!(element, ArrayElement::StoredCallable(_, _))
            && value_hint.is_some_and(|value| matches!(value, Type::Callback(id) if self.callables.callbacks[id].kind == CallbackKind::Reusable));
        let receiver_mode = if lends_callable {
            Mode::Mutate
        } else {
            Mode::Read
        };
        let mut active = loans.clone();
        for root in Self::loan_roots(base, receiver_mode) {
            active
                .entry(root)
                .or_default()
                .push((receiver_mode, base.span));
        }
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        let checked_hint = callback_hint.filter(|ty| !matches!(ty, Type::Closure(id) if self.closures[*id].polymorphic && matches!(args[0].kind, ExprKind::Call(_, _))));
        let accumulator = accumulator_hint
            .or_else(|| args.get(1).and_then(|initial| self.expression_type_hint(context, initial)))
            .unwrap_or(element.ty());
        Ok(Callback { active, checked_hint, supplied: [accumulator, element.ty(), Type::Int, receiver], receiver, element, lends_callable, boundary: capture_boundary })
    }
    pub(in crate::analysis) fn check_reduction_callback(&self, context: &Context, args: &[Expr], frame: Callback, callback: Type) -> Result<Initial, Diagnostic> {
        let (parameters, result, once) = match callback {
            Type::Closure(id) => {
                let closure = &self.closures[id];
                (
                    closure
                        .parameters
                        .iter()
                        .map(|&parameter| {
                            ArgumentContract::from(&closure.function.bindings[parameter])
                        })
                        .collect::<Vec<_>>(),
                    closure.function.result,
                    closure.invocation == Mode::Consume
                        || args[0].binding.is_some_and(|binding| {
                            context.bindings[binding].callback_invocation == Some(Mode::Consume)
                        }),
                )
            }
            Type::Callback(id) | Type::StoredCallable(id) | Type::BorrowedCallable(id) => {
                let id = if matches!(callback, Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
                    self.callables.storage.declaration(id).expect("checked traversal storage")
                } else { id };
                let contract = &self.callables.callbacks[id];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.kind.is_once(),
                )
            }
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "Array reduction requires a typed reusable callback.",
                    args[0].span,
                ));
            }
        };
        if once {
            return Err(Diagnostic::new(
                "callback-access",
                "Reduction callbacks must be reusable; their captured environment cannot be consumed.",
                args[0].span,
            ));
        }

        if !matches!(
            result,
            Type::Int
                | Type::Number
                | Type::Boolean
                | Type::Null
                | Type::Undefined
                | Type::String
                | Type::Record(_)
                | Type::Array(_)
        ) {
            return Err(Diagnostic::new(
                "type",
                "Reduction accumulators must be int, number, boolean, string, named records, or arrays.",
                args[0].span,
            ));
        }
        Ok(Initial { active: frame.active, result, receiver: frame.receiver, element: frame.element, lends_callable: frame.lends_callable, boundary: frame.boundary, callback, parameters })
    }
    pub(in crate::analysis) fn complete_reduction_arguments(&mut self, args: &[Expr], span: Span, frame: Initial, actual: Option<Type>) -> Result<CompletionFrame, Diagnostic> {
        let Initial { result, receiver, element, lends_callable, boundary, callback, parameters, .. } = frame;
        if let Some(actual) = actual { self.require(actual, Some(result), args[1].span)?; }
        else { self.require(result, Some(element.ty()), args[0].span)?; }
        let mut method = None;
        if let Some(accumulator) = parameters.first().filter(|parameter| !parameter.rest) {
            self.require(result, Some(accumulator.ty), args[0].span)?;
            if !self.effects_only && result.is_copy() && accumulator.mode != Mode::Copy {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Copy reduction accumulators require copy access.",
                    args[0].span,
                ));
            }
        }
        let value = element.ty();
        let expected = [
            ArgumentContract { rest: false,
                ty: value,
                mode: if lends_callable {
                    Mode::Mutate
                } else if value.is_copy() {
                    Mode::Copy
                } else {
                    Mode::Read
                },
             omission: crate::ParameterOmission::Required, },
            ArgumentContract { rest: false,
                ty: Type::Int,
                mode: Mode::Copy,
                omission: crate::ParameterOmission::Required,
            },
            ArgumentContract { rest: false,
                ty: receiver,
                mode: Mode::Read,
                omission: crate::ParameterOmission::Required,
            },
        ];
        if lends_callable {
            if args.len() != 2 {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Callable array reductions require an independent initial accumulator.",
                    span,
                ));
            }
            if parameters.iter().skip(3).any(|parameter| !parameter.rest && !parameter.omission.is_omittable()) {
                return Err(Diagnostic::new(
                    "conflicting-access",
                    "A mutable callable element cannot be borrowed together with its source array.",
                    args[0].span,
                ));
            }
            self.require(value, Some(parameters[1].ty), args[0].span)?;
            if parameters[1].mode != Mode::Mutate {
                return Err(Diagnostic::new(
                    "callback-access",
                    "Callable element invocation requires a mutate callback parameter.",
                    args[0].span,
                ));
            }
            self.require_array_callback_parameters(&parameters[2..], &expected[1..2], args[0].span)?;
            let Type::Closure(origin) = callback else {
                return Err(Diagnostic::new(
                    "unsupported",
                    "Callable element reduction requires a concrete arrow environment.",
                    args[0].span,
                ));
            };
            let mut types = vec![result];
            types.extend(
                Self::array_callback_parameter_types(&parameters[1..], &expected[..2].iter().map(|parameter| parameter.ty).collect::<Vec<_>>()),
            );
            method = Some(super::transform::MethodRequest { origin, types });
        } else if parameters.first().is_some_and(|parameter| parameter.rest) {
            let mut supplied = vec![ArgumentContract { ty: result, mode: if result.is_copy() { Mode::Copy } else { Mode::Consume }, omission: crate::ParameterOmission::Required, rest: false }];
            supplied.extend_from_slice(&expected);
            self.require_array_callback_parameters(&parameters, &supplied, args[0].span)?;
        } else if parameters.len() > 1 {
            self.require_array_callback_parameters(&parameters[1..], &expected, args[0].span)?;
        }
        if !lends_callable && let Type::Closure(origin) = callback
            && self.closures[origin].polymorphic {
            let mut supplied = vec![result];
            supplied.extend(expected.iter().map(|parameter| parameter.ty));
            let types = Self::array_callback_parameter_types(&parameters, &supplied);
            method = Some(super::transform::MethodRequest { origin, types });
        }
        Ok(CompletionFrame { result, lends_callable, boundary, method })
    }
    pub(in crate::analysis) fn finish_reduction(&mut self, context: &mut Context, base: &Expr, args: &mut [Expr], loans: &Loans, span: Span, frame: CompletionFrame, specialized: Option<(usize, Type)>) -> Result<Type, Diagnostic> {
        let CompletionFrame { mut result, lends_callable, boundary, .. } = frame;
        if let Some((method, specialized_result)) = specialized {
            self.require(specialized_result, Some(result), args[0].span)?;
            result = specialized_result;
            args[0].traversal_method = Some(TraversalMethod { id: method, access: if lends_callable { Mode::Mutate } else { Mode::Read } });
            if lends_callable { Self::storage_access(context, base, Mode::Mutate, span, loans)?; }
        }
        boundary.unwind(&mut context.capture_loans);
        Ok(result)
    }
}
