//! Attach checked declaration payloads to retained capture-loan ownership paths.
use super::*;
use crate::capture_loan_owners::Place;

impl Analyzer {
    pub(super) fn attach_binding_capture_loans(
        &self,
        context: &mut Context,
        value: &Expr,
        binding: usize,
        consume: bool,
    ) {
        let mut pending = vec![(
            value,
            Place {
                binding,
                fields: vec![],
            },
            consume,
        )];
        while let Some((value, destination, consume)) = pending.pop() {
            if let Some((_, fields)) = Self::direct_fields(value) {
                if let Some(binding) = Self::root_binding(value) {
                    context.capture_loan_owners.extend_from(
                        &Place { binding, fields },
                        &destination,
                        consume,
                    );
                }
                continue;
            }
            match &value.kind {
                ExprKind::Record(fields) => {
                    for (name, value) in fields {
                        let mut destination = destination.clone();
                        destination.fields.push(name.text.clone());
                        pending.push((value, destination, consume));
                    }
                }
                ExprKind::Closure { id, .. } => {
                    let ids = context
                        .capture_loans
                        .iter()
                        .filter(|(_, _, span)| *span == value.span)
                        .map(|loan| context.capture_loan_ids.intern(*loan))
                        .collect::<Vec<_>>();
                    context.capture_loan_owners.attach(destination.clone(), ids);
                    if let Some(closure) = self.closures.get(*id) {
                        for capture in &closure.captures {
                            let mut target = destination.clone();
                            target.fields.push(format!("$capture{}", capture.inner));
                            context.capture_loan_owners.extend_from(
                                &Place {
                                    binding: capture.outer,
                                    fields: vec![],
                                },
                                &target,
                                capture.storage == Mode::Consume,
                            );
                        }
                    }
                }
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push((consequent, destination.clone(), consume));
                    pending.push((alternate, destination, consume));
                }
                ExprKind::TypeAssertion { value, .. }
                | ExprKind::Project { value, .. }
                | ExprKind::Inject { value, .. }
                | ExprKind::Unary {
                    op:
                        UnaryOp::CallableStore
                        | UnaryOp::CallableOnce
                        | UnaryOp::CoalesceSelected
                        | UnaryOp::OptionalSelected
                        | UnaryOp::NonNull,
                    value,
                    ..
                } => pending.push((value, destination, consume)),
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn record_field_origins_follow_a_checked_declaration_move() {
        let parsed =
            crate::syntax::parse("function main(){const pair={left:()=>1,right:()=>2};}").unwrap();
        let StmtKind::Let { value, .. } = &parsed.functions[0].body[0].kind else {
            panic!("record declaration");
        };
        let ExprKind::Record(fields) = &value.kind else {
            panic!("record fields");
        };
        let left_loan = (0, Mode::Read, fields[0].1.span);
        let right_loan = (1, Mode::Read, fields[1].1.span);
        let analyzer = Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context = Context::new("main".into());
        context.capture_loans.extend([left_loan, right_loan]);
        analyzer.attach_binding_capture_loans(&mut context, value, 2, true);
        let left_id = context.capture_loan_ids.intern(left_loan);
        let right_id = context.capture_loan_ids.intern(right_loan);
        assert_eq!(
            context.capture_loan_owners.live(),
            std::collections::BTreeSet::from([left_id, right_id])
        );
        let mut root = Expr::new(ExprKind::Name("pair".into()), Span::new(0, 1));
        root.binding = Some(2);
        let moved = Expr::new(
            ExprKind::Field(Box::new(root), "left".into()),
            Span::new(0, 2),
        );
        analyzer.attach_binding_capture_loans(&mut context, &moved, 3, false);
        assert!(context.capture_loan_owners.leave_scope(3).is_empty());
        assert_eq!(
            context.capture_loan_owners.live(),
            std::collections::BTreeSet::from([left_id, right_id])
        );
        analyzer.attach_binding_capture_loans(&mut context, &moved, 3, true);
        assert_eq!(
            context.capture_loan_owners.leave_scope(3),
            std::collections::BTreeSet::from([left_id])
        );
        assert_eq!(
            context.capture_loan_owners.live(),
            std::collections::BTreeSet::from([right_id])
        );
    }
}
