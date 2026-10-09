//! Rebuild only replacement environments admitted by this particular call.
use super::*;

impl Emitter<'_> {
    pub(super) fn record_callback_reification(
        &self,
        caller: &CheckedFunction,
        callee: borrowed_owner_provenance::Scope,
        parameter: usize,
        names: &[String],
        arguments: &[Expr],
        ty: Type,
        place: &str,
    ) -> Option<String> {
        let Type::BorrowedCallable(family) = self.callback_payload(ty).unwrap_or(ty) else {
            return None;
        };
        let borrows = callee
            .borrows(self.program)
            .iter()
            .filter(|borrow| borrow.target.binding == parameter && borrow.target.fields == names)
            .collect::<Vec<_>>();
        let replacements = callee
            .replacements(self.program)
            .iter()
            .filter(|(slot, path, _)| *slot == parameter && path == names)
            .flat_map(
                |(_, _, ty)| match self.callback_payload(*ty).unwrap_or(*ty) {
                    Type::Closure(origin) => vec![origin],
                    Type::BorrowedCallable(id) => self
                        .program
                        .stored_contracts
                        .origins(id)
                        .unwrap_or_default()
                        .to_vec(),
                    _ => Vec::new(),
                },
            )
            .collect::<Vec<_>>();
        let payloads = self.borrowed_payloads(family);
        let selected = payloads
            .iter()
            .enumerate()
            .filter(|(_, payload)| {
                let sources = borrows
                    .iter()
                    .filter(|borrow| borrow.origin == payload.origin)
                    .collect::<Vec<_>>();
                if sources.is_empty() {
                    return replacements.contains(&payload.origin);
                }
                if !payload.owners.iter().all(|(inner, owner)| {
                    sources
                        .iter()
                        .filter(|borrow| {
                            self.program.closures[borrow.origin].captures[borrow.capture].inner
                                == *inner
                        })
                        .any(|borrow| {
                            arguments
                                .get(borrow.source)
                                .and_then(|argument| argument.binding)
                                .and_then(|binding| self.owner_binding_marker(caller, binding))
                                .is_some_and(|marker| marker == Self::owner_name(owner))
                        })
                }) {
                    return false;
                }
                payload.captures.iter().all(|(inner, actual)| {
                    let facts = sources
                        .iter()
                        .filter(|borrow| {
                            self.program.closures[borrow.origin].captures[borrow.capture].inner
                                == *inner
                        })
                        .filter_map(|borrow| {
                            arguments
                                .get(borrow.source)
                                .and_then(|argument| argument.ty)
                        })
                        .collect::<Vec<_>>();
                    // An abstract argument cannot exclude a concrete environment.
                    facts.is_empty()
                        || facts.iter().any(|ty| match ty {
                            Type::Closure(origin) => origin == actual,
                            Type::BorrowedCallable(id) => self
                                .program
                                .stored_contracts
                                .origins(*id)
                                .is_none_or(|origins| origins.contains(actual)),
                            _ => true,
                        })
                })
            })
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return None;
        }
        let mut present = selected.iter().map(|(position, _)| {
            let variant = crate::borrowed_callable_dispatch::variant(*position, payloads.len());
            format!("BorrowedCallable{family}::{variant}(payload)=>BorrowedCallable{family}::{variant}(payload)")
        }).collect::<Vec<_>>();
        if selected.len() != payloads.len() {
            present.push(
                "_=>unreachable!(\"checked replacement retained an old callback environment\")"
                    .into(),
            );
        }
        let present = present.join(",");
        let Type::Union(union) = ty else {
            return Some(format!("{place}=match {place}{{{present}}};"));
        };
        let arms = self.program.unions[union.index()]
            .members
            .iter()
            .enumerate()
            .map(|(index, member)| {
                let converted = if matches!(member, Type::Null | Type::Undefined) {
                    "payload".to_owned()
                } else {
                    format!("match payload{{{present}}}")
                };
                format!(
                    "Union{}::V{index}(payload)=>Union{}::V{index}({converted})",
                    union.index(),
                    union.index()
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        Some(format!("{place}=match {place}{{{arms}}};"))
    }
}
