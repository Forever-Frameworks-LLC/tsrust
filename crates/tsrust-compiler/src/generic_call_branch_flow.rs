//! Iterative source flow for blocks and branches; runtime environments stay separate.
use super::*;

struct Node<'a> {
    statement: Option<&'a Stmt>,
    next: Vec<usize>,
}

fn graph(body: &[Stmt]) -> Option<Vec<Node<'_>>> {
    let mut nodes = vec![Node {
        statement: None,
        next: vec![0],
    }];
    let mut pending = vec![(body, 0usize, 0usize, 0usize)];
    while let Some((body, successor, parent, edge)) = pending.pop() {
        let mut entry = successor;
        for statement in body.iter().rev() {
            let index = nodes.len();
            let mut next = vec![entry];
            match &statement.kind {
                StmtKind::Block(body) => pending.push((body.as_slice(), entry, index, 0)),
                StmtKind::If {
                    then_body,
                    else_body,
                    ..
                } => {
                    next.push(entry);
                    pending.push((then_body.as_slice(), entry, index, 0));
                    pending.push((else_body.as_slice(), entry, index, 1));
                }
                StmtKind::Return(_) | StmtKind::Throw(_) => next.clear(),
                StmtKind::Let { .. } | StmtKind::Assign { .. } | StmtKind::Expr(_) => {}
                _ => return None,
            }
            nodes.push(Node {
                statement: Some(statement),
                next,
            });
            entry = index;
        }
        nodes[parent].next[edge] = entry;
    }
    Some(nodes)
}

fn binding_at(bindings: &[Binding], name: &str, span: Span) -> Option<usize> {
    bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| {
            binding.name == name
                && binding.scope.start <= span.start
                && span.end <= binding.scope.end
        })
        .min_by_key(|(_, binding)| binding.scope.end - binding.scope.start)
        .map(|(index, _)| index)
}

fn join(
    current: &mut [Binding],
    incoming: &[Binding],
    declarations: &HashMap<String, Contract>,
) -> bool {
    let mut changed = false;
    for (current, incoming) in current.iter_mut().zip(incoming) {
        let mut candidates = current.targets.clone();
        candidates.extend(incoming.targets.clone());
        let joined =
            resolve(candidates, &[], declarations).map_or(Target::Unknown, Target::Contract);
        let targets = vec![joined];
        if targets != current.targets {
            current.targets = targets;
            changed = true;
        }
    }
    changed
}

/// A missing map means this function needs the conservative effects path.
pub(super) fn analyze(
    function: &Function,
    initial: &[Binding],
    writes: &[targets::Write],
    declarations: &HashMap<String, Contract>,
) -> Result<Option<HashMap<(usize, usize), Option<Contract>>>, Diagnostic> {
    let Some(nodes) = graph(&function.body) else {
        return Ok(None);
    };
    let direct = nodes
        .iter()
        .filter_map(|node| match &node.statement?.kind {
            StmtKind::Assign { target, .. } => Some((target.span.start, target.span.end)),
            _ => None,
        })
        .collect::<HashSet<_>>();
    if writes.iter().any(|write| {
        function.scope_span.start <= write.span.start
            && write.span.end <= function.scope_span.end
            && !direct.contains(&(write.span.start, write.span.end))
    }) {
        return Ok(None);
    }
    let mut probe = function.clone();
    let mut supported = true;
    crate::source_walk::function_body(&mut probe, &mut |_| Ok(()), &mut |expression| {
        if matches!(
            expression.kind,
            ExprKind::Closure { .. }
                | ExprKind::Assign { .. }
                | ExprKind::CompoundAssign { .. }
                | ExprKind::CoalesceAssign { .. }
                | ExprKind::LogicalAssign { .. }
                | ExprKind::Update { .. }
        ) {
            supported = false;
        }
        Ok(())
    })?;
    if !supported {
        return Ok(None);
    }
    let mut seed = initial.to_vec();
    // Locals become known only after their initializer executes. Parameters have
    // stable annotated contracts; lexical ranges still govern name lookup.
    for binding in &mut seed {
        if binding.scope.start > function.scope_span.start && binding.contract.is_none() {
            binding.targets = vec![Target::Unknown];
        }
    }
    let mut states = vec![None; nodes.len()];
    let entry = nodes[0].next[0];
    if entry == 0 {
        return Ok(Some(HashMap::new()));
    }
    states[entry] = Some(seed);
    let mut work = vec![entry];
    let mut facts: HashMap<(usize, usize), Option<Contract>> = HashMap::new();
    while let Some(index) = work.pop() {
        let node = &nodes[index];
        let Some(statement) = node.statement else {
            continue;
        };
        let mut state = states[index].as_ref().unwrap().clone();
        let expression = match &statement.kind {
            StmtKind::Let { value, .. }
            | StmtKind::Assign { value, .. }
            | StmtKind::Expr(value)
            | StmtKind::Throw(value) => Some(value),
            StmtKind::Return(value) => value.as_ref(),
            StmtKind::If { condition, .. } => Some(condition),
            _ => None,
        };
        if let Some(expression) = expression {
            let mut body = vec![Stmt {
                kind: StmtKind::Expr(expression.clone()),
                span: expression.span,
                labels: vec![],
                control_target: None,
            }];
            crate::source_walk::body(&mut body, &mut |_| Ok(()), &mut |expression| {
                if let ExprKind::Call(callee, _) = &expression.kind {
                    let contract = resolve(targets::value(callee), &state, declarations);
                    let key = (expression.span.start, expression.span.end);
                    // Repeated visits join path facts rather than replacing an
                    // earlier path with the last branch processed.
                    if let Some(previous) = facts.get(&key) {
                        let joined = match (previous, &contract) {
                            (Some(previous), Some(current)) => resolve(
                                vec![
                                    Target::Contract(previous.clone()),
                                    Target::Contract(current.clone()),
                                ],
                                &[],
                                declarations,
                            ),
                            _ => None,
                        };
                        facts.insert(key, joined);
                    } else {
                        facts.insert(key, contract);
                    }
                }
                Ok(())
            })?;
        }
        let destination = match &statement.kind {
            StmtKind::Let { name, value, .. } => state
                .iter()
                .position(|binding| {
                    binding.name == name.text && binding.scope.start == value.span.end
                })
                .map(|index| (index, targets::value(value))),
            StmtKind::Assign {
                target,
                value,
                operator,
                ..
            } => {
                if let ExprKind::Name(name) = &target.kind {
                    binding_at(&state, name, target.span).map(|index| {
                        (
                            index,
                            if operator.is_none() {
                                targets::value(value)
                            } else {
                                vec![Target::Unknown]
                            },
                        )
                    })
                } else {
                    None
                }
            }
            _ => None,
        };
        if let Some((destination, candidates)) = destination {
            if state[destination].contract.is_none() {
                let snapshot = resolve(candidates, &state, declarations)
                    .map_or(Target::Unknown, Target::Contract);
                state[destination].targets = vec![snapshot];
            }
        }
        for &successor in &node.next {
            if successor == 0 {
                continue;
            }
            let changed = if let Some(previous) = &mut states[successor] {
                join(previous, &state, declarations)
            } else {
                states[successor] = Some(state.clone());
                true
            };
            if changed {
                work.push(successor);
            }
        }
    }
    Ok(Some(facts))
}
