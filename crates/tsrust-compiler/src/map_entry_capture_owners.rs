//! Stable entry identities refine ownership without weakening unknown-key loans.
use super::*;
use crate::capture_loan_owners::Place;

impl Analyzer {
    pub(super) fn map_entry_capture_place(base: &Expr, key: &Expr) -> Option<Place> {
        let (_, mut fields) = Self::direct_fields(base)?;
        let binding = Self::root_binding(base)?;
        let identity = match (&key.kind, key.ty?) {
            (ExprKind::String(value), Type::String) => format!("string:{value}"),
            (ExprKind::Integer(value), Type::Int) => format!("int:{}", value.parse::<i64>().ok()?),
            (ExprKind::Integer(value), Type::Number) => number_identity(value.parse::<f64>().ok()?),
            (ExprKind::Number(value), Type::Number) => number_identity(*value),
            (ExprKind::Boolean(value), Type::Boolean) => format!("boolean:{value}"),
            (ExprKind::Null, Type::Null) => "null".into(),
            (ExprKind::Undefined, Type::Undefined) => "undefined".into(),
            _ => return None,
        };
        // A NUL cannot occur in a source record field identifier. Decoded key
        // contents stay intact, so quoting and escape spelling cannot alias slots.
        fields.push(format!("\0map-entry:{identity}"));
        Some(Place { binding, fields })
    }

    pub(super) fn replace_map_entry_capture_loans(
        &self,
        context: &mut Context,
        place: Place,
        value: &Expr,
    ) {
        let temporary = Place {
            binding: usize::MAX,
            fields: vec![],
        };
        self.attach_payload_capture_loans(context, value, temporary.clone(), true);
        context.capture_loan_owners.attach_map_payload(temporary.clone(),value.ty.expect("checked Map payload"));
        context.capture_loan_owners.transfer(&temporary, &place);
        Self::restore_live_capture_loans(context);
    }

    pub(super) fn discard_map_entry_capture_loans(context: &mut Context, place: &Place) {
        let binding = &context.bindings[place.binding];
        if binding.parameter || binding.capture_reference {
            return;
        }
        context.capture_loan_owners.discard(place);
        Self::restore_live_capture_loans(context);
    }
}

fn number_identity(value: f64) -> String {
    // Match existing Map SameValueZero semantics, including signed zero.
    let bits = if value == 0.0 {
        0
    } else if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    };
    format!("number:{bits}")
}
