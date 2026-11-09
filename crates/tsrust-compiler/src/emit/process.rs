//! Convert checked process records and move owned input exactly once.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 fn process_record(&self,id:usize,fields:Vec<String>)->String{let fields=fields.into_iter().enumerate().map(|(index,value)|format!("field{index}:{value}")).collect::<Vec<_>>().join(",");format!("Record{}{{{fields}{}}}",self.native_record_declaration(id),self.schema_record_order(id))}
 fn process_value(&self,ty:Type,value:&str)->String{match ty{
  Type::Record(id)=>match self.program.records[id].native_type{
   Some(NativeTypeId::Bytes)=>format!("Record{}{{bytes:{value}}}",self.native_record_declaration(id)),
   Some(NativeTypeId::ProcessExitStatus)=>self.process_record(id,self.program.records[id].fields.iter().map(|field|{let name=field.name.text.as_str();self.process_value(field.ty.unwrap(),&format!("({value}).{name}"))}).collect()),
   Some(NativeTypeId::ProcessOutput)=>self.process_record(id,self.program.records[id].fields.iter().map(|field|self.process_value(field.ty.unwrap(),&format!("({value}).{}",field.name.text))).collect()),
   Some(NativeTypeId::ProcessError)=>self.process_record(id,vec![format!("String::Borrowed(({value}).code)"),format!("String::Borrowed(({value}).operation)"),format!("String::Owned(({value}).message)"),self.process_value(self.program.records[id].fields[3].ty.unwrap(),&format!("({value}).os_code"))]),
   _=>value.into(),
  },
  Type::Union(id)=>{let members=&self.program.unions[id.index()].members;let absent=members.iter().position(|ty|*ty==Type::Undefined).expect("process optional output");let present=1-absent;let converted=self.process_value(members[present],"value");format!("match {value}{{Some(value)=>Union{}::V{present}({converted}),None=>Union{}::V{absent}(TsrUndefined)}}",id.index(),id.index())},
  _=>value.into(),
 }}
 fn process_result(&self,ty:Type,call:&str)->String{let Type::Union(id)=ty else{unreachable!("process Result")};let members=&self.program.unions[id.index()].members;let[ok,err]=crate::result_identity::variants(&self.program.records,members).unwrap();let tag=|record|members.iter().position(|ty|*ty==Type::Record(record)).unwrap();let fields=self.program.records[ok].fields.first().map(|field|vec![self.process_value(field.ty.unwrap(),"value")]).unwrap_or_default();let success=self.process_record(ok,fields);let failure=self.process_record(err,vec![self.process_value(self.program.records[err].fields[0].ty.unwrap(),"error")]);format!("match {call}{{Ok(value)=>Union{}::V{}({success}),Err(error)=>Union{}::V{}({failure})}}",id.index(),tag(ok),id.index(),tag(err))}
 fn process_options(&self,id:usize,value:&str,owned:bool)->String{
  let convert=|position:usize|{let Type::Union(union)=self.program.records[id].fields[position].ty.unwrap()else{unreachable!("optional command option")};let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(tag,member)|{
   let output=if *member==Type::Undefined{"None".into()}else{let converted:String=match position{
    0=>if owned{"value.into_owned()".into()}else{"value.as_ref()".into()},
    1=>if owned{"value.tsr_owned_items().map(|entry|(entry.field0.into_owned(),entry.field1.into_owned())).collect::<Vec<_>>()".into()}else{"value.iter().map(|entry|(entry.field0.as_ref(),entry.field1.as_ref())).collect::<Vec<_>>()".into()},
    2|7|8=>if owned{"value".into()}else{"*value".into()},
    3..=5=>"match value.as_ref(){\"inherit\"=>tsrust_native_process::StdioMode::Inherit,\"null\"=>tsrust_native_process::StdioMode::Null,\"pipe\"=>tsrust_native_process::StdioMode::Pipe,_=>unreachable!(\"checked stdio literal\")}".into(),
    6=>{let Type::Record(record)=member else{unreachable!("command input")};match(self.program.records[*record].native_type,owned){(Some(NativeTypeId::Bytes),true)=>"tsrust_native_process::Input::Bytes(value.bytes)".into(),(Some(NativeTypeId::Bytes),false)=>"tsrust_native_process::InputRef::Bytes(&value.bytes)".into(),(Some(NativeTypeId::IoReader),true)=>"tsrust_native_process::Input::Reader(value)".into(),(Some(NativeTypeId::IoReader),false)=>"tsrust_native_process::InputRef::Reader(value)".into(),_=>unreachable!("checked process input")}},
    _=>unreachable!(),
   };format!("Some({converted})")};format!("Union{}::V{tag}(value)=>{output}",union.index())}).collect::<Vec<_>>().join(",");format!("match {}({value}).field{position}{{{arms}}}",if owned{""}else{"&"})};
  let fields=["cwd","env","clear_env","stdin","stdout","stderr","input","deadline_ms","max_output_bytes"].iter().enumerate().map(|(position,name)|{let converted=convert(position);format!("{name}:{}",if matches!(position,1|2){format!("({converted}).unwrap_or_default()")}else{converted})}).collect::<Vec<_>>().join(",");format!("tsrust_native_process::{}{{{fields}}}",if owned{"Options"}else{"OptionsRef"})
 }
 pub(super) fn process_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:process"{return None;}self.native_bindings.borrow_mut().insert(id);let values=args.iter().zip(binding.parameters).map(|(value,parameter)|self.expr(fun,value,parameter.access)).collect::<Vec<_>>();
  let call=if matches!(binding.name,"spawn"|"__run"){
   let owned=binding.name=="__run";
   let options=if let Some(argument)=args.get(2){let optional=self.native_optional_argument(&binding.parameters[2],"process_options_input".into(),argument.ty.unwrap());let ty=argument.ty.unwrap();let record=match ty{Type::Record(record)=>Some(record),Type::Union(union)|Type::BorrowedUnion(union)=>self.program.unions[union.index()].members.iter().find_map(|ty|if let Type::Record(record)=ty{Some(*record)}else{None}),_=>None};if let Some(record)=record{format!("({optional}).map(|options|{}).unwrap_or_default()",self.process_options(record,"options",owned))}else{format!("{{let _={optional};Default::default()}}")}}else{"Default::default()".into()};
   let setup=if args.len()>2{format!("let process_options_input={};",values[2])}else{String::new()};format!("{{let process_program={};let process_args={};{setup}let process_options={options};let process_arguments=process_args.iter().map(|value|value.as_ref()).collect::<Vec<&str>>();tsrust_native_process::{}(process_program.as_ref(),&process_arguments,process_options){}}}",values[0],values[1],if owned{"run"}else{"spawn"},if owned{".await"}else{""})
  }else{let receiver=&values[0];let method=match binding.name{"__id"=>"id","__takeStdin"=>"take_stdin","__takeStdout"=>"take_stdout","__takeStderr"=>"take_stderr","__wait"=>"wait","__tryWait"=>"try_wait","__kill"=>"kill",_=>unreachable!()};format!("({receiver}).{method}(){}",if matches!(binding.name,"__wait"|"__kill"){".await"}else{""})};
  Some(if matches!(binding.result,crate::native_bindings::NativeType::Structured(crate::native_signatures::Signature::Result(..))){self.process_result(expr.ty.unwrap(),&call)}else{self.process_value(expr.ty.unwrap(),&call)})
 }
}
