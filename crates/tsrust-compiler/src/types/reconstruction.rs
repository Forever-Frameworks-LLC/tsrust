//! Postorder reconstruction of finite checked type graphs into source annotations.
use super::syntax::{CallableOwnership, CallableParameter, ValueAnnotation, ValueAnnotationKind as Kind};
use crate::syntax::Type;
use crate::{Diagnostic, Span};
use std::collections::HashSet;

pub(crate) enum Plan {
    Leaf(ValueAnnotation),
    Array(Type),
    Union(Vec<Type>),
    Callable { ownership: CallableOwnership, parameters: Vec<(CallableParameter, Type)>, result: Type },
}

pub(crate) fn reconstruct(root: Type, span: Span, mut describe: impl FnMut(Type) -> Result<Plan, Diagnostic>) -> Result<ValueAnnotation, Diagnostic> {
    enum Work { Visit(Type), Finish(Type, Plan, usize) }
    let mut pending = vec![Work::Visit(root)];
    let mut active = HashSet::new();
    let mut values = Vec::new();
    while let Some(work) = pending.pop() {
        match work {
            Work::Visit(ty) => {
                if active.contains(&ty) {
                    return Err(Diagnostic::new("unsupported", "Checked annotation contains a recursive structural contract.", span));
                }
                let plan = describe(ty)?;
                if let Plan::Leaf(value) = plan { values.push(value); continue; }
                let children = match &plan {
                    Plan::Array(child) => vec![*child],
                    Plan::Union(children) => children.clone(),
                    Plan::Callable { parameters, result, .. } => parameters.iter().map(|(_, ty)| *ty).chain(std::iter::once(*result)).collect(),
                    Plan::Leaf(_) => unreachable!(),
                };
                active.insert(ty);
                pending.push(Work::Finish(ty, plan, values.len()));
                pending.extend(children.into_iter().rev().map(Work::Visit));
            }
            Work::Finish(ty, plan, start) => {
                active.remove(&ty);
                let mut children = values.drain(start..).collect::<Vec<_>>();
                let kind = match plan {
                    Plan::Array(_) => Kind::Array(Box::new(children.pop().expect("array element"))),
                    Plan::Union(_) => Kind::Union(children),
                    Plan::Callable { ownership, parameters, .. } => {
                        let result = Box::new(children.pop().expect("callable result"));
                        let parameters = parameters.into_iter().zip(children).map(|((mut parameter, _), annotation)| {
                            parameter.annotation = annotation;
                            parameter
                        }).collect();
                        Kind::Callable { ownership, parameters, result }
                    }
                    Plan::Leaf(_) => unreachable!(),
                };
                values.push(ValueAnnotation { kind, span });
            }
        }
    }
    Ok(values.pop().expect("reconstructed annotation"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::type_syntax::TypeName;

    #[test]
    fn shared_children_are_rebuilt_and_first_errors_stop_later_siblings() {
        let span = Span::new(20, 31);
        let mut visited = Vec::new();
        let annotation = reconstruct(Type::Callable(0), span, |ty| {
            visited.push(ty);
            Ok(if ty == Type::Callable(0) { Plan::Union(vec![Type::Int, Type::Int]) }
                else { Plan::Leaf(ValueAnnotation { kind: Kind::Named(TypeName { text: "int".into(), span }), span }) })
        }).unwrap();
        assert_eq!(visited, vec![Type::Callable(0), Type::Int, Type::Int]);
        assert!(matches!(annotation.kind, Kind::Union(ref members) if members.len() == 2));
        visited.clear();
        let error = reconstruct(Type::Callable(0), span, |ty| {
            visited.push(ty);
            if ty == Type::Callable(0) { Ok(Plan::Union(vec![Type::Void, Type::Boolean])) }
            else { Err(Diagnostic::new("unsupported", "first child", span)) }
        }).unwrap_err();
        assert_eq!(error.span, span);
        assert_eq!(error.message, "first child");
        assert_eq!(visited, vec![Type::Callable(0), Type::Void]);
    }
}
