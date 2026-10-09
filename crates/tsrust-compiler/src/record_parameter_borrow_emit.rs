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
                    let markers = closure
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
                                        && self.program.closures[origin].captures[borrow.capture]
                                            .inner
                                            == capture.inner
                                })
                                .map(|borrow| format!("O{}", borrow.source))
                                .expect("retained capture owner input")
                        })
                        .collect::<Vec<_>>()
                        .join(",");
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
