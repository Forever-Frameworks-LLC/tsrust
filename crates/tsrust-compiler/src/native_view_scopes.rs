//! Retained native holders cannot depend on storage destroyed at a lexical boundary.
use super::*;

pub(super) fn check(context: &Context, first_local: usize) -> Result<(), Diagnostic> {
    if context.effects_only || !context.native_view_calls {
        return Ok(());
    }
    for (holder, binding) in context.bindings[..first_local].iter().enumerate() {
        for loan in binding.native_view_loans.as_deref().into_iter().flatten() {
            if !loan.retained
                || loan.root < first_local
                || !native_view_projections::possible(context, holder, loan)
            {
                continue;
            }
            let owner = context.bindings.get(loan.root);
            let name = owner.map_or("its owner", |owner| owner.name.as_str());
            let mut diagnostic = Diagnostic::new(
                "native-view-escape",
                format!(
                    "Native view {:?} retains storage from {:?} that leaves this scope. Keep the owner outside the holder's scope, or replace the retained view before leaving.",
                    binding.name, name
                ),
                loan.creation,
            );
            if let Some(span) = owner
                .and_then(|owner| {
                    owner
                        .declaration_span
                        .or_else(|| owner.initializer.as_ref().map(|value| value.span))
                })
                .or_else(|| {
                    context.source_bindings.iter().find_map(|(&start, &id)| {
                        (id == loan.root).then(|| Span::new(start, start + name.len()))
                    })
                })
            {
                diagnostic = diagnostic.related(
                    span,
                    "Borrowed storage originates in the scope ending here.",
                );
            }
            return Err(diagnostic);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::native_view_loans::{LifetimeSource, Loan};
    use super::*;
    use crate::native_return_borrows::BorrowAccess;

    // The factory interface is injected while public borrowed signatures remain
    // guarded. The tested blocks use the real parser, calls, assignments and cleanup.
    fn check_body(body: &str) -> Result<bool, Diagnostic> {
        let source = format!(
            "type Window={{}};function borrow(owner:ReadOnly<string>):string{{return clone(owner);}}function main(){{{body}}}"
        );
        let formatted = crate::formatting::format(&source).unwrap();
        assert_eq!(crate::formatting::format(&formatted).unwrap(), formatted);
        let mut parsed = crate::syntax::parse(&formatted).unwrap();
        parsed.records[0].native_type = Some(crate::native_signatures::NativeTypeId::ByteView);
        let mut analyzer = Analyzer::new(parsed.clone()).unwrap();
        analyzer.function(0).unwrap();
        analyzer.signatures[0].result = Type::Record(0);
        let function = analyzer.checked[0].as_mut().unwrap();
        function.result = Type::Record(0);
        function.native_returns = Some(Box::new(vec![Loan {
            root: 0,
            owner_fields: vec![],
            source: LifetimeSource::Storage,
            access: BorrowAccess::ReadOnly,
            creation: Span::default(),
            retained: true,
            path: vec![],
            guards: vec![],
        }]));
        let mut context = Context::new("outer".into());
        for name in ["view", "safe"] {
            let token = Token {
                text: name.into(),
                kind: TokenKind::Name,
                span: Span::default(),
            };
            let id = Analyzer::bind(&mut context, &token, Type::Record(0), false).unwrap();
            context.bindings[id].rebindable = true;
        }
        let mut statements = parsed.functions.remove(1).body;
        analyzer.block(&mut context, &mut statements, Type::Void)
    }
    #[test]
    fn native_view_scope_exit_rejects_outer_holder_with_local_owner() {
        let error = check_body("{const owner='bytes';view=borrow(owner);}").unwrap_err();
        assert_eq!(error.code, "native-view-escape");
        assert!(error.message.contains("owner"));
        assert!(!error.related.is_empty());
    }
    #[test]
    fn native_view_scope_exit_allows_replacement_and_local_holders() {
        check_body("{const owner='bytes';view=borrow(owner);view=safe;}").unwrap();
        check_body("{const owner='bytes';const local=borrow(owner);}").unwrap();
    }
    #[test]
    fn native_view_scope_shadowing_keeps_outer_holder_alive() {
        let error =
            check_body("let owner='bytes';view=borrow(owner);{const view=1;owner='changed';}")
                .unwrap_err();
        assert_eq!(error.code, "native-view-conflict");
    }
    #[test]
    fn native_view_scope_dead_local_holder_releases_owner() {
        check_body("let owner='bytes';{const local=borrow(owner);}owner='changed';").unwrap();
    }
}
