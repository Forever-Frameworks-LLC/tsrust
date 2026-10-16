//! Return inference and escape checks resume after the value expression completes.
use super::super::*;

pub(super) struct ReturnFrame { result_type: Type, span: Span, pub(super) expected: Option<Type> }

impl Analyzer {
    pub(in crate::analysis) fn return_statement(&mut self, context: &mut Context, value: &mut Option<Expr>, result_type: Type, span: Span) -> Result<(), Diagnostic> {
        let frame = self.prepare_return(context, value, result_type, span);
        let actual = match value {
            Some(value) => self.expression(context, value, Mode::Consume, &Loans::new(), frame.expected)?,
            None => Type::Void,
        };
        self.finish_return(context, value, frame, actual)
    }

    pub(super) fn prepare_return(&mut self, context: &mut Context, value: &Option<Expr>, result_type: Type, span: Span) -> ReturnFrame {
        context.returning_closure = self.callable_result_type(result_type) || matches!(result_type,Type::Closure(_))
            || (context.inferred_body_result && value.as_ref().and_then(|value|self.expression_type_hint(context,value)).is_some_and(|ty|matches!(self.unions.payload(ty).unwrap_or(ty),Type::Closure(_)) || self.callable_result_type(ty)));
        ReturnFrame { result_type, span, expected: if context.inferred_body_result { None } else { Some(result_type) } }
    }

    pub(super) fn finish_return(&mut self, context: &mut Context, value: &mut Option<Expr>, frame: ReturnFrame, mut actual: Type) -> Result<(), Diagnostic> {
        let ReturnFrame { result_type, span, .. } = frame;
        context.returning_closure = false;
        // shortcut: borrowed child tasks need result lifetime lowering before they can escape an async body.
        if context.task_body && self.is_task_type(actual) && self.borrows_local_storage(actual) {
            return Err(Diagnostic::new("task-result-borrow", "A task cannot return another task that retains borrowed data. Await the child and return its value instead.", span));
        }
        let facts = self.aggregate_callable_facts(actual);
        // An absent nullable record holds no callback instance to escape.
        let absent_record = self.nullable_scoped_record_declaration(actual).is_some()
            && value.as_ref().is_some_and(|value| matches!(&value.kind, ExprKind::Inject { value, .. } if matches!(value.kind, ExprKind::Null | ExprKind::Undefined)));
        if (facts.unknown || facts.scoped && self.borrows_local_storage(actual)) && !absent_record && !(self.is_task_type(actual) && context.capture_loans.iter().all(|(root,access,_)|(context.bindings[*root].parameter && !context.task_owned_inputs.as_ref().is_some_and(|owned|owned.contains(root))) || (context.http_retained_state && *root<context.capture_boundary && *access==Mode::Read))) {
            return Err(Diagnostic::new("callback-escape","A returned value cannot contain a callable that borrows local storage.",span));
        }
        let converted =
            self.callable_result_value(context, result_type, actual, span);
        if converted != actual
            && let Some(value) = value
            && let Type::Closure(origin) = converted
        {
            value.wrap_once(origin);
        }
        actual = converted;
        if !context.inferred_body_result { self.require(actual, Some(result_type), span)?; }
        if let Some(value)=value {self.record_native_return(context,value)?;}
        if self.discovering_results && self.has_pending_result(actual) {
            return Ok(());
        }
        if matches!(
            self.unions.payload(actual).unwrap_or(actual),
            Type::Closure(_) | Type::Callback(_) | Type::StoredCallable(_)
        ) {
            // A template body is symbolic: a call it returns names a concrete
            // copy, whose closure literals have their own identities. The copy
            // itself is checked against this rule.
            let symbolic_call = context.template
                && value.as_ref().is_some_and(|value| matches!(value.kind, ExprKind::Call(_, _)));
            // A returned literal replaces a symbolic call; two literals must agree.
            let symbolic = symbolic_call || context.returned_symbolic_call;
            if symbolic && context.returned_callable.is_some() {
                if !symbolic_call {
                    context.returned_callable = Some(actual);
                    context.returned_symbolic_call = false;
                }
                self.record_typed_completion(context,crate::completion::CompletionKind::Return,actual,span);
                return Ok(());
            }
            if !self.effects_only
                && context.returned_callable.is_some_and(|previous| {
                    previous != actual
                        && !self.absent_callable_type(previous)
                        && !self.absent_callable_type(actual)
                })
            {
                return Err(Diagnostic::new(
                    "unsupported",
                    "A factory must return one closure origin; mixed environments are unsupported.",
                    span,
                ));
            }
            if !self.absent_callable_type(actual) || context.returned_callable.is_none()
            {
                context.returned_callable = Some(actual);
                context.returned_symbolic_call = symbolic_call;
            }
        }
        self.record_typed_completion(context,crate::completion::CompletionKind::Return,actual,span);
        Ok(())
    }

}
