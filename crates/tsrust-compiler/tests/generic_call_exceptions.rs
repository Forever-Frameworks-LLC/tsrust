mod support;

const DECLARATIONS: &str = "function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}";

fn source(body: &str) -> String {
    format!("{DECLARATIONS}function unused<T>(flag:boolean,value:T):T{{{body}}}function main(){{}}")
}

#[test]
fn exception_source_contracts_reject_invalid_reachable_calls() {
    let cases = [
        ("index failure carries current target", "let action=one;try{action=two;const item=[1][99];action=one;}catch(error){return action<T>(value);}return value;"),
        ("loop condition failure enters catch", "let action=one;try{action=two;do{}while([1][99]===1);action=one;}catch(error){return action<T>(value);}return value;"),
        ("cleanup throw overrides return", "let action=one;try{try{return value;}finally{action=two;throw 1;}}catch(error){return action<T>(value);}"),
        ("try local replacement", "let action=one;try{action=two;action<T>(value);}catch(error){}return value;"),
        ("catch local replacement", "let action=one;try{throw 1;}catch(error){action=two;action<T>(value);}return value;"),
        ("finally overwrite", "let action=one;try{}finally{action=two;}return action<T>(value);"),
        ("return still runs finally", "let action=one;try{action=two;return value;}finally{action<T>(value);}"),
        ("throw enters catch with current target", "let action=one;try{action=two;throw 1;}catch(error){return action<T>(value);}"),
        ("break runs finally", "let action=one;while(flag){try{break;}finally{action=two;}}return action<T>(value);"),
        ("continue runs finally", "let action=one;for(;flag;){try{action<T>(value);continue;}finally{action=two;}}return value;"),
        ("nested cleanup", "let action=one;try{try{return value;}finally{action=two;}}finally{action<T>(value);}"),
    ];
    let failures = cases.into_iter().filter_map(|(label, body)| {
        let source = source(body);
        match tsrust_compiler::check(&source) {
            Ok(_) => Some(format!("{label}: invalid call accepted")),
            Err(error) if error.code == "arity" && !error.related.is_empty() => None,
            Err(error) => Some(format!("{label}: {}", error.render("input.tsr", &source))),
        }
    }).collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn exception_contract_precision_preserves_snapshots_and_finally_completion() {
    for (label, body) in [
        ("saved snapshot", "let action=one;const saved=action;try{action=two;}finally{}return saved<T>(value);"),
        ("cleanup restores target", "let action=one;try{action=two;}finally{action=one;}return action<T>(value);"),
        ("returned target does not join", "let action=one;try{if(flag){action=two;return value;}}finally{}return action<T>(value);"),
        ("catch restores target", "let action=one;try{action=two;throw 1;}catch(error){action=one;}return action<T>(value);"),
        ("finally return overrides throw", "let action=one;try{throw 1;}finally{return action<T>(value);}"),
        ("shadowed cleanup", "let action=one;try{}finally{let action=two;action<T>(value,value);}return action<T>(value);"),
    ] {
        let source = source(body);
        let formatted = tsrust_compiler::format(&source).unwrap();
        assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
        if let Err(error) = tsrust_compiler::check(&formatted) {
            panic!("{label}: {}", error.render("input.tsr", &formatted));
        }
    }
}

#[test]
fn exception_source_contracts_execute_formatted_cleanup_and_snapshots() {
    let directory = support::TempDirectory::new();
    let source = include_str!("fixtures/source-call-exceptions/contracts.tsr");
    let formatted = tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    let path = directory.source(&formatted);
    let output = support::cli().arg("run").arg(path).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty());
}

#[test]
fn nested_cleanup_flow_remains_iterative_and_preserves_completion() {
    let mut body = "return action<T>(value);".to_owned();
    for _ in 0..24 {
        body = format!("try{{{body}}}finally{{action=action;}}");
    }
    let source = source(&format!("let action=one;{body}"));
    let formatted = tsrust_compiler::format(&source).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    tsrust_compiler::check(&formatted).unwrap();
}
