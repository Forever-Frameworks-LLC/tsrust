//! Finite source callable sets retain signatures and declaration provenance.
use super::*;

/// Resolve a finite set without erasing distinct callable cardinalities.
pub(super) fn resolve_all(
    initial: Vec<Target>,
    bindings: &[Binding],
    declarations: &HashMap<String, Contract>,
) -> Option<Vec<Contract>> {
    resolve_with_index(initial, bindings, declarations, None)
}

pub(super) fn resolve_indexed(initial: Vec<Target>, bindings: &[Binding], declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index) -> Option<Vec<Contract>> {
    resolve_with_index(initial, bindings, declarations, Some(effects))
}

fn resolve_with_index(initial: Vec<Target>, bindings: &[Binding], declarations: &HashMap<String, Contract>, effects: Option<&effect_bodies::Index>) -> Option<Vec<Contract>> {
    let mut pending = initial;
    let mut visited = HashSet::new();
    let mut candidates: Vec<Contract> = Vec::new();
    while let Some(target) = pending.pop() {
        let candidate = match target {
            Target::Unknown => return None,
            Target::Contract(contract) => contract,
            Target::Name(name, span) => {
                let binding = if let Some(effects) = effects {
                    branch_flow::binding_at(bindings, &name, span, effects).map(|index| (index, &bindings[index]))
                } else {
                    bindings.iter().enumerate().filter(|(_, binding)| {
                        binding.name == name && binding.scope.start <= span.start && span.end <= binding.scope.end
                    }).min_by_key(|(_, binding)| binding.scope.end - binding.scope.start)
                };
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
                    if effects.is_some_and(|effects| effects.binding(&name, span).is_some()) { return None; }
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
    (!candidates.is_empty()).then_some(candidates)
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    #[test]
    fn captured_alias_candidates_use_declaration_identity() {
        let source = crate::format("function outer<T>(){function change(){action=two;}let action=one;function shadow(){let action=two;action=one;}}function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function action<T>(value:T):T{return value;}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        let effects = effect_bodies::Index::new(&program).unwrap();
        let declarations = program.functions.iter().map(|function| (function.name.text.clone(), contract(function.type_parameters.len(), &function.parameter_initializers, function.rest_parameter, function.name.span))).collect();
        let outer = program.functions.iter().find(|function| function.name.text == "outer").unwrap();
        let mut bindings = Vec::new();
        collect(&outer.body, &mut bindings, &effects);
        for (suffix, expected) in [("#change", Some(1)), ("#shadow", None)] {
            let callee = program.functions.iter().find(|function| function.name.text.ends_with(suffix)).unwrap();
            let span = callee.body.iter().find_map(|statement| match &statement.kind { StmtKind::Assign { target, .. } => Some(target.span), _ => None }).unwrap();
            let resolved = resolve_indexed(vec![Target::Name("action".into(), span)], &bindings, &declarations, &effects);
            assert_eq!(resolved.as_ref().map(|contracts| contracts[0].required), expected, "{suffix}");
            assert!(resolve_indexed(vec![Target::Name("action".into(), span)], &[], &declarations, &effects).is_none(), "an absent captured/local binding must not resolve to global action");
        }
    }
    #[test]
    fn empty_candidate_set_is_unknown_not_a_known_callable() {
        assert!(resolve_all(Vec::new(), &[], &HashMap::new()).is_none());
    }
}
