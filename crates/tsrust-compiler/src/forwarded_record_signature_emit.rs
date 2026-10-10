//! Forwarded record installers use the named target's native obligations.
use super::*;

impl Emitter<'_> {
    pub(super) fn forwarded_record_signature(
        &self,
        closure: &crate::analysis::CheckedClosure,
    ) -> Option<(usize, native_function_signature_emit::NativeSignature)> {
        let Some(ClosureForward::Function(target)) = closure.forward else {
            return None;
        };
        if closure
            .parameters
            .iter()
            .copied()
            .ne(0..self.program.functions[target].parameter_count)
            || !closure.captures.is_empty()
            || self
                .program
                .named_capture_access
                .get(&target)
                .is_some_and(|captures| !captures.is_empty())
            || self
                .program
                .record_parameter_borrows
                .get(&target)
                .is_none_or(Vec::is_empty)
                && self.program.functions[target].native_returns.is_none()
        {
            return None;
        }
        Some((target, self.native_function_signature(target, &[])))
    }

    pub(super) fn forwarded_record_trait_arguments(
        &self,
        target: usize,
        arguments: &str,
    ) -> String {
        let mut values = if arguments.is_empty() {
            Vec::new()
        } else {
            vec![arguments.to_owned()]
        };
        values.extend(
            self.owner_inputs(target)
                .iter()
                .map(|_| "std::marker::PhantomData::<()>".to_owned()),
        );
        values.join(", ")
    }

    pub(super) fn forwarded_record_trait_where(&self, target: usize) -> String {
        let mut bounds = self.nullable_parameter_replacement_bounds(target);
        bounds.extend(self.nullable_callback_replacement_bounds(target));
        let fun = &self.program.functions[target];
        for parameter in 0..fun.parameter_count {
            let Some(layout) = self.record_parameter_layout(fun, parameter) else {
                continue;
            };
            for leaf in &layout.leaves {
                let Type::Callback(callback) = leaf.ty else {
                    continue;
                };
                let replacements = self.record_replacement_bounds(target, parameter, &leaf.path);
                bounds.extend(
                    replacements
                        .into_iter()
                        .map(|bound| format!("C{callback}: {bound}")),
                );
            }
        }
        let groups = self.record_borrow_lifetime_groups(target);
        let mut replacements = groups
            .values()
            .copied()
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .map(|group| {
                let source = *groups
                    .iter()
                    .find(|(_, candidate)| **candidate == group)
                    .unwrap()
                    .0;
                (format!("'record_input{group}"), format!("'p{source}"))
            })
            .collect::<Vec<_>>();
        replacements.sort_by_key(|(name, _)| std::cmp::Reverse(name.len()));
        for bound in &mut bounds {
            for (source, destination) in &replacements {
                *bound = bound.replace(source, destination);
            }
        }
        for (&source, &group) in &groups {
            let first = *groups
                .iter()
                .find(|(_, candidate)| **candidate == group)
                .unwrap()
                .0;
            if source != first {
                bounds.push(format!("'p{source}: 'p{first}"));
            }
        }
        let owners = self
            .owner_inputs(target)
            .iter()
            .map(|source| (format!("O{source}"), "()".to_owned()))
            .collect();
        for bound in &mut bounds {
            *bound = callback_capture_emit::substitute_parameters(bound, &owners);
        }
        bounds.sort();
        bounds.dedup();
        if bounds.is_empty() {
            String::new()
        } else {
            format!(" where {}", bounds.join(", "))
        }
    }
}
