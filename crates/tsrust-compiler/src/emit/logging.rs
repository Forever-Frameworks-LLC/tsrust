//! Every field schema is compiled into ordinary static tracing callsites.
use super::*;
impl Emitter<'_>{
 fn logging_text(&self,ty:Type,value:&str,optional:bool)->String{
  if let Some(id)=ty.union(){let tag=if matches!(ty,Type::BorrowedUnion(_)){"ViewUnion"}else{"Union"};let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(index,&member)|format!("{tag}{}::V{index}(value)=>{}",id.index(),self.logging_text(member,"value",optional))).collect::<Vec<_>>().join(",");return format!("match &{value}{{{arms}}}");}
  if ty==Type::Undefined{return "None".into();}let text=if let Type::Literal(id,crate::literal_types::LiteralKind::String)=ty{let crate::literal_types::LiteralValue::String(value)=self.program.records[id].literal_definition.as_ref().unwrap()else{unreachable!()};format!("{value:?}")}else{format!("({value}).as_ref()")};if optional{format!("Some({text})")}else{text}
 }
 pub(super) fn logging_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:log-runtime"{return None;}self.native_bindings.borrow_mut().insert(id);
  let values=args.iter().zip(binding.parameters).map(|(argument,parameter)|self.expr(fun,argument,parameter.access)).collect::<Vec<_>>();
  let call=match binding.name{
   "create"=>{let Type::Record(record)=args[0].ty.unwrap()else{unreachable!()};let options=&self.program.records[record];let text=|index:usize,optional|self.logging_text(options.fields[index].ty.unwrap(),&format!("options.field{index}"),optional);format!("{{let options={};tsrust_native_logging::Logger::create({},{},{})}}",values[0],text(0,false),text(1,true),text(2,true))},
   "enabled"=>format!("({}).enabled(({}).as_ref())",values[0],values[1]),
   "flush"=>format!("({}).flush()",values[0]),"close"=>format!("({}).close()",values[0]),
   "child"|"emit"=>{let index=if binding.name=="child"{1}else{2};let Type::Record(record)=args[index].ty.unwrap()else{unreachable!()};let payload_type=self.native_type(args[index].ty.unwrap());let fields=self.program.records[record].fields.iter().enumerate().map(|(index,field)|{let value=if field.ty.unwrap().scalar_base()==Type::String{format!("payload.field{index}.as_ref()")}else{format!("payload.field{index}")};format!("{:?}={value}",field.name.text)}).collect::<Vec<_>>().join(",");
    if binding.name=="child"{format!("{{let logger={};let payload:{payload_type}={};logger.child(move|emit|{{let span=tsr_tracing::debug_span!(\"child\",{fields});span.in_scope(emit);}})}}",values[0],values[1])}
    else{let branches=[("debug","DEBUG"),("info","INFO"),("warn","WARN"),("error","ERROR")].into_iter().map(|(level,native)|format!("{level:?}=>tsr_tracing::event!(tsr_tracing::Level::{native},{fields})")).collect::<Vec<_>>().join(",");format!("{{let logger={};let level={};let payload:{payload_type}={};logger.emit(||match level.as_ref(){{{branches},_=>unreachable!(\"checked logging level\")}});}}",values[0],values[1],values[2])}
   },_=>unreachable!("checked logging leaf")
  };
  Some(if matches!(binding.name,"create"|"flush"|"close"){self.http_result(expr.ty.unwrap(),&call)}else{call})
 }
}
