//! Source target sets contain callable metadata, never runtime environments.
use super::*;

#[derive(Clone, PartialEq, Eq)]
pub(super) enum Target {
    Name(String, Span),
    Contract(Contract),
    Unknown,
}
pub(super) struct Write {
    pub name: String,
    pub span: Span,
    pub completed: usize,
    pub targets: Vec<Target>,
}

pub(super) fn value(value: &Expr) -> Vec<Target> {
    let mut pending = vec![value];
    let mut targets = Vec::new();
    while let Some(value) = pending.pop() {
        if let Some((name, span)) = name(value) {
            targets.push(Target::Name(name.to_owned(), span));
            continue;
        }
        match &value.kind {
            ExprKind::Conditional {
                consequent,
                alternate,
                ..
            } => {
                pending.push(consequent);
                pending.push(alternate);
            }
            ExprKind::Sequence(values) => {
                if let Some(last) = values.last() {
                    pending.push(last);
                } else {
                    targets.push(Target::Unknown);
                }
            }
            ExprKind::TypeAssertion { annotation, .. } => {
                targets.push(
                    crate::callable_annotations::callback(annotation)
                        .map_or(Target::Unknown, |callback| {
                            Target::Contract(parameters(&callback.params, value.span))
                        }),
                );
            }
            ExprKind::Closure { params, body, .. } => {
                let forwarded = if let [statement] = body.as_slice()
                    && let StmtKind::Return(Some(call)) = &statement.kind
                    && let ExprKind::Call(callee, _) = &call.kind
                    && call.span == value.span
                    && callee.span == value.span
                {
                    name(callee)
                } else {
                    None
                };
                targets.push(forwarded.map_or_else(
                    || Target::Contract(parameters(params, value.span)),
                    |(name, span)| Target::Name(name.to_owned(), span),
                ));
            }
            _ => targets.push(Target::Unknown),
        }
    }
    targets
}

fn body_writes(body: &[Stmt], writes: &mut Vec<Write>) {
    let mut pending = body.iter().collect::<Vec<_>>();
    while let Some(statement) = pending.pop() {
        match &statement.kind {
            StmtKind::Assign {
                target,
                value: assigned,
                operator,
                ..
            } => {
                if let ExprKind::Name(name) = &target.kind {
                    writes.push(Write {
                        name: name.clone(),
                        span: target.span,
                        completed: assigned.span.end,
                        targets: if operator.is_none() {
                            value(assigned)
                        } else {
                            vec![Target::Unknown]
                        },
                    });
                }
            }
            StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => {
                pending.extend(body)
            }
            StmtKind::If {
                then_body,
                else_body,
                ..
            } => pending.extend(then_body.iter().chain(else_body)),
            StmtKind::Loop { body, update, .. } => pending.extend(body.iter().chain(update)),
            StmtKind::Try(region) => pending.extend(region.statements()),
            _ => {}
        }
    }
}

pub(super) fn writes(program: &mut Program) -> Result<Vec<Write>, Diagnostic> {
    let mut writes = Vec::new();
    for function in &mut program.functions {
        body_writes(&function.body, &mut writes);
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            match &expression.kind {
                ExprKind::Closure { body, .. } => body_writes(body, &mut writes),
                ExprKind::Assign {
                    target,
                    value: assigned,
                    ..
                } => {
                    if let ExprKind::Name(name) = &target.kind {
                        writes.push(Write {
                            name: name.clone(),
                            span: target.span,
                            completed: assigned.span.end,
                            targets: value(assigned),
                        });
                    }
                }
                ExprKind::CompoundAssign { target, .. }
                | ExprKind::CoalesceAssign { target, .. }
                | ExprKind::LogicalAssign { target, .. }
                | ExprKind::Update { target, .. } => {
                    if let ExprKind::Name(name) = &target.kind {
                        writes.push(Write {
                            name: name.clone(),
                            span: target.span,
                            completed: expression.span.end,
                            targets: vec![Target::Unknown],
                        });
                    }
                }
                _ => {}
            }
            Ok(())
        })?;
    }
    Ok(writes)
}
