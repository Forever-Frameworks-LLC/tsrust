//! Retained source loans belong to binding identities, never source spans.
use super::*;
use crate::native_return_borrows::{BorrowAccess, ReturnedBorrow};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Loan {
    pub root: usize,
    pub owner_fields: Vec<String>,
    pub source: LifetimeSource,
    pub access: BorrowAccess,
    pub creation: Span,
    pub retained: bool,
    pub path: Vec<Segment>,
    pub guards: Vec<VariantGuard>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum LifetimeSource {
    Storage,
    Carried(Vec<Segment>),
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Segment {
    Field(String),
    Element,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct VariantGuard {
    pub path: Vec<Segment>,
    pub success: usize,
}

pub(super) fn join(
    left: Option<Box<Vec<Loan>>>,
    right: Option<Box<Vec<Loan>>>,
) -> Option<Box<Vec<Loan>>> {
    match (left, right) {
        (None, other) | (other, None) => other,
        (Some(mut left), Some(right)) => {
            left.extend(*right);
            native_view_projections::normalize(&mut left);
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
        Self::check_native_view_place(
            context,
            root,
            native_view_places::access_fields(context, root),
            mode,
            span,
        )
    }

    pub(super) fn check_native_view_place(
        context: &Context,
        root: usize,
        fields: &[String],
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
                    && native_view_places::overlaps(&loan.owner_fields, fields)
                    && native_view_projections::possible(context, holder, loan)
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
        enum Task<'a> {
            Value(&'a Expr),
            Returned(&'a Expr, &'a Loan),
            FinishReturn(usize, &'a Loan),
            Wrapped(&'a Expr, Segment),
            Wrap(usize, Segment),
            Select(usize, Segment),
            Narrow(usize, Type),
            Guard(usize, usize),
        }
        let mut pending = vec![Task::Value(expr)];
        let mut result: Vec<Loan> = Vec::new();
        while let Some(task) = pending.pop() {
            let value = match task {
                Task::Returned(value, output) => {
                    pending.push(Task::FinishReturn(result.len(), output));
                    pending.push(Task::Value(value));
                    continue;
                }
                Task::FinishReturn(start, output) => {
                    let mut index=0;
                    result.retain_mut(|loan| {
                        index+=1;
                        index<=start || super::native_view_calls::transport(loan,output)
                    });
                    continue;
                }
                Task::Value(value) => value,
                Task::Wrapped(value, segment) => {
                    pending.push(Task::Wrap(result.len(), segment));
                    pending.push(Task::Value(value));
                    continue;
                }
                Task::Wrap(start, segment) => {
                    for loan in &mut result[start..] {
                        native_view_projections::wrap(loan, &segment);
                    }
                    continue;
                }
                Task::Guard(start, variant) => {
                    for loan in &mut result[start..] {
                        native_view_projections::guard(loan, variant);
                    }
                    continue;
                }
                Task::Select(start, segment) => {
                    let mut index = 0;
                    result.retain_mut(|loan| {
                        index += 1;
                        index <= start || native_view_projections::select(loan, &segment)
                    });
                    continue;
                }
                Task::Narrow(start, ty) => {
                    let mut index = 0;
                    result.retain_mut(|loan| {
                        index += 1;
                        index <= start || native_view_projections::narrow(loan, ty)
                    });
                    continue;
                }
            };
            if value.ty.is_some_and(Type::is_copy) {
                continue;
            }
            match &value.kind {
                ExprKind::Name(_) => {
                    if let Some(id) = value.binding
                        && let Some(loans) = &context.bindings[id].native_view_loans
                    {
                        result.extend(loans.iter().cloned());
                    }
                }
                ExprKind::Call(_, args) => {
                    if let Some(CallTarget::Function(id)|CallTarget::ForwardFunction(id))=value.target
                        && let Some(outputs)=context.native_view_function_returns.as_ref().and_then(|summaries|summaries.get(&id)) {
                        for output in outputs {
                            let argument=args.get(output.root).ok_or_else(||Diagnostic::new("native-view-call", "Returned capture lifetime transport is not yet planned.",value.span))?;
                            match &output.source {
                                LifetimeSource::Carried(_)=>pending.push(Task::Returned(argument,output)),
                                LifetimeSource::Storage=>{
                                    result.extend(super::native_view_calls::storage(context,argument,output)?);
                                    pending.push(Task::Returned(argument,output));
                                }
                            }
                        }
                    }
                    if let Some(CallTarget::NativeBinding(id)) = value.target
                        && let Some(contract) = crate::native_bindings::BINDINGS[id].returned_borrow
                    {
                        let mut loans = Self::native_view_source_loans(
                            context,
                            &args[contract.source_parameter],
                            contract,
                        )?;
                        if contract.slot == crate::native_return_borrows::ReturnSlot::ResultOk {
                            let variant = value.ty.and_then(Type::union).and_then(|union| context.native_view_results.as_ref()?.get(&union).copied()).ok_or_else(|| Diagnostic::new("native-view-result", "A native Result view requires its checked success identity.",value.span))?;
                            for loan in &mut loans {
                                native_view_projections::wrap(
                                    loan,
                                    &Segment::Field("payload".into()),
                                );
                                native_view_projections::guard(loan, variant);
                            }
                        }
                        result.extend(loans);
                    }
                }
                ExprKind::Sequence(values) => {
                    if let Some(last) = values.last() {
                        pending.push(Task::Value(last));
                    }
                }
                ExprKind::Inject { value, .. } => {
                    if let Some(Type::Record(id)) = value.ty {
                        pending.push(Task::Guard(result.len(), id));
                    }
                    pending.push(Task::Value(value));
                }
                ExprKind::TypeAssertion { value, .. } => pending.push(Task::Value(value)),
                ExprKind::Project {
                    value, narrowed, ..
                } => {
                    pending.push(Task::Narrow(result.len(), *narrowed));
                    pending.push(Task::Value(value));
                }
                ExprKind::Field(value, field) => {
                    pending.push(Task::Select(result.len(), Segment::Field(field.clone())));
                    pending.push(Task::Value(value));
                }
                ExprKind::Index(value, _) => {
                    pending.push(Task::Select(result.len(), Segment::Element));
                    pending.push(Task::Value(value));
                }
                ExprKind::Unary {
                    op: UnaryOp::NonNull | UnaryOp::RestPack,
                    value,
                    ..
                } => pending.push(Task::Value(value)),
                ExprKind::Conditional {
                    consequent,
                    alternate,
                    ..
                } => {
                    pending.push(Task::Value(consequent));
                    pending.push(Task::Value(alternate));
                }
                ExprKind::Array(values) => {
                    for value in values {
                        pending.push(Task::Wrapped(value, Segment::Element));
                    }
                }
                ExprKind::Record(fields) => {
                    for (field, value) in fields {
                        pending.push(Task::Wrapped(value, Segment::Field(field.text.clone())));
                    }
                }
                _ => {}
            }
        }
        for loan in &mut result {
            loan.retained = true;
        }
        native_view_projections::normalize(&mut result);
        Ok(result)
    }

    pub(super) fn native_view_source_loans(
        context: &Context,
        source: &Expr,
        contract: ReturnedBorrow,
    ) -> Result<Vec<Loan>, Diagnostic> {
        let places = native_view_places::sources(context, source)?;
        let mut result = Self::native_view_value_loans(context, source)?;
        for loan in &mut result {
            loan.path.clear();
            loan.guards.clear();
        }
        for place in places {
            result.push(Loan {
                root: place.root,
                owner_fields: place.fields,
                source: LifetimeSource::Storage,
                access: contract.access,
                creation: source.span,
                retained: true,
                path: Vec::new(),
                guards: Vec::new(),
            });
        }
        for loan in &mut result {
            loan.retained = true;
        }
        native_view_projections::normalize(&mut result);
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
                owner_fields: Vec::new(),
                source: LifetimeSource::Storage,
                access,
                creation: Span::new(10, 20),
                retained: true,
                path: Vec::new(),
                guards: Vec::new(),
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
                owner_fields: Vec::new(),
                source: LifetimeSource::Storage,
                access: BorrowAccess::Exclusive,
                creation: Span::new(20, 30),
                retained: true,
                path: Vec::new(),
                guards: Vec::new(),
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
        let formatted =
            crate::formatting::format("function main(){if(flag){view=left;}else{view=right;}}")
                .unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut parsed = crate::syntax::parse(&formatted).unwrap();
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
    fn result_error_projection_drops_success_provenance() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let result = retained(&mut context, owner, "result", BorrowAccess::ReadOnly);
        context.bindings[result].native_view_loans.as_mut().unwrap()[0]
            .guards
            .push(VariantGuard {
                path: vec![],
                success: 10,
            });
        let projected = Expr::new(
            ExprKind::Project {
                value: Box::new(name(&context, result)),
                union: UnionId {
                    index: 0,
                    copy: false,
                },
                narrowed: Type::Record(11),
            },
            Span::default(),
        );
        assert!(
            Analyzer::native_view_value_loans(&context, &projected)
                .unwrap()
                .is_empty()
        );
    }
    #[test]
    fn result_error_flow_releases_success_loan_until_branches_join() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let result = retained(&mut context, owner, "result", BorrowAccess::ReadOnly);
        context.bindings[result].native_view_loans.as_mut().unwrap()[0]
            .guards
            .push(VariantGuard {
                path: vec![],
                success: 10,
            });
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_err()
        );
        context.flow.insert(
            crate::flow::Place {
                root: result,
                fields: vec![],
            },
            crate::flow::Fact {
                members: vec![Type::Record(11)],
                origin: None,
            },
        );
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_ok()
        );
        context.flow.clear();
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_err()
        );
    }
    #[test]
    fn record_selection_keeps_only_selected_field_provenance() {
        let mut context = Context::new("test".into());
        let first = local(&mut context, "first");
        let second = local(&mut context, "second");
        let left = retained(&mut context, first, "left", BorrowAccess::ReadOnly);
        let right = retained(&mut context, second, "right", BorrowAccess::ReadOnly);
        let token = |name: &str| Token {
            text: name.into(),
            kind: TokenKind::Symbol,
            span: Span::default(),
        };
        let record = Expr::new(
            ExprKind::Record(vec![
                (token("left"), name(&context, left)),
                (token("right"), name(&context, right)),
            ]),
            Span::default(),
        );
        let selected = Expr::new(
            ExprKind::Field(Box::new(record), "left".into()),
            Span::default(),
        );
        let loans = Analyzer::native_view_value_loans(&context, &selected).unwrap();
        assert_eq!(loans.len(), 1);
        assert_eq!(loans[0].root, first);
        assert!(loans[0].path.is_empty());
    }
    #[test]
    fn success_payload_projection_removes_guard_and_payload_path() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let result = retained(&mut context, owner, "result", BorrowAccess::ReadOnly);
        let loan = &mut context.bindings[result].native_view_loans.as_mut().unwrap()[0];
        loan.path.push(Segment::Field("payload".into()));
        loan.guards.push(VariantGuard {
            path: vec![],
            success: 10,
        });
        let projected = Expr::new(
            ExprKind::Project {
                value: Box::new(name(&context, result)),
                union: UnionId {
                    index: 0,
                    copy: false,
                },
                narrowed: Type::Record(10),
            },
            Span::default(),
        );
        let selected = Expr::new(
            ExprKind::Field(Box::new(projected), "payload".into()),
            Span::default(),
        );
        let loans = Analyzer::native_view_value_loans(&context, &selected).unwrap();
        assert_eq!(loans.len(), 1);
        assert!(loans[0].guards.is_empty() && loans[0].path.is_empty());
        let view = local(&mut context, "view");
        Analyzer::set_native_view_loans(&mut context, view, loans);
        context.names.remove("result");
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_err()
        );
    }
    #[test]
    fn checked_result_match_allows_owner_replacement_in_error_arm() {
        let source = r#"type Outcome=Result<string,int>;type Holder={outcome:Outcome;};function main(){match(result){Ok(value)=>{}Err(error)=>{owner="changed";}}}"#;
        let formatted = crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut parsed = crate::syntax::parse(&formatted).unwrap();
        let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
        let holder = analyzer
            .program
            .records
            .iter()
            .position(|record| record.name.text == "Holder")
            .unwrap();
        let ty = analyzer.program.records[holder].fields[0].ty.unwrap();
        let (_, variants) = analyzer.result_variant_records(ty).unwrap();
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        context.bindings[owner].rebindable = true;
        let result = retained(&mut context, owner, "result", BorrowAccess::ReadOnly);
        context.bindings[result].ty = ty;
        let loan = &mut context.bindings[result].native_view_loans.as_mut().unwrap()[0];
        loan.path.push(Segment::Field("payload".into()));
        loan.guards.push(VariantGuard {
            path: vec![],
            success: variants[0],
        });
        let before = context.clone();
        let mut body = parsed.functions.remove(0).body;
        analyzer.block(&mut context, &mut body, Type::Void).unwrap();
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_ok()
        );
        let source =
            r#"function main(){match(result){Ok(value)=>{owner="changed";}Err(error)=>{}}}"#;
        let formatted = crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut body = crate::syntax::parse(&formatted)
            .unwrap()
            .functions
            .remove(0)
            .body;
        let error = analyzer
            .block(&mut before.clone(), &mut body, Type::Void)
            .unwrap_err();
        assert_eq!(error.code, "native-view-conflict");
    }
    #[test]
    fn error_field_guard_does_not_release_unconditional_sibling_loan() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let holder = retained(&mut context, owner, "holder", BorrowAccess::ReadOnly);
        let mut guarded = context.bindings[holder].native_view_loans.as_ref().unwrap()[0].clone();
        guarded.path = vec![
            Segment::Field("first".into()),
            Segment::Field("payload".into()),
        ];
        guarded.guards = vec![VariantGuard {
            path: vec![Segment::Field("first".into())],
            success: 10,
        }];
        let mut direct = guarded.clone();
        direct.path = vec![Segment::Field("second".into())];
        direct.guards.clear();
        let mut loans = vec![guarded, direct];
        native_view_projections::normalize(&mut loans);
        assert_eq!(loans.len(), 2);
        Analyzer::set_native_view_loans(&mut context, holder, loans);
        context.flow.insert(
            crate::flow::Place {
                root: holder,
                fields: vec!["first".into()],
            },
            crate::flow::Fact {
                members: vec![Type::Record(11)],
                origin: None,
            },
        );
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_err()
        );
        context.bindings[holder]
            .native_view_loans
            .as_mut()
            .unwrap()
            .retain(|loan| !loan.guards.is_empty());
        assert!(
            Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                .is_ok()
        );
    }
    #[test]
    fn native_owner_field_loan_allows_disjoint_sibling_access() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let view = retained(&mut context, owner, "view", BorrowAccess::Exclusive);
        context.bindings[view].native_view_loans.as_mut().unwrap()[0].owner_fields =
            vec!["left".into()];
        context.projection = Some(Projection {
            root: "owner".into(),
            path: vec!["right".into()],
            replacing: false,
            span: Span::default(),
        });
        assert!(
            Analyzer::access(
                &mut context,
                owner,
                Mode::Mutate,
                Span::default(),
                &Loans::default()
            )
            .is_ok()
        );
        context.projection.as_mut().unwrap().path = vec!["left".into(), "nested".into()];
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
        context.projection = None;
        assert!(
            Analyzer::access(
                &mut context,
                owner,
                Mode::Read,
                Span::default(),
                &Loans::default()
            )
            .is_err()
        );
    }
    #[test]
    fn moving_one_holder_field_does_not_retain_its_old_loan() {
        let mut context = Context::new("test".into());
        let first = local(&mut context, "first");
        let second = local(&mut context, "second");
        let holder = retained(&mut context, first, "holder", BorrowAccess::ReadOnly);
        let mut left = context.bindings[holder].native_view_loans.as_ref().unwrap()[0].clone();
        left.path = vec![Segment::Field("left".into())];
        let mut right = left.clone();
        right.root = second;
        right.path = vec![Segment::Field("right".into())];
        Analyzer::set_native_view_loans(&mut context, holder, vec![left, right]);
        context.projection = Some(Projection {
            root: "holder".into(),
            path: vec!["left".into()],
            replacing: false,
            span: Span::default(),
        });
        Analyzer::access(
            &mut context,
            holder,
            Mode::Consume,
            Span::default(),
            &Loans::default(),
        )
        .unwrap();
        context.projection = None;
        assert!(
            Analyzer::check_native_view_access(&context, first, Mode::Mutate, Span::default())
                .is_ok()
        );
        assert!(
            Analyzer::check_native_view_access(&context, second, Mode::Mutate, Span::default())
                .is_err()
        );
    }
    fn record_context() -> (Analyzer, Context, usize, usize, usize) {
        let analyzer = Analyzer::new(
            crate::syntax::parse("type Pair={left:string;right:string;};function main(){}")
                .unwrap(),
        )
        .unwrap();
        let pair = analyzer
            .program
            .records
            .iter()
            .position(|record| record.name.text == "Pair")
            .unwrap();
        let mut context = Context::new("test".into());
        let first = local(&mut context, "first");
        let second = local(&mut context, "second");
        let holder = retained(&mut context, first, "holder", BorrowAccess::ReadOnly);
        context.bindings[holder].ty = Type::Record(pair);
        let mut left = context.bindings[holder].native_view_loans.as_ref().unwrap()[0].clone();
        left.path = vec![Segment::Field("left".into())];
        let mut right = left.clone();
        right.root = second;
        right.path = vec![Segment::Field("right".into())];
        Analyzer::set_native_view_loans(&mut context, holder, vec![left, right]);
        (analyzer, context, holder, first, second)
    }
    fn source_body(source: &str) -> Vec<Stmt> {
        let formatted = crate::formatting::format(source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        crate::syntax::parse(&formatted)
            .unwrap()
            .functions
            .remove(0)
            .body
    }
    #[test]
    fn checked_field_replacement_releases_only_replaced_origins() {
        let (mut analyzer, mut context, _, first, second) = record_context();
        let third = local(&mut context, "third");
        retained(&mut context, third, "replacement", BorrowAccess::ReadOnly);
        let mut body = source_body("function main(){holder.left=replacement;}");
        analyzer.block(&mut context, &mut body, Type::Void).unwrap();
        assert!(
            Analyzer::check_native_view_access(&context, first, Mode::Mutate, Span::default())
                .is_ok()
        );
        for owner in [second, third] {
            assert!(
                Analyzer::check_native_view_access(&context, owner, Mode::Mutate, Span::default())
                    .is_err()
            );
        }
    }
    #[test]
    fn checked_field_move_releases_old_holder_and_keeps_sibling_loan() {
        let (mut analyzer, mut context, _, first, second) = record_context();
        context.bindings[first].rebindable = true;
        let before = context.clone();
        let mut body = source_body("function main(){const moved=holder.left;}");
        analyzer.block(&mut context, &mut body, Type::Void).unwrap();
        assert!(
            Analyzer::check_native_view_access(&context, first, Mode::Mutate, Span::default())
                .is_ok()
        );
        assert!(
            Analyzer::check_native_view_access(&context, second, Mode::Mutate, Span::default())
                .is_err()
        );
        let mut body = source_body(r#"function main(){const moved=holder.left;first="changed";}"#);
        assert_eq!(
            analyzer
                .block(&mut before.clone(), &mut body, Type::Void)
                .unwrap_err()
                .code,
            "native-view-conflict"
        );
    }
    #[test]
    fn checked_owner_field_access_keeps_sibling_mutable() {
        let (mut analyzer, mut context, owner, _, _) = record_context();
        context.bindings[owner].native_view_loans = None;
        let view = retained(&mut context, owner, "view", BorrowAccess::Exclusive);
        context.bindings[view].native_view_loans.as_mut().unwrap()[0].owner_fields =
            vec!["left".into()];
        let mut body = source_body(r#"function main(){holder.right="changed";}"#);
        analyzer.block(&mut context, &mut body, Type::Void).unwrap();
        let mut body = source_body(r#"function main(){holder.left="changed";}"#);
        assert_eq!(
            analyzer
                .block(&mut context, &mut body, Type::Void)
                .unwrap_err()
                .code,
            "native-view-conflict"
        );
    }
    #[test]
    fn conditional_sources_keep_distinct_fields_of_one_owner() {
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner");
        let field = |field: &str| {
            Expr::new(
                ExprKind::Field(Box::new(name(&context, owner)), field.into()),
                Span::default(),
            )
        };
        let source = Expr::new(
            ExprKind::Conditional {
                predicate: Box::new(Expr::new(ExprKind::Boolean(true), Span::default())),
                consequent: Box::new(field("left")),
                alternate: Box::new(field("right")),
            },
            Span::default(),
        );
        let contract = ReturnedBorrow {
            source_parameter: 0,
            access: BorrowAccess::ReadOnly,
            payload: crate::native_signatures::NativeTypeId::ByteView,
            slot: crate::native_return_borrows::ReturnSlot::Direct,
        };
        let loans = Analyzer::native_view_source_loans(&context, &source, contract).unwrap();
        assert_eq!(loans.len(), 2);
        assert_eq!(loans[0].owner_fields, vec!["left"]);
        assert_eq!(loans[1].owner_fields, vec!["right"]);
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
