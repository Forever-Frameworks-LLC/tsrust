//! Read-only values own their evaluator; retained node bindings hold weak references.
use std::{cell::RefCell,rc::{Rc,Weak}};
type Evaluate<T> = RefCell<Box<dyn FnMut()->Result<T,String>>>;
enum Reference<T> {Owner(Rc<Evaluate<T>>),Binding(Weak<Evaluate<T>>)}
pub struct Computed<T>(Reference<T>);
impl<T:'static> Computed<T> {
 pub fn new(evaluate:impl FnMut()->Result<T,String>+'static)->Self {Self(Reference::Owner(Rc::new(RefCell::new(Box::new(evaluate)))))}
 pub fn constant(value:T)->Self where T:Clone {Self::new(move||Ok(value.clone()))}
 pub fn handle(&self)->Self {Self(Reference::Binding(match &self.0{Reference::Owner(value)=>Rc::downgrade(value),Reference::Binding(value)=>value.clone()}))}
 pub fn get(&self)->Result<T,String>{
  let value=match &self.0{Reference::Owner(value)=>Some(Rc::clone(value)),Reference::Binding(value)=>value.upgrade()}.ok_or("Computed UI value is disposed")?;
  let mut evaluate=value.try_borrow_mut().map_err(|_|"Computed UI value recursively reads itself")?;evaluate()
 }
}
#[cfg(test)]mod tests {
 use super::*;
 #[test]fn values_are_live_and_bindings_never_retain_evaluators(){
  struct Probe(Rc<RefCell<usize>>);impl Drop for Probe{fn drop(&mut self){*self.0.borrow_mut()+=1;}}
  let released=Rc::new(RefCell::new(0));let probe=Probe(released.clone());let state=crate::Signal::new(1);let read=state.handle();
  let owner=Computed::new(move||{let _=&probe;read.get().map(|value|value*2)});let binding=owner.handle();let nested=binding.handle();assert_eq!(nested.get(),Ok(2));state.set(4).unwrap();assert_eq!(binding.get(),Ok(8));drop(owner);assert_eq!(*released.borrow(),1);assert_eq!(nested.get(),Err("Computed UI value is disposed".into()));
 }
 #[test]fn nullable_exact_values_errors_and_recursive_reads_are_checked(){
  assert_eq!(Computed::constant(Some(i64::MAX)).get(),Ok(Some(i64::MAX)));assert_eq!(Computed::constant(None::<bool>).get(),Ok(None));
  let failed=Computed::<i64>::new(||Err("application getter failed".into()));assert_eq!(failed.get(),Err("application getter failed".into()));
  let slot=Rc::new(RefCell::new(None::<Computed<i64>>));let recursive=slot.clone();let owner=Computed::new(move||recursive.borrow().as_ref().unwrap().get());*slot.borrow_mut()=Some(owner.handle());assert_eq!(owner.get(),Err("Computed UI value recursively reads itself".into()));
 }
}
