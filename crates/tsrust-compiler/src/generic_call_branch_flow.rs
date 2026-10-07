//! Iterative source flow for blocks and branches; runtime environments stay separate.
use super::*;

#[path = "generic_call_flow_graph.rs"]
mod flow_graph;
use flow_graph::graph;
#[path = "generic_call_expression_flow.rs"]
mod expression_flow;
#[path = "generic_call_node_flow.rs"]
mod node_flow;
#[path = "generic_call_summary_flow.rs"]
mod summary_flow;
#[path = "generic_call_invocation_frame.rs"]
mod invocation_frame;
#[path = "generic_call_invocation_worklist.rs"]
mod invocation_worklist;

pub(super) fn binding_at(bindings: &[Binding], name: &str, span: Span, effects: &effect_bodies::Index) -> Option<usize> {
    if let Some(identity) = effects.binding(name, span) {
        return bindings.iter().position(|binding| binding.identity == Some(identity));
    }
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
        let selection = Selection {
            truth: (current.selection.truth == incoming.selection.truth).then_some(current.selection.truth).flatten(),
            nullish: (current.selection.nullish == incoming.selection.nullish).then_some(current.selection.nullish).flatten(),
            undefined: (current.selection.undefined == incoming.selection.undefined).then_some(current.selection.undefined).flatten(),
        };
        if selection != current.selection { current.selection = selection; changed = true; }
        let origins = body_origins::canonical(current.origins.iter().chain(&incoming.origins).cloned().collect());
        if origins != current.origins { current.origins = origins; changed = true; }
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
    effects: &effect_bodies::Index,
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
        if binding.scope.start > function.scope_span.start {
            binding.origins = vec![body_origins::Origin::Unknown];
            binding.selection = Selection::default();
            if binding.contract.is_none() { binding.targets = vec![Target::Unknown]; }
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
        let mut state = states[index].as_ref().unwrap().clone();
        let transfer = node_flow::apply(node, &mut state, &mut facts, declarations, effects);
        let normal_edges = transfer.normal_edges(node);
        let failures = transfer.failures;
        let successors = normal_edges.iter().map(|&next| (next, &state))
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

#[cfg(test)]
mod identity_tests {
    use super::*;
    #[test]
    fn callee_writes_select_captured_identity_without_aliasing_local_shadows() {
        let source = crate::format("function outer<T>(){function change(){action=two;}let action=one;function shadow(){let action=two;action=one;}}function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let effects = effect_bodies::Index::new(&program).unwrap();
        let root = program.functions.iter().find(|function| function.name.text == "outer").unwrap();
        let mut bindings = Vec::new();
        collect(&root.body, &mut bindings, &effects);
        for (suffix, expected) in [("#change", Some(0)), ("#shadow", None)] {
            let callee = program.functions.iter().find(|function| function.name.text.ends_with(suffix)).unwrap();
            let target = callee.body.iter().find_map(|statement| match &statement.kind {
                StmtKind::Assign { target, .. } => Some(target), _ => None,
            }).unwrap();
            assert_eq!(binding_at(&bindings, "action", target.span, &effects), expected, "{suffix}");
        }
    }
}

#[cfg(test)]
mod provenance_join_tests {
    use super::*;
    #[test]
    fn body_join_is_finite_and_keeps_unknown_without_changing_signature() {
        let source=crate::format("function main(){let action:Function<int>=():int=>7;}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let mut current=Vec::new();collect(&program.functions[0].body,&mut current,&effects);
        let mut incoming=current.clone();
        incoming[0].origins=vec![body_origins::Origin::Unknown];
        let contract=current[0].contract.clone();
        assert!(join(&mut current,&incoming,&HashMap::new()));
        assert_eq!(current[0].origins.len(),2);
        assert!(current[0].origins.contains(&body_origins::Origin::Unknown));
        assert!(!join(&mut current,&incoming,&HashMap::new()));
        assert!(current[0].contract==contract);
    }
}
