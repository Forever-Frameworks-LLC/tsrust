//! Immutable aggregate-write eligibility is computed once per indexed body.
use super::*;
pub(super) fn writes(body: &[Stmt], parameters: &[Parameter]) -> Result<bool, Diagnostic> {
    let mut pending = vec![body];
    while let Some(body) = pending.pop() {
        for statement in body {
            match &statement.kind {
                StmtKind::Assign { target, .. } if !matches!(target.kind, ExprKind::Name(_)) => return Ok(true),
                StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => pending.push(body),
                StmtKind::Loop { body, update, .. } => { pending.push(body); pending.push(update); }
                StmtKind::If { then_body, else_body, .. } => { pending.push(then_body); pending.push(else_body); }
                StmtKind::Try(region) => { pending.push(&region.body); if let Some(handler) = &region.catch { pending.push(&handler.body); } if let Some(body) = &region.finally { pending.push(body); } }
                StmtKind::LocalFunction(function) => pending.push(&function.body),
                _ => {}
            }
        }
    }
    let mut probe = body.to_vec();
    for parameter in parameters {
        if let ParameterInitializer::Default { value, .. } = &parameter.initializer {
            probe.push(Stmt { kind: StmtKind::Expr((**value).clone()), span: value.span, labels: Vec::new(), control_target: None });
        }
    }
    let mut writes = false;
    crate::source_walk::body(&mut probe, &mut |_| Ok(()), &mut |expression| {
        let target = match &expression.kind {
            ExprKind::Assign { target, .. } | ExprKind::CompoundAssign { target, .. }
            | ExprKind::LogicalAssign { target, .. } | ExprKind::CoalesceAssign { target, .. }
            | ExprKind::Update { target, .. } => Some(target),
            _ => None,
        };
        writes |= target.is_some_and(|target| !matches!(target.kind, ExprKind::Name(_)));
        Ok(())
    })?;
    Ok(writes)
}
