//! Assertion and rest wrappers suspend children on the shared checker driver.
use super::super::*;

pub(super) enum Frame {
    Rest,
    Assertion { target: Type, satisfies: bool, borrowed_injection: bool, expected: Option<Type> },
}

impl Frame {
    pub(super) fn child<'a>(&self, expr: &'a mut Expr) -> &'a mut Box<Expr> {
        match (&self, &mut expr.kind) {
            (Self::Rest, ExprKind::Unary { op: UnaryOp::RestPack, value, .. })
            | (Self::Assertion { .. }, ExprKind::TypeAssertion { value, .. }) => value,
            _ => unreachable!("wrapper operand"),
        }
    }
}

impl Analyzer {
    pub(super) fn prepare_assertion_work(&mut self, context: &Context, expr: &Expr, mode: Mode, expected: Option<Type>) -> Result<(Frame, Option<Type>), Diagnostic> {
        let ExprKind::TypeAssertion { value, annotation, satisfies } = &expr.kind else { unreachable!("assertion") };
        let target = self.callable_resolver().resolve_value_annotation(annotation, false)?;
        let numeric_widening = target == Type::Number && self.expression_type_hint(context, value) == Some(Type::Int);
        let needs_context = matches!(&value.kind, ExprKind::Record(_) | ExprKind::Closure { .. })
            || matches!(&value.kind, ExprKind::Array(items) if items.is_empty());
        let contextual = ((!*satisfies || needs_context) && !numeric_widening).then_some(target);
        let borrowed_injection = !*satisfies && mode == Mode::Read
            && matches!(target, Type::Union(id) if !id.copy)
            && self.expression_type_hint(context, value).is_some_and(|ty| ty.union().is_none())
            && (self.borrowed_place_hint(context, value) || self.indexed_element_hint(context, value));
        Ok((Frame::Assertion { target, satisfies: *satisfies, borrowed_injection, expected }, if borrowed_injection { None } else { contextual }))
    }

    pub(super) fn finish_wrapper_work(&mut self, expr: &mut Expr, frame: Frame, checked: Result<Type, Diagnostic>) -> Result<Type, Diagnostic> {
        let mut actual = checked?;
        let Frame::Assertion { target, satisfies, borrowed_injection, expected } = frame else {
            let ExprKind::Unary { value, .. } = &mut expr.kind else { unreachable!("rest pack") };
            value.ty = Some(actual);
            expr.ty = Some(actual);
            return Ok(actual);
        };
        let ExprKind::TypeAssertion { value, annotation, .. } = &mut expr.kind else { unreachable!("assertion") };
        if borrowed_injection {
            self.require(actual, Some(target), annotation.span)?;
            let union = target.union().expect("borrowed union target");
            value.inject(union);
            value.ty = Some(Type::BorrowedUnion(union));
            actual = Type::BorrowedUnion(union);
        }
        if !(actual == Type::Int && target == Type::Number) {
            self.require(actual, Some(target), annotation.span).map_err(|mut error| {
                error.message.push_str(" Type assertions cannot narrow an incompatible value; use typeof to narrow a union.");
                error
            })?;
        }
        let result = if satisfies || matches!(actual, Type::BorrowedUnion(_)) || matches!((actual, target), (Type::Closure(_), Type::Callback(_))) { actual } else { target };
        self.require(result, expected, expr.span)?;
        expr.binding = value.binding;
        expr.callable_origin = value.callable_origin;
        expr.ty = Some(result);
        if let Some(Type::Union(id)) = expected {
            if result != Type::Union(id) {
                expr.inject(id);
                expr.ty = Some(Type::Union(id));
                return Ok(Type::Union(id));
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyzer() -> Analyzer {
        Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap()
    }

    #[test]
    fn mixed_assertion_and_rest_wrappers_complete_and_restore_errors_on_normal_stacks() {
        let annotation = crate::type_syntax::parse("int").unwrap();
        let leaf_span = Span::new(7, 14);
        for invalid in [false, true] {
            let mut analyzer = analyzer();
            let mut context = Context::new("main".into());
            let mut expr = Expr::new(if invalid { ExprKind::Name("missing".into()) } else { ExprKind::Integer("7".into()) }, leaf_span);
            for layer in 0..16_384 {
                expr = Expr::new(if layer % 2 == 0 {
                    ExprKind::TypeAssertion { value: Box::new(expr), annotation: Box::new(annotation.clone()), satisfies: layer % 4 == 0 }
                } else {
                    ExprKind::Unary { op: UnaryOp::RestPack, operator: leaf_span, value: Box::new(expr) }
                }, leaf_span);
            }
            let result = analyzer.expression(&mut context, &mut expr, Mode::Consume, &Loans::default(), Some(Type::Int));
            if invalid {
                let error = result.err().expect("unknown leaf");
                assert_eq!(error.code, "name");
                assert_eq!(error.span, leaf_span);
            } else {
                assert_eq!(result.unwrap(), Type::Int);
            }
            let mut cursor = &expr;
            let mut wrappers = 0;
            loop {
                match &cursor.kind {
                    ExprKind::TypeAssertion { value, .. } | ExprKind::Unary { op: UnaryOp::RestPack, value, .. } => {
                        if !invalid { assert_eq!(cursor.ty, Some(Type::Int)); }
                        cursor = value;
                        wrappers += 1;
                    },
                    _ => break,
                }
            }
            assert_eq!(wrappers, 16_384);
            assert_eq!(cursor.span, leaf_span);
            assert!(matches!(&cursor.kind, ExprKind::Name(name) if invalid && name == "missing") || matches!(&cursor.kind, ExprKind::Integer(value) if !invalid && value == "7"));
            drop(expr);
        }
    }

    #[test]
    fn suspended_assertion_error_keeps_the_annotation_location_and_explanation() {
        let mut analyzer = analyzer();
        let mut context = Context::new("main".into());
        let mut annotation = crate::type_syntax::parse("string").unwrap();
        annotation.span = Span::new(19, 25);
        let mut expr = Expr::new(ExprKind::TypeAssertion {
            value: Box::new(Expr::new(ExprKind::Integer("7".into()), Span::new(0, 1))),
            annotation: Box::new(annotation),
            satisfies: true,
        }, Span::new(0, 25));
        let error = analyzer.expression(&mut context, &mut expr, Mode::Consume, &Loans::default(), None).unwrap_err();
        assert_eq!(error.span, Span::new(19, 25));
        assert!(error.message.contains("Type assertions cannot narrow an incompatible value"));
        let ExprKind::TypeAssertion { value, .. } = &expr.kind else { panic!("restored assertion") };
        assert!(matches!(&value.kind, ExprKind::Integer(value) if value == "7"));
        assert_eq!(value.ty, Some(Type::Int));
    }
}
