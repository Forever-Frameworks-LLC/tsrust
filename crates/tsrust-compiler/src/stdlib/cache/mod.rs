//! Owned local caches with operation-driven expiration.
use crate::native_bindings::{NativeBackend,NativeBinding,NativeParameter,NativeType};
use crate::Mode;
const RECEIVER:NativeParameter=NativeParameter{name:"cache",ty:NativeType::ObservedValue,access:Mode::Mutate,optional:false,rest:false};
const KEY:NativeParameter=NativeParameter{name:"key",access:Mode::Read,..RECEIVER};
const fn operation(name:&'static str,parameters:&'static [NativeParameter])->NativeBinding{
 NativeBinding{module:"std:cache",name,parameters,result:NativeType::ObservedValue,returned_borrow:None,backend:NativeBackend::Bundled{symbol:"cache_intrinsic"},features:&[]}
}
pub(crate) const CREATE:NativeBinding=operation("__create",&[NativeParameter{name:"capacity",ty:NativeType::Int,access:Mode::Read,..RECEIVER}]);
pub(crate) const GET:NativeBinding=operation("__get",&[RECEIVER,KEY]);
pub(crate) const PEEK:NativeBinding=operation("__peek",&[RECEIVER,KEY]);
pub(crate) const REMOVE:NativeBinding=operation("__remove",&[RECEIVER,KEY]);
pub(crate) const SIZE:NativeBinding=operation("__size",&[RECEIVER]);
pub(crate) const CLEAR:NativeBinding=operation("__clear",&[RECEIVER]);
pub(crate) const SET:NativeBinding=operation("__set",&[RECEIVER,NativeParameter{access:Mode::Consume,..KEY},NativeParameter{name:"value",access:Mode::Consume,..RECEIVER},NativeParameter{name:"ttlMs",ty:NativeType::Int,access:Mode::Read,optional:true,..RECEIVER}]);
pub(crate) fn prepare(program:&mut crate::syntax::Program)->Result<(),crate::Diagnostic>{
 use crate::native_signatures::NativeTypeId;
 if program.records.iter().any(|record|record.native_type==Some(NativeTypeId::LruCache)){
  for id in [NativeTypeId::CacheEntry,NativeTypeId::CacheInsertError]{crate::native_type_resolution::intern(&mut program.records,id,crate::Span::default())?;}
 }
 Ok(())
}
pub(crate) fn runtime()->String{
 format!("mod tsr_cache{{pub mod lru{{{}}}pub use lru::{{KeyPolicy,Lru}};pub mod timed{{{}}}}}\n{}\n{}",include_str!("lru.rs"),include_str!("timed.rs").replacen("//! Operation-driven expiration; the clock is injected for deterministic checks.","",1),include_str!("key_policy.rs"),include_str!("runtime.rs"))
}

/// Cache payloads own all retained storage. Callable and task storage needs a separate contract.
pub(crate) fn validate_payload(root:crate::syntax::Type,records:&[crate::syntax::Record],members:impl Fn(crate::syntax::UnionId)->Vec<crate::syntax::Type>,span:crate::Span)->Result<(),crate::Diagnostic>{
 use crate::syntax::Type;use crate::native_signatures::NativeTypeId;
 let mut pending=vec![root];let mut seen=std::collections::HashSet::new();
 while let Some(ty)=pending.pop(){
  if !seen.insert(ty){continue;}
  let valid=match ty{
   Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined|Type::Enum(..)|Type::Literal(..)=>true,
   Type::Array(element)=>{pending.push(element.ty());true},
   Type::Union(id)=>{pending.extend(members(id));true},
   Type::Map(id)=>{let map=records[id].map_definition.as_ref().unwrap();pending.extend([map.key,map.value]);true},
   Type::Set(id)=>{pending.push(records[id].set_definition.as_ref().unwrap().key);true},
   Type::Record(id)=>{
    let record=&records[id];
    if let Some(native)=record.native_type && (crate::native_types::declaration(native).borrow_access().is_some()||matches!(native,NativeTypeId::Job|NativeTypeId::Thread|NativeTypeId::ScopedThread|NativeTypeId::TaskContext)){false}else{
     if let Some(application)=&record.application{pending.extend(&application.resolved_arguments);}
     pending.extend(record.fields.iter().filter_map(|field|field.ty));true
    }
   },_=>false,
  };
  if !valid{return Err(crate::Diagnostic::new("cache-payload","Cache values require owned data or resources; borrowed views, callbacks and task handles are unsupported.",span));}
 }
 Ok(())
}
