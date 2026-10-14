//! Private program inputs for test registration and execution.
use crate::syntax::*;
use crate::{Diagnostic, Span};

pub(crate) const REGISTRY: &str = "@program-test-registry";

pub(crate) fn operation(name: &str) -> Option<&str> {
    let name = name.strip_prefix("@native:std:testing-runtime#")?;
    matches!(name, "invokeRegistration" | "registerTest" | "registerHook" | "runTests" | "finishTests" | "beginSuite" | "endSuite" | "beginFile").then_some(name)
}

pub(crate) fn executes(name: &str) -> bool { matches!(name, "invokeRegistration" | "runTests" | "finishTests") }

/// Only the trusted standard-library source receives this internal contract.
pub(crate) fn facade(program: &mut Program) {
    for function in &mut program.functions {
        if !matches!(function.name.text.as_str(), "test" | "it" | "beforeAll" | "afterAll" | "beforeEach" | "afterEach" | "before" | "after" | "describe" | "suite") { continue; }
        let immediate = matches!(function.name.text.as_str(), "describe" | "suite");
        for (_, annotation, callback) in &mut function.params {
            if let Some(callback) = callback {
                callback.kind = if immediate { CallbackKind::ProgramReusable } else { CallbackKind::Program };
                if let crate::type_syntax::ValueAnnotationKind::Callable { ownership, .. } = &mut annotation.kind {
                    *ownership = if immediate { crate::type_syntax::CallableOwnership::ProgramReusable } else { crate::type_syntax::CallableOwnership::Program };
                }
            }
        }
        if immediate {
            crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
                if let ExprKind::Call(callee, arguments) = &mut expression.kind
                    && matches!(&callee.kind, ExprKind::Name(name) if name == "callback") {
                    arguments.push(Expr::new(ExprKind::Name("callback".into()), callee.span));
                    callee.kind = ExprKind::Name("@native:std:testing-runtime#invokeRegistration".into());
                }
                Ok(())
            }).expect("private registration callback lowering");
        }
    }
}

pub(crate) fn registry_reachable(program: &mut Program) -> Result<bool, Diagnostic> {
    use std::collections::{BTreeMap, BTreeSet};
    let mut edges = BTreeMap::new();
    for function in &mut program.functions {
        let mut names = Vec::new();
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Name(name) = &expression.kind { names.push(name.clone()); }
            Ok(())
        })?;
        edges.insert(function.name.text.clone(), names);
    }
    let mut pending = vec!["main".to_owned()];
    crate::source_walk::body(&mut program.module_bindings, &mut |_| Ok(()), &mut |expression| {
        if let ExprKind::Name(name) = &expression.kind { pending.push(name.clone()); }
        Ok(())
    })?;
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if operation(&name).is_some() { return Ok(true); }
        if seen.insert(name.clone()) && let Some(children) = edges.get(&name) { pending.extend(children.iter().cloned()); }
    }
    Ok(false)
}

pub(crate) fn lower(program: &mut Program) -> Result<(), Diagnostic> {

    if program.module_bindings.iter().any(|statement| matches!(&statement.kind, StmtKind::Let { name, .. } if name.text == REGISTRY)) { return Ok(()); }
    let globals = program.module_bindings.iter().filter_map(|statement| match &statement.kind {
        StmtKind::Let { name, .. } => Some(name.clone()), _ => None,
    }).collect::<Vec<_>>();
    let mut used = false;
    let mut visit = |expression: &mut Expr| {
        let ExprKind::Call(callee, arguments) = &mut expression.kind else { return Ok(()); };
        let ExprKind::Name(name) = &callee.kind else { return Ok(()); };
        let Some(operation) = operation(name) else { return Ok(()); };
        used = true;
        arguments.push(Expr::new(ExprKind::Name(REGISTRY.into()), expression.span));
        if executes(operation) {
            arguments.extend(globals.iter().map(|name| Expr::new(ExprKind::Name(name.text.clone()), expression.span)));
        }
        Ok(())
    };
    for function in &mut program.functions {
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut visit)?;
    }
    // Initializers keep their original order. A registry operation in an
    // initializer is diagnosed by the ordinary lexical readiness checks.
    crate::source_walk::body(&mut program.module_bindings, &mut |_| Ok(()), &mut visit)?;
    if !used { return Ok(()); }
    let end = program.functions.iter().map(|function| function.scope_span.end)
        .chain(program.module_bindings.iter().map(|statement| statement.span.end)).max().unwrap_or(0);
    let span = Span::new(end + 1, end + 1);
    crate::native_binding_types::prepare(&mut program.records, &super::bindings::CREATE_REGISTRY, span)?;
    program.module_bindings.push(Stmt { labels: Vec::new(), control_target: None, span,
        kind: StmtKind::Let { rebindable: false, name: Token { text: REGISTRY.into(), kind: TokenKind::Symbol, span }, annotation: None,
            value: Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@native:std:testing-runtime#createRegistry".into()), span)), Vec::new()), span), binding: None }
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn registry_is_declared_after_user_storage_without_reordering_initializers() {
        use crate::syntax::*;
        let mut program = parse_module("const first='first';let second=1;function main(){}", false, Default::default(), Default::default()).unwrap();
        program.functions[0].body.push(Stmt { labels: Vec::new(), control_target: None, span: crate::Span::default(), kind: StmtKind::Expr(Expr::new(ExprKind::Call(Box::new(Expr::new(ExprKind::Name("@native:std:testing-runtime#runTests".into()), crate::Span::default())), Vec::new()), crate::Span::default())) });
        let declarations=program.module_bindings.iter().map(|s|s.span).collect::<Vec<_>>();
        super::lower(&mut program).unwrap();
        let names=program.module_bindings.iter().map(|s|match &s.kind {StmtKind::Let{name,..}=>name.text.as_str(),_=>panic!("binding")}).collect::<Vec<_>>();
        assert_eq!(names,["first","second",super::REGISTRY]);
        assert_eq!(declarations,program.module_bindings[..2].iter().map(|s|s.span).collect::<Vec<_>>());
    }
}

/// Do not introduce a program owner for assertion-only imports. Only unused
/// trusted facades are omitted; user declarations keep their ordinary checks.
pub(crate) fn omit_unused_facades(program: &mut Program, names: &std::collections::BTreeMap<String,String>) -> Result<(), Diagnostic> {
    let facades = ["test","it","describe","suite","beforeAll","afterAll","beforeEach","afterEach","before","after","runTests"]
        .into_iter().filter_map(|name|names.get(name).cloned()).collect::<std::collections::BTreeSet<_>>();
    let mut referenced = false;
    let mut visit = |expression: &mut Expr| {
        if let ExprKind::Name(name) = &expression.kind { referenced |= facades.contains(name); }
        Ok(())
    };
    for function in &mut program.functions {
        if !facades.contains(&function.name.text) { crate::source_walk::function_body(function, &mut |_|Ok(()), &mut visit)?; }
    }
    crate::source_walk::body(&mut program.module_bindings, &mut |_|Ok(()), &mut visit)?;
    if !referenced { program.functions.retain(|function|!facades.contains(&function.name.text)); }
    Ok(())
}
