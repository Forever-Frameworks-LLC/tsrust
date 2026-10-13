//! Opt-in phase observation on the ordinary test worker stack.
//! No admission policy or compiler execution path depends on this probe.
use std::time::Instant;

fn event(start: Instant, phase: &str, status: &str, detail: serde_json::Value) {
    eprintln!("TSRUST_NESTING_EVENT {}", serde_json::json!({
        "phase": phase, "status": status, "elapsed_ns": start.elapsed().as_nanos(), "detail": detail
    }));
}

#[test]
#[ignore = "Opt-in pipeline discovery; requires source path and an external process budget"]
fn observe_pipeline_stages() {
    let path = std::env::var("TSRUST_NESTING_PROBE_SOURCE").expect("source path required");
    let source = std::fs::read_to_string(path).unwrap();
    let start = Instant::now();
    event(start, "parse", "start", serde_json::Value::Null);
    let program = match crate::syntax::parse(&source) {
        Ok(program) => program,
        Err(error) => { event(start, "parse", "diagnostic", serde_json::json!({"code":error.code,"message":error.message,"start":error.span.start,"end":error.span.end})); return; }
    };
    event(start, "parse", "finish", serde_json::Value::Null);
    event(start, "analysis", "start", serde_json::Value::Null);
    let checked = match crate::analysis::analyze(program) {
        Err(error) if error.code == "recursion-inference" => {
            // Match check(): speculative trees are discarded, then source is reparsed.
            event(start, "recursive-reparse", "start", serde_json::Value::Null);
            let program = crate::syntax::parse(&source).unwrap();
            event(start, "recursive-reparse", "finish", serde_json::Value::Null);
            event(start, "recursive-analysis", "start", serde_json::Value::Null);
            crate::analysis::analyze_recursive(program)
        }
        result => result,
    };
    let checked = match checked {
        Ok(program) => crate::Analysis { program },
        Err(error) => { event(start, "analysis", "diagnostic", serde_json::json!({"code":error.code,"message":error.message,"start":error.span.start,"end":error.span.end})); return; }
    };
    event(start, "analysis", "finish", serde_json::Value::Null);
    event(start, "emit", "start", serde_json::Value::Null);
    let output = checked.emit_rust();
    event(start, "emit", "finish", serde_json::json!({"bytes":output.len()}));
    event(start, "dispose", "start", serde_json::Value::Null);
    drop(output);
    drop(checked);
    event(start, "dispose", "finish", serde_json::Value::Null);
}
