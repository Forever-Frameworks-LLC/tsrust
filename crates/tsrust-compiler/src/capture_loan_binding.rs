//! Attach checked declaration payloads to retained capture-loan ownership paths.
use super::*;
use crate::capture_loan_owners::Place;

impl Analyzer {
    pub(super) fn resolve_completed_capture_invocation(&self, context: &mut Context, expression: &Expr, result: Type) {
        if self.expression_result_retains_captures(result) { return; }
        let ExprKind::Call(callee, _) = &expression.kind else { return; };
        let Some((name, fields)) = Self::direct_fields(callee) else { return; };
        let Some(binding) = Self::root_binding(callee).or_else(|| context.names.get(&name).copied()) else { return; };
        let ty = callee.ty.unwrap_or(context.bindings[binding].ty);
        if !matches!(ty, Type::Closure(_) | Type::StoredCallable(_) | Type::BorrowedCallable(_) | Type::Callback(_) | Type::Callable(_)) { return; }
        let source = Place { binding, fields };
        let moved = context.capture_loan_owners.loans_under(&source);
        context.capture_loan_owners.resolve_consumption(&source, Self::capture_payload_site(callee), &moved);
    }

    #[inline(never)]
    pub(super) fn restore_live_capture_loans(context: &mut Context) {
        let live = context.capture_loan_owners.live();
        context.capture_loans.retain(|loan| {
            context.capture_loan_ids.get(*loan).is_none_or(|id|
                !context.capture_loan_owners.tracked().contains(&id) || live.contains(&id))
        });
        let mut present = context.capture_loans.iter().copied().collect::<std::collections::HashSet<_>>();
        for id in live {
            if let Some(loan) = context.capture_loan_ids.loan(id) {
                if present.insert(loan) {
                    context.capture_loans.push(loan);
                }
            }
        }
    }

    pub(super) fn attach_binding_capture_loans(
        &self,
        context: &mut Context,
        value: &Expr,
        binding: usize,
        consume: bool,
    ) {
        self.attach_payload_capture_loans(
            context,
            value,
            Place {
                binding,
                fields: vec![],
            },
            consume,
        );
    }

    pub(super) fn replace_target_capture_loans(
        &self,
        context: &mut Context,
        target: &Expr,
        value: &Expr,
    ) {
        let Some((_, fields)) = Self::direct_fields(target) else {
            return;
        };
        let Some(binding) = Self::root_binding(target) else {
            return;
        };
        // Stage before replacement: self-assignment and moving fields back into
        // their own record must not discard the RHS environment. This internal
        // path never becomes a source binding or survives the completed store.
        let temporary = Place {
            binding: usize::MAX,
            fields: vec![],
        };
        self.attach_payload_capture_loans(context, value, temporary.clone(), true);
        let _ = context
            .capture_loan_owners
            .transfer(&temporary, &Place { binding, fields });
        Self::restore_live_capture_loans(context);
    }

    /// Insertion adds another payload without discarding existing element loans.
    pub(super) fn append_target_capture_loans(&self, context: &mut Context, target: &Expr, value: &Expr) {
        let Some((_, fields)) = Self::direct_fields(target) else { return; };
        let Some(binding) = Self::root_binding(target) else { return; };
        self.attach_payload_capture_loans(context, value, Place { binding, fields }, true);
    }

    fn attach_payload_capture_loans(
        &self,
        context: &mut Context,
        value: &Expr,
        destination: Place,
        consume: bool,
    ) {
        let mut pending = vec![(value, destination, consume)];
        while let Some((value, destination, consume)) = pending.pop() {
            if let Some((_, fields)) = Self::direct_fields(value) {
                if let Some(binding) = Self::root_binding(value) {
                    let source = Place { binding, fields };
                    let moved = context.capture_loan_owners.loans_under(&source);
                    context.capture_loan_owners.extend_from(
                        &source,
                        &destination,
                        consume,
                    );
                    if consume {
                        context.capture_loan_owners.resolve_consumption(
                            &source, Self::capture_payload_site(value), &moved,
                        );
                    }
                }
                continue;
            }
            match &value.kind {
                ExprKind::Array(values) => {
                    // Dense storage retains every element environment. Until
                    // runtime element identity is known, aliases conservatively
                    // retain all possible capture owners of the container.
                    for value in values {
                        pending.push((value, destination.clone(), consume));
                    }
                }
                ExprKind::Index(base, _) if matches!(base.ty, Some(Type::Array(_))) => {
                    pending.push((base, destination, false));
                }
                ExprKind::Call(callee, _) => {
                    if let ExprKind::Field(base, method) = &callee.kind
                        && matches!(base.ty, Some(Type::Array(_)))
                        && matches!(method.as_str(), "pop" | "shift")
                    {
                        // Removing one runtime-selected element must not release
                        // the capture owners of elements still in the container.
                        pending.push((base, destination, false));
                    }
                }
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
                            let source = Place {
                                    binding: capture.outer,
                                    fields: vec![],
                                };
                            let moved = context.capture_loan_owners.loans_under(&source);
                            context.capture_loan_owners.extend_from(
                                &source,
                                &target,
                                capture.storage == Mode::Consume,
                            );
                            if capture.storage == Mode::Consume {
                                context.capture_loan_owners.resolve_consumption(&source, value.span, &moved);
                            }
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
                        | UnaryOp::RecordCarrier
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

    fn capture_payload_site(mut value: &Expr) -> Span {
        loop {
            match &value.kind {
                ExprKind::Sequence(values) => value = values.last().expect("nonempty sequence"),
                ExprKind::TypeAssertion { value: inner, .. }
                | ExprKind::Project { value: inner, .. }
                | ExprKind::Inject { value: inner, .. }
                | ExprKind::Unary { value: inner, .. } => value = inner,
                _ => return value.span,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn formatted(source: &str) -> String {
        let formatted = crate::format(source).unwrap();
        assert_eq!(crate::format(&formatted).unwrap(), formatted);
        formatted
    }

    #[test]
    fn reconciliation_removes_only_dead_tracked_loans_and_keeps_opaque_retention() {
        let mut context = Context::new("main".into());
        let dead = (0, Mode::Mutate, Span::new(1, 2));
        let live = (1, Mode::Read, Span::new(3, 4));
        let unknown = (2, Mode::Read, Span::new(5, 6));
        let dead_id = context.capture_loan_ids.intern(dead);
        let live_id = context.capture_loan_ids.intern(live);
        context.capture_loan_ids.intern(unknown);
        let source = Place { binding: 2, fields: vec![] };
        context.capture_loan_owners.attach(source.clone(), [dead_id]);
        context.capture_loan_owners.attach(Place { binding: 1, fields: vec![] }, [live_id]);
        let mut opaque = context.clone();
        opaque.capture_loan_owners.begin_consumption(&source, Span::new(7, 8));
        context.capture_loan_owners.leave_scope(2);
        context.capture_loans = vec![unknown, dead, live];
        Analyzer::restore_live_capture_loans(&mut context);
        assert_eq!(context.capture_loans, vec![unknown, live]);
        opaque.capture_loan_owners.leave_scope(2);
        opaque.capture_loans = vec![unknown, dead, live];
        Analyzer::restore_live_capture_loans(&mut opaque);
        assert_eq!(opaque.capture_loans, vec![unknown, dead, live]);
    }

    #[test]
    fn restored_owner_snapshot_recovers_missing_raw_loans_without_dropping_unknown_borrows() {
        let mut context = Context::new("main".into());
        let retained = (0, Mode::Mutate, Span::new(1, 2));
        let unknown = (1, Mode::Read, Span::new(3, 4));
        let id = context.capture_loan_ids.intern(retained);
        context.capture_loan_owners.attach(Place { binding: 2, fields: vec![] }, [id]);
        context.capture_loans = vec![unknown];
        Analyzer::restore_live_capture_loans(&mut context);
        assert_eq!(context.capture_loans, vec![unknown, retained]);
        Analyzer::restore_live_capture_loans(&mut context);
        assert_eq!(context.capture_loans, vec![unknown, retained]);
    }

    #[test]
    fn conditional_arm_restores_entry_loans_after_removal_and_collects_new_members() {
        let mut analyzer = Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context = Context::new("main".into());
        let first = (0, Mode::Read, Span::new(1, 2));
        let second = (1, Mode::Read, Span::new(3, 4));
        let added = (2, Mode::Mutate, Span::new(5, 6));
        context.capture_loans = vec![first, second];
        let predicate = Expr::new(ExprKind::Boolean(true), Span::new(7, 8));
        let mut consequent = Expr::new(ExprKind::Boolean(true), Span::new(9, 10));
        let mut alternate = Expr::new(ExprKind::Boolean(false), Span::new(11, 12));
        let mut plan = analyzer.conditional_plan(&mut context, &predicate, &mut consequent,
            &mut alternate, Mode::Read, Some(Type::Boolean)).unwrap();
        context.capture_loans.remove(0);
        context.capture_loans.push(added);
        analyzer.conditional_between(&mut context, &consequent, &alternate, &mut plan,
            Ok(Type::Boolean)).unwrap();
        assert_eq!(plan.left_loans, vec![added]);
        assert_eq!(context.capture_loans, vec![first, second]);
    }

    #[test]
    fn opaque_consumption_survives_source_disposal_but_wrapped_move_resolves_its_site() {
        let source = formatted(r#"
            type Holder = { run: ReadOnly<Function<int>>; };
            function main() {
                const values = [3];
                const holder: Holder = { run: (): int => values[0] };
            }
        "#);
        let mut analyzer = Analyzer::new(crate::syntax::parse(&source).unwrap()).unwrap();
        let mut body = std::mem::take(&mut analyzer.program.functions[0].body);
        let mut context = Context::new("main".into());
        analyzer.block_with_scope(&mut context, &mut body, Type::Void, true).unwrap();
        let holder = context.names["holder"];
        let ids = context.capture_loan_owners.live();
        assert!(!ids.is_empty());
        let site = Span::new(source.len(), source.len() + 1);
        Analyzer::access(&mut context, holder, Mode::Consume, site, &Loans::new()).unwrap();
        let mut opaque = context.clone();
        assert!(opaque.capture_loan_owners.leave_scope(holder).is_empty());
        assert_eq!(opaque.capture_loan_owners.live(), ids);

        let mut value = Expr::new(ExprKind::Name("holder".into()), site);
        value.binding = Some(holder);
        let wrapped = Expr::new(ExprKind::Unary {
            op: UnaryOp::NonNull, operator: Span::new(site.end, site.end + 1), value: Box::new(value),
        }, Span::new(site.start, site.end + 1));
        analyzer.attach_binding_capture_loans(&mut context, &wrapped, holder + 1, true);
        assert_eq!(context.capture_loan_owners.leave_scope(holder), ids);
        assert!(context.capture_loan_owners.live().is_empty());
    }

    #[test]
    fn named_record_destructuring_moves_only_selected_field_loan_owners() {
        let source = r#"
            type Pair = { left: ReadOnly<Function<int>>; right: ReadOnly<Function<int>>; };
            function main() {
                const leftValues = [3];
                const rightValues = [11];
                const pair: Pair = { left: (): int => leftValues[0], right: (): int => rightValues[0] };
                const { left: extracted } = pair;
            }
        "#;
        let mut analyzer =
            Analyzer::new(crate::syntax::parse(&formatted(source)).unwrap()).unwrap();
        let mut body = std::mem::take(&mut analyzer.program.functions[0].body);
        let mut context = Context::new("main".into());
        analyzer
            .block_with_scope(&mut context, &mut body, Type::Void, true)
            .unwrap();
        let extracted = context.names["extracted"];
        let left = context
            .capture_loans
            .iter()
            .find(|loan| loan.0 == context.names["leftValues"])
            .copied()
            .unwrap();
        let right = context
            .capture_loans
            .iter()
            .find(|loan| loan.0 == context.names["rightValues"])
            .copied()
            .unwrap();
        let left = context.capture_loan_ids.intern(left);
        let right = context.capture_loan_ids.intern(right);
        assert_eq!(
            context.capture_loan_owners.leave_scope(extracted),
            std::collections::BTreeSet::from([left])
        );
        assert_eq!(
            context.capture_loan_owners.live(),
            std::collections::BTreeSet::from([right])
        );
    }

    #[test]
    fn assignment_returns_payload_owners_to_outer_record_before_temporary_disposal() {
        for assignment in [
            "const temporary = holder; holder = temporary;",
            "const temporary = holder.run; holder.run = temporary;",
            "holder = holder;",
        ] {
            let source = format!(
                r#"
                type Holder = {{ run: ReadOnly<Function<int>>; }};
                function main() {{
                    const values = [3];
                    let holder: Holder = {{ run: (): int => values[0] }};
                    {{ {assignment} }}
                }}
            "#
            );
            let mut analyzer =
                Analyzer::new(crate::syntax::parse(&formatted(&source)).unwrap()).unwrap();
            let mut body = std::mem::take(&mut analyzer.program.functions[0].body);
            let mut context = Context::new("main".into());
            analyzer
                .block_with_scope(&mut context, &mut body, Type::Void, true)
                .unwrap();
            let holder = context.names["holder"];
            let before = context.capture_loan_owners.live();
            assert!(!before.is_empty());
            assert!(
                context
                    .capture_loan_owners
                    .leave_scope(holder + 1)
                    .is_empty(),
                "outer holder lost its borrowed payload: {assignment}"
            );
            assert_eq!(context.capture_loan_owners.live(), before);
        }
    }

    #[test]
    fn record_field_origins_follow_a_checked_declaration_move() {
        let parsed = crate::syntax::parse(&formatted(
            "function main(){const pair={left:()=>1,right:()=>2};}",
        ))
        .unwrap();
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
