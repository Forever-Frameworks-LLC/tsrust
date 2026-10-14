//! Lexical named declarations resolve before template, module and ownership passes.
use crate::syntax::*;
use crate::{Diagnostic, Span};
use std::collections::HashMap;

/// Separates the written name of a named function expression from the
/// source position that makes its block-level declaration name unique.
pub(crate) const EXPRESSION_MARK: &str = "#expression";

/// The name as written, without the expression marker.
pub(crate) fn written_name(name: &str) -> &str {
    name.split_once(EXPRESSION_MARK).map_or(name, |(written, _)| written)
}

#[derive(Clone, Debug)]
pub(crate) struct Declaration {
    pub id: usize,
    pub scope: usize,
    pub function_owner: usize,
    pub enclosing_owner: usize,
    pub statement_span: Span,
    pub source_name: Token,
    pub emitted_name: String,
    pub function_scope: Span,
}
#[derive(Clone)]
enum Symbol {
    Function { emitted: String, id: usize },
    Variable { owner: usize, span: Span },
}
type Environment = HashMap<String, Symbol>;
struct Resolver {
    next_owner: usize,
    next_scope: usize,
    declarations: Vec<Declaration>,
    lifted: Vec<Function>,
    lift_order: Vec<String>,
    capture_plan: crate::local_capture_plan::Planner,
    local_owners: HashMap<usize, usize>,
    /// A local function name occurs outside the callee position of a call.
    value_uses: bool,
}

pub(crate) fn resolve(program: &mut Program) -> Result<(), Diagnostic> {
    if program.unresolved_local_function.is_none() {
        return Ok(());
    }
    let mut resolver = Resolver {
        next_owner: program.functions.len(),
        next_scope: 0,
        declarations: Vec::new(),
        lifted: Vec::new(),
        lift_order: Vec::new(),
        capture_plan: Default::default(),
        local_owners: HashMap::new(),
        value_uses: false,
    };
    for (owner, function) in program.functions.iter_mut().enumerate() {
        resolver
            .capture_plan
            .owner(owner, function.name.text.clone());
        resolver.function(function, owner)?;
    }
    resolver.collect_lifted(&mut program.functions)?;
    let plan = resolver
        .capture_plan
        .has_captures()
        .then(|| resolver.capture_plan.finish());
    if resolver.value_uses {
        let ids: HashMap<&str, usize> = resolver
            .declarations
            .iter()
            .map(|declaration| (declaration.emitted_name.as_str(), declaration.id))
            .collect();
        crate::local_function_values::lower(
            &mut program.functions,
            &mut resolver.lifted,
            &|name| {
                plan.as_ref()
                    .is_some_and(|plan| !plan.requirements(ids[name]).is_empty())
            },
        )?;
    }
    program.local_capture_plans.extend(plan);
    for function in program.functions.iter_mut().chain(resolver.lifted.iter_mut()) {
        remove_lifted_placeholders(&mut function.body)?;
    }
    program.functions.extend(resolver.lifted);
    program.local_declarations = resolver.declarations;
    program.unresolved_local_function = None;
    Ok(())
}
/// A lifted declaration leaves an empty block where it stood. Remove those:
/// a declaration is hoisted, so it may follow a `return` and must not count
/// as an unreachable statement or as the value of the block.
fn remove_lifted_placeholders(body: &mut Vec<Stmt>) -> Result<(), Diagnostic> {
    fn statements(body: &mut Vec<Stmt>) {
        let mut work = vec![body];
        while let Some(body) = work.pop() {
            body.retain(|statement| !matches!(&statement.kind, StmtKind::Block(inner) if inner.is_empty()));
            for statement in body.iter_mut().rev() {
                match &mut statement.kind {
                    StmtKind::Block(body) | StmtKind::While { body, .. } | StmtKind::ForOf { body, .. } => work.push(body),
                    StmtKind::If { then_body, else_body, .. } => { work.push(else_body); work.push(then_body); },
                    StmtKind::Loop { body, update, .. } => { work.push(update); work.push(body); },
                    StmtKind::Try(region) => {
                        if let Some(finally) = &mut region.finally { work.push(finally); }
                        if let Some(catch) = &mut region.catch { work.push(&mut catch.body); }
                        work.push(&mut region.body);
                    },
                    _ => {},
                }
            }
        }
    }
    statements(body);
    crate::source_walk::body(body, &mut |_| Ok(()), &mut |expression| {
        if let ExprKind::Closure { body, .. } | ExprKind::Match { body, .. } = &mut expression.kind {
            statements(body);
        }
        Ok(())
    })
}

#[path = "local_functions_work.rs"]
mod work;

#[cfg(test)]
mod tests {
    #[test]
    fn lexical_declaration_identity_survives_checked_projection() {
        let source = "function main(){function value():int{return 1;} {function value():int{return 2;} console.log(value());} console.log(value());}";
        let checked = crate::check(source).unwrap();
        let declarations = &checked.program.local_declarations;
        assert_eq!(declarations.len(), 2);
        assert_ne!(declarations[0].id, declarations[1].id);
        assert_ne!(declarations[0].scope, declarations[1].scope);
        assert_ne!(declarations[0].emitted_name, declarations[1].emitted_name);
        for declaration in declarations {
            assert_eq!(declaration.source_name.text, "value");
            assert_eq!(
                &source[declaration.source_name.span.start..declaration.source_name.span.end],
                "value"
            );
            assert!(
                source[declaration.function_scope.start..declaration.function_scope.end]
                    .starts_with("value():int")
            );
        }
    }
    #[test]
    fn transitive_capture_diagnostic_points_to_the_owner_not_a_sibling_function() {
        let source = "function main(){function forward():int{return read();}const data=[1];function read():int{return data[0];}console.log(forward());}";
        // The capture is planned for its owner; the program checks now that
        // captures lower to hidden arguments.
        let parsed = crate::syntax::parse(source).unwrap();
        let plan = &parsed.local_capture_plans[0];
        assert!(plan.requirements(0).iter().all(|capture| capture.name == "data"));
        assert!(plan.requirements(1).iter().all(|capture| capture.name == "data"));
        assert!(crate::check(source).is_ok());
        // The diagnostic for uses that are still pending names the captured
        // owner binding, not the sibling function.
        let error = plan.pending_diagnostic().expect("captured owner");
        assert_eq!(error.code, "unsupported");
        assert_eq!(&source[error.span.start..error.span.end], "data");
        let binding = error.related[0].0;
        assert_eq!(&source[binding.start..binding.end], "data");
    }
    #[test]
    fn captured_declaration_syntax_reaches_the_checked_context_boundary() {
        let source = "function main(){function read():int{return data[0];}const data=[1];console.log(read());}";
        let parsed = crate::syntax::parse(source).expect("capture syntax must survive parsing");
        assert_eq!(parsed.local_declarations.len(), 1);
        assert_eq!(parsed.local_capture_plans.len(), 1);
        let plan = &parsed.local_capture_plans[0];
        assert_eq!(plan.requirements(0)[0].name, "data");
        let declaration = &parsed.local_declarations[0];
        assert_eq!(declaration.source_name.text, "read");
        assert_eq!(declaration.enclosing_owner, 0);
        assert_ne!(declaration.function_owner, declaration.enclosing_owner);
        assert!(
            source[declaration.statement_span.start..declaration.statement_span.end]
                .starts_with("function read")
        );
        assert!(
            parsed
                .functions
                .iter()
                .any(|function| function.name.text == declaration.emitted_name)
        );
        // The retained plan reaches checking, which lowers the capture.
        assert!(crate::analysis::analyze(parsed).is_ok());
    }
}
