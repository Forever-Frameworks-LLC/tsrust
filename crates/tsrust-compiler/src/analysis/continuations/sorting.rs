//! Sorting separates receiver/comparator checks from final mutation access.
use super::super::*;
pub(in crate::analysis) struct Start {
    pub(in crate::analysis) hint: Option<Type>,
    pub(in crate::analysis) copying: bool,
    pub(in crate::analysis) named: bool,
    callback_hint: Option<Type>,
}
pub(in crate::analysis) struct Callback {
    pub(in crate::analysis) active: Loans,
    pub(in crate::analysis) hint: Option<Type>,
    pub(in crate::analysis) supplied: [Type; 2],
    copying: bool, named: bool, element: ArrayElement,
    boundary: crate::capture_loan_identity::Boundary,
}
pub(in crate::analysis) struct CompletionFrame {
    copying: bool, named: bool, element: ArrayElement,
    boundary: Option<crate::capture_loan_identity::Boundary>, result: Option<Type>,
    pub(in crate::analysis) method: Option<super::transform::MethodRequest>,
}
pub(in crate::analysis) enum Received { Callback(Callback), Ready(CompletionFrame) }
impl Analyzer {
    pub(in crate::analysis) fn prepare_sorting(&mut self, context: &Context, base: &Expr, args: &[Expr], span: Span, options: (bool, Option<Type>)) -> Result<Start, Diagnostic> {
        let (copying, expected) = options;
        if args.len() > 1 {
            return Err(Diagnostic::new(
                "arity",
                "Sorting accepts zero or one comparator.",
                span,
            ));
        }
        let callback_hint = args
            .first()
            .map(|arg| self.callback_value_hint(context, arg))
            .transpose()?
            .flatten();
        let value_hint = args
            .first()
            .and_then(|arg| match &arg.kind {
                ExprKind::Closure { params, .. } => params.first().map(|p| match &p.ty {
                    ParameterAnnotation::Named(token) => {
                        self.resolve_value_annotation(token, false)
                    }
                    ParameterAnnotation::Callback(annotation) => self.resolve_callback(annotation),
                    ParameterAnnotation::Inferred(token) => match &p.initializer {
                        ParameterInitializer::Default { value, .. } => self.expression_type_hint(context, value).ok_or_else(|| Diagnostic::new("parameter-inference", "This parameter needs its checked initializer or contextual type.", token.span)),
                        _ => Ok(Type::Void),
                    },
                }),
                _ => None,
            })
            .transpose()?
            .or_else(|| match callback_hint {
                Some(Type::Closure(id)) => self.closures[id]
                    .parameters
                    .first()
                    .map(|&p| self.closures[id].function.bindings[p].ty),
                Some(Type::Callback(id)) => {
                    self.callables.callbacks[id].params.first().map(|p| p.ty)
                }
                _ => None,
            });
        let value_hint = value_hint.map(|ty| Self::array_callback_hint_type(ty, args.first().and_then(|arg| self.array_callback_rest_position(arg, callback_hint)) == Some(0)));
        let value_hint = value_hint.map(|ty| self.unions.payload(ty).unwrap_or(ty));
        let hint = value_hint
            .filter(|ty| *ty != Type::Void)
            .map(|value| Self::transform_array_type(value, base.span))
            .transpose()?
            .or_else(|| expected.filter(|ty| matches!(ty, Type::Array(_))));
        let named = Self::stable_at_receiver(base);
        Ok(Start { hint, copying, named, callback_hint })
    }
    pub(in crate::analysis) fn receive_sorting(&mut self, context: &mut Context, base: &Expr, args: &mut [Expr], loans: &Loans, span: Span, start: Start, receiver: Type) -> Result<Received, Diagnostic> {
        let Start { copying, named, callback_hint, .. } = start;
        let Type::Array(element) = receiver else {
            return Err(Diagnostic::new(
                "type",
                "Sorting requires an array.",
                base.span,
            ));
        };
        if let Some(callback) = args.first_mut() {
            self.contextual_arrow_parameters(callback, &[element.ty(), element.ty()])?;
            let mut active = loans.clone();
            for root in Self::loan_roots(base, Mode::Read) {
                active
                    .entry(root)
                    .or_default()
                    .push((if copying { Mode::Read } else { Mode::Mutate }, base.span));
            }
            let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
            callback.traversal_method = None;
            return Ok(Received::Callback(Callback { active, hint: callback_hint, supplied: [element.ty(), element.ty()], copying, named, element, boundary: capture_boundary }));
        }
        if element.depth() > 0
            || matches!(
                element,
                ArrayElement::Record(_, _) | ArrayElement::StoredCallable(_, _)
            )
        {
            return Err(Diagnostic::new(
                "type",
                "Nested arrays, records and callable elements require an explicit sorting comparator.",
                span,
            ));
        }
        Ok(Received::Ready(CompletionFrame { copying, named, element, boundary: None, result: None, method: None }))
    }
    pub(in crate::analysis) fn check_sorting_callback(&mut self, context: &Context, callback: &mut Expr, frame: Callback, callback_type: Type) -> Result<CompletionFrame, Diagnostic> {
        let Callback { copying, named, element, boundary, .. } = frame;
        let (parameters, result, once) = match callback_type {
            Type::Closure(id) => {
                let closure = &self.closures[id];
                (
                    closure
                        .parameters
                        .iter()
                        .map(|&p| ArgumentContract::from(&closure.function.bindings[p]))
                        .collect::<Vec<_>>(),
                    closure.function.result,
                    closure.invocation == Mode::Consume
                        || callback.binding.is_some_and(|binding| {
                            context.bindings[binding].callback_invocation == Some(Mode::Consume)
                        }),
                )
            }
            Type::Callback(id) | Type::StoredCallable(id) | Type::BorrowedCallable(id) => {
                let id = if matches!(callback_type, Type::StoredCallable(_) | Type::BorrowedCallable(_)) {
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
                    "Sorting requires a typed reusable comparator.",
                    callback.span,
                ));
            }
        };
        if once {
            return Err(Diagnostic::new(
                "callback-access",
                "Sorting comparators must be reusable.",
                callback.span,
            ));
        }

        if !matches!(result, Type::Int | Type::Number) {
            return Err(Diagnostic::new(
                "type",
                "Sorting comparators return int or number.",
                callback.span,
            ));
        }
        let value = element.ty();
        let parameter = ArgumentContract { rest: false,
            ty: value,
            mode: if value.is_copy() {
                Mode::Copy
            } else {
                Mode::Read
            },
         omission: crate::ParameterOmission::Required, };
        self.require_array_callback_parameters(&parameters, &[parameter, parameter], callback.span)?;
        let method = if let Type::Closure(origin) = callback_type && self.closures[origin].polymorphic {
            Some(super::transform::MethodRequest { origin, types: Self::array_callback_parameter_types(&parameters, &[value, value]) })
        } else { None };
        Ok(CompletionFrame { copying, named, element, boundary: Some(boundary), result: Some(result), method })
    }
    pub(in crate::analysis) fn finish_sorting(&mut self, context: &mut Context, base: &Expr, args: &mut [Expr], loans: &Loans, span: Span, frame: CompletionFrame, specialized: Option<(usize, Type)>) -> Result<Type, Diagnostic> {
        let CompletionFrame { copying, named, element, boundary, result, .. } = frame;
        if let Some((method, specialized_result)) = specialized {
            self.require(specialized_result, result, args[0].span)?;
            args[0].traversal_method = Some(TraversalMethod { id: method, access: Mode::Read });
        }
        if let Some(boundary) = boundary { boundary.unwind(&mut context.capture_loans); }
        if !copying && named { Self::storage_access(context, base, Mode::Mutate, span, loans)?; }
        Ok(if copying { Type::Array(element) } else { Type::Void })
    }
}
