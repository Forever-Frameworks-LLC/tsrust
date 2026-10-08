//! Invocation-free return summaries can complete without suspending a caller.
use super::*;

pub(super) fn eligible(declaration:Span,effects:&effect_bodies::Index)->bool {
    let Some([statement])=effects.body(declaration) else {return false;};
    let StmtKind::Return(Some(value))=&statement.kind else {return false;};
    let Some(parameters)=effects.parameters(declaration) else {return false;};
    if parameters.iter().any(|parameter|parameter.rest || !matches!(parameter.initializer,ParameterInitializer::Required))
        || effects.captures(declaration).is_some_and(|captures|!captures.is_empty())
        || effects.aggregate_writes(declaration) {return false;}
    let mut pending=vec![value];let mut parameter_seen=false;
    while let Some(value)=pending.pop() {
        match &value.kind {
            ExprKind::Name(name)=> {
                let identity=effects.binding(name,value.span);
                if identity.is_none() || !parameters.iter().any(|parameter|parameter.binding.identity==identity) {return false;}
                parameter_seen=true;
            },
            ExprKind::Binary {left,right,..}=>{pending.push(left);pending.push(right);},
            ExprKind::Integer(_)|ExprKind::Number(_)|ExprKind::Boolean(_)|ExprKind::String(_)=>{},
            _=>return false,
        }
    }
    parameter_seen
}
