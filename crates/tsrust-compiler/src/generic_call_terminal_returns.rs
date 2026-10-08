//! Invocation-free return summaries can complete without suspending a caller.
use super::*;

pub(super) fn eligible(declaration:Span,effects:&effect_bodies::Index)->bool {
    let Some([statement])=effects.body(declaration) else {return false;};
    let StmtKind::Return(Some(value))=&statement.kind else {return false;};
    let Some(parameters)=effects.parameters(declaration) else {return false;};
    if parameters.iter().any(|parameter|parameter.rest || !matches!(parameter.initializer,ParameterInitializer::Required))
        || effects.captures(declaration).is_some_and(|captures|!captures.is_empty())
        || effects.aggregate_writes(declaration) {return false;}
    pure_value(value,parameters,effects)
}

// Only one fixed wrapper layer is eligible. The engine bounds eager wrapper
// dispatch independently, so recursive or dynamic calls stay on the worklist.
pub(super) fn wrapper(declaration:Span,declarations:&HashMap<String,Contract>,effects:&effect_bodies::Index)->bool {
    let Some([statement])=effects.body(declaration) else {return false;};
    let StmtKind::Return(Some(value))=&statement.kind else {return false;};
    let ExprKind::Call(callee,arguments)=&value.kind else {return false;};
    let ExprKind::Name(name)=&callee.kind else {return false;};
    if effects.binding(name,callee.span).is_some() {return false;}
    let Some(target)=declarations.get(name) else {return false;};
    if !eligible(target.declaration,effects) {return false;}
    let Some(parameters)=effects.parameters(declaration) else {return false;};
    if parameters.iter().any(|parameter|parameter.rest || !matches!(parameter.initializer,ParameterInitializer::Required))
        || effects.captures(declaration).is_some_and(|captures|!captures.is_empty())
        || effects.aggregate_writes(declaration) {return false;}
    arguments.iter().all(|value|pure_value(value,parameters,effects))
}

fn pure_value(value:&Expr,parameters:&[effect_bodies::Parameter],effects:&effect_bodies::Index)->bool {
    let mut pending=vec![value];
    while let Some(value)=pending.pop() {
        match &value.kind {
            ExprKind::Name(name)=> {
                let identity=effects.binding(name,value.span);
                if identity.is_none() || !parameters.iter().any(|parameter|parameter.binding.identity==identity) {return false;}
            },
            ExprKind::Binary {left,right,..}=>{pending.push(left);pending.push(right);},
            ExprKind::Array(values)=>pending.extend(values),
            ExprKind::Record(fields)=>pending.extend(fields.iter().map(|(_,value)|value)),
            ExprKind::Integer(_)|ExprKind::Number(_)|ExprKind::Boolean(_)|ExprKind::String(_)=>{},
            _=>return false,
        }
    }
    true
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test]
 fn eager_wrappers_require_fixed_leaf_targets_and_pure_required_arguments(){
  for (root,expected) in [
   ("function root(value:int):int{return leaf(value);}",true),
   ("function root(leaf:(value:int)=>int,value:int):int{return leaf(value);}",false),
   ("function root(value:int=1):int{return leaf(value);}",false),
   ("function root(...values:int[]):int{return leaf(1);}",false),
   ("function root(value:int):int{return root(value);}",false),
   ("function root(value:int):int{return middle(value);}",false),
   ("function root(value:int):int{return leaf(leaf(value));}",false),
   ("function root(value:int):int{return leaf(++value);}",false),
  ] {
   let source=crate::format(&format!("function leaf(value:int):int{{return value;}}function middle(value:int):int{{return leaf(value);}}{root}function main(){{}}")).unwrap();
   assert_eq!(crate::format(&source).unwrap(),source);
   let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
   let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
   let root_function=program.functions.iter().find(|function|function.name.text=="root").unwrap();
   assert_eq!(wrapper(root_function.name.span,&declarations,&effects),expected,"{root}");
  }
 }
}
