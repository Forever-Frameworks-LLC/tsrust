//! Stable dense-array slots refine capture ownership independently of layout.
use super::*;
use crate::capture_loan_owners::Place;

impl Analyzer {
    pub(super) fn array_removal_snapshot(context: &Context, value: &Expr) -> Option<Box<crate::capture_loan_owners::Owners>> {
        let ExprKind::Call(callee, arguments) = &value.kind else { return None; };
        let ExprKind::Field(base, method) = &callee.kind else { return None; };
        if !arguments.is_empty() || !matches!(method.as_str(), "pop" | "shift") { return None; }
        let (name, fields) = Self::direct_fields(base)?;
        let binding = *context.names.get(&name)?;
        if context.bindings[binding].parameter || context.bindings[binding].capture_reference { return None; }
        context.capture_loan_owners.array_length(&Place { binding, fields })?;
        Some(Box::new(context.capture_loan_owners.clone()))
    }

    pub(super) fn finish_discarded_array_removal(context: &mut Context, value: &Expr, snapshot: Option<Box<crate::capture_loan_owners::Owners>>) {
        Self::finish_array_removal(context, value, snapshot, None);
    }

    /// A bound removal transfers only the removed slot to its result owner.
    /// The result remains live until normal move and lexical-scope disposal.
    pub(super) fn finish_bound_array_removal(context: &mut Context, value: &Expr, snapshot: Option<Box<crate::capture_loan_owners::Owners>>, binding: usize) {
        Self::finish_array_removal(context, value, snapshot, Some(Place { binding, fields: vec![] }));
    }

    fn finish_array_removal(context: &mut Context, value: &Expr, snapshot: Option<Box<crate::capture_loan_owners::Owners>>, destination: Option<Place>) {
        let Some(mut owners) = snapshot else { return; };
        let ExprKind::Call(callee, _) = &value.kind else { return; };
        let ExprKind::Field(base, _) = &callee.kind else { return; };
        let Some((_, fields)) = Self::direct_fields(base) else { return; };
        let Some(binding) = Self::root_binding(base) else { return; };
        let place = Place { binding, fields };
        let Some(length) = owners.array_length(&place) else { return; };
        let front = value.target == Some(CallTarget::ArrayShift);
        if !front && value.target != Some(CallTarget::ArrayPop) { return; }
        if length > 0 {
            let slot = |index| {
                let mut slot = place.clone(); slot.fields.push(format!("\0array-slot:{index}")); slot
            };
            let removed = slot(if front { 0 } else { length - 1 });
            if let Some(destination) = &destination {
                owners.transfer(&removed, destination);
            } else {
                owners.discard(&removed);
            }
            if front {
                for index in 1..length { owners.transfer(&slot(index), &slot(index - 1)); }
            }
            owners.set_array_length(place, length - 1);
        }
        // A direct named receiver with no arguments has no intervening payload
        // effects. Bound results keep the selected slot; discarded results drop it.
        context.capture_loan_owners = *owners;
        Self::restore_live_capture_loans(context);
    }

    pub(super) fn array_capture_place(value: &Expr) -> Option<Place> {
        let mut value = value;
        let mut fields = Vec::new();
        loop {
            match &value.kind {
                ExprKind::Index(base, index) => {
                    let ExprKind::Integer(spelling) = &index.kind else { return None; };
                    let index = spelling.parse::<u64>().ok()?;
                    fields.push(format!("\0array-slot:{index}"));
                    value = base;
                }
                ExprKind::Field(base, field) => {
                    fields.push(field.clone());
                    value = base;
                }
                ExprKind::Name(_) => {
                    fields.reverse();
                    return Some(Place { binding: value.binding?, fields });
                }
                _ => return None,
            }
        }
    }

    pub(super) fn collapse_array_capture_slots(context: &mut Context, base: &Expr) {
        let Some(place) = Self::array_capture_place(base).or_else(|| {
            Self::scoped_array_storage_path(base).map(|(binding, fields)| Place { binding, fields })
        }) else { return; };
        context.capture_loan_owners.invalidate_array_shape(&place);
        let payload = context.capture_loan_owners.payload(&place);
        // Reordering invalidates positional identity. Keep all its loans at the
        // container root so later slot replacement cannot erase them.
        for (path, loans) in payload {
            if path.iter().any(|field| field.starts_with("\0array-slot:")) {
                let mut source = place.clone();
                source.fields.extend(path);
                context.capture_loan_owners.discard(&source);
                context.capture_loan_owners.attach(place.clone(), loans);
            }
        }
    }
}
