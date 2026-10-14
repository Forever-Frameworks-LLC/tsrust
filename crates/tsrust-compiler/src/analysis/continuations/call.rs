//! Call continuations retain capture-loan boundaries through callee and arguments.
use super::super::*;

pub(in crate::analysis) enum CallDispatch {
    Ready(Type),
    Transform(super::transform::Request),
    ReadyMethod { origin: usize, method: usize, result: Type },
    Method { binding: usize, origin: usize, params: Vec<ArgumentContract>, invocation: Mode },
    Conditional { publish_type: bool },
    Declared { resolve_completed: bool },
    Bound { binding: usize, params: Vec<ArgumentContract>, result: Type, invocation: Mode, target: CallTarget },
    StoredField { field_type: Type, params: Vec<ArgumentContract>, result: Type, invocation: Mode, family: usize },
}

pub(super) struct CalleeFrame {
    capture_boundary: crate::capture_loan_identity::Boundary,
    pub(super) checked_callee: bool,
}

pub(super) struct CallCompletion {
    capture_boundary: crate::capture_loan_identity::Boundary,
    pub(super) checked_callee: bool,
    pub(super) hint: Type,
    span: Span,
}

pub(super) struct CallableInvocation {
    pub(super) params: Vec<ArgumentContract>,
    pub(super) invocation: Mode,
    hint: Type,
    result: Type,
    target: CallTarget,
}

pub(super) enum PreparedInvocation { Ready((CallTarget, Type)), Check(CallableInvocation) }

impl Analyzer {
    pub(in crate::analysis) fn prepare_builtin_call_work(&mut self, context: &mut Context, expr: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<CallDispatch, Diagnostic> {
        let ExprKind::Call(callee, _) = &expr.kind else { unreachable!("call dispatch") };
        let stored_field = matches!(callee.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
            && self.expression_type_hint(context, callee).is_some_and(|ty| matches!(ty, Type::StoredCallable(_) | Type::BorrowedCallable(_)));
        if stored_field {
            let field_type = self.expression_type_hint(context, callee).expect("stored field hint");
            let (Type::StoredCallable(family) | Type::BorrowedCallable(family)) = field_type else { unreachable!("stored field") };
            let declaration = self.callables.storage.declaration(family).expect("checked field storage signature");
            let contract = &self.callables.callbacks[declaration];
            let params = contract.params.clone();
            let result = contract.result;
            let invocation = contract.invocation(&self.callables.shapes);
            let ExprKind::Call(_, args) = &mut expr.kind else { unreachable!("stored field call") };
            Self::prepare_arguments(args, &params, expr.span)?;
            return Ok(CallDispatch::StoredField { field_type, params, result, invocation, family });
        }
        let callable_field = matches!(callee.kind, ExprKind::Field(_, _) | ExprKind::Index(_, _))
            && self.expression_type_hint(context, callee).is_some_and(|ty| matches!(ty, Type::Closure(_) | Type::Callback(_)));
        if callable_field || matches!(callee.kind,
            ExprKind::Selected { .. } | ExprKind::CoalesceAssign { .. } | ExprKind::SpreadValue(_)
                | ExprKind::Call(_, _) | ExprKind::Sequence(_) | ExprKind::Closure { .. } | ExprKind::Conditional { .. }
                | ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault, .. }
                | ExprKind::Unary { op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull, .. }) {
            return Ok(CallDispatch::Conditional { publish_type: true });
        }
        if let Some(request) = self.transform_request(context, expr, expected, false) { return Ok(CallDispatch::Transform(request)); }
        self.builtin_or_method_call(context, expr, loans, expected).map(CallDispatch::Ready)
    }

    pub(super) fn prepare_conditional_callee(&mut self, context: &mut Context, callee: &mut Expr, args: &mut Vec<Expr>) -> Result<CalleeFrame, Diagnostic> {
        let capture_boundary = crate::capture_loan_identity::Boundary::capture(&context.capture_loans);
        let distinct_conditional = if let ExprKind::Conditional { consequent, alternate, .. } = &callee.kind {
            let left = self.callback_value_hint(context, consequent)?;
            let right = self.callback_value_hint(context, alternate)?;
            left.is_none() || right.is_none() || left != right
        } else { false };
        let checked_callee = matches!(callee.kind, ExprKind::Closure { .. } | ExprKind::Call(_, _)) || distinct_conditional;
        if matches!(callee.kind, ExprKind::Closure { .. }) { self.contextual_expression_callee(context, callee, args)?; }
        Ok(CalleeFrame { capture_boundary, checked_callee })
    }

    pub(super) fn finish_conditional_callee(&mut self, callee: &Expr, arguments: usize, span: Span, frame: CalleeFrame, hint: Option<Type>) -> Result<CallCompletion, Diagnostic> {
        let hint = hint.map(|hint| if let Type::Callable(id) = hint { self.result_shape(id) } else { hint });
        let hint = hint.ok_or_else(|| Diagnostic::new("unsupported", "Conditional invocation requires a resolved callable origin.", callee.span))?;
        let hint = if let Type::Closure(origin) = hint
            && let Some(ClosureForward::Native(id)) = self.closures[origin].forward {
            self.native_function_shape(id, span, arguments, false)?
        } else { hint };
        Ok(CallCompletion { capture_boundary: frame.capture_boundary, checked_callee: frame.checked_callee, hint, span })
    }

    pub(super) fn finish_conditional_call(&mut self, context: &mut Context, callee: &mut Expr, frame: CallCompletion, result: (CallTarget, Type)) -> Result<(CallTarget, Type), Diagnostic> {
        let CallCompletion { capture_boundary, checked_callee, hint, span } = frame;
        self.remember_native_expression_call(context,result.0,callee,hint,span)?;
        if checked_callee && !self.expression_result_retains_captures(result.1){capture_boundary.unwind(&mut context.capture_loans);}
        Ok(result)
    }

    pub(super) fn prepare_callable_invocation(&mut self, context: &mut Context, callee: &mut Expr, args: &mut Vec<Expr>, loans: &Loans, span: Span, hint: Type, checked_callee: bool, expected: Option<Type>) -> Result<PreparedInvocation, Diagnostic> {
        if let Type::Closure(origin) = hint {
            let (origin, once) = match self.closures[origin].forward {
                Some(ClosureForward::Once(inner)) => (inner, true),
                _ => (origin, false),
            };
            if self.deferred_generic_values.contains_key(&origin) {
                let (method, result) = self.deferred_generic_call(context, None, origin, callee, args, loans, span, once, expected, checked_callee)?;
                return Ok(PreparedInvocation::Ready((CallTarget::ClosureMethod(method), result)));
            }
        }
        let (params, result, invocation, target) = match hint {
            Type::Closure(id) => {
                let closure = &self.closures[id];
                let invocation = Self::conditional_callback_metadata(context, callee)?
                    .and_then(|x| x.0)
                    .unwrap_or(closure.invocation);
                (
                    closure
                        .parameters
                        .iter()
                        .map(|&p| ArgumentContract::from(&closure.function.bindings[p]))
                        .collect::<Vec<_>>(),
                    closure.function.result,
                    invocation,
                    CallTarget::ConditionalClosure(id),
                )
            }
            Type::Callback(id) => {
                let contract = &self.callables.callbacks[id];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.invocation(&self.callables.shapes),
                    CallTarget::ConditionalCallback(id),
                )
            }
            Type::StoredCallable(family)|Type::BorrowedCallable(family) => {
                let declaration = self
                    .callables
                    .storage
                    .declaration(family)
                    .expect("stored signature");
                let contract = &self.callables.callbacks[declaration];
                (
                    contract.params.clone(),
                    contract.result,
                    contract.invocation(&self.callables.shapes),
                    CallTarget::StoredCallableField(family),
                )
            }
            _ => {
                return Err(Diagnostic::new(
                    "type",
                    "This expression must produce a callable value with a compatible signature.",
                    callee.span,
                ));
            }
        };
        Self::prepare_arguments(args, &params, span)?;
        Ok(PreparedInvocation::Check(CallableInvocation { params, invocation, hint, result, target }))
    }

    pub(super) fn finish_invocation_callee(&mut self, context: &mut Context, callee: &Expr, frame: &mut CallableInvocation, actual: Type, loans: &Loans) -> Result<super::arguments::Arguments, Diagnostic> {
        self.require(actual, Some(frame.hint), callee.span)?;
        Self::conditional_callback_metadata(context, callee)?;
        let mut active = loans.clone();
        for root in Self::loan_roots(callee, frame.invocation) {
            active.entry(root).or_default().push((frame.invocation, callee.span));
        }
        Ok(super::arguments::Arguments::new(std::mem::take(&mut frame.params), active, context, true, false, false))
    }

    pub(super) fn finish_callable_invocation(&mut self, context: &mut Context, args: &mut Vec<Expr>, span: Span, frame: CallableInvocation) -> Result<FinishedInvocation, Diagnostic> {
        let CallableInvocation { hint, result, target, .. } = frame;
        if let Type::Closure(origin) = hint {
            let (origin, _) = match self.closures[origin].forward {
                Some(ClosureForward::Once(inner)) => (inner, true),
                _ => (origin, false),
            };
            if let Some(ClosureForward::Function(function)) = self.closures[origin].forward {
                let (function, result) = self.resolved_call(context, function, args, span)?;
                return Ok(FinishedInvocation::Ready(CallTarget::ForwardFunction(function), result));
            }
            if self.closures[origin].polymorphic {
                return Ok(FinishedInvocation::Method(origin, self.closure_method_argument_types(origin, args)));
            }
        }
        let result = if let Type::Callable(contract) = result {
            if context.template || self.discovering_results {
                self.result_shape(contract)
            } else {
                return Err(Self::pending_result(span));
            }
        } else {
            result
        };
        Ok(FinishedInvocation::Ready(target, result))
    }


}

pub(super) enum FinishedInvocation { Ready(CallTarget, Type), Method(usize, Vec<Type>) }
