//! Point-sensitive capture facts are separate from stable allocation records.
use super::*;
use std::collections::BTreeMap;
#[derive(Clone,Copy,PartialEq,Eq,Hash,Debug)]
pub(super) enum Instance { OwnedSingle, Merged }
#[derive(Clone,PartialEq,Eq,Hash)]
#[cfg_attr(not(test), expect(dead_code, reason = "Live and consumed states are exercised by retained point-sensitive factory witnesses; production currently joins unknown states."))]
pub(super) enum Capture { Live { bindings:Vec<Binding>, instance:Instance }, Consumed, Unknown }
#[derive(Clone,Default,PartialEq,Eq,Hash)]
pub(super) struct State { pub bindings:Vec<Binding>, pub captures:BTreeMap<environment_store::Handle,Capture> }
impl From<Vec<Binding>> for State {fn from(bindings:Vec<Binding>)->Self {Self {bindings,captures:BTreeMap::new()}}}
impl std::ops::Deref for State {type Target=Vec<Binding>;fn deref(&self)->&Self::Target {&self.bindings}}
impl std::ops::DerefMut for State {fn deref_mut(&mut self)->&mut Self::Target {&mut self.bindings}}
#[cfg(test)]
fn canonical(bindings:&[Binding])->Option<Vec<Binding>> {
    if bindings.iter().any(|binding|binding.identity.is_none()) {return None;}
    let mut result=bindings.to_vec();result.sort_by_key(|binding|binding.identity);
    if result.windows(2).any(|pair|pair[0].identity==pair[1].identity) {return None;}
    for binding in &mut result {binding.origins=body_origins::canonical(std::mem::take(&mut binding.origins));}
    Some(result)
}
fn same_slots(left:&[Binding],right:&[Binding])->bool {
    left.len()==right.len() && left.iter().zip(right).all(|(left,right)|left.identity==right.identity && left.name==right.name && left.scope==right.scope)
}
fn joined_bindings(left:&[Binding],right:&[Binding],declarations:&HashMap<String,Contract>)->Option<Vec<Binding>> {
    if !same_slots(left,right) {return None;}
    let mut result=left.to_vec();join(&mut result,right,declarations);
    for (binding,incoming) in result.iter_mut().zip(right) {
        if binding.contract!=incoming.contract {binding.contract=None;}
    }
    Some(result)
}
impl State {
    /// The caller must derive OwnedSingle from checked capture and instance facts.
    #[cfg(test)]
    pub(super) fn install(&mut self,handle:environment_store::Handle,bindings:&[Binding],instance:Instance)->Option<()> {
        if self.captures.contains_key(&handle) {return None;}
        let bindings=canonical(bindings)?;
        self.captures.insert(handle,Capture::Live {bindings,instance});Some(())
    }
    #[cfg(test)]
    pub(super) fn update(&mut self,handle:environment_store::Handle,incoming:&[Binding],declarations:&HashMap<String,Contract>)->Option<()> {
        let incoming=canonical(incoming)?;
        let Capture::Live {bindings,instance}=self.captures.get_mut(&handle)? else {return None;};
        if !same_slots(bindings,&incoming) {return None;}
        *bindings=match instance {Instance::OwnedSingle=>incoming,Instance::Merged=>joined_bindings(bindings,&incoming,declarations)?};Some(())
    }
    #[cfg(test)]
    pub(super) fn consume(&mut self,handle:environment_store::Handle)->Option<()> {
        let capture=self.captures.get_mut(&handle)?;
        let Capture::Live {instance,..}=capture else {return None;};
        *capture=if *instance==Instance::OwnedSingle {Capture::Consumed}else{Capture::Unknown};Some(())
    }
    pub(super) fn join(&mut self,incoming:&Self,declarations:&HashMap<String,Contract>)->Option<bool> {
        let bindings=joined_bindings(&self.bindings,&incoming.bindings,declarations)?;
        let mut captures=BTreeMap::new();
        for handle in self.captures.keys().chain(incoming.captures.keys()) {
            if captures.contains_key(handle) {continue;}
            let value=match (self.captures.get(handle),incoming.captures.get(handle)) {
                (Some(Capture::Live {bindings:left,instance:a}),Some(Capture::Live {bindings:right,instance:b}))=>Capture::Live {
                    bindings:joined_bindings(left,right,declarations)?,
                    instance:if *a==Instance::OwnedSingle && *b==Instance::OwnedSingle {Instance::OwnedSingle}else{Instance::Merged},
                },
                (Some(Capture::Consumed),Some(Capture::Consumed))=>Capture::Consumed,
                _=>Capture::Unknown,
            };
            captures.insert(*handle,value);
        }
        let next=Self {bindings,captures};let changed=*self!=next;*self=next;Some(changed)
    }
}
#[cfg(test)]
mod tests {
 use super::*;
 fn binding(body:usize)->Binding {Binding {alternatives:None,identity:Some(crate::local_capture_plan::BindingId {owner:0,declaration_start:1}),name:"action".into(),scope:Span::new(0,100),targets:vec![Target::Unknown],origins:vec![body_origins::Origin::Body(Span::new(body,body+1))],selection:Selection::default(),elements:None,mutable:true,contract:None}}
 fn handle(store:&mut environment_store::Store,site:usize)->environment_store::Handle {store.allocate(Span::new(site,site+1),Span::new(90,91),&[binding(5)]).unwrap()}
 fn origins(state:&State,handle:environment_store::Handle)->Vec<body_origins::Origin> {match &state.captures[&handle] {Capture::Live {bindings,..}=>bindings[0].origins.clone(),_=>panic!("live capture required")}}
 #[test] fn single_instance_restores_exact_state_without_changing_other_instance() {
  let mut store=environment_store::Store::default();let first=handle(&mut store,20);let second=handle(&mut store,30);let mut state=State::default();
  state.install(first,&[binding(5)],Instance::OwnedSingle).unwrap();state.install(second,&[binding(7)],Instance::OwnedSingle).unwrap();
  state.update(first,&[binding(9)],&HashMap::new()).unwrap();assert!(origins(&state,first)==binding(9).origins);
  state.update(first,&[binding(5)],&HashMap::new()).unwrap();assert!(origins(&state,first)==binding(5).origins);assert!(origins(&state,second)==binding(7).origins);
 }
 #[test] fn merged_instances_keep_old_and_new_alternatives() {
  let mut store=environment_store::Store::default();let first=handle(&mut store,20);let mut state=State::default();state.install(first,&[binding(5)],Instance::Merged).unwrap();
  state.update(first,&[binding(9)],&HashMap::new()).unwrap();assert!(origins(&state,first)==vec![body_origins::Origin::Body(Span::new(5,6)),body_origins::Origin::Body(Span::new(9,10))]);
 }
 #[test] fn consumed_or_absent_handles_cannot_be_updated_or_revived() {
  let mut store=environment_store::Store::default();let first=handle(&mut store,20);let absent=handle(&mut store,30);let mut state=State::default();state.install(first,&[binding(5)],Instance::OwnedSingle).unwrap();state.consume(first).unwrap();
  assert!(state.update(first,&[binding(9)],&HashMap::new()).is_none());assert!(state.install(first,&[binding(5)],Instance::OwnedSingle).is_none());assert!(state.update(absent,&[binding(9)],&HashMap::new()).is_none());
 }
 #[test] fn branch_join_preserves_consumed_and_missing_possibilities() {
  let mut store=environment_store::Store::default();let first=handle(&mut store,20);let mut left=State::default();left.install(first,&[binding(5)],Instance::OwnedSingle).unwrap();let mut right=left.clone();right.consume(first).unwrap();
  assert!(left.join(&right,&HashMap::new()).unwrap());assert!(matches!(left.captures[&first],Capture::Unknown));assert!(!left.join(&right,&HashMap::new()).unwrap());
  let mut live=State::default();live.install(first,&[binding(5)],Instance::OwnedSingle).unwrap();live.join(&State::default(),&HashMap::new()).unwrap();assert!(matches!(live.captures[&first],Capture::Unknown));
 }
 #[test] fn slot_identity_mismatch_rejects_update_without_mutation() {
  let mut store=environment_store::Store::default();let first=handle(&mut store,20);let mut state=State::default();state.install(first,&[binding(5)],Instance::OwnedSingle).unwrap();let before=state.clone();let mut wrong=binding(9);wrong.identity=None;
  assert!(state.update(first,&[wrong],&HashMap::new()).is_none());assert!(state==before);
 }
 #[test] fn branch_joins_converge_commute_and_drop_conflicting_contract_certainty() {
  let mut store=environment_store::Store::default();let first=handle(&mut store,20);
  let mut a=State::default();let mut b=State::default();let mut c=State::default();
  let mut left=binding(5);left.contract=Some(contract(1,&[ParameterInitializer::Required],false,Span::new(5,6)));
  let mut right=binding(9);right.contract=Some(contract(1,&[ParameterInitializer::Required,ParameterInitializer::Required],false,Span::new(9,10)));
  a.install(first,&[left],Instance::OwnedSingle).unwrap();b.install(first,&[right],Instance::Merged).unwrap();c.install(first,&[binding(11)],Instance::OwnedSingle).unwrap();
  let mut ab=a.clone();ab.join(&b,&HashMap::new()).unwrap();let mut ba=b.clone();ba.join(&a,&HashMap::new()).unwrap();assert!(ab==ba);assert!(!ab.join(&b,&HashMap::new()).unwrap());
  let Capture::Live {bindings,instance}= &ab.captures[&first] else {panic!("live");};assert!(bindings[0].contract.is_none());assert_eq!(*instance,Instance::Merged);
  let mut abc=ab;abc.join(&c,&HashMap::new()).unwrap();let mut bc=b;bc.join(&c,&HashMap::new()).unwrap();a.join(&bc,&HashMap::new()).unwrap();assert!(a==abc);
 }

 #[test] fn consuming_a_merged_instance_does_not_claim_every_instance_disposed() {
  let mut store=environment_store::Store::default();let first=handle(&mut store,20);let mut state=State::default();state.install(first,&[binding(5)],Instance::Merged).unwrap();state.consume(first).unwrap();
  assert!(matches!(state.captures[&first],Capture::Unknown));assert!(state.update(first,&[binding(9)],&HashMap::new()).is_none());
 }

}
