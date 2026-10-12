//! Owned-data transfer eligibility, separate from closure capture ownership.
use crate::syntax::{Record,Type,UnionId};

#[derive(Debug,PartialEq,Eq)]
pub(super) enum CompletionFailure { Result(Type), Thrown(Type), UnknownError }

/// Both successful and exceptional completion cross the same thread boundary.
/// Unknown error contracts must be resolved before transfer can be accepted.
#[allow(dead_code)] // Connected to join transport when the source API is introduced.
pub(super) fn require_owned_completion(result:Type,errors:&crate::error_effects::FailureSet,runtime:Option<Type>,records:&[Record],members:&impl Fn(UnionId)->Vec<Type>)->Result<(),CompletionFailure>{
 require_owned_data(result,records,members).map_err(CompletionFailure::Result)?;
 if errors.opaque {return Err(CompletionFailure::UnknownError);}
 for &ty in &errors.explicit {
  require_owned_data(ty,records,members).map_err(CompletionFailure::Thrown)?;
 }
 if errors.runtime {
  let ty=runtime.ok_or(CompletionFailure::UnknownError)?;
  require_owned_data(ty,records,members).map_err(CompletionFailure::Thrown)?;
 }
 Ok(())
}

/// Returns the capture index that cannot move independently of its caller.
/// A copied scalar is owned storage; a borrowed scalar is still a caller loan.
/// Invocation access does not change ownership of an already moved environment.
#[allow(dead_code)] // Connected to spawn checking after lifecycle semantics are settled.
pub(super) fn require_owned_captures(closure:&super::CheckedClosure,records:&[Record],members:&impl Fn(UnionId)->Vec<Type>)->Result<(),usize>{
 for (index,capture) in closure.captures.iter().enumerate(){
  let binding=closure.function.bindings.get(capture.inner).ok_or(index)?;
  match capture.storage {
   super::Mode::Consume=>{},
   super::Mode::Copy if binding.ty.is_copy()=>{},
   _=>return Err(index),
  }
  require_owned_data(binding.ty,records,members).map_err(|_|index)?;
 }
 Ok(())
}

/// Returns the first unsupported type without granting callable capture transfer.
/// Cycles terminate through checked type identity; every reachable field is inspected.
#[allow(dead_code)] // Connected to spawn/channel checking when their API is introduced.
pub(super) fn require_owned_data(root:Type,records:&[Record],members:&impl Fn(UnionId)->Vec<Type>)->Result<(),Type>{
 let mut pending=vec![root];let mut seen=std::collections::HashSet::new();
 while let Some(ty)=pending.pop(){
  if !seen.insert(ty){continue;}
  match ty {
   Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined|Type::Void|Type::Enum(..)|Type::Literal(..)=>{},
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
 #[test]
 fn unresolved_failure_contracts_cannot_transfer(){
  let members=|_:UnionId|panic!("scalar contract has no unions");
  for errors in [
   crate::error_effects::FailureSet{opaque:true,..Default::default()},
   crate::error_effects::FailureSet{runtime:true,..Default::default()},
  ] {
   assert_eq!(require_owned_completion(Type::Int,&errors,None,&[],&members),Err(CompletionFailure::UnknownError));
  }
 }
 #[test]
 fn completion_transfer_checks_returned_and_thrown_payloads(){
  for (source,expected) in [
   ("function probe():string{return 'result';}function main(){}",true),
   ("function probe():int{throw 'failure';}function main(){}",true),
   ("function probe(values:int[],index:int):int{return values[index];}function main(){}",true),
   ("function probe():Function<int>{return ():int=>1;}function main(){}",false),
   ("function probe():int{throw ():int=>1;}function main(){}",false),
  ] {
   let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
   let mut program=crate::check(&source).unwrap().program;
   let runtime=crate::error_effects::materialize_runtime_record(&mut program);
   let index=program.functions.iter().position(|f|f.name=="probe").unwrap();
   assert_eq!(require_owned_completion(program.functions[index].result,&program.error_effects.function_errors[index],Some(runtime),&program.records,&|id|program.unions[id.index()].members.clone()).is_ok(),expected,"{source}");
  }
 }
 #[test]
 fn capture_storage_must_own_transferable_data(){
  for (source,expected) in [
   ("function factory(owner:string):Function<int>{return ():int=>owner.length;}function main(){const callback=factory('bytes');callback();}",true),
   ("function main(){const owner='bytes';const callback=():int=>owner.length;callback();}",false),
   ("function main(){const owner=42;const callback=():int=>owner;callback();}",true),
   ("function main(){let owner=42;const callback=():int=>{owner+=1;return owner;};callback();}",false),
   ("type Packet={callback:()=>int;};function factory(owner:Packet):Function<int>{return ():int=>owner.callback();}function main(){}",false),
  ] {
   let formatted=crate::format(source).unwrap();assert_eq!(crate::format(&formatted).unwrap(),formatted);
   let checked=crate::check(&formatted).unwrap();let program=&checked.program;
   let closure=program.closures.iter().find(|closure|!closure.captures.is_empty()).unwrap();
   assert_eq!(require_owned_captures(closure,&program.records,&|id|program.unions[id.index()].members.clone()).is_ok(),expected,"{source}");
  }
 }
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
