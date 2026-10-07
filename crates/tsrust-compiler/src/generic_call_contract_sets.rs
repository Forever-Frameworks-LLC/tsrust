//! Finite source callable sets retain signatures and declaration provenance.
use super::*;

/// Resolve a finite set without erasing distinct callable cardinalities.
pub(super) fn resolve_all(
    initial: Vec<Target>,
    bindings: &[Binding],
    declarations: &HashMap<String, Contract>,
) -> Option<Vec<Contract>> {
    let mut pending = initial;
    let mut visited = HashSet::new();
    let mut candidates: Vec<Contract> = Vec::new();
    while let Some(target) = pending.pop() {
        let candidate = match target {
            Target::Unknown => return None,
            Target::Contract(contract) => contract,
            Target::Name(name, span) => {
                let binding = bindings
                    .iter()
                    .enumerate()
                    .filter(|(_, binding)| {
                        binding.name == name
                            && binding.scope.start <= span.start
                            && span.end <= binding.scope.end
                    })
                    .min_by_key(|(_, binding)| binding.scope.end - binding.scope.start);
                if let Some((index, binding)) = binding {
                    if let Some(contract) = &binding.contract {
                        contract.clone()
                    } else {
                        if visited.insert(index) {
                            pending.extend(binding.targets.clone());
                        }
                        continue;
                    }
                } else {
                    declarations.get(&name)?.clone()
                }
            }
        };
        if let Some(existing) = candidates.iter_mut().find(|existing| {
            (existing.generic, existing.required, existing.maximum)
                == (candidate.generic, candidate.required, candidate.maximum)
        }) {
            let mut spans = vec![existing.declaration, candidate.declaration];
            spans.extend(existing.alternatives.iter().copied());
            spans.extend(candidate.alternatives);
            spans.sort_by_key(|span| (span.start, span.end));
            spans.dedup();
            existing.declaration = spans.remove(0);
            existing.alternatives = spans;
        } else {
            candidates.push(candidate);
        }
    }
    candidates.sort_by_key(|contract| {
        (
            contract.declaration.start,
            contract.declaration.end,
            contract.generic,
            contract.required,
            contract.maximum,
        )
    });
    Some(candidates)
}
