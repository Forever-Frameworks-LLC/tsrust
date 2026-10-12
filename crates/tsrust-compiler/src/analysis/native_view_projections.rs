//! Value-relative storage paths keep variant guards attached to their payloads.
use super::native_view_loans::{Loan, Segment, VariantGuard};
use super::*;

pub(super) fn normalize(loans: &mut Vec<Loan>) {
    loans.sort_by(|left, right| {
        (
            left.root,
            &left.owner_fields,
            &left.source,
            left.access == crate::native_return_borrows::BorrowAccess::Exclusive,
            &left.path,
            &left.guards,
            left.whole,
        )
            .cmp(&(
                right.root,
                &right.owner_fields,
                &right.source,
                right.access == crate::native_return_borrows::BorrowAccess::Exclusive,
                &right.path,
                &right.guards,
                right.whole,
            ))
    });
    loans.dedup_by(|right, left| {
        if right.root != left.root
            || right.owner_fields != left.owner_fields
            || right.source != left.source
            || right.access != left.access
            || right.path != left.path
            || right.guards != left.guards
            || right.whole != left.whole
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
    if loan.whole && loan.path.is_empty() { return true; }
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::native_view_loans::LifetimeSource;

    fn recursive() -> Loan {
        Loan { whole: true, root: 0, owner_fields: vec![], source: LifetimeSource::Carried(vec![]),
            access: crate::native_return_borrows::BorrowAccess::ReadOnly, creation: Span::default(),
            retained: true, path: vec![], guards: vec![] }
    }

    #[test]
    fn recursive_loan_survives_deep_projection_without_reaching_outer_siblings() {
        let mut loan = recursive();
        wrap(&mut loan, &Segment::Field("chain".into()));
        assert!(!select(&mut loan.clone(), &Segment::Field("other".into())));
        assert!(select(&mut loan, &Segment::Field("chain".into())));
        for _ in 0..256 {
            assert!(select(&mut loan, &Segment::Field("children".into())));
            assert!(select(&mut loan, &Segment::Element));
            assert!(loan.path.is_empty());
        }
        assert_eq!(loan.root, 0);
        assert_eq!(loan.source, LifetimeSource::Carried(vec![]));
    }

    #[test]
    fn moving_a_descendant_does_not_release_whole_record_loan() {
        let mut loans = vec![recursive()];
        assert!(native_view_places::under(&loans[0], &["children".into()]));
        native_view_places::consume(&mut loans, &["children".into()]);
        assert!(loans[0].retained);
        native_view_places::consume(&mut loans, &[]);
        assert!(!loans[0].retained);
    }

    #[test]
    fn recursive_and_exact_loans_keep_distinct_projection_contracts() {
        let mut exact = recursive();
        exact.whole = false;
        let mut loans = vec![recursive(), exact];
        normalize(&mut loans);
        assert_eq!(loans.len(), 2);
        assert!(!select(loans.iter_mut().find(|loan| !loan.whole).unwrap(), &Segment::Element));
    }
}
