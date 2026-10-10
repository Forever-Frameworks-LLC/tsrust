//! Value-relative storage paths keep variant guards attached to their payloads.
use super::native_view_loans::{Loan, Segment, VariantGuard};
use super::*;

pub(super) fn normalize(loans: &mut Vec<Loan>) {
    loans.sort_by(|left, right| {
        (
            left.root,
            left.access == crate::native_return_borrows::BorrowAccess::Exclusive,
            &left.path,
            &left.guards,
        )
            .cmp(&(
                right.root,
                right.access == crate::native_return_borrows::BorrowAccess::Exclusive,
                &right.path,
                &right.guards,
            ))
    });
    loans.dedup_by(|right, left| {
        if right.root != left.root
            || right.access != left.access
            || right.path != left.path
            || right.guards != left.guards
        {
            return false;
        }
        left.retained |= right.retained;
        true
    });
}

pub(super) fn wrap(loan: &mut Loan, segment: &Segment) {
    loan.path.insert(0, segment.clone());
    for guard in &mut loan.guards {
        guard.path.insert(0, segment.clone());
    }
}

pub(super) fn select(loan: &mut Loan, segment: &Segment) -> bool {
    if loan.path.first() != Some(segment) {
        return false;
    }
    loan.path.remove(0);
    for guard in &mut loan.guards {
        if guard.path.first() == Some(segment) {
            guard.path.remove(0);
        }
    }
    true
}

pub(super) fn narrow(loan: &mut Loan, narrowed: Type) -> bool {
    if loan
        .guards
        .iter()
        .any(|guard| guard.path.is_empty() && narrowed != Type::Record(guard.success))
    {
        return false;
    }
    loan.guards.retain(|guard| !guard.path.is_empty());
    true
}

pub(super) fn possible(context: &Context, holder: usize, loan: &Loan) -> bool {
    loan.guards.iter().all(|guard| {
        context
            .flow
            .iter()
            .find(|(place, _)| {
                place.root == holder
                    && place.fields.len() == guard.path.len()
                    && place.fields.iter().zip(&guard.path).all(
                        |(field, segment)| matches!(segment,Segment::Field(name) if name == field),
                    )
            })
            .is_none_or(|(_, fact)| fact.members.contains(&Type::Record(guard.success)))
    })
}

pub(super) fn guard(loan: &mut Loan, success: usize) {
    loan.guards.push(VariantGuard {
        path: Vec::new(),
        success,
    });
    loan.guards.sort();
    loan.guards.dedup();
}
