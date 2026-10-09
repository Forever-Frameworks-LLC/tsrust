//! Concrete closure owner transport requires complete, direct invocation contexts.
use super::*;
use std::collections::BTreeSet;

pub(super) fn incomplete_targets(program: &CheckedProgram) -> BTreeSet<usize> {
    let mut result = BTreeSet::new();
    for function in program
        .functions
        .iter()
        .chain(program.closures.iter().map(|closure| &closure.function))
    {
        let mut callees = BTreeSet::new();
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.target, Some(CallTarget::Closure(_)))
                && let ExprKind::Call(callee, _) = &expr.kind
            {
                callees.insert((callee.span.start, callee.span.end));
            }
        });
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.kind, ExprKind::Name(_))
                && expr.binding.is_some()
                && !callees.contains(&(expr.span.start, expr.span.end))
            {
                if let Some(Type::Closure(origin)) = expr.ty {
                    result.insert(origin);
                }
            }
        });
    }
    for (origin, closure) in program.closures.iter().enumerate() {
        let captured_record = closure.captures.iter().any(|capture| {
            matches!(capture.storage, Mode::Read | Mode::Mutate)
                && matches!(closure.function.bindings[capture.inner].ty, Type::Record(_))
        });
        if closure.escapes
            || closure.polymorphic
            || captured_record
            || program.closure_record_capture_borrows.contains_key(&origin)
        {
            result.insert(origin);
        }
    }
    for (_, _, origins) in program.stored_contracts.families() {
        result.extend(origins);
    }
    for ty in program
        .records
        .iter()
        .flat_map(|record| record.fields.iter().filter_map(|field| field.ty))
        .chain(
            program
                .unions
                .iter()
                .flat_map(|union| union.members.iter().copied()),
        )
    {
        if let Type::Closure(origin) = ty {
            result.insert(origin);
        }
    }
    result
}
