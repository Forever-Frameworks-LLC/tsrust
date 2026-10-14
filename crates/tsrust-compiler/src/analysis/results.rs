//! Consume a Result and propagate its original owned error payload.
use super::*;
impl Analyzer {
    pub(super) fn result_call(&mut self, context: &mut Context, expression: &mut Expr, loans: &Loans, expected: Option<Type>) -> Result<Option<Type>, Diagnostic> {
        let ExprKind::Call(callee, arguments) = &mut expression.kind else { return Ok(None); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(None); };
        if !crate::native_bindings::by_symbol(name).is_some_and(|(_, binding)| binding.module == "std:result" && binding.name == "unwrap") { return Ok(None); }
        if arguments.len() != 1 { return Err(Diagnostic::new("arity", "unwrap expects one Result argument.", expression.span)); }
        let actual = self.expression(context, &mut arguments[0], Mode::Consume, loans, None)?;
        let members = match actual { Type::Union(id) => self.unions.members(id), ty => vec![ty] };
        let invalid = || Diagnostic::new("result-unwrap", "unwrap consumes a Result<T, E> and throws its Err payload.", expression.span);
        let mut output = None;
        let mut error = None;
        for member in members {
            let Type::Record(id) = member else { return Err(invalid()); };
            let record = &self.program.records[id];
            let Some(application) = &record.application else { return Err(invalid()); };
            if !matches!(application.declaration.text.as_str(), "Result#Ok" | "Result#Err") { return Err(invalid()); }
            let next_output = application.resolved_arguments.first().copied();
            let next_error = application.resolved_arguments.get(1).copied();
            if output.is_some() && (output != next_output || error != next_error) { return Err(invalid()); }
            output = next_output;
            error = next_error;
        }
        let (Some(output), Some(error)) = (output, error) else { return Err(Diagnostic::new("result-unwrap", "unwrap consumes a Result<T, E> and throws its Err payload.", expression.span)); };
        if super::threads::contains_handle(error, &self.program.records, &|id| self.unions.members(id)) {
            return Err(Diagnostic::new("thread-lifecycle", "Result propagation cannot throw a thread owner; settle it explicitly in the Err branch.", expression.span));
        }
        self.require(output, expected, expression.span)?;
        expression.ty = Some(output);
        expression.target = Some(CallTarget::ResultUnwrap(error));
        Ok(Some(output))
    }
}
