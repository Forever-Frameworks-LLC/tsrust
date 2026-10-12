//! Cleanup entries preserve the suspended completion and the outer handler.
use super::*;

type CleanupKey = (usize, usize, usize, usize, Vec<(usize, Option<usize>)>);

pub(super) fn cleanup<'a>(
    body: Option<&'a [Stmt]>,
    successor: usize,
    outer: &Body<'a>,
    nodes: &mut Vec<Node<'a>>,
    pending: &mut Vec<Body<'a>>,
    cache: &mut HashMap<CleanupKey, usize>,
) -> usize {
    let Some(body) = body.filter(|body| !body.is_empty()) else { return successor; };
    let key = (body.as_ptr() as usize, successor, outer.returning, outer.throwing,
        outer.controls.iter().map(|control| (control.breaking, control.continuing)).collect());
    if let Some(&entry) = cache.get(&key) { return entry; }
    let entry = push(nodes, None, vec![successor]);
    cache.insert(key, entry);
    pending.push(Body { body, successor, parent: entry, edge: 0,
        controls: outer.controls.clone(), returning: outer.returning, throwing: outer.throwing });
    entry
}

/// Calls and checked operations can fail before a destination assignment commits.
/// Expression writes are handled by the separate conservative-effects guard.
pub(super) fn may_fail(expression: &Expr) -> bool {
    let mut failure = false;
    let mut body = vec![Stmt { kind: StmtKind::Expr(expression.clone()), span: expression.span,
        labels: vec![], control_target: None }];
    let _ = crate::source_walk::body(&mut body, &mut |_| Ok(()), &mut |value| {
        if matches!(value.kind, ExprKind::Call(..) | ExprKind::Index(..) | ExprKind::Binary { .. }
            | ExprKind::Unary { .. } | ExprKind::Update { .. }) { failure = true; }
        Ok(())
    });
    failure
}
