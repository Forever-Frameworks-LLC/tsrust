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
