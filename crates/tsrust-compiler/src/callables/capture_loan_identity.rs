//! Stable capture-loan identities shared by cloned checker paths.
use crate::{Mode, Span};
use std::{cell::RefCell, collections::{HashMap, HashSet}, rc::Rc};

pub(crate) type Loan = (usize, Mode, Span);

/// Lexical membership remains valid when earlier loans are removed or reordered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Boundary(HashSet<Loan>);
impl Boundary {
    pub(crate) fn capture(loans: &[Loan]) -> Self {
        Self(loans.iter().copied().collect())
    }
    pub(crate) fn unwind(&self, loans: &mut Vec<Loan>) {
        loans.retain(|loan| self.0.contains(loan));
    }
    pub(crate) fn added(&self, loans: &[Loan]) -> Vec<Loan> {
        loans.iter().filter(|loan| !self.0.contains(loan)).copied().collect()
    }
}

#[derive(Clone, Default)]
pub(crate) struct Registry(Rc<RefCell<Entries>>);

#[derive(Default)]
struct Entries {
    ids: HashMap<Loan, usize>,
    loans: Vec<Loan>,
}

impl Registry {
    pub(crate) fn intern(&self, loan: Loan) -> usize {
        let mut entries = self.0.borrow_mut();
        if let Some(id) = entries.ids.get(&loan) {
            return *id;
        }
        let id = entries.loans.len();
        entries.loans.push(loan);
        entries.ids.insert(loan, id);
        id
    }
    pub(crate) fn get(&self, loan: Loan) -> Option<usize> {
        self.0.borrow().ids.get(&loan).copied()
    }
    pub(crate) fn loan(&self, id: usize) -> Option<Loan> {
        self.0.borrow().loans.get(id).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lexical_boundary_does_not_keep_new_loans_after_entry_removal() {
        let first = (0, Mode::Read, Span::new(1, 2));
        let surviving = (1, Mode::Mutate, Span::new(3, 4));
        let local = (2, Mode::Read, Span::new(5, 6));
        let mut loans = vec![first, surviving];
        let boundary = Boundary::capture(&loans);
        loans.remove(0);
        loans.push(local);
        boundary.unwind(&mut loans);
        assert_eq!(loans, vec![surviving]);
    }
    #[test]
    fn branch_identity_restores_the_exact_loan_after_vector_disposal() {
        let registry = Registry::default();
        let loan = (4, Mode::Mutate, Span::new(10, 20));
        let branch = registry.clone();
        let id = branch.intern(loan);
        assert_eq!(registry.loan(id), Some(loan));
        assert_eq!(registry.loan(id + 1), None);
        let later = registry.intern((4, Mode::Read, Span::new(10, 20)));
        assert_eq!(branch.loan(id), Some(loan));
        assert_eq!(branch.loan(later), Some((4, Mode::Read, Span::new(10, 20))));
    }
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
