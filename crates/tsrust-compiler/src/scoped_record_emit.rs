//! Native generic slots preserve nested scoped record environments.
use super::*;

impl Emitter<'_> {
    pub(super) fn native_record_declaration(&self, id: usize) -> usize {
        let base = self.program.scoped_records.declaration(id);
        if self.native_views.recursive(id) { id } else { base }
    }

    /// A recursive component uses one lifetime for every contained native view.
    /// Acyclic containers outside this component still use their ordinary slots.
    pub(super) fn recursive_view_native_type(&self, ty: Type, lifetime: &str) -> String {
        if let Type::StoredCallable(family) = ty
            && let Some(layout) = self.program.stored_contracts.scoped_native_family(family) {
            return format!("NativeViewCallable{family}<{}>", vec![lifetime; layout.lifetimes()].join(", "));
        }
        if !self.native_views.contains(ty) { return self.native_type(ty); }
        match ty {
            Type::Record(id) if native_view_emit::access(self.program, id).is_some() || self.native_views.recursive(id) => {
                format!("Record{id}<{lifetime}>")
            }
            Type::Record(id) => {
                let base = self.native_record_declaration(id);
                let arguments = self.program.records[id].fields.iter().enumerate().filter_map(|(position, field)| {
                    let ty = field.ty.expect("checked field");
                    self.scoped_record_native_field(id, position, ty).then(|| self.recursive_view_native_type(ty, lifetime))
                }).collect::<Vec<_>>();
                format!("Record{base}{}", lifetime_parameters(&arguments))
            }
            Type::Array(element) => {
                let payload = self.recursive_view_native_type(element.ty(), lifetime);
                match element {
                    crate::syntax::ArrayElement::Record(id, 0) => {
                        if let Some((_, arguments)) = payload.split_once('<') { format!("RecordArray{id}<{arguments}") }
                        else { format!("RecordArray{id}") }
                    }
                    _ => format!("Vec<{payload}>"),
                }
            }
            Type::Union(id) | Type::BorrowedUnion(id) => {
                let arguments = self.program.unions[id.index()].members.iter().filter(|ty| !ty.is_copy())
                    .map(|&member| {let native=self.recursive_view_native_type(member,lifetime);if matches!(ty,Type::Union(_)){self.union_stored_type(id,member,native)}else{native}}).collect::<Vec<_>>().join(", ");
                if matches!(ty, Type::BorrowedUnion(_)) { format!("ViewUnion{}<{lifetime}, {arguments}>", id.index()) }
                else { format!("Union{}<{arguments}>", id.index()) }
            }
            _ => self.native_type(ty),
        }
    }

    pub(super) fn scoped_record_native_parameter(&self, ty: Type) -> bool {
        self.native_views.contains(ty) || self.callback_record_native_parameter(ty)
    }

    pub(super) fn callback_record_native_parameter(&self, ty: Type) -> bool {
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
            Type::Map(id) if self.native_views.contains(ty) => {
                let payload=self.record_environment_type(self.program.records[id].map_definition.as_ref().unwrap().value,lifetimes,next);
                format!("Map{id}<{payload}>")
            }
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
                        let inner = format!("Closure{origin}{}", lifetime_parameters(&arguments));
                        if payload.owners.is_empty() {
                            inner
                        } else {
                            let markers = payload
                                .owners
                                .iter()
                                .map(|(_, owner)| Self::owner_name(owner))
                                .collect::<Vec<_>>()
                                .join(",");
                            format!("TsrOwnedCapture<{inner},({markers},)>")
                        }
                    })
                    .collect::<Vec<_>>();
                format!("BorrowedCallable{id}<{}>", arguments.join(", "))
            }
            Type::Record(id) if native_view_emit::access(self.program,id).is_some() || self.native_views.recursive(id) => {
                let lifetime=&lifetimes[*next];*next+=1;
                format!("Record{id}<{lifetime}>")
            }
            Type::StoredCallable(id) if self.program.stored_contracts.scoped_native_family(id).is_some() => {
                let count=self.program.stored_contracts.scoped_native_family(id).expect("scoped family").lifetimes();
                let arguments=lifetimes[*next..*next+count].to_vec();*next+=count;
                format!("NativeViewCallable{id}{}",lifetime_parameters(&arguments))
            }
            Type::Array(element) if self.native_views.contains(ty) => {
                let payload=self.record_environment_type(element.ty(),lifetimes,next);
                match element {
                    crate::syntax::ArrayElement::Record(id,0)=>format!("RecordArray{id}<{}>",payload.split_once('<').expect("native view array argument").1),
                    _=>format!("Vec<{payload}>"),
                }
            }
            Type::Record(id) => {
                let arguments = self.program.records[id]
                    .fields
                    .iter()
                    .enumerate()
                    .filter_map(|(position,field)| {
                        let ty = field.ty.expect("checked record field");
                        self.scoped_record_native_field(id,position,ty)
                            .then(|| self.record_environment_type(ty, lifetimes, next))
                    })
                    .collect::<Vec<_>>();
                format!(
                    "Record{}{}",
                    self.native_record_declaration(id),
                    lifetime_parameters(&arguments)
                )
            }
            Type::Union(id) | Type::BorrowedUnion(id) if self.native_views.contains(ty) => {
                let borrowed=matches!(ty,Type::BorrowedUnion(_));
                let prefix=if borrowed {let lifetime=lifetimes[*next].clone();*next+=1;format!("{lifetime}, ")}else{String::new()};
                let arguments=self.program.unions[id.index()].members.iter().filter(|member|!member.is_copy())
                    .map(|&member|{let native=self.record_environment_type(member,lifetimes,next);if borrowed{native}else{self.union_stored_type(id,member,native)}}).collect::<Vec<_>>().join(", ");
                format!("{}Union{}<{prefix}{arguments}>",if borrowed {"View"}else{""},id.index())
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
