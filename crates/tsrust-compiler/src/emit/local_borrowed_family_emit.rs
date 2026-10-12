//! Local callback alternatives retain their payloads before owner-tagged storage.
use super::*;

impl Emitter<'_> {
    pub(super) fn local_borrowed_family(&self, fun: &CheckedFunction, family: usize) -> bool {
        if !self.program.stored_contracts.is_borrowed(family) {
            return false;
        }
        let Some(origins) = self.program.stored_contracts.origins(family) else {
            return false;
        };
        let Some(function) = self
            .program
            .functions
            .iter()
            .position(|candidate| std::ptr::eq(candidate, fun))
        else {
            return false;
        };
        origins.iter().any(|origin| {
            self.borrowed_owners.contains_key(origin)
                || self.borrowed_capture_alternatives.contains_key(origin)
        }) && self
            .program
            .record_parameter_borrows
            .get(&function)
            .is_some_and(|borrows| {
                origins
                    .iter()
                    .all(|origin| borrows.iter().any(|borrow| borrow.origin == *origin))
            })
    }

    pub(super) fn local_borrowed_family_type(
        &self,
        fun: &CheckedFunction,
        ty: Type,
    ) -> Option<String> {
        let Type::BorrowedCallable(family) = ty else {
            return None;
        };
        if !self.local_borrowed_family(fun, family) {
            return None;
        }
        let types = self
            .program
            .stored_contracts
            .origins(family)
            .unwrap()
            .iter()
            .enumerate()
            .map(|(arm, &origin)| {
                self.borrowed_payload_type(
                    &borrowed_payload_emit::Payload {
                        origin,
                        captures: Vec::new(),
                        owners: Vec::new(),
                    },
                    &mut Vec::new(),
                    &mut Vec::new(),
                    arm,
                    true,
                )
            })
            .collect::<Vec<_>>();
        Some(format!(
            "LocalBorrowedCallable{family}<{}>",
            types.join(",")
        ))
    }

    pub(super) fn local_borrowed_family_constructor(
        &self,
        fun: &CheckedFunction,
        family: usize,
        value: &str,
    ) -> Option<String> {
        let name = self
            .local_borrowed_family_type(fun, Type::BorrowedCallable(family))?
            .replacen("<", "::<", 1);
        Some(format!("{name}::from({value})"))
    }

    pub(super) fn local_borrowed_family_store(
        &self,
        fun: &CheckedFunction,
        family: usize,
        destination: &str,
        code: &str,
    ) -> Option<String> {
        if !self.local_borrowed_family(fun, family) {
            return None;
        }
        let origins = self.program.stored_contracts.origins(family)?;
        let arms = origins.iter().enumerate().map(|(index, &origin)| {
            let variant = crate::borrowed_callable_dispatch::variant(index, origins.len());
            let tagged = self.tag_owner_constructor(fun, origin, "payload");
            format!("LocalBorrowedCallable{family}::{variant}(payload)=><{destination}>::from({tagged})")
        }).collect::<Vec<_>>().join(",");
        Some(format!("match {code} {{{arms}}}"))
    }
}
