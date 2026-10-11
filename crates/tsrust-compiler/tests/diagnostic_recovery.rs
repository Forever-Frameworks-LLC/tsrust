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

#[test]
fn semantic_recovery_reports_independent_types_and_suppresses_poisoned_binding_cascades() {
    let source="function main(){const first:int=\"bad\";const dependent=first;console.log(dependent);const second:boolean=42;}";
    let errors=tsrust_compiler::diagnostics(source);
    assert_eq!(errors.iter().map(|error|error.code).collect::<Vec<_>>(),vec!["type","type"],"{errors:?}");
    assert_eq!(errors[0].span.start,source.find("\"bad\"").unwrap());
    assert_eq!(errors[1].span.start,source.find("42").unwrap());
    assert!(tsrust_compiler::check(source).is_err());
}

#[test]
fn semantic_recovery_preserves_moves_and_lexical_shadowing() {
    let source="function main(){const value=\"owned\";const moved=value;console.log(value);const bad:int=\"bad\";console.log(value);}";
    let errors=tsrust_compiler::diagnostics(source);
    assert_eq!(errors.iter().map(|error|error.code).collect::<Vec<_>>(),vec!["transferred","type","transferred"],"{errors:?}");
    let shadow="function main(){const value=7;{const value:int=\"bad\";const dependent=value;console.log(dependent);}const other:boolean=42;console.log(value);}";
    let errors=tsrust_compiler::diagnostics(shadow);
    assert_eq!(errors.iter().map(|error|error.code).collect::<Vec<_>>(),vec!["type","type"],"{errors:?}");
}

#[test]
fn semantic_recovery_preserves_return_boundaries_and_nested_callback_scopes() {
    let errors=tsrust_compiler::diagnostics("function value():int{return \"bad\";}");
    assert_eq!(errors.len(),1,"{errors:?}");assert!(errors[0].message.contains("Expected int"));
    let source="function main(){const callback=():void=>{const first:int=\"bad\";const dependent=first;console.log(dependent);const second:boolean=42;};callback();}";
    let errors=tsrust_compiler::diagnostics(source);
    assert_eq!(errors.iter().map(|error|error.code).collect::<Vec<_>>(),vec!["type","type"],"{errors:?}");
    let independent="function first(){const bad:int=\"bad\";}function second(){const bad:boolean=42;}";
    assert_eq!(tsrust_compiler::diagnostics(independent).len(),2);
}

#[test]
fn semantic_replay_has_a_bounded_error_count() {
    let source=format!("function main(){{{}}}",(0..80).map(|i|format!("const value{i}:int=\"bad\";")).collect::<String>());
    let errors=tsrust_compiler::diagnostics(&source);
    assert_eq!(errors.len(),64);
    assert!(errors.iter().all(|error|error.code=="type"));
    assert!(errors.windows(2).all(|pair|pair[0].span.start<pair[1].span.start));
}

#[test]
fn semantic_replay_keeps_overlapping_access_errors_and_related_locations() {
    let source=r#"type User={name:string;};function both(a:User,b:User){a.name="Sam";console.log(b.name);}function main(){const bad:int="bad";const user:User={name:"Pat"};both(user,user);both(user,user);}"#;
    let errors=tsrust_compiler::diagnostics(source);
    assert_eq!(errors.iter().map(|error|error.code).collect::<Vec<_>>(),vec!["type","conflicting-access","conflicting-access"],"{errors:?}");
    assert!(errors[1..].iter().all(|error|error.related.len()==1));
}
