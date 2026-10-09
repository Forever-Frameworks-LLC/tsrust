//! Native payload alternatives keep source origins separate from capture types.
use super::*;
use std::collections::BTreeMap;

pub(super) type CaptureVector = Vec<(usize, usize)>;
pub(super) type Alternatives = BTreeMap<usize, Vec<CaptureVector>>;

#[derive(Clone, Debug)]
pub(super) struct Payload {
    pub(super) origin: usize,
    pub(super) captures: CaptureVector,
}

pub(super) fn capture_alternatives(program: &CheckedProgram) -> Alternatives {
    fn resolve(function: &CheckedFunction, inputs: &[Option<usize>], arg: &Expr) -> Option<usize> {
        if let Some(Type::Closure(origin)) = arg.ty {
            return Some(origin);
        }
        let mut binding = arg.binding?;
        let mut seen = HashSet::new();
        while seen.insert(binding) {
            if let Some(Some(origin)) = inputs.get(binding) {
                return Some(*origin);
            }
            let view = function.bindings.get(binding)?;
            if view.rebindable || view.view_roots.len() != 1 {
                return None;
            }
            binding = view.view_roots[0];
        }
        None
    }
    fn calls(
        function: &CheckedFunction,
        inputs: &[Option<usize>],
        pending: &mut Vec<(usize, Vec<Option<usize>>)>,
    ) {
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if let (Some(CallTarget::Function(callee)), ExprKind::Call(_, args)) =
                (expr.target, &expr.kind)
            {
                pending.push((
                    callee,
                    args.iter()
                        .map(|arg| resolve(function, inputs, arg))
                        .collect(),
                ));
            }
        });
    }
    let mut result = Alternatives::new();
    let mut pending = program
        .functions
        .iter()
        .enumerate()
        .map(|(id, function)| (id, vec![None; function.parameter_count]))
        .collect::<Vec<_>>();
    for closure in &program.closures {
        calls(&closure.function, &[], &mut pending);
    }
    let mut seen = std::collections::BTreeSet::new();
    while let Some((id, mut inputs)) = pending.pop() {
        let function = &program.functions[id];
        inputs.resize(function.parameter_count, None);
        if function.template || !seen.insert((id, inputs.clone())) {
            continue;
        }
        // Summaries can combine several relay calls. Collect only origins defined
        // in this body, then propagate whole input vectors across individual calls.
        let mut owned = HashSet::new();
        crate::fixed_storage::visit_function_expressions(function, &mut |expr| {
            if matches!(expr.kind, ExprKind::Closure { .. })
                && let Some(Type::Closure(origin)) = expr.ty
            {
                owned.insert(origin);
            }
        });
        let mut vectors: BTreeMap<usize, Vec<(usize, usize)>> = BTreeMap::new();
        let mut unresolved = HashSet::new();
        for borrow in program
            .record_parameter_borrows
            .get(&id)
            .into_iter()
            .flatten()
        {
            if !owned.contains(&borrow.origin) {
                continue;
            }
            let closure = &program.closures[borrow.origin];
            let capture = &closure.captures[borrow.capture];
            if !matches!(
                closure.function.bindings[capture.inner].ty,
                Type::Callback(_)
            ) {
                continue;
            }
            let Some(input) = inputs.get(borrow.source).copied().flatten() else {
                unresolved.insert(borrow.origin);
                continue;
            };
            if program.closures[input].captures.iter().any(|capture| {
                matches!(
                    program.closures[input].function.bindings[capture.inner].ty,
                    Type::Callback(_)
                )
            }) {
                unresolved.insert(borrow.origin);
                continue;
            }
            vectors
                .entry(borrow.origin)
                .or_default()
                .push((capture.inner, input));
        }
        for (origin, mut vector) in vectors {
            if unresolved.contains(&origin) {
                continue;
            }
            vector.sort_unstable();
            vector.dedup();
            let alternatives = result.entry(origin).or_default();
            if !alternatives.contains(&vector) {
                alternatives.push(vector);
            }
        }
        calls(function, &inputs, &mut pending);
    }
    // Even a single concrete environment must remain available when a later
    // store widens an earlier capture's native record layout. That capture has
    // no generic callback input of its own to supply the wrapper's parameter.
    for alternatives in result.values_mut() {
        alternatives.sort();
    }
    result
}

impl Emitter<'_> {
    pub(super) fn borrowed_payloads(&self, family: usize) -> Vec<Payload> {
        self.program
            .stored_contracts
            .origins(family)
            .expect("checked borrowed origins")
            .iter()
            .flat_map(
                |&origin| match self.borrowed_capture_alternatives.get(&origin) {
                    Some(alternatives) => alternatives
                        .iter()
                        .map(|captures| Payload {
                            origin,
                            captures: captures.clone(),
                        })
                        .collect(),
                    None => vec![Payload {
                        origin,
                        captures: Vec::new(),
                    }],
                },
            )
            .collect()
    }

    pub(super) fn borrowed_payload_type(
        &self,
        payload: &Payload,
        lifetimes: &mut Vec<String>,
        bounds: &mut Vec<String>,
        arm: usize,
        inferred: bool,
    ) -> String {
        let closure = &self.program.closures[payload.origin];
        let mut next = 0;
        let mut next_lifetime = || {
            let name = if inferred {
                "'_".to_owned()
            } else {
                format!("'b{arm}c{next}")
            };
            next += 1;
            lifetimes.push(name.clone());
            name
        };
        let mut arguments = (0..self.native_captures.lifetimes[payload.origin])
            .map(|_| next_lifetime())
            .collect::<Vec<_>>();
        for bound in self.captured_callback_bounds(closure) {
            let (parameter, _) = bound.split_once(':').expect("capture bound");
            if let Some((_, input)) = payload
                .captures
                .iter()
                .find(|(inner, _)| parameter == format!("K{inner}"))
            {
                let input_lifetimes = (0..self.native_captures.lifetimes[*input])
                    .map(|_| next_lifetime())
                    .collect::<Vec<_>>();
                arguments.push(
                    self.named_record_environment_type(Type::Closure(*input), &input_lifetimes),
                );
            } else if inferred {
                arguments.push("_".to_owned());
            } else {
                let renamed = format!("B{arm}{parameter}");
                arguments.push(renamed.clone());
                bounds.push(bound.replacen(parameter, &renamed, 1));
            }
        }
        format!(
            "Closure{}{}",
            payload.origin,
            lifetime_parameters(&arguments)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relay_contexts_keep_capture_vectors_correlated_and_recursive_calls_finite() {
        let checked = crate::check(r#"
            type Holder = { run: ReadOnly<Function<int>>; };
            function install(holder: Mutable<Holder>, left: ReadOnly<Function<int>>, right: ReadOnly<Function<int>>): void {
                holder.run = (): int => left() * 10 + right();
            }
            function relay(holder: Mutable<Holder>, left: ReadOnly<Function<int>>, right: ReadOnly<Function<int>>, n: int): void {
                const alias = left;
                install(holder, alias, right);
            }
            function recursive(action: ReadOnly<Function<int>>, n: int): void {
                if (n > 0) { recursive(action, n - 1); }
            }
            function main() {
                const values = [3];
                const first = (): int => values[0];
                const second = (): int => 9;
                const holder: Holder = { run: (): int => 0 };
                recursive(first, 2);
                relay(holder, first, second, 2);
                relay(holder, second, first, 1);
                console.log(holder.run());
            }
        "#).unwrap();
        let plan = capture_alternatives(&checked.program);
        assert_eq!(plan.len(), 1);
        let vectors = plan.values().next().unwrap();
        assert_eq!(vectors.len(), 2, "No Cartesian-product alternatives");
        assert_eq!(vectors[0].len(), 2);
        assert_eq!(vectors[1].len(), 2);
        assert_eq!(vectors[0][0].0, vectors[1][0].0);
        assert_eq!(vectors[0][1].0, vectors[1][1].0);
        assert_ne!(vectors[0][0].1, vectors[0][1].1);
        assert_eq!(vectors[0][0].1, vectors[1][1].1);
        assert_eq!(vectors[0][1].1, vectors[1][0].1);
        assert_eq!(plan, capture_alternatives(&checked.program));
    }
}
