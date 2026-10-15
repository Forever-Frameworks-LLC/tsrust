use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 fn sql_parameter(&self,ty:Type,value:&str)->String{
  let variant=match ty{Type::Int=>"Integer",Type::Number=>"Real",Type::Boolean=>"Boolean",Type::String=>return format!("tsrust_native_sql::Parameter::Text(({value}).as_ref().to_owned())"),Type::Null=>return "tsrust_native_sql::Parameter::Null".into(),Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::Bytes)=>return format!("tsrust_native_sql::Parameter::Blob(({value}).bytes.clone())"),Type::Union(id)=>{
   let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",id.index(),self.sql_parameter(*ty,"value"))).collect::<Vec<_>>().join(",");return format!("match {value}{{{arms}}}");
  },_=>unreachable!("checked SQL parameter")};format!("tsrust_native_sql::Parameter::{variant}(*({value}))")
 }
 pub(super) fn sql_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:sql"{return None;}self.native_bindings.borrow_mut().insert(id);
  let values=args.iter().zip(binding.parameters).map(|(arg,param)|{let value=self.expr(fun,arg,param.access);if arg.ty==Some(Type::String){format!("({value}).as_ref()")}else{value}}).collect::<Vec<_>>();let receiver=&values[0];
  let call=match binding.name{
   "__connect"=>format!("tsrust_native_sql::Pool::connect({receiver},{},{}).await",values.get(1).map(|v|format!("Some({v})")).unwrap_or("None".into()),values.get(2).map(|v|format!("Some({v})")).unwrap_or("None".into())),
   "__execute"|"__query"|"__txexecute"|"__txquery"=>{
    let parameters=if args.len()==3{let Type::Array(array)=args[2].ty.unwrap()else{unreachable!()};let element=array.ty();format!("({}).iter().map(|value: &{}|{}).collect::<Vec<_>>()",values[2],self.native_type(element),self.sql_parameter(element,"value"))}else{"Vec::new()".into()};
    let query=binding.name.ends_with("query");format!("{{let receiver={receiver};let sql={};let parameters={parameters};receiver.{}(sql,&parameters).await{}}}",values[1],if query{"query"}else{"execute"},if query{".map(Array::Heap)"}else{""})
   },
   "__isNull"=>format!("({receiver}).is_null({})",values[1]),
   name=>format!("({receiver}).{}({}){}",name.trim_start_matches('_'),values[1..].join(","),if crate::stdlib::sql::asynchronous(name){".await"}else{""})
  };Some(self.http_result(expr.ty.unwrap(),&call))
 }
 pub(super) fn sql_record(&self,output:&mut String,id:usize)->bool{
  let (name,borrow)=match self.program.records[id].native_type{Some(NativeTypeId::SqlPool)=>("Pool",false),Some(NativeTypeId::SqlTransaction)=>("Transaction",true),Some(NativeTypeId::SqlRow)=>("Row",false),_=>return false};
  let lifetime=if borrow{"<'a>"}else{""};writeln!(output,"type Record{id}{lifetime}=tsrust_native_sql::{name}{lifetime};").unwrap();true
 }
}
