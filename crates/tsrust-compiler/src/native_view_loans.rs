//! Retained source loans belong to binding identities, never source spans.
use super::*;
use crate::native_return_borrows::{BorrowAccess, ReturnedBorrow};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Loan {
    pub root: usize,
    pub access: BorrowAccess,
    pub creation: Span,
    pub retained: bool,
}

pub(super) fn join(
    left: Option<Box<Vec<Loan>>>,
    right: Option<Box<Vec<Loan>>>,
) -> Option<Box<Vec<Loan>>> {
    match (left, right) {
        (None, other) | (other, None) => other,
        (Some(mut left), Some(right)) => {
            left.extend(*right);
            left.sort_by_key(|loan| (loan.root, loan.access == BorrowAccess::Exclusive));
            left.dedup_by(|right, left| {
                if right.root != left.root || right.access != left.access {
                    return false;
                }
                left.retained |= right.retained;
                true
            });
            Some(left)
        }
    }
}

impl Analyzer {
    pub(super) fn check_native_view_access(
        context: &Context,
        root: usize,
        mode: Mode,
        span: Span,
    ) -> Result<(), Diagnostic> {
        if context.effects_only || !context.native_view_calls {
            return Ok(());
        }
        for (holder, binding) in context.bindings.iter().enumerate() {
            if context.names.get(&binding.name) != Some(&holder) {
                continue;
            }
            for loan in binding.native_view_loans.as_deref().into_iter().flatten() {
                if loan.retained
                    && loan.root == root
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
        } else {
            context.bindings[holder].native_view_loans = Some(Box::new(loans));
            context.native_view_calls = true;
        }
    }

    pub(super) fn native_view_value_loans(
        context: &Context,
        expr: &Expr,
    ) -> Result<Vec<Loan>, Diagnostic> {
        if !context.native_view_calls {
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
        for loan in &mut result {
            loan.retained = true;
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
                retained: true,
            });
        }
        for loan in &mut result {
            loan.retained = true;
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
                retained: true,
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
    fn branch_restore_and_join_keep_each_reaching_owner_loan() {
        let mut context = Context::new("test".into());
        let first = local(&mut context, "first");
        let second = local(&mut context, "second");
        let view = retained(&mut context, first, "view", BorrowAccess::ReadOnly);
        let before = Availability::of(&context.bindings[view]);
        Analyzer::set_native_view_loans(
            &mut context,
            view,
            vec![Loan {
                root: second,
                access: BorrowAccess::Exclusive,
                creation: Span::new(20, 30),
                retained: true,
            }],
        );
        let alternative = Availability::of(&context.bindings[view]);
        before.clone().restore(&mut context.bindings[view]);
        assert!(
            Analyzer::check_native_view_access(&context, first, Mode::Mutate, Span::default())
                .is_err()
        );
        assert!(
            Analyzer::check_native_view_access(&context, second, Mode::Read, Span::default())
                .is_ok()
        );
        before
            .join(alternative)
            .restore(&mut context.bindings[view]);
        assert!(
            Analyzer::check_native_view_access(&context, first, Mode::Mutate, Span::default())
                .is_err()
        );
        assert!(
            Analyzer::check_native_view_access(&context, second, Mode::Read, Span::default())
                .is_err()
        );
    }
    #[test]
    fn transfer_on_one_branch_does_not_release_other_branch_loan() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let view = retained(&mut context, owner, "view", BorrowAccess::ReadOnly);
        let active = Availability::of(&context.bindings[view]);
        Analyzer::access(
            &mut context,
            view,
            Mode::Consume,
            Span::default(),
            &Loans::default(),
        )
        .unwrap();
        let moved = Availability::of(&context.bindings[view]);
        active.join(moved).restore(&mut context.bindings[view]);
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_err()
        );
    }
    #[test]
    fn checked_if_replacement_retains_origins_from_both_arms() {
        let mut context = Context::new("test".into());
        let first = local(&mut context, "first");
        let second = local(&mut context, "second");
        retained(&mut context, first, "left", BorrowAccess::ReadOnly);
        retained(&mut context, second, "right", BorrowAccess::ReadOnly);
        let view = local(&mut context, "view");
        context.bindings[view].rebindable = true;
        Analyzer::bind(
            &mut context,
            &Token {
                text: "flag".into(),
                kind: TokenKind::Symbol,
                span: Span::default(),
            },
            Type::Boolean,
            false,
        )
        .unwrap();
        let mut parsed =
            crate::syntax::parse("function main(){if(flag){view=left;}else{view=right;}}").unwrap();
        let mut body = parsed.functions.remove(0).body;
        let mut analyzer =
            Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        analyzer.block(&mut context, &mut body, Type::Void).unwrap();
        context.names.remove("left");
        context.names.remove("right");
        assert!(
            Analyzer::check_native_view_access(&context, first, Mode::Mutate, Span::default())
                .is_err()
        );
        assert!(
            Analyzer::check_native_view_access(&context, second, Mode::Mutate, Span::default())
                .is_err()
        );
    }
    #[test]
    fn transfer_on_both_paths_releases_retained_loan() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let view = retained(&mut context, owner, "view", BorrowAccess::Exclusive);
        Analyzer::access(
            &mut context,
            view,
            Mode::Consume,
            Span::default(),
            &Loans::default(),
        )
        .unwrap();
        let moved = Availability::of(&context.bindings[view]);
        moved
            .clone()
            .join(moved)
            .restore(&mut context.bindings[view]);
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_ok()
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
        let expression = name(&context, view);
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
        let loans = Analyzer::native_view_value_loans(&context, &expression).unwrap();
        Analyzer::set_native_view_loans(&mut context, moved, loans);
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_err()
        );
    }
}
