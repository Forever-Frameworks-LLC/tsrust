//! Emit local read-only loans and explicitly cloned shared results.
use super::*;
use crate::native_signatures::NativeTypeId;
impl Emitter<'_>{
 pub(super) fn cache_record(&self,output:&mut String,id:usize)->bool{
  let record=&self.program.records[id];
  if !crate::stdlib::cache::is_cache(record.native_type){return false;}
  let types=&record.application.as_ref().expect("applied cache").resolved_arguments;
  let _=writeln!(output,"type Record{id}={}<{},{}>;",if record.native_type==Some(NativeTypeId::SharedLruCache){"TsrSharedCache"}else{"TsrLocalCache"},self.native_type(types[0]),self.native_type(types[1]));true
 }
 pub(super) fn cache_constructor_expression(&self,fun:&CheckedFunction,expr:&Expr,mode:Mode)->Option<String>{
  let Some(Type::Record(record))=expr.ty else{return None;};
  if !crate::stdlib::cache::is_cache(self.program.records[record].native_type){return None;}
  let ExprKind::NativeConstructor(args,_)=&expr.kind else{return None;};
  let capacity=self.expr(fun,&args[0],Mode::Read);
  let failed=self.runtime_failure_exit("RangeError","range","Cache capacity must be a positive int.",expr.span).unwrap_or_else(||"panic!(\"Cache capacity must be a positive int.\")".into());
  let constructed=if self.program.records[record].native_type==Some(NativeTypeId::SharedLruCache){"std::sync::Arc::new(std::sync::Mutex::new(cache))"}else{"cache"};
  let code=format!("match tsr_cache_new({capacity}){{Ok(cache)=>{constructed},Err(_)=>{{{failed}}}}}");
  Some(Self::access_wrapped(expr,mode,code))
 }
 pub(super) fn cache_optional(&self,ty:Type,element:Type,selected:&str)->String{
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
  let shared=binding.name.starts_with("__shared_");let operation=crate::stdlib::cache::operation_name(binding.name);
  let receiver=self.expr(fun,&args[0],if shared{Mode::Read}else{Mode::Mutate});
  if operation=="share"{return Some(format!("std::sync::Arc::clone({receiver})"));}
  let Type::Record(record)=args[0].ty.unwrap() else{unreachable!()};
  let types=&self.program.records[record].application.as_ref().unwrap().resolved_arguments;
  let key=args.get(1).map(|arg|self.expr(fun,arg,if operation=="set"{Mode::Consume}else{Mode::Read})).unwrap_or_default();
  let key=if operation!="set"&&types[0].is_copy(){format!("&({key})")}else{key};
  // Evaluate arguments before locking: their calls may access the same shared cache.
  let mut preparation=String::new();
  let key=if shared && args.len()>1{preparation.push_str(&format!("let prepared_key={key};"));"prepared_key".into()}else{key};
  let code=match operation{
   "size"=>"cache.len() as i64".into(),"clear"=>"cache.clear()".into(),
   "get"|"peek"|"getCloned"|"peekCloned"=>{
    let selected=format!("cache.{}({key}){}",operation.trim_end_matches("Cloned"),if shared{".cloned()"}else if types[1].is_copy(){".copied()"}else{""});
    self.cache_optional(expr.ty.unwrap(),types[1],&selected)
   },
   "remove"=>{
    let ty=expr.ty.unwrap();let entry=self.program.unions[ty.union().unwrap().index()].members.iter().copied().find(|ty|*ty!=Type::Undefined).unwrap();
    self.cache_optional(ty,entry,&format!("cache.remove({key}).map(|(key,value)|{})",self.cache_entry(entry)))
   },
   "set"=>{
    let value=self.expr(fun,&args[2],Mode::Consume);let ttl=args.get(3).map(|arg|format!("Some({})",self.expr(fun,arg,Mode::Read))).unwrap_or_else(||"None".into());
    let (value,ttl)=if shared{preparation.push_str(&format!("let prepared_value={value};let prepared_ttl={ttl};"));("prepared_value".to_string(),"prepared_ttl".to_string())}else{(value,ttl)};
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
  Some(if shared{format!(r#"{{let receiver={receiver};{preparation}let mut guard=receiver.lock().expect("Shared cache lock poisoned");let cache=&mut *guard;{code}}}"#)}else{format!("{{let cache={receiver};{code}}}")})
 }
}
