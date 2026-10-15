use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 fn sqlite_parameter(&self,ty:Type,value:&str)->String{
  let variant=match ty{Type::Int=>"Integer",Type::Number=>"Real",Type::String=>return format!("tsrust_native_sqlite::Parameter::Text(({value}).as_ref().to_owned())"),Type::Null=>return "tsrust_native_sqlite::Parameter::Null".into(),Type::Record(id) if self.program.records[id].native_type==Some(NativeTypeId::Bytes)=>return format!("tsrust_native_sqlite::Parameter::Blob(({value}).bytes.clone())"),Type::Union(id)=>{
   let arms=self.program.unions[id.index()].members.iter().enumerate().map(|(tag,ty)|format!("Union{}::V{tag}(value)=>{}",id.index(),self.sqlite_parameter(*ty,"value"))).collect::<Vec<_>>().join(",");return format!("match {value}{{{arms}}}");
  },_=>unreachable!("checked SQL parameter")};format!("tsrust_native_sqlite::Parameter::{variant}(*({value}))")
 }
 pub(super) fn sqlite_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:sqlite"{return None;}self.native_bindings.borrow_mut().insert(id);
  let values=args.iter().zip(binding.parameters).map(|(arg,param)|{let value=self.expr(fun,arg,param.access);if arg.ty==Some(Type::String){format!("({value}).as_ref()")}else{value}}).collect::<Vec<_>>();let receiver=&values[0];
  let call=match binding.name{
   "open"=>format!("tsrust_native_sqlite::Connection::open({receiver},{})",values.get(1).map(|v|format!("Some({v})")).unwrap_or("None".into())),
   "__execute"|"__query"=>{
    let parameters=if args.len()==2{let Type::Array(array)=args[1].ty.unwrap()else{unreachable!()};let element=array.ty();format!("({}).iter().map(|value: &{}|{}).collect::<Vec<_>>()",values[1],self.native_type(element),self.sqlite_parameter(element,"value"))}else{"Vec::new()".into()};
    format!("{{let statement={receiver};let parameters={parameters};statement.{}(&parameters){}}}",binding.name.trim_start_matches('_'),if binding.name=="__query"{".map(Array::Heap)"}else{""})
   },
   "__isNull"=>format!("({receiver}).is_null({})",values[1]),
   name=>format!("({receiver}).{}({})",name.trim_start_matches('_'),values[1..].join(","))
  };Some(self.http_result(expr.ty.unwrap(),&call))
 }
 pub(super) fn sqlite_record(&self,output:&mut String,id:usize)->bool{
  let (name,borrow)=match self.program.records[id].native_type{Some(NativeTypeId::SqliteConnection)=>("Connection",false),Some(NativeTypeId::SqliteStatement)=>("Statement",true),Some(NativeTypeId::SqliteTransaction)=>("Transaction",true),Some(NativeTypeId::SqliteRow)=>("Row",false),_=>return false};
  let lifetime=if borrow{"<'a>"}else{""};writeln!(output,"type Record{id}{lifetime}=tsrust_native_sqlite::{name}{lifetime};").unwrap();true
 }
}
