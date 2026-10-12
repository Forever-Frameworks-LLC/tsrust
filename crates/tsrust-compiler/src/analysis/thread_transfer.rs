//! Owned-data transfer eligibility, separate from closure capture ownership.
use crate::syntax::{Record,Type,UnionId};

/// Returns the first unsupported type without granting callable capture transfer.
/// Cycles terminate through checked type identity; every reachable field is inspected.
#[allow(dead_code)] // Connected to spawn/channel checking when their API is introduced.
pub(super) fn require_owned_data(root:Type,records:&[Record],members:&impl Fn(UnionId)->Vec<Type>)->Result<(),Type>{
 let mut pending=vec![root];let mut seen=std::collections::HashSet::new();
 while let Some(ty)=pending.pop(){
  if !seen.insert(ty){continue;}
  match ty {
   Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined|Type::Void|Type::Enum(..)=>{},
   Type::Array(element)=>pending.push(element.ty()),
   Type::Union(id)=>pending.extend(members(id)),
   Type::Record(id)=>{
    let record=records.get(id).ok_or(ty)?;
    if let Some(native)=record.native_type {
     let declaration=crate::native_types::declaration(native);
     if declaration.borrow_access().is_some()||!declaration.thread_transfer{return Err(ty);}
    }
    if let Some(application)=&record.application {pending.extend(application.resolved_arguments.iter().copied());}
    for field in &record.fields {pending.push(field.ty.ok_or(ty)?);}
   },
   Type::Map(id)=>{
    let map=records.get(id).and_then(|record|record.map_definition.as_ref()).ok_or(ty)?;
    pending.extend([map.key,map.value]);
   },
   Type::Set(id)=>pending.push(records.get(id).and_then(|record|record.set_definition.as_ref()).ok_or(ty)?.key),
   Type::BorrowedUnion(_)|Type::BorrowedCallable(_)|Type::Closure(_)|Type::Callable(_)|Type::Callback(_)|Type::StoredCallable(_)=>return Err(ty),
  }
 }
 Ok(())
}

#[cfg(test)]
mod tests {
 use super::*;
 fn accepts(source:&str)->bool {
  struct Input<'a>(&'a str);
  impl crate::project::SourceProvider for Input<'_> {
   fn resolve(&self,_:Option<&crate::project::SourceId>,path:&str)->Result<crate::project::SourceId,String>{Ok(crate::project::SourceId(path.into()))}
   fn read(&self,_:&crate::project::SourceId)->Result<String,String>{Ok(self.0.into())}
  }
  let graph=crate::project::load_project(&Input(source),"thread-test.tsr").unwrap();
  let checked=crate::project::check_graph(graph).unwrap();let program=&checked.analysis.program;
  let value=program.functions.iter().find(|f|f.name=="probe"||f.name.ends_with("_probe")).unwrap().bindings[0].ty;
  require_owned_data(value,&program.records,&|id|program.unions[id.index()].members.clone()).is_ok()
 }
 #[test]
 fn recursive_owned_data_can_transfer(){
  assert!(accepts("type Node={value:int;next?:Node;};function probe(value:Node[]){}function main(){}"));
  assert!(accepts("function probe(value:Map<string,int[]>){}function main(){}"));
  assert!(accepts("import {Bytes} from 'std:bytes';type Packet={data:Bytes;};function probe(value:Packet){}function main(){}"));
 }
 #[test]
 fn nested_borrows_resources_and_callables_need_rejection(){
  for source in [
   "import {ByteView} from 'std:bytes';type Node={next?:Node;data:ByteView;};function probe(value:Node){}function main(){}",
   "import {File} from 'std:fs';function probe(value:File[]){}function main(){}",
   "import {JsonRaw} from 'std:json';type Packet={raw:JsonRaw;};function probe(value:Packet){}function main(){}",
   "type Packet={callback:()=>int;};function probe(value:Packet){}function main(){}",
  ]{assert!(!accepts(source),"{source}");}
 }
}
