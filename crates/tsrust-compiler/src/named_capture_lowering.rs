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

/// Whether every use of a capturing function in one checked body is a
/// direct call that recorded its hidden argument slots.
fn lowered(function: &CheckedFunction, capturing: &HashSet<usize>, checked: &CheckedProgram) -> bool {
    crate::error_effects::checked_expressions(function).into_iter().all(|expression| {
        let Some(CallTarget::Function(target) | CallTarget::ForwardFunction(target)) = expression.target else {
            return true;
        };
        if !capturing.contains(&target) {
            return true;
        }
        matches!(expression.kind, ExprKind::Call(_, _))
            && expression.target == Some(CallTarget::Function(target))
            && expression
                .capture_slots
                .as_deref()
                .is_some_and(|slots| slots.len() == checked.named_capture_access[&target].len())
    })
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
    // A function value needs an environment value. That stays pending.
    for closure in &checked.closures {
        if matches!(closure.forward, Some(ClosureForward::Function(target)) if capturing.contains(&target)) {
            return Err(pending(checked));
        }
        for body in std::iter::once(&closure.function).chain(closure.methods.iter().map(|method| &method.function)) {
            if !body.template && !lowered(body, &capturing, checked) {
                return Err(pending(checked));
            }
        }
    }
    for function in &checked.functions {
        if !function.template && !lowered(function, &capturing, checked) {
            return Err(pending(checked));
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
