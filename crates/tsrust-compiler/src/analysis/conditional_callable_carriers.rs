//! A common owned carrier preserves distinct native environment origins.
use super::*;
impl Analyzer {
 pub(super) fn common_owned_conditional_family(&mut self,left:Type,right:Type,span:Span)->Result<Option<usize>,Diagnostic> {
  let (Type::Closure(a),Type::Closure(b))=(left,right) else {return Ok(None);};
  let eligible=|origin:usize|!self.deferred_generic_values.contains_key(&origin) && !self.closures[origin].borrowed && !self.closures[origin].polymorphic && !self.closures[origin].function.template;
  if !eligible(a)||!eligible(b) {return Ok(None);}
  if self.is_task_type(left)!=self.is_task_type(right){return Err(Diagnostic::new("task-type","Conditional alternatives must both be tasks or both ordinary callbacks.",span));}
  let once=self.closures[a].invocation==Mode::Consume || self.closures[b].invocation==Mode::Consume;
  let mut annotation=self.checked_value_annotation(left,span)?;
  if once && !self.is_task_type(left) && let crate::type_syntax::ValueAnnotationKind::Callable {ownership,..}=&mut annotation.kind {*ownership=crate::type_syntax::CallableOwnership::OwnedOnce;}
  Ok(self.callables.resolver(&self.record_names,&self.unions).storage_declaration(&annotation)?.and_then(|(ty,_)|if let Type::StoredCallable(family)=ty {Some(family)}else{None}))
 }
}
