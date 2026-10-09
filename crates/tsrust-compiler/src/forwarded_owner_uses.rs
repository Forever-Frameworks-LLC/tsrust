//! Conservative use classification for concrete forwarded owner transport.
use super::*;
use std::collections::BTreeSet;

pub(super) fn incomplete_targets(program: &CheckedProgram) -> BTreeSet<usize> {
    let mut unresolved_forward = BTreeSet::new();
    for function in program
        .functions
        .iter()
        .chain(program.closures.iter().map(|closure| &closure.function))
    {
        let mut callees = BTreeSet::new();
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.target, Some(CallTarget::ForwardFunction(_)))
                && let ExprKind::Call(callee, _) = &expr.kind
            {
                callees.insert((callee.span.start, callee.span.end));
            }
        });
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            let mut reject = |ty| {
                if let Some(Type::Closure(origin)) = ty
                    && let Some(ClosureForward::Function(target)) = program.closures[origin].forward
                {
                    unresolved_forward.insert(target);
                }
            };
            if expr.binding.is_some()
                && matches!(expr.kind, ExprKind::Name(_))
                && !callees.contains(&(expr.span.start, expr.span.end))
            {
                reject(expr.ty);
            }
            if let ExprKind::Call(_, args) = &expr.kind {
                for arg in args {
                    reject(arg.ty);
                }
            }
        });
        if let Type::Closure(origin) = function.result
            && let Some(ClosureForward::Function(target)) = program.closures[origin].forward
        {
            unresolved_forward.insert(target);
        }
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
        if let Type::Closure(origin) = ty
            && let Some(ClosureForward::Function(target)) = program.closures[origin].forward
        {
            unresolved_forward.insert(target);
        }
    }
    for (_, _, origins) in program.stored_contracts.families() {
        for &origin in origins {
            if let Some(ClosureForward::Function(target)) = program.closures[origin].forward {
                unresolved_forward.insert(target);
            }
        }
    }
    for closure in &program.closures {
        if let Some(ClosureForward::Function(target)) = closure.forward {
            if !closure.captures.is_empty()
                || program
                    .named_capture_access
                    .get(&target)
                    .is_some_and(|captures| !captures.is_empty())
            {
                unresolved_forward.insert(target);
            }
        }
        if let Some(ClosureForward::Once(inner)) = closure.forward
            && let Some(ClosureForward::Function(target)) = program.closures[inner].forward
        {
            unresolved_forward.insert(target);
        }
    }
    unresolved_forward
}
