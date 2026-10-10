//! Retained source loans belong to binding identities, never source spans.
use super::*;
use crate::native_return_borrows::{BorrowAccess, ReturnedBorrow};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Loan {
    pub root: usize,
    pub access: BorrowAccess,
    pub creation: Span,
}

impl Analyzer {
    pub(super) fn check_native_view_access(
        context: &Context,
        root: usize,
        mode: Mode,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if context.effects_only {
            return Ok(());
        }
        for &holder in &context.native_view_holders {
            let binding = &context.bindings[holder];
            if binding.transferred.is_some() || context.names.get(&binding.name) != Some(&holder) {
                continue;
            }
            for loan in binding.native_view_loans.as_deref().into_iter().flatten() {
                if loan.root == root
                    && (loan.access == BorrowAccess::Exclusive
                        || matches!(mode, Mode::Mutate | Mode::Consume))
                {
                    return Err(Diagnostic::new(
                        "native-view-conflict",
                        format!(
                            "Cannot access {:?} while native view {:?} holds conflicting access.",
                            context.bindings[root].name, binding.name
                        ),
                        span,
                    )
                    .related(loan.creation, "Native view borrows this storage here."));
                }
            }
        }
        Ok(())
    }

    pub(super) fn set_native_view_loans(context: &mut Context, holder: usize, loans: Vec<Loan>) {
        if loans.is_empty() {
            context.bindings[holder].native_view_loans = None;
            context.native_view_holders.remove(&holder);
        } else {
            context.bindings[holder].native_view_loans = Some(Box::new(loans));
            context.native_view_holders.insert(holder);
        }
    }

    pub(super) fn native_view_value_loans(
        context: &Context,
        expr: &Expr,
    ) -> Result<Vec<Loan>, Diagnostic> {
        if context.native_view_holders.is_empty() && !context.native_view_calls {
            return Ok(Vec::new());
        }
        let mut pending = vec![expr];
        let mut result = Vec::new();
        while let Some(value) = pending.pop() {
            match &value.kind {
                ExprKind::Name(_) => {
                    if let Some(id) = value.binding
                        && let Some(loans) = &context.bindings[id].native_view_loans
                    {
                        result.extend(loans.iter().cloned());
                    }
                }
                ExprKind::Call(_, args) => {
                    if let Some(CallTarget::NativeBinding(id)) = value.target
                        && let Some(contract) = crate::native_bindings::BINDINGS[id].returned_borrow
                    {
                        result.extend(Self::native_view_source_loans(
                            context,
                            &args[contract.source_parameter],
                            contract,
                        )?);
                    }
                }
                ExprKind::Sequence(values) => {
                    if let Some(last) = values.last() {
                        pending.push(last);
                    }
                }
                ExprKind::Inject { value, .. }
                | ExprKind::TypeAssertion { value, .. }
                | ExprKind::Project { value, .. } => pending.push(value),
                ExprKind::Field(value, _) | ExprKind::Index(value, _) => pending.push(value),
                ExprKind::Unary {
                    op: UnaryOp::NonNull | UnaryOp::RestPack,
                    value,
                    ..
                } => pending.push(value),
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push(consequent);
                    pending.push(alternate);
                }
                ExprKind::Array(values) => pending.extend(values),
                ExprKind::Record(fields) => pending.extend(fields.iter().map(|(_, value)| value)),
                _ => {}
            }
        }
        result.sort_by_key(|loan| (loan.root, loan.access == BorrowAccess::Exclusive));
        result.dedup_by(|right, left| right.root == left.root && right.access == left.access);
        Ok(result)
    }

    pub(super) fn native_view_source_loans(
        context: &Context,
        source: &Expr,
        contract: ReturnedBorrow,
    ) -> Result<Vec<Loan>, Diagnostic> {
        let mut pending = vec![source];
        while let Some(place) = pending.pop() {
            match &place.kind {
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push(consequent);
                    pending.push(alternate);
                }
                ExprKind::Sequence(values) => {
                    if let Some(last) = values.last() {
                        pending.push(last);
                    }
                }
                ExprKind::TypeAssertion { value, .. } | ExprKind::Project { value, .. } => {
                    pending.push(value)
                }
                _ if Self::named_storage(place) => {}
                _ => {
                    return Err(Diagnostic::new(
                        "native-view-source",
                        "Every source alternative of a stored native view requires named storage that outlives the view.",
                        place.span,
                    ));
                }
            }
        }
        let roots = Self::view_sources(context, source);
        if roots.is_empty() {
            return Err(Diagnostic::new(
                "native-view-source",
                "A stored native view requires named source storage that outlives the view.",
                source.span,
            ));
        }
        let mut result = Vec::new();
        for root in roots {
            if let Some(inherited) = &context.bindings[root].native_view_loans {
                result.extend(inherited.iter().cloned());
            }
            result.push(Loan {
                root,
                access: contract.access,
                creation: source.span,
            });
        }
        result.sort_by_key(|loan| (loan.root, loan.access == BorrowAccess::Exclusive));
        result.dedup_by(|right, left| right.root == left.root && right.access == left.access);
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn local(context: &mut Context, name: &str) -> usize {
        Analyzer::bind(
            context,
            &Token {
                text: name.into(),
                kind: TokenKind::Symbol,
                span: Span::default(),
            },
            Type::String,
            false,
        )
        .unwrap()
    }
    fn retained(context: &mut Context, owner: usize, name: &str, access: BorrowAccess) -> usize {
        let id = local(context, name);
        Analyzer::set_native_view_loans(
            context,
            id,
            vec![Loan {
                root: owner,
                access,
                creation: Span::new(10, 20),
            }],
        );
        id
    }
    #[test]
    fn readonly_native_view_blocks_owner_mutation_transfer_and_replacement() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        context.bindings[owner].rebindable = true;
        retained(&mut context, owner, "view", BorrowAccess::ReadOnly);
        assert!(
            Analyzer::access(
                &mut context,
                owner,
                Mode::Read,
                Span::default(),
                &Loans::default()
            )
            .is_ok()
        );
        for mode in [Mode::Mutate, Mode::Consume] {
            let error = Analyzer::access(
                &mut context,
                owner,
                mode,
                Span::default(),
                &Loans::default(),
            )
            .unwrap_err();
            assert_eq!(error.code, "native-view-conflict");
        }
    }
    fn name(context: &Context, id: usize) -> Expr {
        let mut expression = Expr::new(
            ExprKind::Name(context.bindings[id].name.clone()),
            Span::new(30, 40),
        );
        expression.binding = Some(id);
        expression.ty = Some(context.bindings[id].ty);
        expression
    }
    #[test]
    fn retained_native_view_blocks_replacement_without_reading_old_value() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        context.bindings[owner].rebindable = true;
        retained(&mut context, owner, "view", BorrowAccess::ReadOnly);
        let analyzer = Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        assert_eq!(
            analyzer
                .rebind_target(&context, &name(&context, owner))
                .unwrap_err()
                .code,
            "native-view-conflict"
        );
    }
    #[test]
    fn nested_reborrow_keeps_owner_and_suspends_parent() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let parent = retained(&mut context, owner, "parent", BorrowAccess::Exclusive);
        let contract = ReturnedBorrow {
            source_parameter: 0,
            access: BorrowAccess::Exclusive,
            payload: crate::native_signatures::NativeTypeId::ByteViewMut,
            slot: crate::native_return_borrows::ReturnSlot::Direct,
        };
        let loans = Analyzer::native_view_source_loans(&context, &name(&context, parent), contract)
            .unwrap();
        assert_eq!(
            loans.iter().map(|loan| loan.root).collect::<Vec<_>>(),
            vec![owner, parent]
        );
        let child = local(&mut context, "child");
        Analyzer::set_native_view_loans(&mut context, child, loans);
        assert!(
            Analyzer::check_native_view_access(&context, parent, Mode::Read, Span::default())
                .is_err()
        );
        context.names.remove("child");
        assert!(
            Analyzer::check_native_view_access(&context, parent, Mode::Mutate, Span::default())
                .is_ok()
        );
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Read, Span::default())
                .is_err()
        );
    }
    #[test]
    fn aggregate_values_keep_all_roots_and_temporary_sources_are_rejected() {
        let mut context = Context::new("test".into());
        let first = local(&mut context, "first");
        let second = local(&mut context, "second");
        let left = retained(&mut context, first, "left", BorrowAccess::ReadOnly);
        let right = retained(&mut context, second, "right", BorrowAccess::Exclusive);
        let aggregate = Expr::new(
            ExprKind::Array(vec![
                name(&context, left),
                name(&context, right),
                name(&context, left),
            ]),
            Span::default(),
        );
        let loans = Analyzer::native_view_value_loans(&context, &aggregate).unwrap();
        assert_eq!(loans.len(), 2);
        assert_eq!(loans[0].root, first);
        assert_eq!(loans[1].root, second);
        let contract = ReturnedBorrow {
            source_parameter: 0,
            access: BorrowAccess::ReadOnly,
            payload: crate::native_signatures::NativeTypeId::ByteView,
            slot: crate::native_return_borrows::ReturnSlot::ResultOk,
        };
        let temporary = Expr::new(ExprKind::String("temporary".into()), Span::default());
        assert_eq!(
            Analyzer::native_view_source_loans(&context, &temporary, contract)
                .unwrap_err()
                .code,
            "native-view-source"
        );
    }
    #[test]
    fn conditional_source_cannot_hide_a_temporary_arm() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let source = Expr::new(
            ExprKind::Conditional {
                predicate: Box::new(Expr::new(ExprKind::Boolean(true), Span::default())),
                consequent: Box::new(name(&context, owner)),
                alternate: Box::new(Expr::new(
                    ExprKind::String("temporary".into()),
                    Span::default(),
                )),
            },
            Span::default(),
        );
        let contract = ReturnedBorrow {
            source_parameter: 0,
            access: BorrowAccess::ReadOnly,
            payload: crate::native_signatures::NativeTypeId::ByteView,
            slot: crate::native_return_borrows::ReturnSlot::Direct,
        };
        assert_eq!(
            Analyzer::native_view_source_loans(&context, &source, contract)
                .unwrap_err()
                .code,
            "native-view-source"
        );
    }
    #[test]
    fn exclusive_native_view_blocks_reads_until_scope_ends() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        retained(&mut context, owner, "view", BorrowAccess::Exclusive);
        assert_eq!(
            Analyzer::access(
                &mut context,
                owner,
                Mode::Read,
                Span::default(),
                &Loans::default()
            )
            .unwrap_err()
            .code,
            "native-view-conflict"
        );
        context.names.remove("view");
        assert!(
            Analyzer::access(
                &mut context,
                owner,
                Mode::Read,
                Span::default(),
                &Loans::default()
            )
            .is_ok()
        );
    }
    #[test]
    fn transferred_view_releases_old_holder_but_moved_value_keeps_loan() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let view = retained(&mut context, owner, "view", BorrowAccess::ReadOnly);
        let loans = context.bindings[view].native_view_loans.clone();
        Analyzer::access(
            &mut context,
            view,
            Mode::Consume,
            Span::default(),
            &Loans::default(),
        )
        .unwrap();
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_ok()
        );
        let moved = local(&mut context, "moved");
        Analyzer::set_native_view_loans(&mut context, moved, *loans.unwrap());
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_err()
        );
    }
}
