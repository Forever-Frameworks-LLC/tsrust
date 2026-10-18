//! Component state has one strong owner; bindings and captures use weak references.
use std::{cell::Cell, rc::{Rc, Weak}};

pub struct Signal(Reference);
enum Reference { Owner(Rc<Cell<i64>>), Binding(Weak<Cell<i64>>) }
impl Signal {
    pub fn new(value: i64) -> Self { Self(Reference::Owner(Rc::new(Cell::new(value)))) }
    pub fn handle(&self) -> Self {
        Self(Reference::Binding(match &self.0 {
            Reference::Owner(value) => Rc::downgrade(value),
            Reference::Binding(value) => value.clone(),
        }))
    }
    fn live(&self) -> Option<Rc<Cell<i64>>> {
        match &self.0 { Reference::Owner(value) => Some(Rc::clone(value)), Reference::Binding(value) => value.upgrade() }
    }
    pub fn value(&self) -> Option<i64> { self.live().map(|value| value.get()) }
    pub fn get(&self) -> Result<i64,String> { self.value().ok_or_else(||"UI state is disposed".into()) }
    pub fn set(&self, next:i64) -> Result<(),String> {
        let value=self.live().ok_or_else(||String::from("UI state is disposed"))?;
        value.set(next);Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_state_checked_updates_and_nonretaining_handles() {
        let first=Signal::new(0);let second=Signal::new(i64::MAX);
        let a=first.handle();let nested=a.handle();let b=second.handle();
        assert!(a.set(1).is_ok());assert_eq!(nested.value(),Some(1));
        assert!(b.set(i64::MAX).is_ok());assert_eq!(b.value(),Some(i64::MAX));
        drop(first);assert_eq!(a.value(),None);assert!(nested.set(1).is_err());assert!(nested.get().is_err());
        assert_eq!(b.value(),Some(i64::MAX));
    }
}
