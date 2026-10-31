//! Copying array methods and `String(text)` whose receiver is a dead local can
//! move it instead.
//!
//! `const tail = rows.slice(1); return tail.concat();` copies `tail` although
//! nothing reads it afterwards. A call qualifies only when its receiver is a
//! `const` owned array (or, for `String`, a string local or a string field of a
//! record local) that the function references exactly once, from a
//! later statement of the same statement list (so the same loop iteration), with
//! no `match` body in between, and no closure captures it (a local function
//! anywhere in the function disables the move). Anything else keeps the copy.
use std::collections::{HashMap, HashSet};
use crate::analysis::{CheckedClosure, CheckedFunction};
use crate::fixed_storage::{visit_expression, visit_function_expressions, visit_statements};
use crate::syntax::{CallTarget, Expr, ExprKind, Stmt, StmtKind, Type};

/// Bindings of `fun` that a closure captures, or `None` when the function has a
/// local function or an unknown closure, whose effects are not tracked here.
pub(super) fn captured(fun: &CheckedFunction, closures: &[CheckedClosure]) -> Option<HashSet<usize>> {
    let mut nested = false;
    visit_statements(&fun.body, &mut |stmt| nested |= matches!(stmt.kind, StmtKind::LocalFunction(_)));
    let mut captured = HashSet::new();
    visit_function_expressions(fun, &mut |expr| {
        nested |= matches!(expr.kind, ExprKind::Selected { .. });
        if let ExprKind::Closure { id, .. } = expr.kind {
            match closures.get(id) {
                Some(closure) => captured.extend(closure.captures.iter().map(|capture| capture.outer)),
                None => nested = true,
            }
        }
    });
    (!nested).then_some(captured)
}

/// Whether `expr` reads the binding `id` anywhere.
pub(super) fn mentions(expr: &Expr, id: usize) -> bool {
    let mut found = false;
    visit_expression(expr, &mut |value| found |= matches!(value.kind, ExprKind::Name(_)) && value.binding == Some(id));
    found
}

/// Spans of `concat()`, `toReversed()`, `toSorted()`, `slice()` and `String()` calls that may consume their receiver.
pub(super) fn eligible(fun: &CheckedFunction, closures: &[CheckedClosure]) -> HashSet<(usize, usize)> {
    let mut result = HashSet::new();
    let mut nested = false;
    visit_statements(&fun.body, &mut |stmt| nested |= matches!(stmt.kind, StmtKind::LocalFunction(_)));
    let mut references: HashMap<usize, usize> = HashMap::new();
    let mut captured = HashSet::new();
    visit_function_expressions(fun, &mut |expr| {
        nested |= matches!(expr.kind, ExprKind::Selected { .. });
        // A closure's body is not visited here; its captures name the outer bindings it uses.
        if let ExprKind::Closure { id, .. } = expr.kind {
            match closures.get(id) {
                Some(closure) => captured.extend(closure.captures.iter().map(|capture| capture.outer)),
                None => nested = true,
            }
        }
        if let (ExprKind::Name(_), Some(id)) = (&expr.kind, expr.binding) {
            *references.entry(id).or_default() += 1;
        }
    });
    references.retain(|id, _| !captured.contains(id));
    if nested {
        return result;
    }
    let mut lists: Vec<&[Stmt]> = vec![&fun.body];
    while let Some(list) = lists.pop() {
        for (index, stmt) in list.iter().enumerate() {
            match &stmt.kind {
                StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => lists.push(body),
                StmtKind::Loop { body, update, .. } => { lists.push(body); lists.push(update); }
                StmtKind::If { then_body, else_body, .. } => { lists.push(then_body); lists.push(else_body); }
                StmtKind::Try(region) => {
                    lists.push(&region.body);
                    if let Some(catch) = &region.catch { lists.push(&catch.body); }
                    if let Some(finally) = &region.finally { lists.push(finally); }
                }
                StmtKind::Let { rebindable: false, binding: Some(id), .. } if movable(fun, *id) && references.get(id) == Some(&1) => {
                    if let Some(span) = list[index + 1..].iter().find_map(|later| single_use(later, *id)) {
                        result.extend(span);
                    }
                }
                _ => {}
            }
        }
    }
    result
}

fn movable(fun: &CheckedFunction, id: usize) -> bool {
    let binding = &fun.bindings[id];
    matches!(binding.ty, Type::Array(_) | Type::String | Type::Record(_)) && !binding.parameter && binding.borrowed_element.is_none()
        && binding.borrowed_place.is_none() && binding.view_roots.is_empty() && !binding.capture_reference
}

/// `Some(Some(span))` when `stmt` holds the binding's one reference as an
/// eligible receiver; `Some(None)` when it holds it any other way; `None` when
/// the reference is not in this statement.
fn single_use(stmt: &Stmt, id: usize) -> Option<Option<(usize, usize)>> {
    let expressions: Vec<&Expr> = match &stmt.kind {
        StmtKind::Let { value, .. } | StmtKind::Expr(value) | StmtKind::Throw(value) | StmtKind::Return(Some(value)) => vec![value],
        StmtKind::Assign { target, value, .. } => vec![target, value],
        _ => {
            // Control statements hide later loop iterations; a reference there disqualifies.
            let mut found = false;
            visit_statements(std::slice::from_ref(stmt), &mut |inner| found |= mentions_statement(inner, id));
            return found.then_some(None);
        }
    };
    let (mut found, mut matched, mut call) = (false, false, None);
    for expr in expressions {
        visit_expression(expr, &mut |value| {
            found |= matches!(value.kind, ExprKind::Name(_)) && value.binding == Some(id);
            matched |= matches!(value.kind, ExprKind::Match { .. });
            if let ExprKind::Call(callee, args) = &value.kind
                && let ExprKind::Field(base, _) = &callee.kind
                && matches!(base.kind, ExprKind::Name(_)) && base.binding == Some(id)
                && (matches!(value.target, Some(CallTarget::ArrayConcat | CallTarget::ArrayToReversed)) && args.is_empty() || matches!(value.target, Some(CallTarget::ArrayToSorted | CallTarget::ArraySlice)))
            {
                call = Some((value.span.start, value.span.end));
            }
            if value.target == Some(CallTarget::StringConversion)
                && let ExprKind::Call(_, args) = &value.kind
                && let [argument] = args.as_slice()
                && argument.ty == Some(Type::String)
                && (matches!(argument.kind, ExprKind::Name(_)) && argument.binding == Some(id)
                    // `String(record.field)` moves the field out of a dead record local.
                    || matches!(&argument.kind, ExprKind::Field(base, _) if matches!(base.kind, ExprKind::Name(_)) && base.binding == Some(id) && matches!(base.ty, Some(Type::Record(_)))))
            {
                call = Some((value.span.start, value.span.end));
            }
        });
    }
    found.then(|| call.filter(|_| !matched))
}

fn mentions_statement(stmt: &Stmt, id: usize) -> bool {
    let mut found = false;
    let mut look = |expr: &Expr| visit_expression(expr, &mut |value| found |= matches!(value.kind, ExprKind::Name(_)) && value.binding == Some(id));
    match &stmt.kind {
        StmtKind::Let { value, .. } | StmtKind::Expr(value) | StmtKind::Throw(value) | StmtKind::Return(Some(value)) => look(value),
        StmtKind::Assign { target, value, .. } => { look(target); look(value); }
        StmtKind::If { condition, .. } | StmtKind::While { condition, .. } => look(condition),
        StmtKind::ForOf { iterable, .. } => look(iterable),
        StmtKind::Loop { condition: Some(condition), .. } => look(condition),
        _ => {}
    }
    found
}
