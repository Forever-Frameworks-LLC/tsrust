//! Body provenance does not inherit a declared callback signature's identity.
use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum Origin {
    Name(String, Span),
    Body(Span),
    Unknown,
}
pub(super) fn value(expression: &Expr) -> Vec<Origin> {
    let mut pending = vec![expression];
    let mut origins = Vec::new();
    while let Some(expression) = pending.pop() {
        if let Some((name, span)) = name(expression) {
            origins.push(if matches!(name, "undefined" | "null") { Origin::Unknown } else { Origin::Name(name.to_owned(), span) });
            continue;
        }
        match &expression.kind {
            ExprKind::TypeAssertion { value, .. }
            | ExprKind::Assign { value, .. }
            | ExprKind::Unary { op: UnaryOp::NonNull | UnaryOp::OptionalBoundary | UnaryOp::OptionalSelected, value, .. } => pending.push(value),
            ExprKind::Conditional { consequent, alternate, .. } => {
                pending.push(alternate);
                pending.push(consequent);
            }
            ExprKind::Sequence(values) => {
                if let Some(last) = values.last() { pending.push(last); }
                else { origins.push(Origin::Unknown); }
            }
            ExprKind::Closure { .. } => {
                let forwarded = targets::value(expression);
                if let [targets::Target::Name(name, span)] = forwarded.as_slice() {
                    origins.push(Origin::Name(name.clone(), *span));
                } else {
                    origins.push(Origin::Body(expression.span));
                }
            }
            _ => origins.push(Origin::Unknown),
        }
    }
    origins
}

/// Alias traversal follows lexical binding identities, never signature identities.
pub(super) fn resolve(initial: Vec<Origin>, bindings: &[Binding], declarations: &HashMap<String, Contract>, effects: &effect_bodies::Index) -> Vec<Origin> {
    enum Step { Origin(Origin), Leave(usize) }
    let mut pending = initial.into_iter().map(Step::Origin).collect::<Vec<_>>();
    let mut resolved = Vec::new();
    let mut visited = HashSet::new();
    while let Some(step) = pending.pop() {
        let origin = match step { Step::Origin(origin) => origin, Step::Leave(index) => { visited.remove(&index); continue; } };
        match origin {
            Origin::Name(name, span) => {
                if let Some(index) = branch_flow::binding_at(bindings, &name, span, effects) {
                    if visited.insert(index) {
                        pending.push(Step::Leave(index));
                        pending.extend(bindings[index].origins.iter().cloned().map(Step::Origin));
                    } else { resolved.push(Origin::Unknown); }
                } else if effects.binding(&name, span).is_some() {
                    resolved.push(Origin::Unknown);
                } else if let Some(declaration) = declarations.get(&name)
                    && effects.body(declaration.declaration).is_some() {
                    resolved.push(Origin::Body(declaration.declaration));
                } else { resolved.push(Origin::Unknown); }
            }
            origin => resolved.push(origin),
        }
    }
    canonical(resolved)
}

pub(super) fn canonical(mut origins: Vec<Origin>) -> Vec<Origin> {
    if origins.is_empty() { origins.push(Origin::Unknown); }
    fn key(origin: &Origin) -> (u8, usize, usize, &str) { match origin {
        Origin::Unknown => (0, 0, 0, ""),
        Origin::Body(span) => (1, span.start, span.end, ""),
        Origin::Name(name, span) => (2, span.start, span.end, name.as_str()),
    } }
    origins.sort_by(|left, right| key(left).cmp(&key(right)));
    origins.dedup();
    origins
}

#[cfg(test)]
mod tests {
    use super::*;
    fn values(source: &str) -> Vec<Expr> {
        let source = crate::format(source).unwrap();
        assert_eq!(crate::format(&source).unwrap(), source);
        let program = crate::syntax::parse(&source).unwrap();
        program.functions.iter().find(|function| function.name.text == "main").unwrap().body.iter().filter_map(|statement| match &statement.kind {
            StmtKind::Let { value, .. } => Some(value.clone()), _ => None,
        }).collect()
    }
    #[test]
    fn assertions_and_annotations_preserve_actual_arrow_body() {
        for expression in values("function main(){const asserted=(():int=>1) as Function<int>;const annotated:Function<int>=():int=>2;}") {
            let actual = if let ExprKind::TypeAssertion { value, .. } = &expression.kind { value.as_ref() } else { &expression };
            assert!(matches!(actual.kind, ExprKind::Closure { .. }));
            assert_eq!(value(&expression), vec![Origin::Body(actual.span)]);
        }
    }
    #[test]
    fn signature_assertion_does_not_invent_body_for_unknown_value() {
        let expressions = values("function main(){const missing=undefined as Function<int>;}");
        assert_eq!(value(&expressions[0]), vec![Origin::Unknown]);
    }
    #[test]
    fn selected_bodies_and_named_aliases_keep_distinct_provenance() {
        let expressions = values("function identity<T>(value:T):T{return value;}function main(){const alias=identity as Function<int,[value:int]>;const selected=true?(():int=>1):(():int=>2);}");
        let ExprKind::TypeAssertion { value: named, .. } = &expressions[0].kind else { unreachable!() };
        assert_eq!(value(&expressions[0]), vec![Origin::Name("identity".into(), named.span)]);
        let ExprKind::Conditional { consequent, alternate, .. } = &expressions[1].kind else { unreachable!() };
        let origins=value(&expressions[1]);
        assert_eq!(origins.len(),2);
        assert!(origins.contains(&Origin::Body(consequent.span)));
        assert!(origins.contains(&Origin::Body(alternate.span)));
    }
}

#[cfg(test)]
mod binding_tests {
    use super::*;
    #[test]
    fn annotated_alias_keeps_initializer_body_and_unknown_parameter_stays_unknown() {
        let source=crate::format("function inspect<T>(unknown:Function<int>){const callback:Function<int>=():int=>7;const saved=callback as Function<int>;saved();unknown();}function main(){}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let function=program.functions.iter().find(|function|function.name.text=="inspect").unwrap();
        let mut bindings=Vec::new();collect(&function.body,&mut bindings,&effects);
        let body=function.body.iter().find_map(|statement|match &statement.kind {StmtKind::Let {name,value,..} if name.text=="callback"=>Some(value.span),_=>None}).unwrap();
        let calls=function.body.iter().filter_map(|statement|match &statement.kind {StmtKind::Expr(Expr {kind:ExprKind::Call(callee,_),..})=>Some(callee.as_ref()),_=>None}).collect::<Vec<_>>();
        assert_eq!(resolve(value(calls[0]),&bindings,&HashMap::new(),&effects),vec![Origin::Body(body)]);
        assert_eq!(resolve(value(calls[1]),&bindings,&HashMap::new(),&effects),vec![Origin::Unknown]);
    }
}

#[cfg(test)]
mod cycle_tests {
    use super::*;
    #[test]
    fn alias_cycles_keep_unknown_while_shared_alias_paths_keep_known_body() {
        let source=crate::format("function main(){let action=():int=>7;action();}").unwrap();
        assert_eq!(crate::format(&source).unwrap(),source);
        let program=crate::syntax::parse(&source).unwrap();
        let effects=effect_bodies::Index::new(&program).unwrap();
        let mut bindings=Vec::new();collect(&program.functions[0].body,&mut bindings,&effects);
        let callee=program.functions[0].body.iter().find_map(|statement|match &statement.kind {StmtKind::Expr(Expr {kind:ExprKind::Call(callee,_),..})=>Some(callee.as_ref()),_=>None}).unwrap();
        let known=resolve(value(callee),&bindings,&HashMap::new(),&effects);
        assert!(matches!(known.as_slice(),[Origin::Body(_)]));
        let repeated=resolve(vec![value(callee)[0].clone(),value(callee)[0].clone()],&bindings,&HashMap::new(),&effects);
        assert_eq!(repeated,known);
        bindings[0].origins.extend(value(callee));
        let cyclic=resolve(value(callee),&bindings,&HashMap::new(),&effects);
        assert_eq!(cyclic,canonical(known.into_iter().chain([Origin::Unknown]).collect()));
    }
}
