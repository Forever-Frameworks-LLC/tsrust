//! Stable dense-array slots refine capture ownership independently of layout.
use super::*;
use crate::capture_loan_owners::Place;

impl Analyzer {
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
