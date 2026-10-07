//! Stable capture-loan identities shared by cloned checker paths.
use crate::{Mode, Span};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

pub(crate) type Loan = (usize, Mode, Span);

#[derive(Clone, Default)]
pub(crate) struct Registry(Rc<RefCell<HashMap<Loan, usize>>>);

impl Registry {
    pub(crate) fn intern(&self, loan: Loan) -> usize {
        let mut entries = self.0.borrow_mut();
        let next = entries.len();
        *entries.entry(loan).or_insert(next)
    }
    pub(crate) fn get(&self, loan: Loan) -> Option<usize> {
        self.0.borrow().get(&loan).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cloned_branches_share_identity_without_colliding_new_loans() {
        let registry = Registry::default();
        let first = (0, Mode::Read, Span::new(1, 2));
        let left = registry.clone();
        let right = registry.clone();
        let first_id = left.intern(first);
        assert_eq!(right.intern(first), first_id);
        let second_id = right.intern((0, Mode::Mutate, Span::new(1, 2)));
        let third_id = left.intern((1, Mode::Read, Span::new(1, 2)));
        assert_ne!(first_id, second_id);
        assert_ne!(first_id, third_id);
        assert_ne!(second_id, third_id);
        assert_eq!(registry.get(first), Some(first_id));
    }
    #[test]
    fn identities_survive_loan_vector_removal_and_reordering() {
        let registry = Registry::default();
        let mut loans = vec![
            (0, Mode::Read, Span::new(1, 2)),
            (0, Mode::Read, Span::new(3, 4)),
        ];
        let id = registry.intern(loans[1]);
        loans.remove(0);
        assert_eq!(registry.intern(loans[0]), id);
        assert_ne!(registry.intern((0, Mode::Read, Span::new(5, 6))), id);
    }
}
