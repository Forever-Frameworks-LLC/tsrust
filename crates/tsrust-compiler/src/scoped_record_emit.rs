//! Native generic slots preserve nested scoped record environments.
use super::*;

impl Emitter<'_> {
    pub(super) fn scoped_record_native_parameter(&self, ty: Type) -> bool {
        // Generic field positions belong to the nominal declaration even when
        // this instance's callback environment owns all of its captures.
        let ty = match ty {
            Type::Record(id) => Type::Record(self.program.scoped_records.declaration(id)),
            other => other,
        };
        if matches!(
            self.callback_payload(ty).unwrap_or(ty),
            Type::Callback(_) | Type::Closure(_) | Type::BorrowedCallable(_)
        ) {
            return true;
        }
        crate::aggregate_callable_facts::inspect(
            ty,
            &self.program.records,
            |id| self.program.unions[id.index()].members.clone(),
            |ty| match ty {
                Type::BorrowedCallable(_) => true,
                Type::Closure(id) => self.program.closures[id].borrowed,
                Type::Callback(id) => !self.program.callback_contracts[id].kind.is_owned(),
                _ => false,
            },
        )
        .scoped
    }

    pub(super) fn named_record_environment_type(&self, ty: Type, lifetimes: &[String]) -> String {
        let mut next = 0;
        let result = self.record_environment_type(ty, lifetimes, &mut next);
        assert_eq!(next, lifetimes.len(), "checked environment lifetime slots");
        result
    }

    fn record_environment_type(&self, ty: Type, lifetimes: &[String], next: &mut usize) -> String {
        match ty {
            Type::Closure(id) => {
                let count = self.native_captures.lifetimes[id];
                let mut arguments = lifetimes[*next..*next + count].to_vec();
                *next += count;
                arguments.extend(
                    self.captured_callback_bounds(&self.program.closures[id])
                        .iter()
                        .map(|bound| {
                            bound
                                .split(':')
                                .next()
                                .expect("capture generic")
                                .trim()
                                .to_owned()
                        }),
                );
                format!("Closure{id}{}", lifetime_parameters(&arguments))
            }
            Type::BorrowedCallable(id) => {
                let arguments = self
                    .borrowed_payloads(id)
                    .iter()
                    .map(|payload| {
                        let origin = payload.origin;
                        let count = self.native_captures.lifetimes[origin];
                        let mut arguments = lifetimes[*next..*next + count].to_vec();
                        *next += count;
                        for bound in self.captured_callback_bounds(&self.program.closures[origin]) {
                            let parameter = bound.split(':').next().unwrap();
                            if let Some((_, input)) = payload
                                .captures
                                .iter()
                                .find(|(inner, _)| parameter == format!("K{inner}"))
                            {
                                arguments.push(self.record_environment_type(
                                    Type::Closure(*input),
                                    lifetimes,
                                    next,
                                ));
                            } else {
                                arguments.push(parameter.to_owned());
                            }
                        }
                        format!("Closure{origin}{}", lifetime_parameters(&arguments))
                    })
                    .collect::<Vec<_>>();
                format!("BorrowedCallable{id}<{}>", arguments.join(", "))
            }
            Type::Record(id) => {
                let arguments = self.program.records[id]
                    .fields
                    .iter()
                    .filter_map(|field| {
                        let ty = field.ty.expect("checked record field");
                        self.scoped_record_native_parameter(ty)
                            .then(|| self.record_environment_type(ty, lifetimes, next))
                    })
                    .collect::<Vec<_>>();
                format!(
                    "Record{}{}",
                    self.program.scoped_records.declaration(id),
                    lifetime_parameters(&arguments)
                )
            }
            Type::Union(id) | Type::BorrowedUnion(id) => {
                if let Some(payload) = self.callback_payload(ty) {
                    let payload = self.record_environment_type(payload, lifetimes, next);
                    format!("Union{}<{payload}>", id.index())
                } else {
                    self.native_type(ty)
                }
            }
            _ => self.native_type(ty),
        }
    }
}
