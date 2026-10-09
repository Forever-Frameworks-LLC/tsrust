//! Explicit input lifetimes for callbacks retained through mutable records.
use super::*;

impl Emitter<'_> {
    /// One carrier slot has one lifetime even when different calls fill it.
    /// Reborrow all contributing inputs for a common caller-selected lifetime.
    pub(super) fn record_borrow_lifetime_groups(
        &self,
        function: usize,
    ) -> std::collections::BTreeMap<usize, usize> {
        let borrows = self.program.record_parameter_borrows.get(&function);
        let mut parents = borrows
            .into_iter()
            .flatten()
            .map(|borrow| (borrow.source, borrow.source))
            .collect::<std::collections::BTreeMap<_, _>>();
        let mut slots = std::collections::BTreeMap::new();
        for borrow in borrows.into_iter().flatten() {
            let key = (borrow.target.clone(), borrow.origin, borrow.capture);
            if let Some(previous) = slots.insert(key, borrow.source) {
                let left = parents[&previous];
                let right = parents[&borrow.source];
                let common = left.min(right);
                for parent in parents.values_mut() {
                    if *parent == left || *parent == right {
                        *parent = common;
                    }
                }
            }
        }
        parents
    }

    pub(super) fn record_borrow_inputs(&self, function: usize) -> Vec<usize> {
        let mut inputs = self
            .program
            .record_parameter_borrows
            .get(&function)
            .into_iter()
            .flatten()
            .map(|borrow| borrow.source)
            .collect::<Vec<_>>();
        inputs.sort_unstable();
        inputs.dedup();
        inputs
    }

    pub(super) fn record_borrow_replacement_type(
        &self,
        function: usize,
        ty: Type,
        parameter: usize,
        path: &[String],
    ) -> String {
        self.record_borrow_replacement_type_selected(function, ty, parameter, path, None)
    }
    pub(super) fn record_borrow_replacement_types(
        &self,
        function: usize,
        ty: Type,
        parameter: usize,
        path: &[String],
    ) -> Vec<String> {
        if let Type::BorrowedCallable(family) = ty {
            let origins = self
                .program
                .stored_contracts
                .origins(family)
                .expect("checked replacement origins");
            if origins.iter().any(|origin| {
                self.borrowed_owners.contains_key(origin)
                    || self.borrowed_capture_alternatives.contains_key(origin)
            }) {
                return origins
                    .iter()
                    .flat_map(|origin| {
                        self.record_borrow_replacement_types(
                            function,
                            Type::Closure(*origin),
                            parameter,
                            path,
                        )
                    })
                    .collect();
            }
        }
        let Some(Type::Closure(origin)) = self.callback_payload(ty).or(Some(ty)) else {
            return vec![self.record_borrow_replacement_type(function, ty, parameter, path)];
        };
        if !self.borrowed_owners.contains_key(&origin) {
            return vec![self.record_borrow_replacement_type(function, ty, parameter, path)];
        }
        let plan = borrowed_owner_provenance::parameter_vectors(self.program, function);
        let result = plan
            .get(&origin)
            .into_iter()
            .flatten()
            .filter(|owners| {
                owners.iter().all(|(inner, owner)| {
                    owner.function == function
                        && owner.binding < self.program.functions[function].parameter_count
                        && self
                            .program
                            .record_parameter_borrows
                            .get(&function)
                            .into_iter()
                            .flatten()
                            .any(|borrow| {
                                borrow.origin == origin
                                    && borrow.target.binding == parameter
                                    && borrow.target.fields == path
                                    && borrow.source == owner.binding
                                    && self.program.closures[origin].captures[borrow.capture].inner
                                        == *inner
                            })
                })
            })
            .map(|owners| {
                self.record_borrow_replacement_type_selected(
                    function,
                    ty,
                    parameter,
                    path,
                    Some(owners),
                )
            })
            .collect::<Vec<_>>();
        if result.is_empty() {
            vec![self.record_borrow_replacement_type(function, ty, parameter, path)]
        } else {
            result
        }
    }
    fn record_borrow_replacement_type_selected(
        &self,
        function: usize,
        ty: Type,
        parameter: usize,
        path: &[String],
        owners: Option<&borrowed_owner_provenance::Vector>,
    ) -> String {
        match ty {
            Type::Closure(origin) => {
                let closure = &self.program.closures[origin];
                let (mut lifetimes, fields) = self.capture_layout(closure);
                let groups = self.record_borrow_lifetime_groups(function);
                for borrow in self
                    .program
                    .record_parameter_borrows
                    .get(&function)
                    .into_iter()
                    .flatten()
                    .filter(|borrow| {
                        borrow.origin == origin
                            && borrow.target.binding == parameter
                            && borrow.target.fields == path
                            && owners.is_none_or(|owners| {
                                owners.iter().any(|(inner, owner)| {
                                    *inner
                                        == self.program.closures[origin].captures[borrow.capture]
                                            .inner
                                        && owner.binding == borrow.source
                                })
                            })
                    })
                {
                    let field = &fields[borrow.capture];
                    for (slot, lifetime) in lifetimes.iter_mut().enumerate() {
                        let token = format!("'c{slot}");
                        if field
                            .split(|ch: char| !ch.is_alphanumeric() && ch != '\'' && ch != '_')
                            .any(|word| word == token)
                        {
                            *lifetime = format!("'record_input{}", groups[&borrow.source]);
                        }
                    }
                }
                let rendered = self.named_record_environment_type(ty, &lifetimes);
                let mut parameters = HashMap::new();
                for borrow in self
                    .program
                    .record_parameter_borrows
                    .get(&function)
                    .into_iter()
                    .flatten()
                    .filter(|borrow| {
                        borrow.origin == origin
                            && borrow.target.binding == parameter
                            && borrow.target.fields == path
                            && owners.is_none_or(|owners| {
                                owners.iter().any(|(inner, owner)| {
                                    *inner
                                        == self.program.closures[origin].captures[borrow.capture]
                                            .inner
                                        && owner.binding == borrow.source
                                })
                            })
                    })
                {
                    let capture = &closure.captures[borrow.capture];
                    if matches!(
                        closure.function.bindings[capture.inner].ty,
                        Type::Callback(_)
                    ) {
                        parameters
                            .insert(format!("K{}", capture.inner), format!("F{}", borrow.source));
                    }
                }
                let rendered = callback_capture_emit::substitute_parameters(&rendered, &parameters);
                if self.borrowed_owners.contains_key(&origin) {
                    let markers = if let Some(owners) = owners {
                        owners
                            .iter()
                            .map(|(_, owner)| format!("O{}", owner.binding))
                            .collect::<Vec<_>>()
                            .join(",")
                    } else {
                        closure
                            .captures
                            .iter()
                            .filter(|capture| matches!(capture.storage, Mode::Read | Mode::Mutate))
                            .map(|capture| {
                                self.program
                                    .record_parameter_borrows
                                    .get(&function)
                                    .into_iter()
                                    .flatten()
                                    .find(|borrow| {
                                        borrow.origin == origin
                                            && self.program.closures[origin].captures
                                                [borrow.capture]
                                                .inner
                                                == capture.inner
                                    })
                                    .map(|borrow| format!("O{}", borrow.source))
                                    .expect("retained capture owner input")
                            })
                            .collect::<Vec<_>>()
                            .join(",")
                    };
                    format!("TsrOwnedCapture<{rendered},({markers},)>")
                } else {
                    rendered
                }
            }
            Type::BorrowedCallable(family) => {
                let arguments = self
                    .program
                    .stored_contracts
                    .origins(family)
                    .expect("checked origins")
                    .iter()
                    .map(|&origin| {
                        self.record_borrow_replacement_type(
                            function,
                            Type::Closure(origin),
                            parameter,
                            path,
                        )
                    })
                    .collect::<Vec<_>>();
                format!("BorrowedCallable{family}<{}>", arguments.join(", "))
            }
            _ => self.native_type(ty),
        }
    }
}
