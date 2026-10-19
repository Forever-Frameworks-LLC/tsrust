//! Source declaration proofs stay separate from native storage membership.
use super::*;
#[derive(Clone, Default)]
pub(crate) struct Proofs {
    calls: HashMap<String, HashMap<(usize, usize, usize, usize), Option<Vec<Span>>>>,
}
impl Proofs {
    pub(super) fn observe(&mut self, owner: &str, span: Span, callee: Span, contracts: Option<&[Contract]>, declarations: &HashSet<Span>) {
        let incoming = contracts.and_then(|contracts| {
            let mut spans = Vec::new();
            for contract in contracts {
                for span in std::iter::once(contract.declaration).chain(contract.alternatives.iter().copied()) {
                    if !declarations.contains(&span) { return None; }
                    spans.push(span);
                }
            }
            spans.sort_by_key(|span| (span.start, span.end));
            spans.dedup();
            (!spans.is_empty()).then_some(spans)
        });
        let calls = self.calls.entry(owner.to_owned()).or_default();
        let key = (span.start, span.end, callee.start, callee.end);
        let incoming = match (calls.get(&key), incoming) {
            (Some(Some(previous)), Some(mut incoming)) => {
                incoming.extend(previous); incoming.sort_by_key(|span| (span.start, span.end)); incoming.dedup(); Some(incoming)
            }
            (Some(None), _) => None,
            (_, incoming) => incoming,
        };
        calls.insert(key, incoming);
    }
    pub(crate) fn reachable(&self, owner: &str, span: Span, callee: Span) -> Option<&[Span]> {
        self.calls.get(owner)?.get(&(span.start, span.end, callee.start, callee.end)).and_then(|value| value.as_deref())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nongeneric_owner_exports_guard_selected_and_equal_arity_origins() {
        let source = crate::format(r#"
            function one<T>(value: T): T { return value; }
            function same<T>(value: T): T { return value; }
            function two<T>(value: T, other: T): T { return other; }
            function select(flag: boolean): int {
                let action = one;
                if (flag) { action = two; }
                if (flag) { return action<int>(4, 7); }
                return action<int>(4);
            }
            function matching(flag: boolean): int {
                let action = one;
                if (flag) { action = same; }
                return action<int>(3);
            }
            function main() {}
        "#).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let mut program = crate::syntax::parse(&source).unwrap();
        let declarations = program.functions.iter().map(|function| (function.name.text.clone(), function.name.span)).collect::<HashMap<_, _>>();
        let mut callees = HashMap::new();
        for function in &mut program.functions {
            crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
                if let ExprKind::Call(callee, _) = &expression.kind { callees.insert((expression.span.start, expression.span.end), callee.span); }
                Ok(())
            }).unwrap();
        }
        let proofs = validate(&mut program).unwrap();
        for (owner, snippets, expected) in [
            ("select", vec!["action<int>(4, 7)", "action<int>(4)"], vec![vec!["two"], vec!["one"]]),
            ("matching", vec!["action<int>(3)"], vec![vec!["one", "same"]]),
        ] {
            for (snippet, names) in snippets.into_iter().zip(expected) {
                let start = source.find(snippet).unwrap();
                let span = Span::new(start, start + snippet.len());
                let mut expected = names.into_iter().map(|name| declarations[name]).collect::<Vec<_>>();
                expected.sort_by_key(|span| (span.start, span.end));
                assert_eq!(proofs.reachable(owner, span, callees[&(span.start, span.end)]), Some(expected.as_slice()), "{owner}: {snippet}");
            }
        }
    }
    #[test]
    fn unknown_provenance_is_sticky_and_generated_receivers_do_not_collide() {
        let declaration = Span::new(10, 13);
        let call = Span::new(40, 55);
        let receiver = Span::new(40, 46);
        let declarations = HashSet::from([declaration]);
        let contracts = vec![contract(1, &[ParameterInitializer::Required], false, declaration)];
        let mut proofs = Proofs::default();
        proofs.observe("owner", call, receiver, Some(&contracts), &declarations);
        assert_eq!(proofs.reachable("owner", call, receiver), Some([declaration].as_slice()));
        assert_eq!(proofs.reachable("owner", call, call), None, "generated receivers cannot inherit the factory call proof");
        assert_eq!(proofs.reachable("other", call, receiver), None);
        proofs.observe("owner", call, receiver, None, &declarations);
        proofs.observe("owner", call, receiver, Some(&contracts), &declarations);
        assert_eq!(proofs.reachable("owner", call, receiver), None);
        proofs.observe("callback", call, receiver, Some(&contracts), &HashSet::new());
        assert_eq!(proofs.reachable("callback", call, receiver), None, "an annotation span is not a declaration origin");
    }
    #[test]
    fn concrete_generic_instances_keep_original_invocation_proofs() {
        let source = crate::format("function one<T>(value: T): T { return value; } function select<T>(value: T): T { const action = one; return action<T>(value); } function main() { console.log(select<int>(4)); }").unwrap();
        let mut program = crate::syntax::parse(&source).unwrap();
        let declaration = program.functions.iter().find(|function| function.name.text == "one").unwrap().name.span;
        let mut site = None;
        let function = program.functions.iter_mut().find(|function| function.name.text == "select").unwrap();
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Call(callee, _) = &expression.kind { site = Some((expression.span, callee.span)); }
            Ok(())
        }).unwrap();
        let cache = crate::generic_functions::specialize(&mut program).unwrap();
        let instance = program.functions.iter().find(|function| function.name.text.starts_with("select<") && function.application.is_some()).unwrap();
        let (call, receiver) = site.unwrap();
        assert_eq!(cache.reachable_declarations(&instance.name.text, call, receiver), Some([declaration].as_slice()));
    }

}
