//! Structural assertions inspect ordinary data without granting opaque expansion.
use super::*;
impl Analyzer {
 pub(super) fn testing_deep_types(&self,args:&[Expr])->Result<(),Diagnostic>{
  let left=args[0].ty.expect("observed argument").owning();let right=args[1].ty.expect("observed argument").owning();
  self.require(left,Some(right),args[1].span)?;
  if matches!(left,Type::Void)||matches!(right,Type::Void){return Err(Diagnostic::new("unsupported","Void is not an assertion value.",args[0].span));}
  let mut work=vec![left,right];let mut seen=HashSet::new();
  while let Some(ty)=work.pop(){if !seen.insert(ty){continue;}match ty{
   Type::Record(id)=>{let record=&self.program.records[id];if record.native_type.is_some_and(|id|!crate::native_types::declaration(id).source_constructible()){return Err(Diagnostic::new("native-capability","Deep assertions do not expand opaque native values.",args[0].span));}work.extend(record.fields.iter().filter_map(|field|field.ty));},
   Type::Array(element)=>work.push(element.ty()),
   Type::Map(id)=>{let record=&self.program.records[id];if record.native_type.is_some_and(|id|!crate::native_types::declaration(id).source_constructible()){return Err(Diagnostic::new("native-capability","Deep assertions do not expand opaque native values.",args[0].span));}let map=record.map_definition.as_ref().unwrap();work.extend([map.key,map.value]);},
   Type::Set(id)=>work.push(self.program.records[id].set_definition.as_ref().unwrap().key),
   Type::Union(id)|Type::BorrowedUnion(id)=>work.extend(self.unions.members(id)),
   Type::String|Type::Int|Type::Number|Type::Boolean|Type::Null|Type::Undefined|Type::Void|Type::Literal(..)|Type::Enum(..)=>{},
   _=>return Err(Diagnostic::new("unsupported","Deep assertions require structural data; functions are unsupported.",args[0].span)),
  }}Ok(())
 }
}
