//! Source control edges are built iteratively, before generic specialization.
use super::*;
#[path = "generic_call_exception_edges.rs"]
mod exceptions;

pub(super) struct Node<'a> {
    pub statement: Option<&'a Stmt>,
    pub expression: Option<&'a Expr>,
    pub next: Vec<usize>,
    pub exceptional: Option<usize>,
}

#[derive(Clone)]
struct Control<'a> {
    labels: &'a [Token],
    breaking: usize,
    continuing: Option<usize>,
}

struct Body<'a> {
    body: &'a [Stmt],
    successor: usize,
    parent: usize,
    edge: usize,
    controls: Vec<Control<'a>>,
    returning: usize,
    throwing: usize,
}

fn push<'a>(nodes: &mut Vec<Node<'a>>, expression: Option<&'a Expr>, next: Vec<usize>) -> usize {
    let index = nodes.len();
    nodes.push(Node {
        statement: None,
        expression,
        next,
        exceptional: None,
    });
    index
}

fn condition_edges(condition: Option<&Expr>, body: usize, exit: usize) -> Vec<usize> {
    match condition.map(|expression| &expression.kind) {
        Some(ExprKind::Boolean(false)) => vec![exit],
        None | Some(ExprKind::Boolean(true)) => vec![body],
        _ => vec![body, exit],
    }
}

/// A standalone named update cannot reorder another expression's call.
pub(super) fn standalone_update(mut expression: &Expr) -> Option<(&Expr, Span)> {
    while let ExprKind::Sequence(values) = &expression.kind {
        if values.len() != 1 {
            return None;
        }
        expression = &values[0];
    }
    if let ExprKind::Update { target, .. } = &expression.kind
        && matches!(target.kind, ExprKind::Name(_))
    {
        Some((target, expression.span))
    } else {
        None
    }
}

/// None leaves unsupported effect regions on the conservative validation path.
pub(super) fn graph(body: &[Stmt]) -> Option<Vec<Node<'_>>> {
    let mut nodes = vec![Node {
        statement: None,
        expression: None,
        next: vec![0],
        exceptional: None,
    }];
    let mut pending = vec![Body {
        body,
        successor: 0,
        parent: 0,
        edge: 0,
        controls: Vec::new(),
        returning: 0,
        throwing: 0,
    }];
    let mut cleanups = HashMap::new();
    while let Some(task) = pending.pop() {
        let mut entry = task.successor;
        for statement in task.body.iter().rev() {
            let index = nodes.len();
            nodes.push(Node {
                statement: Some(statement),
                expression: None,
                next: vec![entry],
                exceptional: None,
            });
            match &statement.kind {
                StmtKind::Block(body) => {
                    let mut controls = task.controls.clone();
                    if !statement.labels.is_empty() {
                        controls.push(Control {
                            labels: &statement.labels,
                            breaking: entry,
                            continuing: None,
                        });
                    }
                    pending.push(Body {
                        body,
                        successor: entry,
                        parent: index,
                        edge: 0,
                        controls,
                        returning: task.returning,
                        throwing: task.throwing,
                    });
                }
                StmtKind::If {
                    condition,
                    then_body,
                    else_body,
                } => {
                    nodes[index].expression = Some(condition);
                    nodes[index].next.push(entry);
                    pending.push(Body {
                        body: then_body,
                        successor: entry,
                        parent: index,
                        edge: 0,
                        controls: task.controls.clone(),
                        returning: task.returning,
                        throwing: task.throwing,
                    });
                    pending.push(Body {
                        body: else_body,
                        successor: entry,
                        parent: index,
                        edge: 1,
                        controls: task.controls.clone(),
                        returning: task.returning,
                        throwing: task.throwing,
                    });
                }
                StmtKind::While { condition, body } => {
                    let body_gate = push(&mut nodes, None, vec![index]);
                    nodes[index].expression = Some(condition);
                    nodes[index].next = condition_edges(Some(condition), body_gate, entry);
                    let mut controls = task.controls.clone();
                    controls.push(Control {
                        labels: &statement.labels,
                        breaking: entry,
                        continuing: Some(index),
                    });
                    pending.push(Body {
                        body,
                        successor: index,
                        parent: body_gate,
                        edge: 0,
                        controls,
                        returning: task.returning,
                        throwing: task.throwing,
                    });
                }
                StmtKind::Loop {
                    condition,
                    body,
                    update,
                    post_test,
                    ..
                } => {
                    let test = push(&mut nodes, condition.as_ref(), Vec::new());
                    if task.throwing != 0 && condition.as_ref().is_some_and(exceptions::may_fail) {
                        nodes[test].exceptional = Some(task.throwing);
                    }
                    let body_gate = push(&mut nodes, None, vec![test]);
                    let update_gate = push(&mut nodes, None, vec![test]);
                    nodes[test].next = condition_edges(condition.as_ref(), body_gate, entry);
                    nodes[index].next = vec![if *post_test { body_gate } else { test }];
                    let mut controls = task.controls.clone();
                    controls.push(Control {
                        labels: &statement.labels,
                        breaking: entry,
                        continuing: Some(update_gate),
                    });
                    pending.push(Body {
                        body: update,
                        successor: test,
                        parent: update_gate,
                        edge: 0,
                        controls: controls.clone(),
                        returning: task.returning,
                        throwing: task.throwing,
                    });
                    pending.push(Body {
                        body,
                        successor: update_gate,
                        parent: body_gate,
                        edge: 0,
                        controls,
                        returning: task.returning,
                        throwing: task.throwing,
                    });
                }
                StmtKind::ForOf { iterable, body, .. } => {
                    // Evaluate the iterable once; iterator advancement may either
                    // enter the body or finish on each subsequent iteration.
                    let advance = push(&mut nodes, None, Vec::new());
                    let body_gate = push(&mut nodes, None, vec![advance]);
                    nodes[index].expression = Some(iterable);
                    nodes[index].next = vec![advance];
                    let count = if let ExprKind::Array(values) = &iterable.kind {
                        let (count, exact) = cardinality(values);
                        exact.then_some(count)
                    } else {
                        None
                    };
                    let continuing = match count {
                        Some(0) => {
                            nodes[advance].next = vec![entry];
                            advance
                        }
                        Some(1) => {
                            nodes[advance].next = vec![body_gate];
                            entry
                        }
                        _ => {
                            nodes[advance].next = vec![body_gate, entry];
                            advance
                        }
                    };
                    let mut controls = task.controls.clone();
                    controls.push(Control {
                        labels: &statement.labels,
                        breaking: entry,
                        continuing: Some(continuing),
                    });
                    pending.push(Body {
                        body,
                        successor: continuing,
                        parent: body_gate,
                        edge: 0,
                        controls,
                        returning: task.returning,
                        throwing: task.throwing,
                    });
                }
                StmtKind::Try(region) => {
                    let normal = exceptions::cleanup(region.finally.as_deref(), entry, &task, &mut nodes, &mut pending, &mut cleanups);
                    let returning = exceptions::cleanup(region.finally.as_deref(), task.returning, &task, &mut nodes, &mut pending, &mut cleanups);
                    let throwing = exceptions::cleanup(region.finally.as_deref(), task.throwing, &task, &mut nodes, &mut pending, &mut cleanups);
                    let mut controls = task.controls.clone();
                    for control in &mut controls {
                        control.breaking = exceptions::cleanup(region.finally.as_deref(), control.breaking, &task, &mut nodes, &mut pending, &mut cleanups);
                        control.continuing = control.continuing.map(|target| exceptions::cleanup(region.finally.as_deref(), target, &task, &mut nodes, &mut pending, &mut cleanups));
                    }
                    let handler = if let Some(catch) = &region.catch {
                        let gate = push(&mut nodes, None, vec![normal]);
                        pending.push(Body { body: &catch.body, successor: normal, parent: gate, edge: 0, controls: controls.clone(), returning, throwing });
                        gate
                    } else { throwing };
                    pending.push(Body { body: &region.body, successor: normal, parent: index, edge: 0, controls, returning, throwing: handler });
                }
                StmtKind::Break | StmtKind::Continue => {
                    let target = if let Some(label) = statement.labels.first() {
                        task.controls.iter().rev().find(|control| {
                            control.labels.iter().any(|name| name.text == label.text)
                        })
                    } else {
                        task.controls
                            .iter()
                            .rev()
                            .find(|control| control.continuing.is_some())
                    }?;
                    nodes[index].next = vec![if matches!(statement.kind, StmtKind::Continue) {
                        target.continuing?
                    } else {
                        target.breaking
                    }];
                }
                StmtKind::Return(value) => {
                    nodes[index].expression = value.as_ref();
                    nodes[index].next = vec![task.returning];
                }
                StmtKind::Throw(value) => {
                    nodes[index].expression = Some(value);
                    nodes[index].next = vec![task.throwing];
                }
                StmtKind::Let { value, .. }
                | StmtKind::Assign { value, .. }
                | StmtKind::Expr(value) => nodes[index].expression = Some(value),
                _ => return None,
            }
            let assignment_failure = match &statement.kind {
                StmtKind::Assign { target, operator, .. } => operator.is_some() || exceptions::may_fail(target),
                _ => false,
            };
            if task.throwing != 0 && (assignment_failure || nodes[index].expression.is_some_and(exceptions::may_fail)) {
                nodes[index].exceptional = Some(task.throwing);
            }
            entry = index;
        }
        nodes[task.parent].next[task.edge] = entry;
    }
    Some(nodes)
}
