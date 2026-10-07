//! Source snapshots for bodies proven to have only ordered direct writes.
use super::*;

pub(super) fn at_call(
    initial: &[Binding],
    writes: &[targets::Write],
    scope: Span,
    call_start: usize,
    declarations: &HashMap<String, Contract>,
    effects: &effect_bodies::Index,
) -> Vec<Binding> {
    let mut bindings = initial.to_vec();
    // Initializers snapshot callable values. Assignment events replace rather
    // than accumulate targets; saved immutable aliases retain earlier snapshots.
    let mut events = bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| {
            scope.start < binding.scope.start && binding.scope.start <= call_start
        })
        .map(|(index, binding)| (binding.scope.start, false, index))
        .chain(
            writes
                .iter()
                .enumerate()
                .filter(|(_, write)| {
                    scope.start <= write.span.start
                        && write.span.end <= scope.end
                        && write.completed <= call_start
                })
                .map(|(index, write)| (write.completed, true, index)),
        )
        .collect::<Vec<_>>();
    events.sort_by_key(|&(position, assignment, _)| (position, assignment));
    for (_, assignment, index) in events {
        let (binding_index, candidates) = if assignment {
            let write = &writes[index];
            let Some(binding_index) = branch_flow::binding_at(&bindings, &write.name, write.span, effects) else { continue; };
            let binding = &bindings[binding_index];
            if !binding.mutable || binding.contract.is_some() {
                continue;
            }
            (binding_index, write.targets.clone())
        } else {
            if bindings[index].contract.is_some() {
                continue;
            }
            (index, bindings[index].targets.clone())
        };
        let snapshot =
            resolve(candidates, &bindings, declarations, effects).map_or(Target::Unknown, Target::Contract);
        bindings[binding_index].targets = vec![snapshot];
    }
    bindings
}
