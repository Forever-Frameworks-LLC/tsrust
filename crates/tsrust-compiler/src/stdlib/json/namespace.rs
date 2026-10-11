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
    let mut throwing = std::collections::BTreeSet::new();
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
                "parse" => { throwing.insert("parse"); "parse" },
                "stringify" => { throwing.insert("stringify"); "stringify" },
                _ => return Ok(()),
            };
            required.insert(binding, expression.span);
            expression.kind = ExprKind::Name(if member.starts_with("try") {
                format!("@native:std:json#{binding}")
            } else { format!("@json:{binding}") });
            Ok(())
        })?;
    }
    for (name, span) in required {
        let binding = crate::native_bindings::lookup("std:json", name).expect("registered JSON binding");
        crate::native_binding_types::prepare(&mut program.records, binding, span)?;
    }
    for name in throwing {
        if !program.functions.iter().any(|function| function.name.text == format!("@json:{name}")) {
            let wrapper = throwing_adapter(program, name)?;
            program.functions.push(wrapper);
        }
    }
    Ok(())
}

/// Ordinary checked functions preserve typed failure propagation and ownership.
/// Only compiler-owned source is parsed here; no user text is interpolated.
fn throwing_adapter(program: &mut Program, name: &str) -> Result<Function, Diagnostic> {
    use crate::native_signatures::{NativeTypeId, Signature};
    let source = if name == "parse" {
        "function adapter(text:string,limits:JsonLimits={maxBytes:16777216,maxDepth:64,maxEntries:1000000,maxNodes:1000000}):JsonValue {match(backend(text,limits)){Ok(value)=>{return value;},Err(error)=>{throw error;}}}"
    } else {
        "function adapter(value:JsonValue,maxBytes:int=16777216):string {match(backend(value,maxBytes)){Ok(text)=>{return text;},Err(error)=>{throw error;}}}"
    };
    let mut function = crate::syntax::parse(source)?.functions.remove(0);
    let offset = program.functions.iter().map(|function| function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0) + 1;
    let shift = |span: &mut Span| { span.start += offset; span.end += offset; };
    function.name.text = format!("@json:{name}");
    shift(&mut function.name.span);
    shift(&mut function.scope_span);
    for (parameter, _, _) in &mut function.params { shift(&mut parameter.span); }
    for initializer in &mut function.parameter_initializers {
        if let ParameterInitializer::Default { span, .. } = initializer { shift(span); }
    }
    let mut bodies = vec![&mut function.body];
    while let Some(body) = bodies.pop() {
        for statement in body {
            shift(&mut statement.span);
            if let StmtKind::Let { name, .. } = &mut statement.kind { shift(&mut name.span); }
            match &mut statement.kind {
                StmtKind::Block(body) => bodies.push(body),
                StmtKind::If { then_body, else_body, .. } => { bodies.push(then_body); bodies.push(else_body); }
                _ => {}
            }
        }
    }
    crate::source_walk::function_body(&mut function, &mut |_| Ok(()), &mut |expression| {
        shift(&mut expression.span);
        if matches!(&expression.kind, ExprKind::Name(value) if value == "backend") {
            expression.kind = ExprKind::Name(format!("@native:std:json#{name}"));
        }
        Ok(())
    })?;
    let span = function.name.span;
    if name == "parse" {
        function.params[1].1 = crate::native_type_resolution::project(&mut program.records, &Signature::Named(NativeTypeId::JsonLimits), span)?;
        function.result = Some(ResultAnnotation::Named(Box::new(crate::native_type_resolution::project(&mut program.records, &Signature::Named(NativeTypeId::JsonValue), span)?)));
    } else {
        function.params[0].1 = crate::native_type_resolution::project(&mut program.records, &Signature::Named(NativeTypeId::JsonValue), span)?;
    }
    Ok(function)
}
