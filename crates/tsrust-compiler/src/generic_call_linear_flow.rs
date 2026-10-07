//! Source snapshots for bodies proven to have only ordered direct writes.
use super::*;

pub(super) fn at_call(
    initial: &[Binding],
    writes: &[targets::Write],
    scope: Span,
    call_start: usize,
    declarations: &HashMap<String, Contract>,
    effects: &effect_bodies::Index,
) -> Vec<Binding> {
    let mut bindings = initial.to_vec();
    // Initializers snapshot callable values. Assignment events replace rather
    // than accumulate targets; saved immutable aliases retain earlier snapshots.
    let mut events = bindings
        .iter()
        .enumerate()
        .filter(|(_, binding)| {
            scope.start < binding.scope.start && binding.scope.start <= call_start
        })
        .map(|(index, binding)| (binding.scope.start, false, index))
        .chain(
            writes
                .iter()
                .enumerate()
                .filter(|(_, write)| {
                    scope.start <= write.span.start
                        && write.span.end <= scope.end
                        && write.completed <= call_start
                })
                .map(|(index, write)| (write.completed, true, index)),
        )
        .collect::<Vec<_>>();
    events.sort_by_key(|&(position, assignment, _)| (position, assignment));
    for (_, assignment, index) in events {
        let (binding_index, candidates, origins) = if assignment {
            let write = &writes[index];
            let Some(binding_index) = branch_flow::binding_at(&bindings, &write.name, write.span, effects) else { continue; };
            let binding = &bindings[binding_index];
            if !binding.mutable {
                continue;
            }
            (binding_index, write.targets.clone(), write.origins.clone())
        } else {
            (index, bindings[index].targets.clone(), bindings[index].origins.clone())
        };
        let origins = body_origins::resolve(origins, &bindings, declarations, effects);
        bindings[binding_index].alternatives = None;
        bindings[binding_index].origins = origins;
        if bindings[binding_index].contract.is_some() { continue; }
        let snapshot =
            resolve(candidates, &bindings, declarations, effects).map_or(Target::Unknown, Target::Contract);
        bindings[binding_index].targets = vec![snapshot];
    }
    bindings
}

#[cfg(test)]
mod provenance_tests {
    use super::*;
    #[test]
    fn typed_linear_reassignment_does_not_rewrite_saved_body() {
        let source=crate::format("function one():int{return 1;}function two():int{return 2;}function inspect<T>(){let action:Function<int>=one;const saved=action;action=two;saved();action();}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let declarations=program.functions.iter().map(|function|(function.name.text.clone(),contract(0,&function.parameter_initializers,function.rest_parameter,function.name.span))).collect::<HashMap<_,_>>();
        let function=program.functions.iter().find(|function|function.name.text=="inspect").unwrap();
        let mut bindings=Vec::new();collect(&function.body,&mut bindings,&effects);
        let writes=targets::writes(&mut program.clone()).unwrap();
        let saved=function.body.iter().find_map(|statement|match &statement.kind {StmtKind::Expr(Expr {kind:ExprKind::Call(callee,_),..}) if name(callee).is_some_and(|(name,_)|name=="saved")=>Some(callee.as_ref()),_=>None}).unwrap();
        let state=at_call(&bindings,&writes,function.scope_span,saved.span.start,&declarations,&effects);
        for (name,expected) in [("saved","one"),("action","two")] {
            let origin=state.iter().find(|binding|binding.name==name).unwrap().origins.clone();
            let span=program.functions.iter().find(|function|function.name.text==expected).unwrap().name.span;
            assert_eq!(origin,vec![body_origins::Origin::Body(span)],"{name}");
        }
    }
}
