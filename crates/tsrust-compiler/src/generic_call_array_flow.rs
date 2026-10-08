//! Exact local array transitions preserve callable environment provenance.
use super::*;

#[inline(never)]
pub(super) fn invoke(
    expression:&Expr, arguments:&[expression_flow::Value], state:&FlowState,
    effects:&effect_bodies::Index,
)->Option<expression_flow::Invocation> {
    let ExprKind::Call(callee,_)=&expression.kind else {return None;};
    let ExprKind::Field(receiver,method)=&callee.kind else {return None;};
    if !matches!(method.as_str(),"push"|"unshift"|"pop"|"shift") {return None;}
    let ExprKind::Name(name)=&receiver.kind else {return None;};
    let index=binding_at(state,name,receiver.span,effects)?;
    let mut outgoing=state.clone();
    let elements=outgoing[index].elements.as_mut()?;
    let result=match method.as_str() {
        "push"|"unshift"=>{
            let added=expression_flow::Value::array(arguments).elements?;
            if method=="push" {elements.extend(added);} else {elements.splice(0..0,added);}
            expression_flow::Value::index(elements.len())
        }
        "pop"|"shift"=>{
            if !arguments.is_empty() {return None;}
            let element=if method=="pop" {elements.pop()} else if elements.is_empty() {None} else {Some(elements.remove(0))};
            element.as_ref().map_or_else(expression_flow::Value::undefined,expression_flow::Value::element)
        }
        _=>unreachable!(),
    };
    Some(expression_flow::Invocation {normal:Some(outgoing),throwing:Vec::new(),result,facts:HashMap::new()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mutations_preserve_order_and_transfer_provenance_without_changing_input() {
        let source=crate::format("function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function main(){const values=[one];values.push(two);values.unshift(one);values.shift();values.pop();values.pop();values.shift();}").unwrap();
        let program=crate::syntax::parse(&source).unwrap();let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(function.type_parameters.len(),&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let main=program.functions.iter().find(|function|function.name.text=="main").unwrap();
        let mut bindings=Vec::new();collect(&main.body,&mut bindings,&effects);let mut state:FlowState=bindings.into();
        let StmtKind::Let {value,..}=&main.body[0].kind else {panic!("array initializer")};
        let (initial,_)=expression_flow::evaluate(value,&mut state,&mut HashMap::new(),&declarations,false,&effects);
        state[0].elements=initial.elements;
        for (statement,expected) in main.body[1..].iter().zip([None,None,Some(1),Some(2),Some(1),None]) {
            let StmtKind::Expr(call)=&statement.kind else {panic!("array call")};let before=state.clone();
            let (result,_)=expression_flow::evaluate(call,&mut state,&mut HashMap::new(),&declarations,false,&effects);
            if let Some(expected)=expected {assert_eq!(result.contracts.unwrap()[0].required,expected);}
            assert!(before.captures==state.captures,"array transfer must not mutate capture contents");
        }
        assert!(state[0].elements.as_ref().unwrap().is_empty());
    }
}
