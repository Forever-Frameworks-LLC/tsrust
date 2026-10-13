//! Borrow-source predicates preserve short-circuit order on an explicit worklist.
use super::*;

enum Work<'a> { Borrowed(&'a Expr), Indexed(&'a Expr), Query(&'a Expr) }

impl Analyzer {
    /// A const taken from an owned element of a named array, or from a field
    /// of such an element, borrows it for reading; indexing never moves.
    #[inline(never)]
    pub(super) fn indexed_element_hint(&mut self, context: &Context, mut expr: &Expr) -> bool {
        loop { match &expr.kind {
            ExprKind::Sequence(values) => match values.last() { Some(last) => expr = last, None => return false },
            ExprKind::TypeAssertion { value, .. } => expr = value,
            _ => break,
        }}
        let mut place = expr;
        if let ExprKind::Unary{op:UnaryOp::PatternSelection,value,..}=&place.kind {place=value;}
        while let ExprKind::Field(base, _) = &place.kind {
            place = base;
        }
        let ExprKind::Index(base, _) = &place.kind else { return false };
        Self::named_storage(base)
            && matches!(self.expression_type_hint(context, base), Some(Type::Array(_)))
            && self.expression_type_hint(context, expr).is_some_and(|ty| !ty.is_copy())
    }

    pub(super) fn borrowed_query_projection_hint(&self, context:&Context, expr:&Expr)->bool {
        let mut receiver=expr;
        while let ExprKind::Unary{op:UnaryOp::NonNull,value,..}=&receiver.kind {receiver=value;}
        matches!(self.expression_type_hint(context,receiver),Some(Type::BorrowedUnion(_)))
    }

    pub(super) fn borrowed_place_hint(&mut self, context: &Context, expr: &Expr) -> bool {
        let mut pending = vec![Work::Borrowed(expr)];
        while let Some(work) = pending.pop() {
            let expr = match work {
                Work::Indexed(expr) => { if self.indexed_element_hint(context, expr) { return true; } continue; }
                Work::Query(expr) => { if self.borrowed_query_projection_hint(context, expr) { return true; } continue; }
                Work::Borrowed(expr) => expr,
            };
            if self.string_position(context, expr) { continue; }
            match &expr.kind {
                ExprKind::Sequence(values) => if let Some(last) = values.last() {
                    pending.push(Work::Indexed(last)); pending.push(Work::Borrowed(last));
                },
                ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. }
                | ExprKind::Unary { op: UnaryOp::PatternSelection, value, .. } => {
                    pending.push(Work::Indexed(value)); pending.push(Work::Borrowed(value));
                }
                ExprKind::Call(callee, arguments) if matches!(&callee.kind, ExprKind::Name(name) if name == "@match:payload") => {
                    if let Some(value) = arguments.first() {
                        if self.expression_type_hint(context, value).is_some_and(|ty| matches!(ty, Type::BorrowedUnion(_))) { return true; }
                        pending.push(Work::Borrowed(value));
                    }
                }
                ExprKind::Optional { body, .. } => pending.push(Work::Borrowed(body)),
                ExprKind::OptionalChain(value) => if self.optional_borrowed_hint(context, value) { return true; },
                ExprKind::Selected { owned, .. } => if !owned && !expr.ty.is_some_and(Type::is_copy) { return true; },
                ExprKind::Name(name) => if context.names.get(name).is_some_and(|id| {
                    context.bindings[*id].borrowed_place.is_some() || context.bindings[*id].borrowed_element.is_some() || context.readonly_parameters.contains(id)
                }) { return true; },
                ExprKind::CoalesceAssign { target, value, .. } => if self.coalesce_hint(context, target, value, expr.span, false)
                    .or_else(|| self.expression_type_hint(context, target)).is_some_and(|ty| !ty.is_copy()) { return true; },
                ExprKind::Conditional { consequent, alternate, .. } => {
                    pending.push(Work::Borrowed(alternate)); pending.push(Work::Borrowed(consequent));
                }
                ExprKind::Field(value, _) | ExprKind::Index(value, _) => {
                    if self.expression_type_hint(context, expr).is_none_or(|ty| !ty.is_copy()) {
                        pending.push(Work::Query(value)); pending.push(Work::Borrowed(value));
                    }
                }
                ExprKind::Unary { op: UnaryOp::RestPack | UnaryOp::CoalesceSelected | UnaryOp::OptionalSelected | UnaryOp::NonNull, value, .. } => pending.push(Work::Borrowed(value)),
                ExprKind::Binary { op: BinaryOp::Coalesce | BinaryOp::PatternDefault | BinaryOp::And | BinaryOp::Or, left, right, .. } => {
                    pending.push(Work::Borrowed(right)); pending.push(Work::Borrowed(left));
                }
                _ => {},
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn name(value:&str)->Expr { Expr::new(ExprKind::Name(value.into()),Span::default()) }
    fn binding(context:&mut Context,name:&str,ty:Type)->usize {
        Analyzer::bind(context,&Token{text:name.into(),kind:TokenKind::Name,span:Span::default()},ty,false).unwrap()
    }
    #[test]
    fn mixed_borrow_source_walks_keep_positive_negative_and_short_circuit_results() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        let borrowed=binding(&mut context,"borrowed",Type::String);
        binding(&mut context,"owned",Type::String);context.readonly_parameters.insert(borrowed);
        for (leaf,expected) in [("borrowed",true),("owned",false)] {
            let mut value=name(leaf);
            for layer in 0..16_384 {
                value=match layer%3 {
                    0=>Expr::new(ExprKind::Unary{op:UnaryOp::OptionalSelected,value:Box::new(value),operator:Span::default()},Span::default()),
                    1=>Expr::new(ExprKind::Sequence(vec![value]),Span::default()),
                    _=>Expr::new(ExprKind::Conditional{predicate:Box::new(Expr::new(ExprKind::Boolean(true),Span::default())),consequent:Box::new(Expr::new(ExprKind::Integer("0".into()),Span::default())),alternate:Box::new(value)},Span::default()),
                };
            }
            assert_eq!(analyzer.borrowed_place_hint(&context,&value),expected);drop(value);
        }
        // This unselected branch has deliberately invalid type metadata. A hint
        // visit would index the record arena, so it witnesses short-circuiting.
        binding(&mut context,"unselected",Type::Record(usize::MAX));
        let value=Expr::new(ExprKind::Conditional{
            predicate:Box::new(Expr::new(ExprKind::Boolean(true),Span::default())),
            consequent:Box::new(name("borrowed")),
            alternate:Box::new(Expr::new(ExprKind::Field(Box::new(name("unselected")),"value".into()),Span::default())),
        },Span::default());
        assert!(analyzer.borrowed_place_hint(&context,&value));
    }
    #[test]
    fn deep_sequence_index_hints_keep_owned_elements_distinct_from_scalar_copies() {
        let mut analyzer=Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context=Context::new("main".into());
        for (key,ty,expected) in [("strings",Type::Array(ArrayElement::String(0)),true),("integers",Type::Array(ArrayElement::Int(0)),false),("text",Type::String,false)] {
            binding(&mut context,key,ty);
            let mut value=Expr::new(ExprKind::Index(Box::new(name(key)),Box::new(Expr::new(ExprKind::Integer("0".into()),Span::default()))),Span::default());
            for _ in 0..16_384 {value=Expr::new(ExprKind::Sequence(vec![value]),Span::default());}
            assert_eq!(analyzer.indexed_element_hint(&context,&value),expected);drop(value);
        }
    }
}
