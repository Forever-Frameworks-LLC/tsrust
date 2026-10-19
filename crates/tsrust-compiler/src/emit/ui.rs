//! Scalar state is emitted per checked payload; public component code sees no handles.
use super::*;
impl Emitter<'_> {
 pub(super) fn ui_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];
  if binding.module!="std:ui-runtime" || !matches!(binding.name,"signal"|"handle"|"get"|"set"|"ownState"){return None;}
  self.native_bindings.borrow_mut().insert(id);
  let read=|arg:&Expr|self.expr(fun,arg,Mode::Read);
  let scalar=|arg:&Expr|{let text=read(arg);if arg.ty==Some(Type::String){format!("String::clone({text})")}else{text}};
  let call=match binding.name {
   "signal"=>return Some(format!("tsrust_native_ui::Signal::new({})",scalar(&args[0]))),
   "handle"=>return Some(format!("({}).handle()",read(&args[0]))),
   "get"=>format!("({}).get()",read(&args[0])),
   "set"=>format!("{{let state={};let next={};state.set(next)}}",read(&args[0]),scalar(&args[1])),
   "ownState"=>format!("{{let node={};let state={};node.own_state(state)}}",self.expr(fun,&args[0],Mode::Consume),self.expr(fun,&args[1],Mode::Consume)),
   _=>unreachable!(),
  };
  let union=expr.ty.unwrap().union().expect("UI result");let members=&self.program.unions[union.index()].members;let [ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();
  let wrap=|record:usize,payload:&str|{let tag=members.iter().position(|ty|*ty==Type::Record(record)).unwrap();let fields=if self.program.records[record].fields.is_empty(){String::new()}else{format!("field0:{payload},")};format!("Union{}::V{tag}(Record{}{{{fields}{}}})",union.index(),self.native_record_declaration(record),self.schema_record_order(record).trim_start_matches(','))};
  let Type::Record(error)=self.program.records[err].fields[0].ty.unwrap() else{unreachable!()};
  let failure=format!("Record{}{{field0:String::Borrowed(\"UiError\"),field1:String::Owned(message){}}}",self.native_record_declaration(error),self.schema_record_order(error));
  Some(format!("match {call}{{Ok(value)=>{},Err(message)=>{}}}",wrap(ok,"value"),wrap(err,&failure)))
 }
}
