#[test]
fn syntax_recovery_reports_independent_statements_and_preserves_original_spans() {
    let source="function main(){const first=;const second=;console.log(1);}function other(){const third=;}";
    let diagnostics=tsrust_compiler::diagnostics(source);
    assert_eq!(diagnostics.len(),3,"{diagnostics:?}");
    for (error,offset) in diagnostics.iter().zip(source.match_indices("=;").map(|(offset,_)|offset+1)) {
        assert_eq!(error.span.start,offset);
        assert_eq!(error.code,"syntax");
    }
    assert!(tsrust_compiler::check(source).is_err());
}

#[test]
fn syntax_recovery_respects_nested_blocks_strings_and_missing_semicolons() {
    let source="function main(){if(true){const first=;}console.log(\";}\");const second=;}";
    assert_eq!(tsrust_compiler::diagnostics(source).len(),2);
    let missing="function main(){const first=1 const second=;const third=;}";
    assert_eq!(tsrust_compiler::diagnostics(missing).len(),3);
    assert!(tsrust_compiler::diagnostics("function main(){console.log(\";}\");}").is_empty());
}

#[test]
fn syntax_recovery_terminates_on_unclosed_blocks_and_keeps_strict_checking() {
    for source in ["function main(){const value=;", "function main(){const value=(;}", "function main(){const value={a:;};}"] {
        let diagnostics=tsrust_compiler::diagnostics(source);
        assert!(!diagnostics.is_empty());
        assert!(diagnostics.len()<=3,"{diagnostics:?}");
        assert!(tsrust_compiler::check(source).is_err());
    }
    assert_eq!(tsrust_compiler::diagnostics("function main(){const value:int=\"bad\";}")[0].code,"type");
}

#[test]
fn syntax_recovery_is_bounded_and_preserves_the_strict_first_error() {
    let source=format!("function main(){{{}}}","const broken=;".repeat(100));
    let strict=tsrust_compiler::check(&source).err().unwrap();
    let errors=tsrust_compiler::diagnostics(&source);
    assert_eq!(errors.len(),64);
    assert_eq!(errors[0].span,strict.span);
    assert_eq!(errors[0].message,strict.message);
    assert!(errors.windows(2).all(|pair|pair[0].span.start<pair[1].span.start));
}
