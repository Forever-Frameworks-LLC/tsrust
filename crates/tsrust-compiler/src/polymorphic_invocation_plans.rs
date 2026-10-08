//! Concrete invocation alternatives retain environment and method identity together.
use crate::analysis::{CheckedClosure, CheckedClosureMethod};
use crate::{Diagnostic, Mode, Span};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Pair { pub origin: usize, pub method: usize }

/// Intern checked alternatives outside recursive expression nodes. IDs remain
/// stable through origin compaction; method ordinals belong to their environment.
#[derive(Default)]
pub(crate) struct Registry {
    plans: Vec<Vec<Pair>>,
    ids: std::collections::BTreeMap<Vec<Pair>, usize>,
}
impl Registry {
    pub(crate) fn publish(&mut self, pairs: impl IntoIterator<Item = Pair>, closures: &[CheckedClosure], span: Span) -> Result<usize, Diagnostic> {
        let plan = Plan::checked(pairs, closures, span)?;
        let pairs = plan.cases.iter().map(|case| case.pair).collect::<Vec<_>>();
        if let Some(&id) = self.ids.get(&pairs) { return Ok(id); }
        let id = self.plans.len();
        self.ids.insert(pairs.clone(), id);
        self.plans.push(pairs);
        Ok(id)
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (usize, &[Pair])> {
        self.plans.iter().enumerate().map(|(id, pairs)| (id, pairs.as_slice()))
    }

    /// Lookup a canonical pair slice without allocating during emission.
    pub(crate) fn index(&self, pairs: &[Pair]) -> Option<usize> {
        self.ids.get(pairs).copied()
    }

    pub(crate) fn checked<'a>(&self, id: usize, closures: &'a [CheckedClosure], span: Span) -> Result<Plan<'a>, Diagnostic> {
        let pairs = self.plans.get(id).ok_or_else(|| Diagnostic::new("callable-layout", "Generic invocation refers to an unknown checked plan.", span))?;
        Plan::checked(pairs.iter().copied(), closures, span)
    }

    pub(crate) fn validate(&self, closures: &[CheckedClosure], span: Span) -> Result<(), Diagnostic> {
        for id in 0..self.plans.len() { self.checked(id, closures, span)?; }
        Ok(())
    }

    pub(crate) fn remap(&self, origins: &[Option<usize>], span: Span) -> Result<Self, Diagnostic> {
        let mut result = Self::default();
        let mut identities = std::collections::BTreeMap::new();
        for source in &self.plans {
            let mut pairs = Vec::with_capacity(source.len());
            for &pair in source {
                let origin = origins.get(pair.origin).copied().flatten().ok_or_else(|| Diagnostic::new("callable-layout", "A checked invocation lost its environment during compaction.", span))?;
                if identities.insert(origin, pair.origin).is_some_and(|previous| previous != pair.origin) {
                    return Err(Diagnostic::new("callable-layout", "Distinct checked invocation environments cannot merge during compaction.", span));
                }
                pairs.push(Pair { origin, method: pair.method });
            }
            pairs.sort_unstable();
            let id = result.plans.len();
            if result.ids.insert(pairs.clone(), id).is_some() {
                return Err(Diagnostic::new("callable-layout", "Distinct checked invocation plans cannot merge during compaction.", span));
            }
            result.plans.push(pairs);
        }
        Ok(result)
    }
}

pub(crate) struct Case<'a> { pub pair: Pair, pub body: &'a CheckedClosureMethod }
pub(crate) struct Plan<'a> { pub cases: Vec<Case<'a>>, pub invocation: Mode }
impl<'a> Plan<'a> {
    pub(crate) fn checked(pairs: impl IntoIterator<Item = Pair>, closures: &'a [CheckedClosure], span: Span) -> Result<Self, Diagnostic> {
        let mut pairs = pairs.into_iter().collect::<Vec<_>>();
        pairs.sort_unstable(); pairs.dedup();
        if pairs.windows(2).any(|pair| pair[0].origin == pair[1].origin) { return Err(Diagnostic::new("callable-layout", "A reachable generic environment must select one concrete method per invocation.", span)); }
        if pairs.is_empty() { return Err(Diagnostic::new("callable-layout", "A generic invocation needs a reachable environment and method.", span)); }
        let mut cases: Vec<Case<'a>> = Vec::new();
        let mut invocation = Mode::Read;
        for pair in pairs {
            let closure = closures.get(pair.origin).ok_or_else(|| Diagnostic::new("callable-layout", "Generic invocation refers to an unknown environment.", span))?;
            let body = closure.methods.get(pair.method).ok_or_else(|| Diagnostic::new("callable-layout", "Generic invocation refers to an unknown method of its environment.", span).related(closure.span, "Environment is defined here."))?;
            if closure.function.template || body.function.template || body.parameters.iter().any(|&slot| slot >= body.function.bindings.len()) {
                return Err(Diagnostic::new("callable-layout", "Generic invocation requires a concrete checked method layout.", span).related(closure.span, "Environment is defined here."));
            }
            if let Some(first) = cases.first() {
                let compatible = first.body.function.result == body.function.result && first.body.parameters.len() == body.parameters.len()
                    && first.body.parameters.iter().zip(&body.parameters).all(|(&left, &right)| {
                        let left = &first.body.function.bindings[left]; let right = &body.function.bindings[right];
                        left.ty == right.ty && left.mode == right.mode && left.omission == right.omission && left.rest == right.rest
                    });
                if !compatible { return Err(Diagnostic::new("callable-layout", "Reachable generic invocation methods require compatible prepared arguments and results.", span)); }
            }
            invocation = invocation.max(body.invocation);
            cases.push(Case { pair, body });
        }
        Ok(Self { cases, invocation })
    }

    /// Receiver and prepared arguments are evaluated once, before variant selection.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render(&self, receiver: &str, arguments: &[String], supplied_count: &str, counted: impl Fn(Pair) -> bool, mutable: bool, storage: Option<(usize, &[usize])>, span: Span) -> Result<String, Diagnostic> {
        if arguments.len() != self.cases[0].body.parameters.len() {
            return Err(Diagnostic::new("callable-layout", "Generic invocation prepared argument count does not match its methods.", span));
        }
        let mutable = if mutable { "mut " } else { "" };
        let mut text = format!("{{ let {mutable}method_environment = {receiver}; ");
        for (index, value) in arguments.iter().enumerate() { text.push_str(&format!("let method_argument{index} = {value}; ")); }
        let arguments = (0..arguments.len()).map(|index| format!("method_argument{index}")).collect::<Vec<_>>().join(", ");
        let call = |pair: Pair, receiver: &str| {
            if counted(pair) {
                let separator = if arguments.is_empty() { "" } else { ", " };
                format!("{receiver}.call{}_with_count({supplied_count}{separator}{arguments})", pair.method)
            } else { format!("{receiver}.call{}({arguments})", pair.method) }
        };
        if let Some((family, origins)) = storage {
            if origins.is_empty() || origins.windows(2).any(|pair| pair[0] >= pair[1]) { return Err(Diagnostic::new("callable-layout", "Generic invocation storage requires canonical distinct environment origins.", span)); }
            text.push_str("match method_environment { ");
            for case in &self.cases {
                let variant = origins.iter().position(|&origin| origin == case.pair.origin).ok_or_else(|| Diagnostic::new("callable-layout", "Reachable generic invocation origin is absent from its storage family.", span))?;
                text.push_str(&format!("TsrGenericFamily{family}::V{variant}(method_payload) => {}, ", call(case.pair, "method_payload")));
            }
            if origins.iter().any(|origin| !self.cases.iter().any(|case| case.pair.origin == *origin)) {
                text.push_str("_ => panic!(\"generic environment is outside the verified invocation alternatives\"), ");
            }
            text.push_str("} }");
        } else {
            if self.cases.len() != 1 { return Err(Diagnostic::new("callable-layout", "Multiple generic invocation environments require explicit family storage.", span)); }
            text.push_str(&call(self.cases[0].pair, "method_environment")); text.push_str(" }");
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_invocations_publish_stable_checked_pairs_and_remap_together() {
        let checked = environments();
        let table = &checked.program.polymorphic_invocations;
        assert_eq!(table.plans.len(), 3, "repeated concrete calls reuse their checked pair plan");
        let first = [Pair { origin: 0, method: 0 }];
        let second = [Pair { origin: 1, method: 1 }];
        let first_id = table.index(&first).expect("source invocation published");
        let second_id = table.index(&second).expect("independent method published");
        assert_ne!(first_id, second_id);
        assert_eq!(table.checked(first_id, &checked.program.closures, Span::default()).unwrap().cases[0].pair, first[0]);
        let remapped = table.remap(&[Some(1), Some(0)], Span::default()).unwrap();
        assert_eq!(remapped.index(&[Pair { origin: 1, method: 0 }]), Some(first_id));
        assert_eq!(remapped.index(&[Pair { origin: 0, method: 1 }]), Some(second_id));
        assert!(table.remap(&[Some(0), None], Span::default()).is_err());
        assert!(table.remap(&[Some(0), Some(0)], Span::default()).is_err());
        assert!(table.checked(usize::MAX, &checked.program.closures, Span::default()).is_err());
    }
    #[test]
    fn unused_templates_do_not_publish_concrete_invocations() {
        let source = crate::format("function identity<T>(value:T):T{return value;}function unused<T>(value:T):T{const action=identity;return action<T>(value);}function replace<T>(value:T):T{const action=identity;return action<T>(value);}function main(){replace<int>(7);replace<string>('owned');}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        let table = &checked.program.polymorphic_invocations;
        assert_eq!(table.plans.len(), 2);
        for id in 0..table.plans.len() {
            let plan = table.checked(id, &checked.program.closures, Span::default()).unwrap();
            assert_eq!(plan.cases.len(), 1);
            assert!(!checked.program.closures[plan.cases[0].pair.origin].function.template);
        }
    }
    #[test]
    fn source_selected_fresh_environments_materialize_independent_method_ordinals() {
        let source = crate::format("function identity<T>(value:T):T{return value;}function main(){let action=identity;action<int>(7);action=identity;action<string>('owned');}").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).unwrap();
        assert_eq!(checked.program.closures[0].methods.len(), 2);
        assert_eq!(checked.program.closures[1].methods.len(), 1);
        let pairs = [Pair { origin: 0, method: 1 }, Pair { origin: 1, method: 0 }];
        let table = &checked.program.polymorphic_invocations;
        let id = table.index(&pairs).expect("source-selected environments publish one invocation plan");
        let plan = table.checked(id, &checked.program.closures, Span::default()).unwrap();
        assert_eq!(plan.cases.iter().map(|case| case.pair).collect::<Vec<_>>(), pairs);
        let main = checked.program.functions.iter().find(|function| function.name == "main").unwrap();
        let selected = crate::error_effects::checked_expressions(main).into_iter().filter_map(|expression| expression.generic_invocation).collect::<Vec<_>>();
        assert_eq!(selected, vec![id], "the actual source receiver retains its point-specific plan");
    }
    #[test]
    fn equivalent_family_layouts_do_not_exhaust_body_specialization_budget() {
        let mut source = String::from("function identity<T>(value:T):T{return value;}function use(action:Function<int>):int{return action();}function main(){let action=identity;action<int>(7);");
        for _ in 0..130 { source.push_str("action=identity;action<int>(7);"); }
        source.push_str("use(()=>9);const inline=(callback:Function<int>):int=>callback();inline(()=>10);}");
        let source = crate::format(&source).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let checked = crate::check(&source).expect("fresh storage layouts do not create new checked generic bodies");
        assert_eq!(checked.program.deferred_generic_values.len(), 131);
        assert!(checked.program.deferred_generic_values.iter().all(|&origin| checked.program.closures[origin].methods.len() == 1));
    }
    #[test]
    fn one_environment_cannot_select_two_methods_at_one_invocation() {
        let checked = crate::check("function action<T>(): int { return 7; } function main() { const alias = action; alias<int>(); alias<string>(); }").unwrap();
        let origin = *checked.program.deferred_generic_values.iter().next().unwrap();
        assert_eq!(checked.program.closures[origin].methods.len(), 2);
        let plan = Plan::checked([Pair { origin, method: 0 }, Pair { origin, method: 1 }], &checked.program.closures, Span::new(3, 4));
        let error = plan.err().expect("variant selection cannot choose between two methods of the same environment");
        assert_eq!(error.code, "callable-layout");
        assert_eq!(error.span, Span::new(3, 4));
    }
    fn environments() -> crate::Analysis {
        let source = crate::format("function first<T>(value: T): T { return value; } function second<T>(value: T): T { return value; } function main() { const one = first; const two = second; one<int>(3); one<int>(3); two<string>('warm'); two<int>(4); }").unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        crate::check(&source).unwrap()
    }
    #[test]
    fn origins_keep_independent_method_ordinals_and_checked_contracts() {
        let checked = environments();
        let closures = &checked.program.closures;
        assert_eq!(closures[0].methods.len(), 1);
        assert_eq!(closures[1].methods.len(), 2);
        let pairs = [Pair { origin: 0, method: 0 }, Pair { origin: 1, method: 1 }];
        let plan = Plan::checked(pairs.into_iter().rev(), closures, Span::default()).unwrap();
        assert_eq!(plan.cases.iter().map(|case| case.pair).collect::<Vec<_>>(), pairs);
        assert_eq!(plan.invocation, Mode::Read);
        assert!(Plan::checked([pairs[0], Pair { origin: 1, method: 0 }], closures, Span::default()).is_err(), "different concrete string and integer methods are not interchangeable");
        assert!(Plan::checked([Pair { origin: 0, method: 1 }], closures, Span::default()).is_err(), "another environment's method ordinal cannot be reused");
        assert!(Plan::checked([Pair { origin: usize::MAX, method: 0 }], closures, Span::default()).is_err());
        assert!(Plan::checked([], closures, Span::default()).is_err());
        assert!(plan.render("receiver", &[], "0", |_| false, false, Some((4, &[0, 1])), Span::default()).is_err());
        assert!(plan.render("receiver", &["argument".into()], "1", |_| false, false, Some((4, &[0, 0, 1])), Span::default()).is_err());
        assert!(plan.render("receiver", &["argument".into()], "1", |_| false, false, Some((4, &[0])), Span::default()).is_err());
        assert!(plan.render("receiver", &["argument".into()], "1", |_| false, false, None, Span::default()).is_err());
    }
    #[test]
    fn native_dispatch_keeps_method_pairs_evaluation_order_and_payload_ownership() {
        let checked = environments();
        let mut registry = Registry::default();
        let pairs = [Pair { origin: 0, method: 0 }, Pair { origin: 1, method: 1 }];
        let id = registry.publish(pairs, &checked.program.closures, Span::default()).unwrap();
        assert_eq!(registry.publish([pairs[1], pairs[0], pairs[1]], &checked.program.closures, Span::default()).unwrap(), id);
        assert!(registry.publish([pairs[0], Pair { origin: 0, method: 1 }], &checked.program.closures, Span::default()).is_err());
        let plan = registry.checked(id, &checked.program.closures, Span::default()).unwrap();
        let code = plan.render("{ evaluations.push(1); &family }", &["{ evaluations.push(2); 4i64 }".into()], "1", |_| false, false, Some((4, &[0, 1])), Span::default()).unwrap();
        let failure = plan.render("{ evaluations.push(1); TsrGenericFamily4::V0(Closure0(Payload(3))) }", &["fail_argument(&mut evaluations)".into()], "1", |_| false, false, Some((4, &[0, 1])), Span::default()).unwrap();
        crate::scoped_record_places::tests::native(&format!(r#"
            use std::sync::atomic::{{AtomicUsize, Ordering}};
            static DROPS: AtomicUsize = AtomicUsize::new(0);
            static CALLS: AtomicUsize = AtomicUsize::new(0);
            fn fail_argument(evaluations: &mut Vec<i32>) -> i64 {{ evaluations.push(2); panic!("argument failed"); }}
            struct Payload(i64);
            impl Drop for Payload {{ fn drop(&mut self) {{ DROPS.fetch_add(1, Ordering::SeqCst); }} }}
            struct Closure0(Payload);
            struct Closure1(Payload);
            impl Closure0 {{ fn call0(&self, value: i64) -> i64 {{ CALLS.fetch_add(1, Ordering::SeqCst); self.0.0 + value }} }}
            impl Closure1 {{ fn call1(&self, value: i64) -> i64 {{ CALLS.fetch_add(1, Ordering::SeqCst); self.0.0 + value * 2 }} }}
            enum TsrGenericFamily4 {{ V0(Closure0), V1(Closure1) }}
            fn main() {{
                for (family, expected) in [(TsrGenericFamily4::V0(Closure0(Payload(3))), 7), (TsrGenericFamily4::V1(Closure1(Payload(7))), 15)] {{
                    let before = DROPS.load(Ordering::SeqCst);
                    let mut evaluations = Vec::new();
                    assert_eq!({code}, expected);
                    assert_eq!(evaluations, [1, 2]);
                    assert_eq!(DROPS.load(Ordering::SeqCst), before, "borrowed invocation preserves owned payload");
                    drop(family);
                    assert_eq!(DROPS.load(Ordering::SeqCst), before + 1);
                }}
                assert_eq!(DROPS.load(Ordering::SeqCst), 2);
                assert_eq!(CALLS.load(Ordering::SeqCst), 2);
                std::panic::set_hook(Box::new(|_| {{}}));
                let mut evaluations = Vec::new();
                let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {failure}));
                assert!(failed.is_err());
                assert_eq!(evaluations, [1, 2]);
                assert_eq!(DROPS.load(Ordering::SeqCst), 3, "owned receiver drops once on argument failure");
                assert_eq!(CALLS.load(Ordering::SeqCst), 2, "argument failure prevents invocation");
            }}
        "#));
    }

}
