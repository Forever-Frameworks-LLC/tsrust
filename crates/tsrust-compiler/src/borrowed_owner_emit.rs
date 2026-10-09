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
    pub(super) fn closure_owner_inputs(&self, origin: usize) -> Vec<usize> {
        let closure = &self.program.closures[origin];
        let mut inputs = self
            .program
            .closure_record_parameter_borrows
            .get(&origin)
            .into_iter()
            .flatten()
            .filter(|borrow| self.borrowed_owners.contains_key(&borrow.origin))
            .map(|borrow| closure.parameters[borrow.source])
            .collect::<Vec<_>>();
        inputs.extend(
            self.program
                .closure_record_capture_borrows
                .get(&origin)
                .into_iter()
                .flatten()
                .enumerate()
                .filter(|(_, borrow)| self.borrowed_owners.contains_key(&borrow.origin))
                .filter_map(|(position, borrow)| {
                    if borrow.source_projection.is_direct() {
                        Some(closure.captures[borrow.source_capture].inner)
                    } else {
                        // An opaque projection has no capture path and no owner input.
                        (!borrow.source_projection.captures().is_empty()).then(|| Self::projected_owner_key(position))
                    }
                }),
        );
        inputs.sort();
        inputs.dedup();
        inputs
    }
    /// A retained source inside a captured environment has its own owner
    /// input: two retained fields of one receiver can have different owners.
    /// The key is the position of its captured borrow, outside binding IDs.
    pub(super) fn projected_owner_key(position: usize) -> usize {
        usize::MAX - position
    }
    fn projected_owner_position(key: usize) -> Option<usize> {
        (key > usize::MAX / 2).then(|| usize::MAX - key)
    }
    /// The storage binding in `function` that a captured environment retains
    /// along `path`, when every environment on the way was constructed there.
    pub(super) fn constructed_capture_binding(&self, function: &CheckedFunction, mut binding: usize, path: &[usize]) -> Option<usize> {
        for &inner in path {
            let value = function.bindings.get(binding)?;
            if value.parameter || value.capture_reference {
                return None;
            }
            let Type::Closure(origin) = value.ty else { return None; };
            binding = self.program.closures[origin].captures.iter().find(|capture| capture.inner == inner)?.outer;
        }
        Some(binding)
    }
    /// The owner marker, in `function`, of what the closure value `receiver`
    /// retains along `path`.
    fn receiver_owner_marker(&self, function: &CheckedFunction, receiver: usize, path: &[usize]) -> Option<String> {
        let value = function.bindings.get(receiver)?;
        if let Some((origin, closure)) = self.program.closures.iter().enumerate().find(|(_, closure)| std::ptr::eq(function, &closure.function))
            && let Some(capture) = closure.captures.iter().position(|capture| capture.inner == receiver)
        {
            // A held environment: this closure names the owner through its own input.
            let position = self.program.closure_record_capture_borrows.get(&origin)?.iter().position(|borrow| {
                borrow.source_capture == capture && borrow.source_projection.captures() == path
            })?;
            let key = Self::projected_owner_key(position);
            return self.closure_owner_inputs(origin).contains(&key).then(|| format!("O{key}"));
        }
        if value.parameter || value.capture_reference {
            return None;
        }
        let binding = self.constructed_capture_binding(function, receiver, path)?;
        self.owner_binding_marker(function, binding)
    }
    /// Hidden retained sources use their capture binding, not a written argument slot.
    pub(super) fn retained_call_binding(
        &self,
        scope: borrowed_owner_provenance::Scope,
        source: usize,
        arguments: &[Expr],
    ) -> Option<usize> {
        match scope {
            borrowed_owner_provenance::Scope::Closure(origin)
                if source >= self.program.closures[origin].parameters.len() =>
            {
                self.program.closures[origin]
                    .captures
                    .get(source - self.program.closures[origin].parameters.len())
                    .map(|capture| capture.outer)
            }
            _ => arguments.get(source).and_then(|argument| argument.binding),
        }
    }
    pub(super) fn closure_owner_arguments(
        &self,
        function: &CheckedFunction,
        origin: usize,
        arguments: &[Expr],
        receiver: usize,
    ) -> Vec<String> {
        let closure = &self.program.closures[origin];
        self.closure_owner_inputs(origin)
            .into_iter()
            .map(|parameter| {
                let marker = if let Some(slot) = closure.parameters.iter().position(|candidate| *candidate == parameter) {
                    arguments.get(slot).and_then(|argument| argument.binding)
                        .and_then(|binding| self.owner_binding_marker(function, binding))
                } else {
                    // A hidden source: resolve it through the receiver value.
                    let path = match Self::projected_owner_position(parameter) {
                        Some(position) => self.program.closure_record_capture_borrows.get(&origin).and_then(|borrows| borrows.get(position)).map(|borrow| {
                            std::iter::once(closure.captures[borrow.source_capture].inner).chain(borrow.source_projection.captures().iter().copied()).collect::<Vec<_>>()
                        }),
                        None => Some(vec![parameter]),
                    };
                    path.and_then(|path| self.receiver_owner_marker(function, receiver, &path))
                };
                format!("std::marker::PhantomData::<{}>", marker.unwrap_or_else(|| "()".into()))
            })
            .collect()
    }
    fn function_id(&self, function: &CheckedFunction) -> Option<usize> {
        self.program
            .functions
            .iter()
            .position(|candidate| std::ptr::eq(candidate, function))
    }
    pub(super) fn owner_name(owner: &borrowed_owner_provenance::Owner) -> String {
        match owner.scope {
            borrowed_owner_provenance::Scope::Function(id) => {
                format!("TsrOwner{id}_{}", owner.binding)
            }
            borrowed_owner_provenance::Scope::Closure(id) => {
                format!("TsrClosureOwner{id}_{}", owner.binding)
            }
        }
    }
    pub(super) fn owner_binding_marker(
        &self,
        function: &CheckedFunction,
        mut binding: usize,
    ) -> Option<String> {
        if let Some((origin, closure)) = self
            .program
            .closures
            .iter()
            .enumerate()
            .find(|(_, closure)| std::ptr::eq(function, &closure.function))
        {
            if Self::projected_owner_position(binding).is_some() {
                return self.closure_owner_inputs(origin).contains(&binding).then(|| format!("O{binding}"));
            }
            let mut seen = HashSet::new();
            while seen.insert(binding) {
                if closure.parameters.contains(&binding)
                    || closure
                        .captures
                        .iter()
                        .any(|capture| capture.inner == binding)
                {
                    return self
                        .closure_owner_inputs(origin)
                        .contains(&binding)
                        .then(|| format!("O{binding}"));
                }
                let value = function.bindings.get(binding)?;
                if !value.rebindable
                    && (value.callback_reference || value.borrowed_place.is_some())
                    && value.view_roots.len() == 1
                {
                    binding = value.view_roots[0];
                } else {
                    if value.parameter || value.capture_reference {
                        return None;
                    }
                    return Some(Self::owner_name(&borrowed_owner_provenance::Owner {
                        scope: borrowed_owner_provenance::Scope::Closure(origin),
                        binding,
                    }));
                }
            }
            return None;
        }
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
                scope: borrowed_owner_provenance::Scope::Function(id),
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
    /// Owner markers for a callback trait implementation, where no caller
    /// names the owners. One owner keeps the unit marker. Several owners need
    /// distinct markers: replacement bounds that differ only in a lifetime
    /// would otherwise be ambiguous.
    pub(super) fn unnamed_owner_markers(&self, count: usize) -> Vec<String> {
        if count <= 1 {
            return vec!["()".to_owned(); count];
        }
        (0..count).map(|position| format!("TsrUnnamedOwner<{position}>")).collect()
    }
    pub(super) fn emit_owner_markers(&self, output: &mut String) {
        if self.borrowed_owners.is_empty() {
            return;
        }
        output.push_str("struct TsrOwnedCapture<T,O>(T,std::marker::PhantomData<O>);\nstruct TsrUnnamedOwner<const N: usize>;\n");
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
                                        owner.scope.function(program).bindings[owner.binding].ty
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
                                    owner.scope.function(program).bindings[owner.binding].ty
                                })
                        };
                        ty(left) == ty(right)
                    })
                })
            })
        })
        .collect()
}
