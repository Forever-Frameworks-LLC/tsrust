//! Installer closures have a native interface for direct calls only.
//!
//! A closure that stores a borrowing callback through its record parameter is
//! generic over that record's view and carries replacement obligations. A
//! direct call, also through a closure that captured it, supplies both. Using
//! the closure as a value does not: an abstract callback invocation sees only
//! the callback contract. Such uses are source errors, not native failures.
use super::*;

pub(super) fn check(
    functions: &[Option<CheckedFunction>],
    closures: &[CheckedClosure],
    function_borrows: &HashMap<usize, Vec<record_parameter_borrows::Borrow>>,
    parameter_borrows: &HashMap<usize, Vec<record_parameter_borrows::Borrow>>,
    capture_borrows: &HashMap<usize, Vec<record_parameter_borrows::CapturedBorrow>>,
) -> Result<(), Diagnostic> {
    // A named installer passed as a callback argument is invoked abstractly.
    let forwarded_installer = |ty: Option<Type>| matches!(ty, Some(Type::Closure(origin))
        if matches!(closures[origin].forward, Some(ClosureForward::Function(target)) if function_borrows.contains_key(&target)));
    let mut passed: Option<Span> = None;
    for function in functions.iter().flatten().chain(closures.iter().map(|closure| &closure.function)).filter(|function| !function.template) {
        // A saved installer is called through its own name; any other use of
        // that name, such as a second alias, is abstract as well.
        let mut forwarded_callees = HashSet::new();
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.target, Some(CallTarget::ForwardFunction(_)))
                && let ExprKind::Call(callee, _) = &expr.kind
            {
                forwarded_callees.insert((callee.span.start, callee.span.end));
            }
        });
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.kind, ExprKind::Name(_))
                && expr.binding.is_some()
                && !forwarded_callees.contains(&(expr.span.start, expr.span.end))
                && forwarded_installer(expr.ty)
                && passed.is_none_or(|span| expr.span.start < span.start)
            {
                passed = Some(expr.span);
            }
        });
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if let ExprKind::Call(_, arguments) = &expr.kind {
                for argument in arguments {
                    if forwarded_installer(argument.ty) && passed.is_none_or(|span| argument.span.start < span.start) {
                        passed = Some(argument.span);
                    }
                }
            }
        });
    }
    if let Some(span) = passed {
        return Err(Diagnostic::new(
            "unsupported",
            "A function that stores a borrowing callback through its record parameter cannot be passed as a callback or aliased twice yet. Call it directly, or save it with const and call that name.",
            span,
        ));
    }
    // The record that receives the callback must be a parameter or a local of
    // the calling closure: a captured record has no retained-borrow interface.
    for closure in closures.iter().filter(|closure| !closure.function.template) {
        let mut captured: Option<Span> = None;
        crate::fixed_storage::visit_function_expressions(&closure.function, &mut |expr| {
            let ExprKind::Call(_, arguments) = &expr.kind else { return; };
            let targets = match expr.target {
                Some(CallTarget::Function(id) | CallTarget::ForwardFunction(id)) => function_borrows.get(&id).into_iter().flatten().map(|borrow| borrow.target.binding).collect::<Vec<_>>(),
                Some(CallTarget::Closure(binding)) => match closure.function.bindings.get(binding).map(|value| value.ty) {
                    Some(Type::Closure(origin)) => parameter_borrows.get(&origin).into_iter().flatten().map(|borrow| borrow.target.binding)
                        .chain(capture_borrows.get(&origin).into_iter().flatten().map(|borrow| borrow.target.binding))
                        .filter_map(|parameter| closures[origin].parameters.iter().position(|candidate| *candidate == parameter)).collect(),
                    _ => Vec::new(),
                },
                _ => Vec::new(),
            };
            for slot in targets {
                if let Some(argument) = arguments.get(slot)
                    && let Some(root) = Analyzer::root_binding(argument)
                    && closure.captures.iter().any(|capture| capture.inner == root)
                    && captured.is_none_or(|span| argument.span.start < span.start)
                {
                    captured = Some(argument.span);
                }
            }
        });
        if let Some(span) = captured {
            return Err(Diagnostic::new(
                "unsupported",
                "A closure cannot store a borrowing callback into a record that it captured. Pass the record to the closure as a Mutable parameter.",
                span,
            ));
        }
        // A direct assignment has the same limit: the captured record's
        // callback carrier is fixed where the record was declared.
        for write in record_storage_writes::collect(&closure.function.body) {
            if write.path.is_empty() || !closure.captures.iter().any(|capture| capture.inner == write.binding) {
                continue;
            }
            // An owning callback field stores an owned callable and is unaffected.
            let mut value = write.value;
            while let ExprKind::Inject { value: inner, .. } = &value.kind {
                value = inner;
            }
            if matches!(value.ty, Some(Type::Closure(_) | Type::BorrowedCallable(_) | Type::Callback(_))) {
                return Err(Diagnostic::new(
                    "unsupported",
                    "A closure cannot replace a callback field of a record that it captured. Pass the record to the closure as a Mutable parameter.",
                    write.span,
                ));
            }
        }
    }
    let installer = |origin: usize| parameter_borrows.contains_key(&origin) || capture_borrows.contains_key(&origin);
    let bodies = functions
        .iter()
        .flatten()
        .filter(|function| !function.template)
        .chain(closures.iter().map(|closure| &closure.function).filter(|function| !function.template));
    let mut found: Option<Span> = None;
    for function in bodies {
        let mut callees = HashSet::new();
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
                && let Some(Type::Closure(origin)) = expr.ty
                && installer(origin)
                && found.is_none_or(|span| expr.span.start < span.start)
            {
                found = Some(expr.span);
            }
        });
    }
    if let Some(span) = found {
        return Err(Diagnostic::new(
            "unsupported",
            "A closure that stores a borrowing callback through its record parameter can only be called directly, or from a closure that captures it. It cannot be passed as a callback, aliased or converted yet.",
            span,
        ));
    }
    // A held installer lends a mutable capture through two receivers at once;
    // only a shared captured source has a lifetime independent of them.
    for (&origin, borrows) in capture_borrows {
        let closure = &closures[origin];
        for borrow in borrows {
            let path = borrow.source_projection.captures();
            if path.is_empty() {
                continue;
            }
            let held = closure.function.bindings[closure.captures[borrow.source_capture].inner].ty;
            if retained_capture_projection::mode(closures, held, path) == Some(Mode::Mutate) {
                let span = closure.function.body.first().map_or(Span::default(), |statement| statement.span);
                return Err(Diagnostic::new(
                    "unsupported",
                    "A closure cannot call a captured installer whose stored callback mutates a variable that the installer captured. Call the installer directly, or capture the variable for reading.",
                    span,
                ));
            }
        }
    }
    Ok(())
}
