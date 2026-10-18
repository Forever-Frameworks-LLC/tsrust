//! A component owns state; bindings and handlers retain only weak handles.
use std::{cell::Cell, rc::{Rc, Weak}};

pub struct Signal(Rc<Cell<i64>>);
pub struct SignalHandle(Weak<Cell<i64>>);
impl Signal {
    pub fn new(value: i64) -> Self { Self(Rc::new(Cell::new(value))) }
    pub fn handle(&self) -> SignalHandle { SignalHandle(Rc::downgrade(&self.0)) }
}
impl SignalHandle {
    pub fn value(&self) -> Option<i64> { self.0.upgrade().map(|value| value.get()) }
    /// Rejected updates leave state unchanged, including integer overflow.
    pub fn add(&self, delta: i64) -> bool {
        let Some(value) = self.0.upgrade() else { return false; };
        let Some(next) = value.get().checked_add(delta) else { return false; };
        value.set(next);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_state_checked_updates_and_nonretaining_handles() {
        let first=Signal::new(0);let second=Signal::new(i64::MAX);
        let a=first.handle();let b=second.handle();
        assert!(a.add(1));assert_eq!(a.value(),Some(1));
        assert!(!b.add(1));assert_eq!(b.value(),Some(i64::MAX));
        drop(first);assert_eq!(a.value(),None);assert!(!a.add(1));
        assert_eq!(b.value(),Some(i64::MAX));
    }
}
