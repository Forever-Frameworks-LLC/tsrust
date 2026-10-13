//! Opt-in parser stage observation on the ordinary test worker stack.
//! No admission policy or compiler execution path depends on this probe.
use super::*;
use std::time::Instant;

fn event(start: Instant, phase: &str, status: &str, detail: serde_json::Value) {
    eprintln!("TSRUST_PARSER_EVENT {}", serde_json::json!({
        "phase": phase, "status": status, "elapsed_ns": start.elapsed().as_nanos(), "detail": detail
    }));
}

fn diagnostic(error: &Diagnostic) -> serde_json::Value {
    serde_json::json!({"code": error.code, "message": error.message, "start": error.span.start, "end": error.span.end})
}

#[test]
#[ignore = "Opt-in parser discovery; requires source path and an external process budget"]
fn observe_parser_stages() {
    let path = std::env::var("TSRUST_PARSER_PROBE_SOURCE").expect("source path required");
    let kind = std::env::var("TSRUST_PARSER_PROBE_KIND").unwrap_or_else(|_| "program".into());
    let source = std::fs::read_to_string(path).unwrap();
    // Consumer stages are optional so a slow consumer cannot end parser discovery.
    let stages = std::env::var("TSRUST_PARSER_PROBE_STAGES").unwrap_or_else(|_| "core,recovery,format,bounded".into());
    let stage = |name: &str| stages.split(',').any(|selected| selected == name);
    let start = Instant::now();
    if kind == "type" {
        event(start, "type-parse", "start", serde_json::Value::Null);
        match crate::type_syntax::parse_with(&source, NestingPolicy::UNBOUNDED) {
            Ok(annotation) => {
                event(start, "type-parse", "finish", serde_json::json!({"span_end": annotation.span.end}));
                event(start, "type-clone", "start", serde_json::Value::Null);
                let copy = annotation.clone();
                event(start, "type-clone", "finish", serde_json::json!({"equal": copy == annotation}));
                event(start, "dispose", "start", serde_json::Value::Null);
                drop(copy);
                drop(annotation);
                event(start, "dispose", "finish", serde_json::Value::Null);
            }
            Err(error) => event(start, "type-parse", "diagnostic", diagnostic(&error)),
        }
        if stage("bounded") {
            event(start, "bounded-type-parse", "start", serde_json::Value::Null);
            match crate::type_syntax::parse(&source) {
                Ok(_) => event(start, "bounded-type-parse", "finish", serde_json::Value::Null),
                Err(error) => event(start, "bounded-type-parse", "diagnostic", diagnostic(&error)),
            }
        }
        return;
    }
    event(start, "sizes", "finish", serde_json::json!({
        "token": std::mem::size_of::<Token>(), "expression": std::mem::size_of::<Expr>(),
        "statement": std::mem::size_of::<Stmt>(), "work": std::mem::size_of::<scheduler::Work>(),
    }));
    event(start, "lex", "start", serde_json::Value::Null);
    match lex_with(&source, NestingPolicy::UNBOUNDED) {
        Ok(tokens) => event(start, "lex", "finish", serde_json::json!({"tokens": tokens.len()})),
        Err(error) => event(start, "lex", "diagnostic", diagnostic(&error)),
    }
    event(start, "parse", "start", serde_json::Value::Null);
    match parse_unbounded(&source) {
        Ok(program) => {
            event(start, "parse", "finish", serde_json::json!({"functions": program.functions.len(), "records": program.records.len(), "module_bindings": program.module_bindings.len()}));
            event(start, "clone", "start", serde_json::Value::Null);
            let copy = program.clone();
            event(start, "clone", "finish", serde_json::Value::Null);
            event(start, "dispose", "start", serde_json::Value::Null);
            drop(copy);
            drop(program);
            event(start, "dispose", "finish", serde_json::Value::Null);
        }
        Err(error) => event(start, "parse", "diagnostic", diagnostic(&error)),
    }
    // Statement recovery retains every suspended scope while it resynchronizes.
    if stage("recovery") {
        event(start, "recovery", "start", serde_json::Value::Null);
        let errors = diagnostics_unbounded(&source);
        event(start, "recovery", "finish", serde_json::json!({
            "count": errors.len(),
            "first": errors.first().map(diagnostic),
            "last": errors.last().map(diagnostic),
        }));
        drop(errors);
    }
    // The formatter consumes the lexer and the annotation probe.
    if stage("format") {
        event(start, "format", "start", serde_json::Value::Null);
        match crate::format(&source) {
            Ok(formatted) => {
                let stable = crate::format(&formatted).is_ok_and(|again| again == formatted);
                event(start, "format", "finish", serde_json::json!({"bytes": formatted.len(), "idempotent": stable}));
            }
            Err(error) => event(start, "format", "diagnostic", diagnostic(&error)),
        }
    }
    // The public entry keeps the documented admission bound for tree consumers.
    if stage("bounded") {
        event(start, "bounded-parse", "start", serde_json::Value::Null);
        match parse_raw(&source, false, Default::default(), Default::default()) {
            Ok(_) => event(start, "bounded-parse", "finish", serde_json::Value::Null),
            Err(error) => event(start, "bounded-parse", "diagnostic", diagnostic(&error)),
        }
    }
}
