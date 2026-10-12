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
        function.native_returns = Some(Box::new(vec![Loan { whole: false,
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
        if body.contains("try") {
            return analyzer.check_exception_callable_body(
                &mut context,
                &mut statements,
                Type::Void,
            );
        }
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
    #[test]
    fn native_view_scope_control_exits_reject_dangling_holders() {
        for body in [
            "while(true){const owner='bytes';view=borrow(owner);break;}",
            "let count=0;while(count<1){const owner='bytes';view=borrow(owner);count=count+1;continue;}",
            "try{const owner='bytes';view=borrow(owner);throw 1;}catch(error){}",
        ] {
            let error = check_body(body).unwrap_err();
            assert_eq!(error.code, "native-view-escape", "{body}: {error:?}");
        }
    }
    #[test]
    fn native_view_scope_control_exits_release_dead_holders() {
        for body in [
            "let owner='bytes';while(true){const local=borrow(owner);break;}owner='changed';",
            "let owner='bytes';let count=0;while(count<1){const local=borrow(owner);count=count+1;continue;}owner='changed';",
            "let owner='bytes';try{const local=borrow(owner);throw 1;}catch(error){}owner='changed';",
        ] {
            check_body(body).unwrap_or_else(|error| panic!("{body}: {error:?}"));
        }
    }
    #[test]
    fn native_view_scope_finally_replacement_releases_owner() {
        check_body("let owner='bytes';try{view=borrow(owner);throw 1;}catch(error){}finally{view=safe;}owner='changed';").unwrap();
        let error=check_body("let owner='bytes';try{view=borrow(owner);throw 1;}catch(error){}owner='changed';view=safe;").unwrap_err();
        assert_eq!(error.code, "native-view-conflict");
    }
}
