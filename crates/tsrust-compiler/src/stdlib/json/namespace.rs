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
    let namespace_shadowed=program.functions.iter().any(|function| function.name.text == "JSON");
    let mut required = std::collections::BTreeMap::new();
    let mut throwing = std::collections::BTreeSet::new();
    let mut typed = std::collections::BTreeSet::new();
    let explicit: std::collections::BTreeSet<_> = program.call_type_arguments.iter().map(|(span, _)| (span.start, span.end)).collect();
    // Named functions have already been lifted out of their lexical bodies.
    // Their variable-use identities retain captured receiver provenance.
    let local_uses: std::collections::BTreeSet<_> = program.local_capture_plans.iter()
        .flat_map(|plan| plan.uses.keys().copied()).collect();
    for function in &mut program.functions {
        let namespace_shadowed=namespace_shadowed || function.params.iter().any(|(name, _, _)| name.text == "JSON");
        let mut scopes = Vec::new();
        let mut typed_members = std::collections::BTreeSet::new();
        shadowed_scopes(&function.body, &mut scopes);
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if let ExprKind::Call(callee, _) = &expression.kind {
                if explicit.contains(&(expression.span.start, expression.span.end)) { typed_members.insert((callee.span.start, callee.span.end)); }
            }
            if let ExprKind::Closure { params, body, .. } = &expression.kind {
                if params.iter().any(|parameter| parameter.name.text == "JSON") {
                    scopes.push(expression.span);
                }
                shadowed_scopes(body, &mut scopes);
            }
            Ok(())
        })?;
        crate::source_walk::function_body(function, &mut |_| Ok(()), &mut |expression| {
            if matches!(&expression.kind,ExprKind::Name(name) if name=="@native:std:json#writeJsonl") {
                required.insert("writeJsonl",expression.span);throwing.insert("writeJsonl");
                expression.kind=ExprKind::Name("@json:writeJsonl".into());return Ok(());
            }
            if typed_members.contains(&(expression.span.start,expression.span.end)) {
                let imported=match &expression.kind {
                    ExprKind::Name(name) if name=="@native:std:json#parseJsonc"=>Some("parseJsonc"),
                    ExprKind::Name(name) if name=="@native:std:json#readJsonl"=>Some("readJsonl"),
                    _=>None,
                };
                if let Some(name)=imported {
                    typed.insert(name.into());required.insert(name,expression.span);
                    expression.kind=ExprKind::Name(format!("@json:typed{name}"));return Ok(());
                }
            }
            if namespace_shadowed {return Ok(());}
            let ExprKind::Field(base, member) = &expression.kind else { return Ok(()); };
            if !matches!(&base.kind, ExprKind::Name(name) if name == "JSON")
                || local_uses.contains(&base.span.start)
                || scopes.iter().any(|scope| scope.start <= expression.span.start && expression.span.end <= scope.end) {
                return Ok(());
            }
            if matches!(member.as_str(), "parse" | "tryParse") && typed_members.contains(&(expression.span.start, expression.span.end)) {
                typed.insert(member.clone());
                required.insert("parse", expression.span);
                expression.kind = ExprKind::Name(format!("@json:typed{member}"));
                return Ok(());
            }
            let binding = match member.as_str() {
                "tryParse" => "parse",
                "tryStringify" => { throwing.insert("tryStringify"); "stringify" },
                "parse" => { throwing.insert("parse"); "parse" },
                "stringify" => { throwing.insert("stringify"); "stringify" },
                _ => return Ok(()),
            };
            required.insert(binding, expression.span);
            expression.kind = ExprKind::Name(if member == "tryStringify" { "@json:tryStringify".into() } else if member.starts_with("try") {
                format!("@native:std:json#{binding}")
            } else { format!("@json:{binding}") });
            Ok(())
        })?;
    }
    for (name, span) in required {
        let binding = crate::native_bindings::lookup("std:json", name).expect("registered JSON binding");
        crate::native_binding_types::prepare(&mut program.records, binding, span)?;
    }
    for name in typed {
        if name=="readJsonl" {let wrapper=typed_adapter(program,"readJsonlView")?;program.functions.push(wrapper);}
        let wrapper = typed_adapter(program, &name)?;
        program.functions.push(wrapper);
    }
    for name in throwing {
        if name=="writeJsonl" {let wrapper=throwing_adapter(program,"writeJsonlView")?;program.functions.push(wrapper);}
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
    let source = if name=="writeJsonlView" {
        "function adapter<T>(writer:Mutable<JsonlWriterView>,value:T):Result<void,JsonlError>{return backend(writer,value);}"
    }else if name=="writeJsonl" {
        "function adapter<T>(writer:Mutable<JsonlWriter>,value:T):Result<void,JsonlError>{return backend(writer,value);}"
    }else if name == "parse" {
        "function adapter(text:string,limits:JsonLimits={maxBytes:16777216,maxDepth:64,maxEntries:1000000,maxNodes:1000000}):JsonValue {match(backend(text,limits)){Ok(value)=>{return value;},Err(error)=>{throw error;}}}"
    } else if name == "tryStringify" {
        "function adapter<T>(value:T,maxBytes:int=16777216):Result<string,JsonError> {return backend(value,maxBytes);}"
    } else {
        "function adapter<T>(value:T,maxBytes:int=16777216):string {const encoded:Result<string,JsonError>=backend(value,maxBytes);match(encoded){Ok(text)=>{return text;},Err(error)=>{throw error;}}}"
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
    let identities=if name=="writeJsonlView" {vec![NativeTypeId::JsonlWriterView,NativeTypeId::JsonlError]}else if name=="writeJsonl" {vec![NativeTypeId::JsonlWriter,NativeTypeId::JsonlError]}else{vec![NativeTypeId::JsonError]};
    let mut replacements=std::collections::HashMap::new();
    for identity in identities {replacements.insert(crate::native_types::declaration(identity).name.to_owned(),crate::native_type_resolution::project(&mut program.records,&Signature::Named(identity),function.name.span)?);}
    crate::source_walk::function_annotations(&mut function,&mut |annotation|crate::type_substitution::apply(annotation,&replacements))?;
    crate::source_walk::function_body(&mut function, &mut |annotation|crate::type_substitution::apply(annotation,&replacements), &mut |expression| {
        shift(&mut expression.span);
        if matches!(&expression.kind, ExprKind::Name(value) if value == "backend") {
            expression.kind = ExprKind::Name(if name == "parse" {format!("@native:std:json#{name}")} else if matches!(name,"writeJsonl"|"writeJsonlView"){"@json:write".into()}else {"@json:encode".into()});
        }
        Ok(())
    })?;
    let span = function.name.span;
    if name == "parse" {
        function.params[1].1 = crate::native_type_resolution::project(&mut program.records, &Signature::Named(NativeTypeId::JsonLimits), span)?;
        function.result = Some(ResultAnnotation::Named(Box::new(crate::native_type_resolution::project(&mut program.records, &Signature::Named(NativeTypeId::JsonValue), span)?)));
    }
    Ok(function)
}

/// Explicit generic calls specialize ordinary checked adapters before analysis.
fn typed_adapter(program: &mut Program, name: &str) -> Result<Function, Diagnostic> {
    use crate::native_signatures::{NativeTypeId, Signature};
    let result = if name == "parse" { "T" } else { "Result<T,JsonError>" };
    let body = if name == "parse" {
        "const decoded:Result<T,JsonError>=backend(text,limits);match(decoded){Ok(value)=>{return value;},Err(error)=>{throw error;}}"
    } else { "return backend(text,limits);" };
    let source = format!("function adapter<T>(text:ReadOnly<string>,limits:ReadOnly<JsonLimits>={{maxBytes:16777216,maxDepth:64,maxEntries:1000000,maxNodes:1000000}}):{result}{{{body}}}");
    let source=if name=="parseJsonc" {
        "function adapter<T>(text:ReadOnly<string>,allowTrailingComma:boolean=false,limits:ReadOnly<JsonLimits>={maxBytes:16777216,maxDepth:64,maxEntries:1000000,maxNodes:1000000}):Result<T,JsonError>{return backend(text,allowTrailingComma,limits);}".to_owned()
    }else if name=="readJsonlView" {
        "function adapter<T>(reader:Mutable<JsonlReaderView>):Result<T|undefined,JsonlError>{return backend(reader);}".to_owned()
    }else if name=="readJsonl" {
        "function adapter<T>(reader:Mutable<JsonlReader>):Result<T|undefined,JsonlError>{return backend(reader);}".to_owned()
    }else{source};
    let mut function = crate::syntax::parse(&source)?.functions.remove(0);
    let offset = program.functions.iter().map(|function| function.scope_span.end.max(crate::source_walk::body_end(&function.body))).max().unwrap_or(0) + 1;
    let shift = |span: &mut Span| { span.start += offset; span.end += offset; };
    function.name.text = format!("@json:typed{name}");
    shift(&mut function.name.span);shift(&mut function.scope_span);
    for (parameter, _, _) in &mut function.params { shift(&mut parameter.span); }
    for initializer in &mut function.parameter_initializers { if let ParameterInitializer::Default { span, .. } = initializer {shift(span);} }
    let mut replacements=std::collections::HashMap::new();
    let identities=if name=="readJsonlView" {vec![NativeTypeId::JsonlReaderView,NativeTypeId::JsonlError,NativeTypeId::JsonLimits]}else if name=="readJsonl" {vec![NativeTypeId::JsonlReader,NativeTypeId::JsonlError,NativeTypeId::JsonLimits]}else{vec![NativeTypeId::JsonLimits,NativeTypeId::JsonError]};
    for identity in identities {
        replacements.insert(crate::native_types::declaration(identity).name.to_owned(),crate::native_type_resolution::project(&mut program.records,&Signature::Named(identity),function.name.span)?);
    }
    crate::source_walk::function_annotations(&mut function, &mut |annotation| crate::type_substitution::apply(annotation, &replacements))?;
    crate::source_walk::function_body(&mut function, &mut |annotation| crate::type_substitution::apply(annotation, &replacements), &mut |expression| {
        shift(&mut expression.span);
        if matches!(&expression.kind, ExprKind::Name(value) if value == "backend") { expression.kind = ExprKind::Name(if name=="parseJsonc"{"@json:decodeJsonc"}else if matches!(name,"readJsonl"|"readJsonlView"){"@json:decodeJsonl"}else{"@json:decode"}.into()); }
        Ok(())
    })?;
    // Statement and binding spans share the adapter's synthetic source region.
    let mut bodies = vec![&mut function.body];
    while let Some(body) = bodies.pop() {
        for statement in body {
            shift(&mut statement.span);
            if let StmtKind::Let { name, .. } = &mut statement.kind {shift(&mut name.span);}
            match &mut statement.kind {
                StmtKind::Block(body) => bodies.push(body),
                StmtKind::If { then_body, else_body, .. } => {bodies.push(then_body);bodies.push(else_body);},
                _ => {}
            }
        }
    }
    Ok(function)
}
