//! Match an actual borrowed payload, retaining its precise owner and lifetime.
use super::*;
impl Emitter<'_> {
 fn is_string_observation(&self,ty:Type)->bool{crate::literal_types::string_observation(ty,|id|self.program.unions[id.index()].members.clone())}
 fn string_observation(&self,fun:&CheckedFunction,value:&Expr)->String{
  let Some(_union)=value.ty.filter(|ty|self.is_string_observation(*ty)).and_then(Type::union) else{return self.expr(fun,value,Mode::Read);};
  // Existing conditional staging owns fresh receivers through the containing
  // expression. Initialization remains at the original evaluation position.
  let source=self.conditional_arm(fun,value,Mode::Read);
  self.string_observation_view(value.ty.unwrap(),source)
 }
 fn string_observation_view(&self,ty:Type,source:String)->String{
  let Some(union)=ty.union().filter(|_|self.is_string_observation(ty)) else{return source;};
  let arms=self.program.unions[union.index()].members.iter().enumerate().map(|(tag,_)|format!("ViewUnion{}::V{tag}(payload)=>payload",union.index())).collect::<Vec<_>>().join(",");
  format!("match {source}{{{arms}}}")
 }
}
