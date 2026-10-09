//! Completed caller arguments ground errors before the caller body is stored.
use super::*;

impl Analyzer {
    pub(super) fn retain_caller_error_origins(
        &self,
        context: &mut Context,
        callee: usize,
        arguments: &[Expr],
    ) {
        for (&formal, actual) in self.signatures[callee].params.iter().zip(arguments) {
            let (Type::Callback(contract), Some(actual)) = (formal, actual.ty) else {
                continue;
            };
            let origin = (contract, actual);
            if !context.caller_error_origins.contains(&origin) {
                context.caller_error_origins.push(origin);
            }
        }
    }
}
