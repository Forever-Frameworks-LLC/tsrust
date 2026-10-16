//! Emit owned entries and read-only views without cloning payloads.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 pub(super) fn cache_record(&self,output:&mut String,id:usize)->bool{
  let record=&self.program.records[id];
  if record.native_type!=Some(NativeTypeId::LruCache){return false;}
  let types=&record.application.as_ref().expect("applied cache").resolved_arguments;
  let _=writeln!(output,"type Record{id}=TsrLocalCache<{},{}>;",self.native_type(types[0]),self.native_type(types[1]));true
 }
 pub(super) fn cache_constructor_expression(&self,fun:&CheckedFunction,expr:&Expr,mode:Mode)->Option<String>{
  let Some(Type::Record(record))=expr.ty else{return None;};
  if self.program.records[record].native_type!=Some(NativeTypeId::LruCache){return None;}
  let ExprKind::NativeConstructor(args,_)=&expr.kind else{return None;};
  let capacity=self.expr(fun,&args[0],Mode::Read);
  let failed=self.runtime_failure_exit("RangeError","range","Cache capacity must be a positive int.",expr.span).unwrap_or_else(||"panic!(\"Cache capacity must be a positive int.\")".into());
  let code=format!("match tsr_cache_new({capacity}){{Ok(cache)=>cache,Err(_)=>{{{failed}}}}}");
  Some(Self::access_wrapped(expr,mode,code))
 }
 fn cache_optional(&self,ty:Type,element:Type,selected:&str)->String{
  if ty==Type::Undefined{return format!("{{let _={selected};TsrUndefined}}");}
  let id=ty.union().expect("optional cache result");let borrowed=matches!(ty,Type::BorrowedUnion(_));
  let absent=self.program.unions[id.index()].members.iter().position(|ty|*ty==Type::Undefined).unwrap();
  let present=self.optional_element_present(id,element,borrowed,"payload");
  let tag=if borrowed{"ViewUnion"}else{"Union"};let nil=if borrowed{"&TsrUndefined"}else{"TsrUndefined"};
  format!("match {selected}{{Some(payload)=>{present},None=>{tag}{}::V{absent}({nil})}}",id.index())
 }
 fn cache_entry(&self,ty:Type)->String{
  let Type::Record(id)=ty else{unreachable!()};
  format!("Record{}{{field0:key,field1:value{}}}",self.native_record_declaration(id),self.schema_record_order(id))
 }
 pub(super) fn cache_native(&self,fun:&CheckedFunction,expr:&Expr,id:usize,args:&[Expr])->Option<String>{
  let binding=&crate::native_bindings::BINDINGS[id];if binding.module!="std:cache"{return None;}
  let receiver=self.expr(fun,&args[0],Mode::Mutate);
  let Type::Record(record)=args[0].ty.unwrap() else{unreachable!()};
  let types=&self.program.records[record].application.as_ref().unwrap().resolved_arguments;
  let key=args.get(1).map(|arg|self.expr(fun,arg,if binding.name=="__set"{Mode::Consume}else{Mode::Read})).unwrap_or_default();
  let key=if binding.name!="__set"&&types[0].is_copy(){format!("&({key})")}else{key};
  let code=match binding.name{
   "__size"=>"cache.len() as i64".into(),"__clear"=>"cache.clear()".into(),
   "__get"|"__peek"=>{
    let selected=format!("cache.{}({key}){}",binding.name.trim_start_matches("__"),if types[1].is_copy(){".copied()"}else{""});
    self.cache_optional(expr.ty.unwrap(),types[1],&selected)
   },
   "__remove"=>{
    let ty=expr.ty.unwrap();let entry=self.program.unions[ty.union().unwrap().index()].members.iter().copied().find(|ty|*ty!=Type::Undefined).unwrap();
    self.cache_optional(ty,entry,&format!("cache.remove({key}).map(|(key,value)|{})",self.cache_entry(entry)))
   },
   "__set"=>{
    let value=self.expr(fun,&args[2],Mode::Consume);let ttl=args.get(3).map(|arg|format!("Some({})",self.expr(fun,arg,Mode::Read))).unwrap_or_else(||"None".into());
    let union=expr.ty.unwrap().union().unwrap();let mut ok=String::new();let mut err=String::new();
    for (tag,member) in self.program.unions[union.index()].members.iter().enumerate(){
     let Type::Record(record)=*member else{unreachable!()};let declaration=&self.program.records[record];let payload=declaration.fields[0].ty.unwrap();
     let success=declaration.application.as_ref().unwrap().declaration.text=="Result#Ok";
     let mapped=if success{
      let entry=self.program.unions[payload.union().unwrap().index()].members.iter().copied().find(|ty|*ty!=Type::Undefined).unwrap();
      self.cache_optional(payload,entry,&format!("displaced.map(|(key,value)|{})",self.cache_entry(entry)))
     }else{
      let Type::Record(error)=payload else{unreachable!()};
      format!("Record{}{{field0:String::Borrowed(code),field1:key,field2:value{}}}",self.native_record_declaration(error),self.schema_record_order(error))
     };
     let result=format!("Union{}::V{tag}(Record{}{{field0:{mapped}{}}})",union.index(),self.native_record_declaration(record),self.schema_record_order(record));
     if success{ok=result;}else{err=result;}
    }
    format!("match tsr_cache_set(cache,{key},{value},{ttl}){{Ok(displaced)=>{ok},Err((code,key,value))=>{err}}}")
   },_=>unreachable!("checked cache operation"),
  };
  Some(format!("{{let cache={receiver};{code}}}"))
 }
}
