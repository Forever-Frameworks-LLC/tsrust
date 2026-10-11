//! Resolve JSON namespace members before native signature specialization.
use crate::{Diagnostic, Span, syntax::*};

fn body_span(body: &[Stmt]) -> Span {
    Span::new(body.first().map_or(0, |stmt| stmt.span.start), crate::source_walk::body_end(body))
}

fn shadowed_scopes(body: &[Stmt], scopes: &mut Vec<Span>) {
    let mut pending = vec![body];
    while let Some(body) = pending.pop() {
        if body.iter().any(|statement| matches!(&statement.kind,
            StmtKind::Let { name, .. } if name.text == "JSON")) {
            scopes.push(body_span(body));
        }
        for statement in body {
            match &statement.kind {
                StmtKind::Block(body) | StmtKind::While { body, .. } => pending.push(body),
                StmtKind::If { then_body, else_body, .. } => pending.extend([then_body.as_slice(), else_body.as_slice()]),
                StmtKind::Loop { body, update, .. } => pending.extend([body.as_slice(), update.as_slice()]),
                StmtKind::ForOf { name, body, .. } => {
                    if name.text == "JSON" { scopes.push(body_span(body)); }
                    pending.push(body);
                }
                StmtKind::Try(region) => {
                    pending.push(&region.body);
                    if let Some(handler) = &region.catch {
                        if handler.name.as_ref().is_some_and(|name| name.text == "JSON") {
                            scopes.push(body_span(&handler.body));
                        }
                        pending.push(&handler.body);
                    }
                    if let Some(body) = &region.finally { pending.push(body); }
                }
                _ => {}
            }
        }
    }
}

pub(crate) fn prepare(program: &mut Program) -> Result<(), Diagnostic> {
    if program.functions.iter().any(|function| function.name.text == "JSON") { return Ok(()); }
    let mut required = std::collections::BTreeMap::new();
    for function in &mut program.functions {
        if function.params.iter().any(|(name, _, _)| name.text == "JSON") { continue; }
        let mut scopes = Vec::new();
        shadowed_scopes(&function.body, &mut scopes);
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Closure { params, body, .. } = &expression.kind {
                if params.iter().any(|parameter| parameter.name.text == "JSON") {
                    scopes.push(expression.span);
                }
                shadowed_scopes(body, &mut scopes);
            }
            Ok(())
        })?;
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            let ExprKind::Field(base, member) = &expression.kind else { return Ok(()); };
            if !matches!(&base.kind, ExprKind::Name(name) if name == "JSON")
                || scopes.iter().any(|scope| scope.start <= expression.span.start && expression.span.end <= scope.end) {
                return Ok(());
            }
            let binding = match member.as_str() {
                "tryParse" => "parse",
                "tryStringify" => "stringify",
                _ => return Ok(()),
            };
            required.insert(binding, expression.span);
            expression.kind = ExprKind::Name(format!("@native:std:json#{binding}"));
            Ok(())
        })?;
    }
    for (name, span) in required {
        let binding = crate::native_bindings::lookup("std:json", name).expect("registered JSON binding");
        crate::native_binding_types::prepare(&mut program.records, binding, span)?;
    }
    Ok(())
}
