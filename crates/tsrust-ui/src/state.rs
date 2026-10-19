//! Component state has one strong owner; bindings and captures use weak references.
use std::{cell::RefCell, rc::{Rc, Weak}};

pub struct Signal<T = i64>(Reference<T>);
pub struct TextBinding(Box<dyn Fn() -> Result<String,String>>);
impl TextBinding { pub fn value(&self) -> Result<String,String> { (self.0)() } }
enum Reference<T> { Owner(Rc<RefCell<T>>), Binding(Weak<RefCell<T>>) }
impl<T: Clone> Signal<T> {
    pub fn new(value: T) -> Self { Self(Reference::Owner(Rc::new(RefCell::new(value)))) }
    pub fn handle(&self) -> Self {
        Self(Reference::Binding(match &self.0 {
            Reference::Owner(value) => Rc::downgrade(value),
            Reference::Binding(value) => value.clone(),
        }))
    }
    pub fn text_binding(&self, format: fn(&T) -> String) -> TextBinding where T: 'static {
        let binding=self.handle();
        TextBinding(Box::new(move || binding.get().map(|value|format(&value))))
    }
    fn live(&self) -> Option<Rc<RefCell<T>>> {
        match &self.0 { Reference::Owner(value) => Some(Rc::clone(value)), Reference::Binding(value) => value.upgrade() }
    }
    pub fn value(&self) -> Option<T> { self.live().map(|value| value.borrow().clone()) }
    pub fn get(&self) -> Result<T,String> { self.value().ok_or_else(||"UI state is disposed".into()) }
    pub fn set(&self, next:T) -> Result<(),String> {
        let value=self.live().ok_or_else(||String::from("UI state is disposed"))?;
        *value.borrow_mut()=next;Ok(())
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

#[cfg(test)]
mod scalar_tests {
 use super::*;
 #[test]
 fn strings_boolean_numbers_and_text_bindings_release_with_the_owner() {
  let text=Signal::new(String::from("first"));let weak=text.handle();let binding=text.text_binding(Clone::clone);
  let snapshot=text.get().unwrap();text.set("second".into()).unwrap();assert_eq!(snapshot,"first");assert_eq!(binding.value().unwrap(),"second");
  let flag=Signal::new(false);assert_eq!(flag.get().unwrap(),false);flag.set(true).unwrap();assert!(flag.get().unwrap());
  let number=Signal::new(-0.0f64);assert_eq!(number.get().unwrap().to_bits(),(-0.0f64).to_bits());number.set(f64::NAN).unwrap();assert!(number.get().unwrap().is_nan());
  drop(text);assert!(weak.get().is_err());assert!(weak.set("gone".into()).is_err());assert!(binding.value().is_err());
 }
}
