//! Iterative source flow for blocks and branches; runtime environments stay separate.
use super::*;

#[path = "generic_call_flow_graph.rs"]
mod flow_graph;
use flow_graph::graph;
#[path = "generic_call_expression_flow.rs"]
mod expression_flow;

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
    let mut direct = nodes.iter().filter_map(|node| match &node.statement?.kind {
        StmtKind::Assign { target, .. } => Some((target.span.start, target.span.end)),
        _ => None,
    }).collect::<HashSet<_>>();
    let mut probe = function.clone();
    let mut supported = true;
    crate::source_walk::function_body(&mut probe, &mut |_| Ok(()), &mut |expression| {
        match &expression.kind {
            ExprKind::Assign { target, .. } | ExprKind::CompoundAssign { target, .. }
            | ExprKind::CoalesceAssign { target, .. } | ExprKind::LogicalAssign { target, .. }
            | ExprKind::Update { target, .. } => { direct.insert((target.span.start, target.span.end)); }
            ExprKind::Conditional { .. } | ExprKind::Sequence(_) | ExprKind::Call(..)
            | ExprKind::Binary { .. } | ExprKind::Index(..) | ExprKind::Unary { .. }
            | ExprKind::TypeAssertion { .. } | ExprKind::Field(..) | ExprKind::Array(_)
            | ExprKind::Template(_) | ExprKind::ArrayOf(..) | ExprKind::SetConstructor(..)
            | ExprKind::MapConstructor(..) | ExprKind::Record(_) | ExprKind::Name(_)
            | ExprKind::String(_) | ExprKind::Integer(_) | ExprKind::Number(_)
            | ExprKind::Boolean(_) | ExprKind::Null | ExprKind::Undefined | ExprKind::EnumMember { .. } => {}
            _ => supported = false,
        }
        Ok(())
    })?;
    if !supported || writes.iter().any(|write| function.scope_span.start <= write.span.start
        && write.span.end <= function.scope_span.end && !direct.contains(&(write.span.start, write.span.end))) {
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
        let mut failures = Vec::new();
        if let Some(StmtKind::Assign { target, .. }) = statement.map(|statement| &statement.kind)
            && !matches!(target.kind, ExprKind::Name(_))
        {
            let (_, failed) = expression_flow::evaluate(target, &mut state, &mut facts, declarations, node.exceptional.is_some());
            failures.extend(failed);
        }
        let expression_contracts = if let Some(expression) = node.expression {
            let (value, failed) = expression_flow::evaluate(expression, &mut state, &mut facts, declarations, node.exceptional.is_some());
            failures.extend(failed);
            value
        } else { None };
        if node.exceptional.is_some() && matches!(statement.map(|statement| &statement.kind), Some(StmtKind::Assign { operator: Some(_), .. })) {
            failures.push(state.clone());
        }
        let destination = match statement.map(|statement| &statement.kind) {
            Some(StmtKind::Let { name, value, .. }) => state
                .iter()
                .position(|binding| {
                    binding.name == name.text && binding.scope.start == value.span.end
                })
                .map(|index| (index, expression_contracts.clone())),
            Some(StmtKind::Assign {
                target,
                operator,
                ..
            }) => {
                if let ExprKind::Name(name) = &target.kind {
                    binding_at(&state, name, target.span).map(|index| {
                        (
                            index,
                            if operator.is_none() {
                                expression_contracts.clone()
                            } else {
                                None
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
                            .map(|index| (index, None))
                    } else {
                        None
                    }
                })
            }
            _ => None,
        };
        if let Some((destination, candidates)) = destination {
            if state[destination].contract.is_none() {
                state[destination].targets = candidates.map_or_else(
                    || vec![Target::Unknown],
                    |contracts| contracts.into_iter().map(Target::Contract).collect(),
                );
            }
        }
        let successors = node.next.iter().map(|&next| (next, &state))
            .chain(node.exceptional.into_iter().flat_map(|next| failures.iter().map(move |state| (next, state))));
        for (successor, outgoing) in successors {
            if successor == 0 {
                continue;
            }
            let changed = if let Some(previous) = &mut states[successor] {
                join(previous, outgoing, declarations)
            } else {
                states[successor] = Some(outgoing.clone());
                true
            };
            if changed {
                work.push(successor);
            }
        }
    }
    Ok(Some(facts))
}
