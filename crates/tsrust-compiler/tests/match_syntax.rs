use tsrust_compiler::match_syntax::{parse, Pattern};

#[test]
fn formatted_match_syntax_remains_parseable_and_idempotent() {
    let source = "match(make()){Ok(value)=>{useValue(value);},Err(_)=>{},_=>{}}";
    let formatted = tsrust_compiler::format(source).unwrap();
    assert!(formatted.starts_with("match (make()) {"), "{formatted}");
    assert!(formatted.contains("\n  Err(_) =>"), "{formatted}");
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    let parsed = parse(&formatted).unwrap();
    assert_eq!(parsed.arms.len(), 3);
    assert_eq!(&formatted[parsed.scrutinee.start..parsed.scrutinee.end], "make()");
}

#[test]
fn contextual_match_calls_keep_ordinary_call_spacing() {
    let source = "function match(value:int):int{return value;}function main(){console.log(match(1));}";
    let formatted = tsrust_compiler::format(source).unwrap();
    assert!(formatted.contains("function match(value: int)"), "{formatted}");
    assert!(formatted.contains("console.log(match(1));"), "{formatted}");
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    tsrust_compiler::check(&formatted).unwrap();
}

#[test]
fn match_parser_preserves_expression_pattern_binding_and_body_spans() {
    let source = "/* 🦀 */ match (produce(1)) { Ok(value) => { consume(value); }, Err(_) => {}, _ => {} }";
    let parsed = parse(source).unwrap();
    assert_eq!(&source[parsed.scrutinee.start..parsed.scrutinee.end], "produce(1)");
    assert_eq!(parsed.arms.len(), 3);
    let Pattern::Variant { name, binding } = &parsed.arms[0].pattern else { panic!("variant"); };
    assert_eq!(&source[name.start..name.end], "Ok");
    assert_eq!(&source[binding.unwrap().start..binding.unwrap().end], "value");
    assert_eq!(&source[parsed.arms[0].body.start..parsed.arms[0].body.end], "{ consume(value); }");
    assert!(matches!(parsed.arms[1].pattern, Pattern::Variant { binding: None, .. }));
    assert!(matches!(parsed.arms[2].pattern, Pattern::Wildcard(_)));
    assert_eq!(&source[parsed.span.start..parsed.span.end], &source["/* 🦀 */ ".len()..]);
}

#[test]
fn malformed_match_patterns_and_bodies_report_bounded_source_diagnostics() {
    for source in [
        "match (x) { Ok(x, y) => {} }",
        "match (x) { Ok() => {} }",
        "match (x) { Ok(value) {} }",
        "match (x) { Ok(value) => {",
        "match (x) { _() => {} }",
        "match (x) { Ok(value) if true => {} }",
        "match (x) {} trailing",
    ] {
        let error = parse(source).unwrap_err();
        assert_eq!(error.code, "syntax", "{source}: {}", error.message);
        assert!(error.span.start <= error.span.end && error.span.end <= source.len(), "{source}");
    }
}

#[test]
fn syntax_preserves_coverage_errors_for_type_checking() {
    for source in ["match (x) {}", "match (x) { _ => {}, Ok(value) => {} }", "match (x) { Ok(a) => {}, Ok(b) => {} }"] {
        parse(source).unwrap();
    }
}

#[test]
fn arm_bodies_share_existing_statement_parser_and_depth_limit() {
    let prefix = "if (true) {";
    let normal = format!("match (value) {{ Ok(x) => {{ {}consume(x);{} }} }}", prefix.repeat(100), "}".repeat(100));
    parse(&normal).unwrap();
    let deep = format!("match (value) {{ Ok(x) => {{ {}consume(x);{} }} }}", prefix.repeat(140), "}".repeat(140));
    let error = parse(&deep).unwrap_err();
    assert_eq!(error.code, "unsupported");
    assert!(error.message.contains("nesting"));
}

#[test]
fn concise_arms_and_literal_patterns_preserve_spans_after_formatting() {
    for source in ["match (x) { Ok(value) => value }", "match (x) { 4 => {} }"] {
        let formatted = tsrust_compiler::format(source).unwrap();
        assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
        let parsed = parse(&formatted).unwrap();
        assert_eq!(parsed.arms.len(), 1);
        let arm = &parsed.arms[0];
        match arm.pattern {
            Pattern::Literal(span) => assert_eq!(&formatted[span.start..span.end], "4"),
            Pattern::Variant { .. } => assert_eq!(&formatted[arm.body.start..arm.body.end], "value"),
            Pattern::Wildcard(_) => panic!("unexpected wildcard"),
        }
    }
}
