//! Direct declarations need no contextual execution to identify their call targets.
use super::*;
type Facts=HashMap<(usize,usize),Option<Vec<Contract>>>;
pub(super) fn facts(function:&Function,bindings:&[Binding],declarations:&HashMap<String,Contract>,effects:&effect_bodies::Index)->Result<Option<Facts>,Diagnostic> {
 let mut facts=HashMap::new();let mut indirect=false;let mut probe=function.clone();
 crate::source_walk::function_body(&mut probe,&mut |_|Ok(()),&mut |expression| {
  let ExprKind::Call(callee,_) = &expression.kind else {return Ok(())};
  let target=if let Some((name,span))=name(callee) {
   if effects.binding(name,span).is_some() || branch_flow::binding_at(bindings,name,span,effects).is_some(){indirect=true;return Ok(());}
   declarations.get(name).cloned().map(|contract|vec![contract])
  } else if matches!(callee.kind,ExprKind::Field(..)) {
   // Member receivers do not carry source callable contracts. Calls inside
   // their receiver, arguments and closure bodies are visited independently.
   None
  } else {indirect=true;return Ok(());};
  facts.insert((expression.span.start,expression.span.end),target);Ok(())
 })?;
 Ok((!indirect).then_some(facts))
}

#[cfg(test)]
mod tests {
 use super::*;
 fn inspect(source:&str)->(Program,HashMap<String,Contract>,effect_bodies::Index) {
  let source=crate::format(source).unwrap();assert_eq!(crate::format(&source).unwrap(),source);
  let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
  let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect();
  (program,declarations,effects)
 }
 #[test]
 fn direct_declarations_keep_contracts_without_interpreting_the_call_graph() {
  let (program,declarations,effects)=inspect("function identity<T>(value:T):T{return value;}function outer(){console.log(identity<int>(7));const callback=()=>{console.log(identity<int>(8));};}function main(){}");
  let function=program.functions.iter().find(|function|function.name.text=="outer").unwrap();let mut bindings=Vec::new();collect(&function.body,&mut bindings,&effects);
  let facts=facts(function,&bindings,&declarations,&effects).unwrap().unwrap();
  assert_eq!(facts.values().filter(|value|value.as_ref().is_some_and(|contracts|contracts==&vec![declarations["identity"].clone()])).count(),2);
  assert_eq!(facts.values().filter(|value|value.is_none()).count(),2);
  assert!(validate(&program).is_ok());
  let bad=crate::syntax::parse("function identity<T>(value:T):T{return value;}function outer<T>(value:T):T{return identity<T>(value,value);}function main(){}").unwrap();
  assert_eq!(validate(&bad).err().unwrap().code,"arity");
 }
 #[test]
 fn aliases_shadowing_returned_and_indexed_calls_keep_contextual_analysis() {
  for body in ["const action=identity;action(1);","const identity=other;identity(1);","const actions=[identity];actions[0](1);","factory()(1);","try{}catch(identity){identity(1);}","const action=()=>identity(1);[1].forEach(()=>{action();});"] {
   let (program,declarations,effects)=inspect(&format!("function identity(value:int):int{{return value;}}function other(value:int):int{{return value;}}function factory():Function<int,[value:int]>{{return identity;}}function outer(){{{body}}}function main(){{}}"));
   let function=program.functions.iter().find(|function|function.name.text=="outer").unwrap();let mut bindings=Vec::new();collect(&function.body,&mut bindings,&effects);
   assert!(facts(function,&bindings,&declarations,&effects).unwrap().is_none(),"{body}");
  }
 }
}
