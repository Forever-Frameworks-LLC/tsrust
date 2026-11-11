//! A borrowed observation exposes the common string API, retaining exact storage.
use super::*;
impl Analyzer {
 pub(super) fn is_string_observation(&self,ty:Type)->bool{crate::literal_types::string_observation(ty,|id|self.unions.members(id))}
 pub(super) fn string_observation_hint(&self,context:&Context,value:&Expr)->bool{self.expression_type_hint(context,value).is_some_and(|ty|self.is_string_observation(ty))}
 pub(super) fn require_string_observation(&self,ty:Type,span:Span)->Result<(),Diagnostic>{if self.is_string_observation(ty){Ok(())}else{self.require(ty,Some(Type::String),span)}}
}
