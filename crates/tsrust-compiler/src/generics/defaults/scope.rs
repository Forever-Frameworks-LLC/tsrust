//! Resolve names in generic defaults even when no specialization is requested.
//! This checks initialization scope, not unrelated generic function bodies.
use crate::Diagnostic;
use crate::syntax::{Expr, ExprKind, ParameterInitializer, Program, Stmt, StmtKind};
use std::collections::HashSet;

fn expression<'a>(value: &'a Expr, names: &HashSet<&'a str>) -> Result<(), Diagnostic> {
    match &value.kind {
        ExprKind::TypeAssertion {value,..} => expression(value,names)?,
        ExprKind::SpreadCall(plan) => { for value in plan.expressions() { expression(value, names)?; } }
        ExprKind::SpreadValue(_) => (),
        ExprKind::Name(name) => {
            if name != "Object#in" && !names.contains(name.as_str())
                && crate::native_bindings::by_symbol(name).is_none()
                && crate::syntax::global_number_member(name, false).is_none()
                && crate::syntax::global_number_member(name, true).is_none()
            {
                return Err(Diagnostic::new(
                    "name",
                    format!("Unknown value {name:?}."),
                    value.span,
                ));
            }
        }
        ExprKind::Match { body, .. } => statements(body, &mut names.clone())?,
                    ExprKind::Closure { params, body, .. } => {
            let mut local = names.clone();
            // All parameter names reserve their scope before any initializer.
            for parameter in params {
                local.remove(parameter.name.text.as_str());
            }
            for parameter in params {
                if let ParameterInitializer::Default { value, .. } = &parameter.initializer {
                    expression(value, &local)?;
                }
                local.insert(parameter.name.text.as_str());
            }
            statements(body, &mut local)?;
        }
        ExprKind::Conditional {
            predicate,
            consequent,
            alternate,
        } => {
            expression(predicate, names)?;
            expression(consequent, names)?;
            expression(alternate, names)?;
        }
        ExprKind::Optional { base, body, .. } => {
            expression(base, names)?;
            expression(body, names)?;
        }
        ExprKind::Binary { left, right, .. } | ExprKind::Index(left, right) => {
            expression(left, names)?;
            expression(right, names)?;
        }
        ExprKind::CompoundAssign { target, value, .. }
        | ExprKind::Assign { target, value, .. }
        | ExprKind::CoalesceAssign { target, value, .. }
        | ExprKind::LogicalAssign { target, value, .. } => {
            expression(target, names)?;
            expression(value, names)?;
        }
        ExprKind::OptionalChain(value)
        | ExprKind::Unary { value, .. }
        | ExprKind::Update { target: value, .. }
        | ExprKind::Field(value, _)
        | ExprKind::Inject { value, .. }
        | ExprKind::Project { value, .. } => expression(value, names)?,
        ExprKind::Record(fields) => {
            for (_, value) in fields {
                expression(value, names)?;
            }
        }
        ExprKind::ArrayOf(values, _) | ExprKind::SetConstructor(values, _) | ExprKind::MapConstructor(values, _) | ExprKind::NativeConstructor(values, _) | ExprKind::Array(values) | ExprKind::Template(values) | ExprKind::Sequence(values) => {
            for value in values {
                expression(value, names)?;
            }
        }
        ExprKind::Call(callee, arguments) => {
            expression(callee, names)?;
            for argument in arguments {
                expression(argument, names)?;
            }
        }
        ExprKind::Selected { .. }
        | ExprKind::String(_)
        | ExprKind::Integer(_)
        | ExprKind::EnumMember { .. } | ExprKind::Number(_)
        | ExprKind::Boolean(_)
        | ExprKind::Null
        | ExprKind::Undefined => {}
    }
    Ok(())
}

fn statements<'a>(body: &'a [Stmt], names: &mut HashSet<&'a str>) -> Result<(), Diagnostic> {
    for statement in body {
        match &statement.kind {
            StmtKind::LocalFunction(_) => unreachable!("local declarations must be resolved before checking"),
            StmtKind::Throw(value) => expression(value,names)?,
            StmtKind::Try(region) => {
                statements(&region.body,&mut names.clone())?;
                if let Some(handler)=&region.catch {
                    let mut local=names.clone();
                    if let Some(name)=&handler.name {local.insert(name.text.as_str());}
                    statements(&handler.body,&mut local)?;
                }
                if let Some(body)=&region.finally {statements(body,&mut names.clone())?;}
            }

            StmtKind::Let { name, value, .. } => {
                expression(value, names)?;
                names.insert(name.text.as_str());
            }
            StmtKind::Block(body) => statements(body, &mut names.clone())?,
            StmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                expression(condition, names)?;
                statements(then_body, &mut names.clone())?;
                statements(else_body, &mut names.clone())?;
            }
            StmtKind::While { condition, body } => {
                expression(condition, names)?;
                statements(body, &mut names.clone())?;
            }
            StmtKind::Loop {
                condition,
                body,
                update,
                ..
            } => {
                if let Some(condition) = condition {
                    expression(condition, names)?;
                }
                statements(body, &mut names.clone())?;
                statements(update, &mut names.clone())?;
            }
            StmtKind::ForOf {
                name,
                iterable,
                body,
                ..
            } => {
                expression(iterable, names)?;
                let mut local = names.clone();
                local.insert(name.text.as_str());
                statements(body, &mut local)?;
            }
            StmtKind::Assign { target, value, .. } => {
                expression(target, names)?;
                expression(value, names)?;
            }
            StmtKind::Return(Some(value)) | StmtKind::Expr(value) => expression(value, names)?,
            StmtKind::Return(None) | StmtKind::Break | StmtKind::Continue => {}
        }
    }
    Ok(())
}

pub(super) fn validate(program: &Program) -> Result<(), Diagnostic> {
    let mut globals = program
        .functions
        .iter()
        .map(|function| function.name.text.as_str())
        .collect::<HashSet<_>>();
    globals.extend([
        "clone", "unwrap",
        "String",
        "Boolean",
        "Array",
        "Set",
        "Map",
        "console",
        "Number",
        "Math",
        "undefined",
    ]);
    for function in &program.functions {
        if function.type_parameters.is_empty() {
            continue;
        }
        let mut names = globals.clone();
        for (name, _, _) in &function.params {
            names.remove(name.text.as_str());
        }
        for ((name, _, _), initializer) in
            function.params.iter().zip(&function.parameter_initializers)
        {
            if let ParameterInitializer::Default { value, .. } = initializer {
                expression(value, &names)?;
            }
            names.insert(name.text.as_str());
        }
    }
    Ok(())
}
