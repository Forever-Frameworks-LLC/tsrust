//! Returned native views keep their source parameters and reject local owners.
use super::native_view_loans::LifetimeSource;
use super::*;

impl Analyzer {
    pub(super) fn record_native_return(
        &self,
        context: &mut Context,
        value: &Expr,
    ) -> Result<(), Diagnostic> {
        if !context.native_view_calls || context.effects_only {
            return Ok(());
        }
        let loans = Self::native_view_value_loans(context, value)?;
        let mut sources = Vec::new();
        for output in loans {
            let mut pending = vec![(
                crate::flow::Place {
                    root: output.root,
                    fields: output.owner_fields.clone(),
                },
                output.source.clone(),
                output.access == crate::native_return_borrows::BorrowAccess::Exclusive,
            )];
            let mut seen = HashSet::new();
            let start = sources.len();
            while let Some((place, kind, exclusive)) = pending.pop() {
                if !seen.insert((place.clone(), kind.clone(), exclusive)) {
                    continue;
                }
                let binding = &context.bindings[place.root];
                if binding.parameter {
                    if binding.mode == Mode::Consume && matches!(kind, LifetimeSource::Storage) {
                        return Err(escape(
                            value.span,
                            binding.declaration_span.unwrap_or(output.creation),
                            "A returned native view cannot borrow a consuming owner parameter.",
                        ));
                    }
                    let mut source = output.clone();
                    source.root = place.root;
                    source.owner_fields = place.fields;
                    source.source = kind;
                    source.access = if exclusive {
                        crate::native_return_borrows::BorrowAccess::Exclusive
                    } else {
                        crate::native_return_borrows::BorrowAccess::ReadOnly
                    };
                    sources.push(source);
                } else {
                    let Some(origins) = &binding.native_view_loans else {
                        return Err(escape(
                            value.span,
                            binding.declaration_span.unwrap_or(output.creation),
                            "A returned native view cannot borrow local owner storage.",
                        ));
                    };
                    let mut found = false;
                    for origin in origins.iter() {
                        if native_view_places::under(origin, &place.fields)
                            && native_view_projections::possible(context, place.root, origin)
                        {
                            pending.push((
                                crate::flow::Place {
                                    root: origin.root,
                                    fields: origin.owner_fields.clone(),
                                },
                                origin.source.clone(),
                                exclusive
                                    || origin.access
                                        == crate::native_return_borrows::BorrowAccess::Exclusive,
                            ));
                            found = true;
                        }
                    }
                    if !found {
                        return Err(escape(
                            value.span,
                            output.creation,
                            "A returned native view has no borrowed parameter source.",
                        ));
                    }
                }
            }
            if sources.len() == start {
                return Err(escape(
                    value.span,
                    output.creation,
                    "A returned native view has cyclic source aliases without a borrowed parameter.",
                ));
            }
        }
        if !sources.is_empty() {
            let retained = context.native_returns.get_or_insert_with(Default::default);
            retained.extend(sources);
            native_view_projections::normalize(retained);
        }
        Ok(())
    }

    pub(super) fn validate_native_returns(&self, context: &Context) -> Result<(), Diagnostic> {
        for source in context.native_returns.as_deref().into_iter().flatten() {
            let binding = &context.bindings[source.root];
            if binding.mode == Mode::Consume && matches!(source.source, LifetimeSource::Storage) {
                return Err(escape(
                    source.creation,
                    binding.declaration_span.unwrap_or(source.creation),
                    "A returned native view cannot borrow a consuming owner parameter.",
                ));
            }
        }
        Ok(())
    }
}

fn escape(span: Span, source: Span, message: &str) -> Diagnostic {
    Diagnostic::new("native-view-escape", message, span)
        .related(source, "Borrowed source storage belongs to this callable.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::native_view_loans::Loan;
    use crate::native_return_borrows::BorrowAccess;
    fn local(context: &mut Context, name: &str, parameter: bool) -> usize {
        Analyzer::bind(
            context,
            &Token {
                text: name.into(),
                kind: TokenKind::Symbol,
                span: Span::default(),
            },
            Type::String,
            parameter,
        )
        .unwrap()
    }
    fn setup(parameter: bool) -> (Analyzer, Context, usize, usize) {
        let analyzer = Analyzer::new(crate::syntax::parse("function main(){}").unwrap()).unwrap();
        let mut context = Context::new("test".into());
        let owner = local(&mut context, "owner", parameter);
        let view = local(&mut context, "view", false);
        Analyzer::set_native_view_loans(
            &mut context,
            view,
            vec![Loan {
                root: owner,
                owner_fields: vec![],
                source: LifetimeSource::Storage,
                access: BorrowAccess::ReadOnly,
                creation: Span::new(10, 20),
                retained: true,
                path: vec![],
                guards: vec![],
            }],
        );
        (analyzer, context, owner, view)
    }
    fn value(context: &Context, id: usize) -> Option<Expr> {
        let mut value = Expr::new(
            ExprKind::Name(context.bindings[id].name.clone()),
            Span::new(30, 40),
        );
        value.binding = Some(id);
        value.ty = Some(context.bindings[id].ty);
        Some(value)
    }
    #[test]
    fn returning_view_of_local_owner_reports_escape() {
        let (mut analyzer, mut context, _, view) = setup(false);
        let mut returned = value(&context, view);
        assert_eq!(
            analyzer
                .return_statement(&mut context, &mut returned, Type::String, Span::new(30, 40))
                .unwrap_err()
                .code,
            "native-view-escape"
        );
    }
    #[test]
    fn returning_view_of_consuming_owner_parameter_reports_escape() {
        let (mut analyzer, mut context, owner, view) = setup(true);
        context.bindings[owner].mode = Mode::Consume;
        let mut returned = value(&context, view);
        assert_eq!(
            analyzer
                .return_statement(&mut context, &mut returned, Type::String, Span::new(30, 40))
                .unwrap_err()
                .code,
            "native-view-escape"
        );
    }
    fn check_return(analyzer: &mut Analyzer, context: &mut Context, view: usize) -> Result<(), Diagnostic> {
        let mut returned = value(context, view);
        analyzer.return_statement(context, &mut returned, Type::String, Span::new(30, 40)).map(|_| ())
    }
    #[test]
    fn borrowed_parameter_return_keeps_exact_source() {
        let (mut analyzer, mut context, owner, view) = setup(true);
        context.bindings[owner].mode = Mode::Read;
        check_return(&mut analyzer, &mut context, view).unwrap();
        let sources = context.native_returns.as_ref().unwrap();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].root, owner);
        assert_eq!(sources[0].source, LifetimeSource::Storage);
    }
    #[test]
    fn final_consuming_mode_rejects_previously_borrowed_return() {
        let (mut analyzer, mut context, owner, view) = setup(true);
        context.bindings[owner].mode = Mode::Read;
        check_return(&mut analyzer, &mut context, view).unwrap();
        context.bindings[owner].mode = Mode::Consume;
        assert_eq!(analyzer.validate_native_returns(&context).unwrap_err().code, "native-view-escape");
    }
    #[test]
    fn carried_input_lifetime_survives_consuming_parameter() {
        let (mut analyzer, mut context, owner, view) = setup(true);
        context.bindings[owner].mode = Mode::Consume;
        context.bindings[view].native_view_loans.as_mut().unwrap()[0].source =
            LifetimeSource::Carried(vec![native_view_loans::Segment::Field("view".into())]);
        check_return(&mut analyzer, &mut context, view).unwrap();
        analyzer.validate_native_returns(&context).unwrap();
        assert!(matches!(context.native_returns.as_ref().unwrap()[0].source, LifetimeSource::Carried(_)));
    }
    #[test]
    fn local_alias_resolves_parameter_origin() {
        let (mut analyzer, mut context, owner, view) = setup(true);
        context.bindings[owner].mode = Mode::Read;
        let alias = local(&mut context, "alias", false);
        let mut origin = context.bindings[view].native_view_loans.as_ref().unwrap()[0].clone();
        origin.root = view;
        Analyzer::set_native_view_loans(&mut context, alias, vec![origin]);
        check_return(&mut analyzer, &mut context, alias).unwrap();
        assert_eq!(context.native_returns.as_ref().unwrap()[0].root, owner);
    }
    #[test]
    fn cyclic_alias_without_parameter_is_diagnosed() {
        let (analyzer, mut context, owner, view) = setup(false);
        let mut origin = context.bindings[view].native_view_loans.as_ref().unwrap()[0].clone();
        origin.root = view;
        Analyzer::set_native_view_loans(&mut context, owner, vec![origin]);
        let returned = value(&context, view).unwrap();
        assert_eq!(analyzer.record_native_return(&mut context, &returned).unwrap_err().code, "native-view-escape");
    }
    #[test]
    fn storage_and_carried_sources_do_not_normalize_together() {
        let (_, context, _, view) = setup(true);
        let storage = context.bindings[view].native_view_loans.as_ref().unwrap()[0].clone();
        let mut carried = storage.clone();
        carried.source = LifetimeSource::Carried(vec![]);
        let mut sources = vec![storage, carried];
        native_view_projections::normalize(&mut sources);
        assert_eq!(sources.len(), 2);
    }

    #[test]
    fn input_view_has_carried_origin_without_self_conflict() {
        let mut parsed = crate::syntax::parse("type Window={};function main(){}").unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let analyzer = Analyzer::new(parsed).unwrap();
        let mut context = Context::new("test".into());
        let input = local(&mut context, "input", true);
        context.bindings[input].ty = Type::Record(0);
        analyzer.seed_native_parameters(&mut context).unwrap();
        let origins = context.bindings[input].native_view_loans.as_ref().unwrap();
        assert_eq!(origins.len(), 1);
        assert_eq!(origins[0].source, LifetimeSource::Carried(vec![]));
        assert!(!origins[0].retained);
        Analyzer::check_native_view_access(&context, input, Mode::Consume, Span::default()).unwrap();
        let returned = value(&context, input).unwrap();
        analyzer.record_native_return(&mut context, &returned).unwrap();
    }

}
