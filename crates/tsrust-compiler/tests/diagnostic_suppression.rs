mod support;

#[test]
fn ignore_hides_check_diagnostics_without_authorizing_codegen() {
    let source = "function main() {\n// @tsrust ignore - intentional negative fixture\nconst value: int = \"wrong\";\n}\n";
    assert!(tsrust_compiler::check(source).is_err());
    let directory = support::TempDirectory::new();
    let path = directory.source(source);
    let checked = support::run(support::cli().arg("check").arg(&path).arg("--json"), 10);
    support::success(&checked);
    assert_eq!(String::from_utf8(checked.stdout).unwrap().trim(), "{\"ok\":true,\"diagnostics\":[]}");
    for command in ["emit", "build"] {
        let output = support::run(support::cli().arg(command).arg(&path), 10);
        assert!(!output.status.success(), "{command} must retain strict errors");
        assert!(!output.stderr.is_empty(), "{command} must report the error");
    }
}

#[test]
fn directive_ranges_follow_nodes_and_survive_formatting() {
    for source in [
        "function main(){\n// @tsrust ignore - multiline\nconst value:int=\n \"wrong\";\nconst other:int=\"visible\";}",
        "// @tsrust ignore - declaration\nfunction hidden():int {return \"wrong\";}\nfunction visible():int{return \"visible\";}",
        "// @tsrust ignore - record definition\ntype Hidden={value:Unknown};\ntype Visible={value:VisibleUnknown};",
        "function main(){\n// @tsrust ignore - branch\nif(true){const value:int=\"wrong\";}else{const other:int=\"wrong\";}\nconst visible:int=\"visible\";}",
    ] {
        for text in [source.to_owned(), support::format_round_trip(source)] {
            let directives=tsrust_compiler::DiagnosticSuppressions::new(&text);
            let hidden=text.find("wrong").or_else(||text.find("Unknown")).unwrap();
            let visible=text.rfind("visible").or_else(||text.find("VisibleUnknown")).unwrap();
            assert!(directives.suppresses(tsrust_compiler::Span::new(hidden,hidden+1)),"{text}");
            assert!(!directives.suppresses(tsrust_compiler::Span::new(visible,visible+1)),"{text}");
        }
    }
}

#[test]
fn ordinary_comment_or_literal_text_does_not_suppress() {
    for prefix in [
        "// @tsrust ignore -\n",
        "// @tsrust ignore\n",
        "/* // @tsrust ignore - fake */\n",
        "const text = `\n// @tsrust ignore - fake\n`;\n",
        "const text = '// @tsrust ignore - fake';\n",
    ] {
        let source=format!("function main(){{{prefix}const value:int=\"wrong\";}}");
        let directives=tsrust_compiler::DiagnosticSuppressions::new(&source);
        for error in tsrust_compiler::diagnostics(&source) { assert!(!directives.suppresses(error.span),"{source}"); }
    }
}

#[test]
fn check_keeps_unsuppressed_siblings_and_imported_errors() {
    let directory=support::TempDirectory::new();
    let source="function main(){\n// @tsrust ignore - one error only\nconst hidden:int=\"hidden\";\nconst visible:int=\"visible\";}";
    let path=directory.source(source);
    let output=support::run(support::cli().arg("check").arg(&path).arg("--json"),10);
    assert!(!output.status.success());
    let value:serde_json::Value=serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["diagnostics"].as_array().unwrap().len(),1);
    let dependency=directory.0.join("dep.tsr");
    std::fs::write(&dependency,"// @tsrust ignore - dependency fixture\nexport function value():int{return \"wrong\";}").unwrap();
    std::fs::write(&path,"import {value} from './dep.tsr';function main(){value();}").unwrap();
    let output=support::run(support::cli().arg("check").arg(&path).arg("--json"),10);
    support::success(&output);
    let output=support::run(support::cli().arg("build").arg(&path),10);
    assert!(!output.status.success());
}

#[test]
fn syntax_errors_and_unicode_comments_keep_strict_analysis() {
    for source in [
        "function main(){\n// @tsrust ignore - incomplete fixture 😀\nconst value:int=;\n}",
        "function main(){\r\n// @tsrust ignore - invalid token\r\nconst value:int=@;\r\n}",
        "// @tsrust ignore - unsupported field\ninterface Hidden { value: Unknown; }\n",
    ] {
        let errors=tsrust_compiler::diagnostics(source);
        assert!(!errors.is_empty());
        let directives=tsrust_compiler::DiagnosticSuppressions::new(source);
        assert!(errors.iter().all(|error|directives.suppresses(error.span)),"{source}: {errors:?}");
        assert!(tsrust_compiler::check(source).is_err());
    }
}

#[test]
fn lexical_recovery_does_not_extend_ignore_into_the_next_statement() {
    let source="function main(){\n// @tsrust ignore - invalid token fixture\nconst hidden:int=@;\nconst visible:int=\"wrong\";}";
    let directives=tsrust_compiler::DiagnosticSuppressions::new(source);
    let hidden=source.find("=@").unwrap()+1;
    let visible=source.find("visible").unwrap();
    assert!(directives.suppresses(tsrust_compiler::Span::new(hidden,hidden+1)));
    assert!(!directives.suppresses(tsrust_compiler::Span::new(visible,visible+1)));
}
