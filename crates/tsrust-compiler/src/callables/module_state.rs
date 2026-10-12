//! Program storage is an ordinary owned lexical scope, never a mutable static.
use crate::syntax::*;
use crate::{Diagnostic, Span};

/// Form the program owner before resolving named captures. All module functions
/// share this lexical owner; the capture solver computes their transitive access.
/// Linking must already have canonicalized module names and initialization order.
pub(crate) fn lower(program: &mut Program) -> Result<(), Diagnostic> {
    if program.module_bindings.is_empty() { return Ok(()); }
    if !program.local_declarations.is_empty() {
        return Err(Diagnostic::new("module-state", "Module ownership must be resolved before local captures.", program.module_bindings[0].span));
    }
    let entry = program.functions.iter().find(|function| function.name.text == "main").cloned();
    let span = program.module_bindings.iter().map(|statement| statement.span)
        .chain(program.functions.iter().map(|function| function.scope_span))
        .fold(Span::default(), |all, next| Span::new(all.start.min(next.start), all.end.max(next.end)));
    program.module_owner_bindings = program.module_bindings.iter().filter_map(|statement| match &statement.kind {
        StmtKind::Let { name, .. } => Some(name.span),
        _ => None,
    }).collect();
    let mut body = std::mem::take(&mut program.module_bindings);
    let mut data_owners = Vec::new();
    for function in std::mem::take(&mut program.functions) {
        // Validated data imports already have canonical, readonly ownership.
        // Their dedicated accessor encoding must keep its linked identity.
        if function.name.text.ends_with(crate::data_lowering::OWNER) {
            data_owners.push(function);
            continue;
        }
        body.push(Stmt { labels: Vec::new(), control_target: None,
            span: function.name.span, kind: StmtKind::LocalFunction(Box::new(function)) });
    }
    let mut owner = if let Some(mut entry) = entry {
        let arguments = entry.params.iter().map(|(name, _, _)| Expr::new(ExprKind::Name(name.text.clone()), name.span)).collect();
        body.push(Stmt { labels: Vec::new(), control_target: None, span: entry.name.span,
            kind: StmtKind::Return(Some(Expr::new(ExprKind::Call(
                Box::new(Expr::new(ExprKind::Name("main".into()), entry.name.span)), arguments), entry.name.span))) });
        entry.body = body;
        entry
    } else {
        Function { application: None, scope_span: span,
            name: Token { text: "main".into(), kind: TokenKind::Name, span },
            type_parameters: Vec::new(), params: Vec::new(), parameter_initializers: Vec::new(),
            parameter_access: Vec::new(), rest_parameter: false, result: None, body }
    };
    owner.scope_span = span;
    program.unresolved_local_function = Some(owner.name.clone());
    program.functions.push(owner);
    program.functions.extend(data_owners);
    Ok(())
}

/// A module binding remains available for the entire program. A named function
/// cannot consume its owner as though it were a one-shot lexical environment.
pub(crate) fn validate(program: &crate::analysis::CheckedProgram) -> Result<(), Diagnostic> {
    if let Some(owner) = program.functions.iter().find(|function| function.name == "main") {
        for binding in &owner.bindings {
            if binding.declaration_span.is_some_and(|span| program.module_owner_bindings.contains(&span))
                && (binding.transferred.is_some() || !binding.moved_fields.is_empty()) {
                return Err(Diagnostic::new("module-state-consume",
                    "Cannot leave program-owned module storage consumed; use explicit clone() for an owned copy.",
                    binding.transferred.or(binding.declaration_span).unwrap_or_default()));
            }
        }
    }
    for accesses in program.named_capture_access.values() {
        for access in accesses {
            if access.binding.owner != 0 || access.mode != crate::Mode::Consume { continue; }
            if let Some(span) = program.module_owner_bindings.iter().find(|span| span.start == access.binding.declaration_start) {
                return Err(Diagnostic::new("module-state-consume",
                    "Cannot consume program-owned module storage; borrow it or use explicit clone() for an owned copy.", *span));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_keeps_initializers_in_order_and_resolves_function_captures() {
        let program = crate::syntax::parse("const first = 1; let second = first; function read(): int { return second; } function main() { console.log(read()); }").unwrap();
        assert!(program.module_bindings.is_empty());
        assert_eq!(program.module_owner_bindings.len(), 2);
        assert_eq!(program.functions[0].name.text, "main");
        assert!(matches!(&program.functions[0].body[0].kind, StmtKind::Let { name, .. } if name.text == "first"));
        assert!(matches!(&program.functions[0].body[1].kind, StmtKind::Let { name, .. } if name.text == "second"));
        assert!(program.local_capture_plans.iter().flat_map(|plan| &plan.direct).flatten().any(|capture| capture.name == "second"));
    }

    #[test]
    fn initialization_does_not_require_an_explicit_main() {
        let program = crate::syntax::parse("const value = 1;").unwrap();
        assert_eq!(program.functions.len(), 1);
        assert_eq!(program.functions[0].name.text, "main");
        assert_eq!(program.functions[0].body.len(), 1);
    }
}
