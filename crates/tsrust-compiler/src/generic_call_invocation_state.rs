//! A contextual frame retains completed body work independently of its callers.
use super::*;
pub(in super::super) struct InvocationState<'a> {
 frame:invocation_frame::Frame,
 entry:parameter_entry::Entry,
 body:Option<body_state::BodyState<'a>>,
 declaration:Span,
 arguments:Vec<expression_flow::Value>,
 resume_entry:bool,
}

#[cfg(test)]
mod tests {
 use super::*;
 struct Delayed {ready:bool,calls:Vec<String>}
 impl expression_flow::Dispatch for Delayed {
  fn invoke(&mut self,expression:&Expr,_callee:&expression_flow::Value,_arguments:&[expression_flow::Value],state:&FlowState)->Option<expression_flow::Invocation>{
   let ExprKind::Call(callee,_)=&expression.kind else {panic!("call")};
   let ExprKind::Name(name)=&callee.kind else {panic!("name")};
   self.calls.push(name.clone());
   Some(expression_flow::Invocation {normal:if name=="target" && !self.ready {None}else{Some(state.clone())},throwing:vec![],result:expression_flow::Value::index(7),facts:HashMap::new()})
  }
 }
 #[test]
 fn retained_invocation_matches_one_shot_entry_and_exits(){
  for declaration in [
   "function root(first:int=before(),second:int=target()):int{return second;}",
   "function root(first:int=before(),...rest:int[]):int{return first;}",
   "function root(first:int=before()):int{try{return target();}finally{before();}}",
   "function root(first:int=before()):int{try{throw first;}catch(error){return target();}}",
  ] {
   let source=crate::format(&format!("function before():int{{return 1;}}function target():int{{return 7;}}{declaration}function main(){{}}")).unwrap();
   assert_eq!(crate::format(&source).unwrap(),source);
   let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
   let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();let declarations=HashMap::new();
   for arguments in [vec![],vec![expression_flow::Value::index(3),expression_flow::Value::index(4)]] {
    let caller=FlowState::default();let mut reference_dispatch=Delayed {ready:true,calls:vec![]};
    let reference=invoke_with_dispatch(root.name.span,&caller,&arguments,&declarations,&effects,&mut reference_dispatch).unwrap();
    let mut dispatch=Delayed {ready:true,calls:vec![]};
    let mut invocation=InvocationState::new(root.name.span,&caller,&arguments,&declarations,&effects,&mut dispatch).unwrap();
    let outcome=invocation.drive(&declarations,&effects,&mut dispatch).unwrap();
    assert!(outcome==reference,"{declaration}");assert_eq!(dispatch.calls,reference_dispatch.calls);
    let calls=dispatch.calls.clone();assert!(invocation.drive(&declarations,&effects,&mut dispatch).unwrap()==outcome);assert_eq!(dispatch.calls,calls);
   }
  }
 }
 #[test]
 fn pending_default_resumes_after_completed_parameter_prefix(){
  let source=crate::format("function before():int{return 1;}function target():int{return 7;}function root(first:int=before(),second:int=target()):int{return second;}function main(){}").unwrap();
  assert_eq!(crate::format(&source).unwrap(),source);
  let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
  let root=program.functions.iter().find(|function|function.name.text=="root").unwrap();let declarations=HashMap::new();
  let mut dispatch=Delayed {ready:false,calls:vec![]};
  let mut invocation=InvocationState::new(root.name.span,&FlowState::default(),&[],&declarations,&effects,&mut dispatch).unwrap();
  assert!(invocation.drive(&declarations,&effects,&mut dispatch).unwrap().returning.is_none());
  assert_eq!(dispatch.calls,["before","target"]);
  dispatch.ready=true;
  invocation.requeue_entry();
  assert!(invocation.drive(&declarations,&effects,&mut dispatch).unwrap().returning.is_some());
  assert_eq!(dispatch.calls,["before","target","target"]);
 }
}
impl<'a> InvocationState<'a> {
 pub(in super::super) fn new(declaration:Span,caller:&FlowState,arguments:&[expression_flow::Value],declarations:&HashMap<String,Contract>,effects:&'a effect_bodies::Index,dispatch:&mut dyn expression_flow::Dispatch)->Option<Self> {
  if effects.aggregate_writes(declaration) {return None;}
  let mut frame=invocation_frame::Frame::build_with_state(declaration,caller,arguments,effects)?;
  let entry=parameter_entry::prepare(declaration,arguments,&mut frame.state,declarations,effects,dispatch)?;
  let body=if entry.normal {Some(body_state::BodyState::new(effects.body(declaration)?,&frame.state,declarations,effects)?)}else{None};
  Some(Self {frame,entry,body,declaration,arguments:arguments.to_vec(),resume_entry:false})
 }
 pub(in super::super) fn requeue_entry(&mut self)->bool{
  if self.entry.resume_at.is_none(){return false;}
  self.resume_entry=true;true
 }
 pub(in super::super) fn requeue(&mut self,node:usize)->Option<bool>{self.body.as_mut()?.requeue(node)}
 pub(in super::super) fn drive(&mut self,declarations:&HashMap<String,Contract>,effects:&'a effect_bodies::Index,dispatch:&mut dyn expression_flow::Dispatch)->Option<Outcome>{
  if std::mem::take(&mut self.resume_entry) && let Some(position)=self.entry.resume_at {
   let next=parameter_entry::prepare_from(self.declaration,&self.arguments,&mut self.frame.state,declarations,effects,dispatch,position)?;
   self.entry.normal=next.normal;self.entry.resume_at=next.resume_at;
   self.entry.failures.extend(next.failures);
   for (key,incoming) in next.facts {
    let value=match self.entry.facts.remove(&key) {
     Some(Some(previous))=>incoming.and_then(|incoming|resolve_all(previous.into_iter().chain(incoming).map(Target::Contract).collect(),&[],declarations)),
     Some(None)=>None,None=>incoming,
    };self.entry.facts.insert(key,value);
   }
   if self.entry.normal {self.body=Some(body_state::BodyState::new(effects.body(self.declaration)?,&self.frame.state,declarations,effects)?);}
  }
  let mut outcome=if let Some(body)=&mut self.body {body.drive(declarations,effects,dispatch)?}else{Outcome::default()};
  for failure in &self.entry.failures {
   if let Some(previous)=&mut outcome.throwing {previous.join(failure,declarations)?;}else{outcome.throwing=Some(failure.clone());}
  }
  for (key,incoming) in &self.entry.facts {
   let value=match outcome.facts.remove(key) {
    Some(Some(previous))=>incoming.as_ref().and_then(|incoming|resolve_all(previous.into_iter().chain(incoming.iter().cloned()).map(Target::Contract).collect(),&[],declarations)),
    Some(None)=>None,None=>incoming.clone(),
   };outcome.facts.insert(*key,value);
  }
  if let (Some(value),Some(state))=(&mut outcome.returned_value,&mut outcome.returning) && !dispatch.validate_escape(value,state) {return None;}
  outcome.normal=outcome.normal.map(|state|self.frame.project(&state));
  outcome.returning=outcome.returning.map(|state|self.frame.project(&state));
  outcome.throwing=outcome.throwing.map(|state|self.frame.project(&state));
  Some(outcome)
 }
}
