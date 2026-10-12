//! Borrowed branch preparation precedes a single consuming commitment.
use super::issues::Issues;

/// Preparation owns generated defaults and other branch-local state, never input.
/// Aborted carries a propagated exception, which must not trigger another branch.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Preparation<P, E> { Ready(P), Rejected, Aborted(E) }
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Branch<A, B> { First(A), Second(B) }

/// Left-to-right selection keeps one global issue budget across failed branches.
/// A successful alternative discards speculative issues, not earlier siblings.
pub(super) fn prepare_choice<I: ?Sized, A, B, E>(
    input: &I,
    issues: &mut Issues,
    first: impl FnOnce(&I, &mut Issues) -> Preparation<A, E>,
    second: impl FnOnce(&I, &mut Issues) -> Preparation<B, E>,
) -> Preparation<Branch<A, B>, E> {
    let checkpoint = issues.checkpoint();
    match first(input, issues) {
        Preparation::Ready(value) => {
            issues.restore(checkpoint);
            Preparation::Ready(Branch::First(value))
        }
        Preparation::Aborted(error) => {
            issues.restore(checkpoint);
            Preparation::Aborted(error)
        }
        Preparation::Rejected => {
            issues.reject();
            match second(input, issues) {
                Preparation::Ready(value) => {
                    issues.restore(checkpoint);
                    Preparation::Ready(Branch::Second(value))
                }
                Preparation::Aborted(error) => {
                    issues.restore(checkpoint);
                    Preparation::Aborted(error)
                }
                Preparation::Rejected => {
                    issues.reject();
                    Preparation::Rejected
                }
            }
        }
    }
}

/// Commitment transfers input exactly once. Failure cannot reopen branch selection.
pub(super) fn commit<I, P, O, E>(
    input: I,
    prepared: Preparation<P, E>,
    build: impl FnOnce(I, P) -> Preparation<O, E>,
) -> Preparation<O, E> {
    match prepared {
        Preparation::Ready(state) => build(input, state),
        Preparation::Rejected => Preparation::Rejected,
        Preparation::Aborted(error) => Preparation::Aborted(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stdlib::schema::issues::{Code, Kind, OwnedSegment, Segment};
    use std::cell::Cell;

    fn reject(issues: &mut Issues, branch: usize) {
        issues.push(&[Segment::Index(branch)], Code::Literal, Kind::String, Kind::String);
    }

    #[test]
    fn failed_branch_borrows_original_storage_and_selected_branch_moves_it_once() {
        struct Input(String); // Deliberately no Clone implementation.
        let input = Input("original owned storage".repeat(4));
        let pointer = input.0.as_ptr();
        let capacity = input.0.capacity();
        let mut issues = Issues::new(1, 4, 32);
        let prepared: Preparation<_, ()> = prepare_choice(&input, &mut issues,
            |input, issues| { assert_eq!(input.0.as_ptr(), pointer); reject(issues, 0); Preparation::<(), ()>::Rejected },
            |input, _| { assert_eq!(input.0.as_ptr(), pointer); Preparation::Ready(input.0.len()) });
        assert!(issues.is_valid());
        assert!(issues.items.is_empty());
        let output = commit(input, prepared, |input, branch| {
            let Branch::Second(length) = branch else { panic!("wrong branch") };
            assert_eq!(input.0.len(), length);
            Preparation::Ready(input.0)
        });
        let Preparation::Ready(output) = output else { panic!("selection failed") };
        assert_eq!(output.as_ptr(), pointer);
        assert_eq!(output.capacity(), capacity);
    }

    #[test]
    fn successful_later_branch_restores_zero_budget_and_existing_sibling_state() {
        for budget in [0, 1, 2] {
            for sibling in [false, true] {
                let mut issues = Issues::new(budget, 2, 8);
                if sibling { reject(&mut issues, 99); }
                let before = (issues.items.len(), issues.truncated, issues.is_valid());
                let result: Preparation<Branch<(), ()>, ()> = prepare_choice(&(), &mut issues,
                    |_, issues| { reject(issues, 0); reject(issues, 1); Preparation::Rejected },
                    |_, _| Preparation::Ready(()));
                assert_eq!(result, Preparation::Ready(Branch::Second(())));
                assert_eq!((issues.items.len(), issues.truncated, issues.is_valid()), before);
                if sibling && budget > 0 { assert_eq!(issues.items[0].path, [OwnedSegment::Index(99)]); }
            }
        }
    }

    #[test]
    fn rejected_alternatives_share_one_budget_and_preserve_declaration_order() {
        for budget in [0, 1, 2] {
            let mut issues = Issues::new(budget, 2, 8);
            let result: Preparation<Branch<(), ()>, ()> = prepare_choice(&(), &mut issues,
                |_, issues| { reject(issues, 0); Preparation::Rejected },
                |_, issues| { reject(issues, 1); Preparation::Rejected });
            assert_eq!(result, Preparation::Rejected);
            assert!(!issues.is_valid());
            assert_eq!(issues.items.len(), budget);
            assert_eq!(issues.truncated, budget < 2);
            for (index, issue) in issues.items.iter().enumerate() {
                assert_eq!(issue.path, [OwnedSegment::Index(index)]);
            }
        }
        let mut issues = Issues::new(0, 0, 0);
        let result: Preparation<Branch<(), ()>, ()> = prepare_choice(&(), &mut issues,
            |_, _| Preparation::Rejected, |_, _| Preparation::Rejected);
        assert_eq!(result, Preparation::Rejected);
        assert!(!issues.is_valid());
    }

    #[test]
    fn nested_selection_restores_only_its_own_errors() {
        let mut issues = Issues::new(2, 4, 32);
        let result: Preparation<_, ()> = prepare_choice(&(), &mut issues,
            |_, issues| { reject(issues, 0); Preparation::<(), ()>::Rejected },
            |input, issues| prepare_choice(input, issues,
                |_, issues| { reject(issues, 1); Preparation::<(), ()>::Rejected },
                |_, _| Preparation::Ready(7)));
        assert_eq!(result, Preparation::Ready(Branch::Second(Branch::Second(7))));
        assert!(issues.is_valid());
        assert!(issues.items.is_empty());
        assert!(!issues.truncated);
    }

    #[test]
    fn default_state_is_built_once_and_rejected_state_is_released_before_retry() {
        struct Default<'a>(&'a Cell<usize>);
        impl Drop for Default<'_> { fn drop(&mut self) { self.0.set(self.0.get() + 1); } }
        let drops = Cell::new(0);
        let calls = Cell::new(0);
        let mut issues = Issues::new(4, 4, 32);
        let prepared: Preparation<Branch<(), Default<'_>>, ()> = prepare_choice(&(), &mut issues,
            |_, issues| { calls.set(calls.get() + 1); let _default = Default(&drops); reject(issues, 0); Preparation::Rejected },
            |_, _| { assert_eq!(drops.get(), 1); calls.set(calls.get() + 1); Preparation::Ready(Default(&drops)) });
        let output = commit((), prepared, |_, branch| match branch {
            Branch::Second(default) => Preparation::Ready(default),
            Branch::First(()) => panic!("wrong branch"),
        });
        assert_eq!(calls.get(), 2);
        assert_eq!(drops.get(), 1);
        drop(output);
        assert_eq!(drops.get(), 2);
    }

    #[test]
    fn exceptions_abort_selection_and_commit_failure_never_retries() {
        let second_calls = Cell::new(0);
        let mut issues = Issues::new(4, 4, 32);
        let prepared: Preparation<Branch<(), ()>, &str> = prepare_choice(&(), &mut issues,
            |_, _| Preparation::Aborted("factory failed"),
            |_, _| { second_calls.set(1); Preparation::Ready(()) });
        let result = commit((), prepared, |_, _| Preparation::<(), &str>::Ready(()));
        assert_eq!(result, Preparation::Aborted("factory failed"));
        assert_eq!(second_calls.get(), 0);
        let input = String::from("original");
        let prepared: Preparation<Branch<(), ()>, &str> = prepare_choice(&input, &mut issues,
            |_, _| Preparation::Ready(()),
            |_, _| { second_calls.set(1); Preparation::Ready(()) });
        let result = commit(input, prepared, |mut input, _| {
            input.clear(); // A consuming transform may irreversibly change storage.
            Preparation::<(), &str>::Rejected
        });
        assert_eq!(result, Preparation::Rejected);
        assert_eq!(second_calls.get(), 0);
    }
}
