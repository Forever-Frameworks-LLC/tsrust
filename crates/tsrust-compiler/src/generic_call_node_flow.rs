//! One source node applies ordered evaluation before its destination commits.
use super::*;

pub(super) struct Transfer {
    pub value: expression_flow::Value,
    pub failures: Vec<Vec<Binding>>,
    pub normal: bool,
}
impl Transfer {
    pub(super) fn normal_edges(&self, node: &flow_graph::Node<'_>) -> Vec<usize> {
        if !self.normal { return Vec::new(); }
        if node.next.len() == 2 && let Some(truth) = self.value.known_truth() {
            vec![node.next[usize::from(!truth)]]
        } else { node.next.clone() }
    }
}

pub(super) fn apply(node: &flow_graph::Node<'_>, state: &mut Vec<Binding>, facts: &mut HashMap<(usize, usize), Option<Vec<Contract>>>, declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index) -> Transfer {
    apply_with_dispatch(node, state, facts, declarations, effects, &mut expression_flow::UnknownDispatch)
}
pub(super) fn apply_with_dispatch(node: &flow_graph::Node<'_>, state: &mut Vec<Binding>, facts: &mut HashMap<(usize, usize), Option<Vec<Contract>>>, declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index, dispatch: &mut dyn expression_flow::Dispatch) -> Transfer {
    let statement = node.statement;
        let mut failures = Vec::new();
        if let Some(StmtKind::Assign { target, .. }) = statement.map(|statement| &statement.kind)
            && !matches!(target.kind, ExprKind::Name(_))
        {
            let evaluation = expression_flow::evaluate_with_dispatch(target, state, facts, declarations, node.exceptional.is_some(), effects, dispatch);
            failures.extend(evaluation.failures);
            if !evaluation.normal { return Transfer { value: expression_flow::Value::default(), failures, normal: false }; }
        }
        let expression_value = if let Some(expression) = node.expression {
            let evaluation = expression_flow::evaluate_with_dispatch(expression, state, facts, declarations, node.exceptional.is_some(), effects, dispatch);
            failures.extend(evaluation.failures);
            if !evaluation.normal { return Transfer { value: evaluation.value, failures, normal: false }; }
            evaluation.value
        } else { expression_flow::Value::default() };
        if node.exceptional.is_some() && matches!(statement.map(|statement| &statement.kind), Some(StmtKind::Assign { operator: Some(_), .. })) {
            failures.push(state.clone());
        }
        let destination = match statement.map(|statement| &statement.kind) {
            Some(StmtKind::Let { name, value, .. }) => state
                .iter()
                .position(|binding| {
                    binding.name == name.text && binding.scope.start == value.span.end
                })
                .map(|index| (index, expression_value.clone())),
            Some(StmtKind::Assign {
                target,
                operator,
                ..
            }) => {
                if let ExprKind::Name(name) = &target.kind {
                    binding_at(state, name, target.span, effects).map(|index| {
                        (
                            index,
                            if operator.is_none() {
                                expression_value.clone()
                            } else {
                                expression_flow::Value::default()
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
                        binding_at(state, name, target.span, effects)
                            .map(|index| (index, expression_flow::Value::default()))
                    } else {
                        None
                    }
                })
            }
            _ => None,
        };
        if matches!(node.statement.map(|statement| &statement.kind), Some(StmtKind::Assign { target, .. }) if !matches!(target.kind, ExprKind::Name(_))) {
            for binding in state.iter_mut() { binding.elements = None; }
        }
        if let Some((destination, candidates)) = destination {
            state[destination].selection = candidates.selection();
            state[destination].elements = candidates.elements.clone();
            state[destination].origins = body_origins::canonical(candidates.origins);
            if state[destination].contract.is_none() {
                state[destination].targets = candidates.contracts.map_or_else(
                    || vec![Target::Unknown],
                    |contracts| contracts.into_iter().map(Target::Contract).collect(),
                );
            }
        }
    Transfer { value: expression_value, failures, normal: true }
}
