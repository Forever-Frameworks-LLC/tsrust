//! Zero-sized native capture markers retain storage provenance across ordinary calls.
use super::*;
impl Emitter<'_> {
    pub(super) fn owner_inputs(&self, function: usize) -> Vec<usize> {
        let mut inputs = self
            .program
            .record_parameter_borrows
            .get(&function)
            .into_iter()
            .flatten()
            .filter(|borrow| self.borrowed_owners.contains_key(&borrow.origin))
            .map(|borrow| borrow.source)
            .collect::<Vec<_>>();
        inputs.sort();
        inputs.dedup();
        inputs
    }
    fn function_id(&self, function: &CheckedFunction) -> Option<usize> {
        self.program
            .functions
            .iter()
            .position(|candidate| std::ptr::eq(candidate, function))
    }
    pub(super) fn owner_name(owner: &borrowed_owner_provenance::Owner) -> String {
        format!("TsrOwner{}_{}", owner.function, owner.binding)
    }
    pub(super) fn owner_binding_marker(
        &self,
        function: &CheckedFunction,
        mut binding: usize,
    ) -> Option<String> {
        let id = self.function_id(function)?;
        let mut seen = HashSet::new();
        while seen.insert(binding) {
            let value = function.bindings.get(binding)?;
            if binding < function.parameter_count {
                return self
                    .owner_inputs(id)
                    .contains(&binding)
                    .then(|| format!("O{binding}"));
            }
            if !value.rebindable && (value.callback_reference || value.borrowed_place.is_some()) {
                let [root] = value.view_roots.as_slice() else {
                    return None;
                };
                binding = *root;
                continue;
            }
            return Some(Self::owner_name(&borrowed_owner_provenance::Owner {
                function: id,
                binding,
            }));
        }
        None
    }
    pub(super) fn owner_marker_parameters(&self, function: usize) -> String {
        self.owner_inputs(function)
            .iter()
            .map(|source| format!(",_owner{source}:std::marker::PhantomData<O{source}>"))
            .collect()
    }
    pub(super) fn owner_call_arguments(
        &self,
        function: &CheckedFunction,
        callee: usize,
        expression: &ExprKind,
    ) -> Vec<String> {
        let ExprKind::Call(_, arguments) = expression else {
            return Vec::new();
        };
        self.owner_argument_markers(function, callee, arguments)
    }
    pub(super) fn owner_argument_markers(
        &self,
        function: &CheckedFunction,
        callee: usize,
        arguments: &[Expr],
    ) -> Vec<String> {
        self.owner_inputs(callee)
            .iter()
            .map(|source| {
                let marker = arguments
                    .get(*source)
                    .and_then(|argument| argument.binding)
                    .and_then(|binding| self.owner_binding_marker(function, binding))
                    .unwrap_or_else(|| "()".into());
                format!("std::marker::PhantomData::<{marker}>")
            })
            .collect()
    }
    pub(super) fn owner_capture_marker(
        &self,
        function: &CheckedFunction,
        origin: usize,
    ) -> Option<String> {
        if !self.borrowed_owners.contains_key(&origin) {
            return None;
        }
        let markers = self.program.closures[origin]
            .captures
            .iter()
            .filter(|capture| matches!(capture.storage, Mode::Read | Mode::Mutate))
            .map(|capture| self.owner_binding_marker(function, capture.outer))
            .collect::<Option<Vec<_>>>()?;
        Some(format!("({},)", markers.join(",")))
    }
    pub(super) fn tag_owner_constructor(
        &self,
        function: &CheckedFunction,
        origin: usize,
        value: &str,
    ) -> String {
        self.owner_capture_marker(function, origin).map_or_else(
            || value.to_owned(),
            |marker| format!("TsrOwnedCapture::<_,{marker}>({value},std::marker::PhantomData)"),
        )
    }
    pub(super) fn emit_owner_markers(&self, output: &mut String) {
        if self.borrowed_owners.is_empty() {
            return;
        }
        output.push_str("struct TsrOwnedCapture<T,O>(T,std::marker::PhantomData<O>);\n");
        let owners = self
            .borrowed_owners
            .values()
            .flatten()
            .flatten()
            .map(|(_, owner)| owner.clone())
            .collect::<std::collections::BTreeSet<_>>();
        for owner in owners {
            let _ = writeln!(output, "struct {};", Self::owner_name(&owner));
        }
    }
}

/// Lifetime slots count each tagged environment independently, even for equal types.
pub(super) fn lifetime_alternatives(
    program: &CheckedProgram,
    types: &borrowed_payload_emit::Alternatives,
    owners: &borrowed_owner_provenance::Plan,
) -> borrowed_payload_emit::Alternatives {
    let mut result = types.clone();
    for (&origin, vectors) in owners {
        let singleton = vec![Vec::new()];
        let types = types.get(&origin).unwrap_or(&singleton);
        result.insert(
            origin,
            types
                .iter()
                .flat_map(|types| {
                    vectors
                        .iter()
                        .filter(|owners| {
                            types.iter().all(|(inner, actual)| {
                                owners
                                    .iter()
                                    .find(|(capture, _)| capture == inner)
                                    .is_none_or(|(_, owner)| {
                                        program.functions[owner.function].bindings[owner.binding].ty
                                            == Type::Closure(*actual)
                                    })
                            })
                        })
                        .map(|_| types.clone())
                })
                .collect(),
        );
    }
    result
}

#[inline(never)]
pub(super) fn marker_plan(program: &CheckedProgram) -> borrowed_owner_provenance::Plan {
    borrowed_owner_provenance::plan(program)
        .into_iter()
        .filter(|(origin, vectors)| {
            let callback_inputs = program.closures[*origin]
                .captures
                .iter()
                .filter(|capture| {
                    matches!(
                        program.closures[*origin].function.bindings[capture.inner].ty,
                        Type::Callback(_)
                    )
                })
                .map(|capture| capture.inner)
                .collect::<Vec<_>>();
            vectors.iter().enumerate().any(|(index, left)| {
                vectors.iter().skip(index + 1).any(|right| {
                    callback_inputs.iter().all(|inner| {
                        let ty = |vector: &borrowed_owner_provenance::Vector| {
                            vector
                                .iter()
                                .find(|(capture, _)| capture == inner)
                                .map(|(_, owner)| {
                                    program.functions[owner.function].bindings[owner.binding].ty
                                })
                        };
                        ty(left) == ty(right)
                    })
                })
            })
        })
        .collect()
}
