//! Iterative source flow for blocks and branches; runtime environments stay separate.
use super::*;

#[path = "generic_call_flow_graph.rs"]
mod flow_graph;
use flow_graph::graph;

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
        let joined = resolve_all(candidates, &[], declarations);
        let targets = joined.map_or_else(
            || vec![Target::Unknown],
            |contracts| contracts.into_iter().map(Target::Contract).collect(),
        );
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
) -> Result<Option<HashMap<(usize, usize), Option<Vec<Contract>>>>, Diagnostic> {
    let Some(nodes) = graph(&function.body) else {
        return Ok(None);
    };
    let updates = nodes
        .iter()
        .filter_map(|node| {
            let statement = node.statement?;
            if let StmtKind::Expr(expression) = &statement.kind {
                flow_graph::standalone_update(expression)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let update_spans = updates
        .iter()
        .map(|(_, span)| (span.start, span.end))
        .collect::<HashSet<_>>();
    let mut direct = nodes
        .iter()
        .filter_map(|node| match &node.statement?.kind {
            StmtKind::Assign { target, .. } => Some((target.span.start, target.span.end)),
            _ => None,
        })
        .collect::<HashSet<_>>();
    direct.extend(
        updates
            .iter()
            .map(|(target, _)| (target.span.start, target.span.end)),
    );
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
        ) || (matches!(expression.kind, ExprKind::Update { .. })
            && !update_spans.contains(&(expression.span.start, expression.span.end)))
        {
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
    let mut facts: HashMap<(usize, usize), Option<Vec<Contract>>> = HashMap::new();
    while let Some(index) = work.pop() {
        let node = &nodes[index];
        let statement = node.statement;
        let mut state = states[index].as_ref().unwrap().clone();
        let expression = node.expression;
        if let Some(expression) = expression {
            let mut body = vec![Stmt {
                kind: StmtKind::Expr(expression.clone()),
                span: expression.span,
                labels: vec![],
                control_target: None,
            }];
            crate::source_walk::body(&mut body, &mut |_| Ok(()), &mut |expression| {
                if let ExprKind::Call(callee, _) = &expression.kind {
                    let contract = resolve_all(targets::value(callee), &state, declarations);
                    let key = (expression.span.start, expression.span.end);
                    // Repeated visits join path facts rather than replacing an
                    // earlier path with the last branch processed.
                    if let Some(previous) = facts.get(&key) {
                        let joined = match (previous, &contract) {
                            (Some(previous), Some(current)) => resolve_all(
                                previous
                                    .iter()
                                    .chain(current)
                                    .cloned()
                                    .map(Target::Contract)
                                    .collect(),
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
        let destination = match statement.map(|statement| &statement.kind) {
            Some(StmtKind::Let { name, value, .. }) => state
                .iter()
                .position(|binding| {
                    binding.name == name.text && binding.scope.start == value.span.end
                })
                .map(|index| (index, targets::value(value))),
            Some(StmtKind::Assign {
                target,
                value,
                operator,
                ..
            }) => {
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
            Some(StmtKind::Expr(expression)) => {
                flow_graph::standalone_update(expression).and_then(|(target, _)| {
                    if let ExprKind::Name(name) = &target.kind {
                        binding_at(&state, name, target.span)
                            .map(|index| (index, vec![Target::Unknown]))
                    } else {
                        None
                    }
                })
            }
            _ => None,
        };
        if let Some((destination, candidates)) = destination {
            if state[destination].contract.is_none() {
                let snapshot = resolve_all(candidates, &state, declarations);
                state[destination].targets = snapshot.map_or_else(
                    || vec![Target::Unknown],
                    |contracts| contracts.into_iter().map(Target::Contract).collect(),
                );
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
