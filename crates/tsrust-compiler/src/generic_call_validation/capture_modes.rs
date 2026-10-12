//! Only native-checked capture representations can authorize environment storage.
use super::*;
use crate::{Mode,local_capture_plan::BindingId};
#[derive(Clone,PartialEq,Eq,Debug)]
pub(super) struct Slot {pub binding:BindingId,pub storage:Mode,pub access:Mode}
#[derive(Clone,PartialEq,Eq,Debug)]
pub(super) struct Environment {pub escapes:bool,pub borrowed:bool,pub polymorphic:bool,pub invocations:Vec<Mode>,pub slots:Vec<Slot>}
#[derive(Default)]
pub(super) struct Facts {entries:HashMap<(usize,usize),Option<Environment>>}
impl Facts {
 pub(super) fn observe(&mut self,checked:&crate::analysis::CheckedProgram,source:&effect_bodies::Index)->bool {
  let mut changed=false;
  for closure in &checked.closures {
   let key=(closure.span.start,closure.span.end);
   let slots=closure.captures.iter().map(|capture| {
    let binding=closure.function.bindings.get(capture.inner)?;
    Some(Slot {binding:source.binding(&binding.name,capture.span)?,storage:capture.storage,access:capture.access})
   }).collect::<Option<Vec<_>>>();
   let environment=slots.and_then(|mut slots| {
    if closure.function.template {return None;}
    slots.sort_by_key(|slot|slot.binding);
    if slots.windows(2).any(|pair|pair[0].binding==pair[1].binding) {return None;}
    let mut invocations=vec![closure.invocation];invocations.extend(closure.methods.iter().map(|method|method.invocation));invocations.sort();invocations.dedup();
    Some(Environment {escapes:closure.escapes,borrowed:closure.borrowed,polymorphic:closure.polymorphic,invocations,slots})
   });
   match self.entries.entry(key) {
    std::collections::hash_map::Entry::Vacant(entry)=>{entry.insert(environment);changed=true;},
    std::collections::hash_map::Entry::Occupied(mut entry)=>{
     if entry.get().is_some() && entry.get()!=&environment {entry.insert(None);changed=true;}
    },
   }
  }
  changed
 }
 pub(super) fn get(&self,span:Span)->Option<&Environment> {self.entries.get(&(span.start,span.end)).and_then(Option::as_ref)}
}
#[cfg(test)]
mod tests {
 use super::*;
 fn checked_facts(source:&str)->(String,Vec<(Span,Environment)>) {
  let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
  let program=crate::syntax::parse(&source).unwrap();let index=effect_bodies::Index::new(&program).unwrap();let checked=crate::analysis::analyze(program).unwrap();
  let mut facts=Facts::default();assert!(facts.observe(&checked,&index));assert!(!facts.observe(&checked,&index),"same checked evidence must be idempotent");
  let environments=checked.closures.iter().filter(|closure|!closure.captures.is_empty()).map(|closure|(closure.span,facts.get(closure.span).expect("checked capture facts must map to lexical identities").clone())).collect::<Vec<_>>();assert!(!environments.is_empty());(source,environments)
 }
 #[test] fn mutable_scalar_captures_borrow_locally_and_copy_when_escaping() {
  for (source,storage,escapes) in [("function main(){let value=0;const action=()=>{value=value+1;return value;};console.log(action());}",Mode::Mutate,false),("function make(start:int){let value=start;return ()=>{value=value+1;return value;};}function main(){const action=make(0);console.log(action(),action());}",Mode::Copy,true)] {
   let (_,facts)=checked_facts(source);assert!(facts.iter().any(|(_,environment)|environment.escapes==escapes && environment.slots.iter().any(|slot|slot.storage==storage && slot.access==Mode::Mutate)));
  }
 }
 #[test] fn record_captures_borrow_locally_and_transfer_when_escaping() {
  for (source,storage,escapes) in [("type Item={id:int;};function main(){const item:Item={id:7};const action=()=>item.id;console.log(action());}",Mode::Read,false),("type Item={id:int;};function make(item:Item){return ()=>item.id;}function main(){const action=make({id:7});console.log(action());}",Mode::Consume,true)] {
   let (_,facts)=checked_facts(source);assert!(facts.iter().any(|(_,environment)|environment.escapes==escapes && environment.slots.iter().any(|slot|slot.storage==storage)));
  }
 }
 #[test] fn specialization_conflicts_never_certify_one_storage_mode() {
  let source=crate::format("type Item={id:int;};function make<T>(value:T){return ()=>value;}function main(){const number=make<int>(7);const item=make<Item>({id:9});console.log(number(),item().id);}").unwrap();
  let program=crate::syntax::parse(&source).unwrap();let index=effect_bodies::Index::new(&program).unwrap();let checked=crate::analysis::analyze(program).unwrap();
  let mut groups:HashMap<(usize,usize),Vec<Mode>>=HashMap::new();for closure in &checked.closures {for capture in &closure.captures {groups.entry((closure.span.start,closure.span.end)).or_default().push(capture.storage);}}
  let (span,_)=groups.iter().find(|(_,modes)|modes.contains(&Mode::Copy)&&modes.contains(&Mode::Consume)).expect("same source capture has distinct checked representations across specializations");
  let mut facts=Facts::default();assert!(facts.observe(&checked,&index));assert!(facts.get(Span::new(span.0,span.1)).is_none());assert!(!facts.observe(&checked,&index),"uncertainty is absorbing");
 }
 #[test] fn missing_source_identity_stays_unknown_even_after_complete_observation() {
  let source=crate::format("function main(){let value=0;const action=()=>{value=value+1;return value;};console.log(action());}").unwrap();let program=crate::syntax::parse(&source).unwrap();let index=effect_bodies::Index::new(&program).unwrap();let checked=crate::analysis::analyze(program).unwrap();
  let span=checked.closures.iter().find(|closure|!closure.captures.is_empty()).unwrap().span;let mut facts=Facts::default();facts.observe(&checked,&effect_bodies::Index::default());assert!(facts.get(span).is_none());facts.observe(&checked,&index);assert!(facts.get(span).is_none());
 }

}
