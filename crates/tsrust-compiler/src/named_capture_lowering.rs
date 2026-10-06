//! Hidden capture arguments of nested named functions.
//!
//! A nested function that uses bindings of an enclosing function receives
//! them as hidden native parameters after its source parameters. A direct
//! call passes each one from the calling function: a copy for a scalar that
//! is only read, otherwise a borrow that lasts for the call. A caller that is
//! itself a nested function forwards its own hidden parameter by reborrow.
//! Recursive groups therefore need no shared environment value, a capture
//! declared after the function is borrowed only once it is initialized, and
//! every block or loop entry borrows its own bindings.
//!
//! This module checks that every use of a capturing function is one the
//! emitter can lower this way. Other uses keep the pending diagnostic.
use crate::analysis::{CheckedFunction, CheckedProgram, ClosureForward, Mode};
use crate::syntax::{CallTarget, ExprKind};
use crate::Diagnostic;
use std::collections::HashSet;

fn pending(checked: &CheckedProgram) -> Diagnostic {
    checked
        .local_capture_plans
        .iter()
        .find_map(crate::local_capture_plan::Plan::pending_diagnostic)
        .expect("capturing function has a capture plan")
}

/// Direct calls of capturing functions in one checked body, or an error for
/// a use that is not a direct call.
fn direct_targets(function: &CheckedFunction, capturing: &HashSet<usize>) -> Result<Vec<usize>, ()> {
    let mut targets = Vec::new();
    for expression in crate::error_effects::checked_expressions(function) {
        let Some(CallTarget::Function(target) | CallTarget::ForwardFunction(target)) = expression.target else {
            continue;
        };
        if !capturing.contains(&target) {
            continue;
        }
        if !matches!(expression.kind, ExprKind::Call(_, _)) || matches!(expression.target, Some(CallTarget::ForwardFunction(_))) {
            return Err(());
        }
        targets.push(target);
    }
    Ok(targets)
}

pub(crate) fn prepare(checked: &mut CheckedProgram) -> Result<(), Diagnostic> {
    let capturing = checked
        .named_capture_access
        .iter()
        .filter(|(_, accesses)| !accesses.is_empty())
        .map(|(&function, _)| function)
        .collect::<HashSet<_>>();
    if capturing.is_empty() {
        // A plan whose capturing functions are never used has nothing to lower.
        return Ok(());
    }
    for &function in &capturing {
        let target = &checked.functions[function];
        let accesses = &checked.named_capture_access[&function];
        let contiguous = accesses
            .iter()
            .enumerate()
            .all(|(position, access)| access.slot == target.parameter_count + position);
        if target.template
            || !contiguous
            || target.bindings[..target.parameter_count].iter().any(|binding| binding.rest)
            || accesses.iter().any(|access| access.mode == Mode::Consume)
        {
            return Err(pending(checked));
        }
    }
    // A function value and a call inside an arrow function need an owned or
    // borrowed environment value. Those stay pending.
    for closure in &checked.closures {
        if matches!(closure.forward, Some(ClosureForward::Function(target)) if capturing.contains(&target)) {
            return Err(pending(checked));
        }
        for body in std::iter::once(&closure.function).chain(closure.methods.iter().map(|method| &method.function)) {
            if direct_targets(body, &capturing).map_or(true, |targets| !targets.is_empty()) {
                return Err(pending(checked));
            }
        }
    }
    for (caller, function) in checked.functions.iter().enumerate() {
        if function.template {
            continue;
        }
        let Ok(targets) = direct_targets(function, &capturing) else {
            return Err(pending(checked));
        };
        for target in targets {
            let known = checked.function_source_bindings.get(&caller);
            if checked.named_capture_access[&target]
                .iter()
                .any(|access| !known.is_some_and(|slots| slots.contains_key(&access.binding.declaration_start)))
            {
                return Err(pending(checked));
            }
        }
    }
    // A scalar that the function changes is passed by mutable reference.
    for &function in &capturing {
        for access in checked.named_capture_access[&function].clone() {
            let binding = &mut checked.functions[function].bindings[access.slot];
            if binding.ty.is_copy() && access.mode == Mode::Mutate {
                binding.capture_reference = true;
            }
        }
    }
    if crate::emit::needs_exception_transport(checked) {
        crate::error_effects::defer_unreferenced_error_definitions(checked);
        crate::emit::validate_exception_program(checked)?;
    }
    Ok(())
}
