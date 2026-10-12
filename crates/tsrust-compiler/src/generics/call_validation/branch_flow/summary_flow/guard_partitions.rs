//! Bounded truth-qualified source states retain repeated guard correlation.
use super::*;
type Pending = Option<expression_flow::Value>;
const MAX_PARTITIONS: usize = 32;
#[derive(Clone, Default)]
pub(super) struct Partitions {
    pub entries: Vec<(FlowState, Pending)>,
    widened: bool,
}
fn key(state: &FlowState) -> Vec<(crate::local_capture_plan::BindingId, bool)> {
    let mut key = state.iter().filter_map(|binding| Some((binding.identity?, binding.selection.truth?))).collect::<Vec<_>>();
    key.sort_unstable();
    key
}
fn merge(previous: &mut (FlowState, Pending), incoming: &(FlowState, Pending), declarations: &HashMap<String, Contract>) -> Option<bool> {
    let mut changed = previous.0.join(&incoming.0, declarations)?;
    let value = match (&previous.1, &incoming.1) {
        (Some(left), Some(right)) => Some(expression_flow::merge_value(left.clone(), right.clone(), declarations)),
        (None, None) => None,
        (Some(value), None) | (None, Some(value)) => Some(expression_flow::merge_value(value.clone(), expression_flow::Value::undefined(), declarations)),
    };
    if previous.1 != value { previous.1 = value; changed = true; }
    Some(changed)
}
impl Partitions {
    pub fn insert(&mut self, state: FlowState, pending: Pending, declarations: &HashMap<String, Contract>) -> Option<bool> {
        let incoming = (state, pending);
        if self.widened { return merge(&mut self.entries[0], &incoming, declarations); }
        let incoming_key = key(&incoming.0);
        if let Some(previous) = self.entries.iter_mut().find(|(state, _)| key(state) == incoming_key) {
            return merge(previous, &incoming, declarations);
        }
        self.entries.push(incoming);
        if self.entries.len() > MAX_PARTITIONS {
            let joined = self.joined(declarations)?.unwrap();
            self.entries = vec![joined];
            self.widened = true;
        }
        Some(true)
    }
    pub fn joined(&self, declarations: &HashMap<String, Contract>) -> Option<Option<(FlowState, Pending)>> {
        let mut entries = self.entries.iter();
        let Some(first) = entries.next() else { return Some(None); };
        let mut joined = first.clone();
        for incoming in entries { merge(&mut joined, incoming, declarations)?; }
        Some(Some(joined))
    }
}
/// Refine only a directly observed binding, after condition evaluation effects.
/// Writes already update or invalidate its truth before the next partition key.
pub(super) fn refine(node: &flow_graph::Node<'_>, successor: usize, state: &mut FlowState, effects: &effect_bodies::Index) {
    if node.next.len() != 2 || node.next[0] == node.next[1] { return; }
    let Some(expression) = node.expression else { return; };
    let ExprKind::Name(name) = &expression.kind else { return; };
    if let Some(index) = binding_at(state, name, expression.span, effects) {
        // Written guards require versioned relations; keep their branch proof
        // conservative until invocation frames preserve those versions.
        if state[index].identity.is_some_and(|identity| effects.may_write(identity)) { return; }
        state[index].selection.truth = Some(successor == node.next[0]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state(bits: usize) -> FlowState {
        (0..6).map(|slot| Binding {
            alternatives: None,
            identity: Some(crate::local_capture_plan::BindingId { owner: 0, declaration_start: slot }),
            name: format!("flag{slot}"), scope: Span::new(0, 100),
            targets: vec![Target::Unknown], origins: vec![body_origins::Origin::Unknown],
            selection: Selection { truth: Some(bits & (1 << slot) != 0), ..Selection::default() },
            elements: None, mutable: true, contract: None,
        }).collect::<Vec<_>>().into()
    }
    #[test]
    fn widening_is_bounded_monotone_and_preserves_all_return_values() {
        let declarations = HashMap::new();
        let mut partitions = Partitions::default();
        for bits in 0..64 {
            partitions.insert(state(bits), Some(expression_flow::Value::index(bits)), &declarations).unwrap();
            assert!(partitions.entries.len() <= MAX_PARTITIONS);
        }
        assert!(partitions.widened);
        assert_eq!(partitions.entries.len(), 1);
        let (joined, returned) = partitions.joined(&declarations).unwrap().unwrap();
        assert!(joined.iter().all(|binding| binding.selection.truth.is_none()));
        assert_eq!(returned.unwrap().known_truth(), None);
        assert!(!partitions.insert(state(0), Some(expression_flow::Value::index(0)), &declarations).unwrap());
        assert_eq!(partitions.entries.len(), 1, "a widened node never recreates partitions");
    }
    #[test]
    fn coincident_branch_edges_do_not_assert_one_truth_value() {
        let source = crate::format("function root(flag0: boolean) { if (flag0) {} } function main() {}").unwrap();
        let program = crate::syntax::parse(&source).unwrap();
        let effects = effect_bodies::Index::new(&program).unwrap();
        let function = program.functions.iter().find(|function| function.name.text == "root").unwrap();
        let graph = flow_graph::summary_graph(&function.body).unwrap();
        let node = graph.nodes.iter().find(|node| node.next.len() == 2).unwrap();
        assert_eq!(node.next[0], node.next[1]);
        let mut state: FlowState = vec![effects.parameters(function.name.span).unwrap()[0].binding.clone()].into();
        let expression = node.expression.unwrap();
        assert!(binding_at(&state, "flag0", expression.span, &effects).is_some());
        refine(node, node.next[0], &mut state, &effects);
        assert_eq!(state[0].selection.truth, None);
    }
    #[test]
    fn partitions_use_binding_identity_instead_of_spelling() {
        let mut left = state(0);
        let mut right = left.clone();
        left[0].name = "same".into();
        right[0].name = "same".into();
        right[0].identity.as_mut().unwrap().owner = 1;
        let mut partitions = Partitions::default();
        partitions.insert(left, None, &HashMap::new()).unwrap();
        partitions.insert(right, None, &HashMap::new()).unwrap();
        assert_eq!(partitions.entries.len(), 2);
    }
}
