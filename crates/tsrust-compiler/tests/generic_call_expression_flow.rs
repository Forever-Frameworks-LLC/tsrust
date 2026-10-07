mod support;

const DECLARATIONS: &str = "function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}function accept<T>(first:T,second:T):T{return first;}";
fn source(body: &str) -> String {
    format!("{DECLARATIONS}function unused<T>(flag:boolean,value:T):T{{{body}}}function main(){{}}")
}

#[test]
fn expression_source_contracts_reject_invalid_calls_in_evaluation_order() {
    let cases = [
        ("receiver effects precede arguments", "let action=one;const holder={run:one};((action=two),holder).run(action<T>(value));return value;"),
        ("nullish selected write", "let action=one;null??(action=two);return action<T>(value);"),
        ("logical assignment branch", "let action=one;let selected=flag;selected&&=(action=two,flag);return action<T>(value);"),
        ("sequence replacement", "let action=one;(action=two,action<T>(value));return value;"),
        ("assignment-result callee", "let action=one;return (action=two)<T>(value);"),
        ("argument replacement before nested call", "let action=one;accept((action=two),action<T>(value));return value;"),
        ("callee snapshot before argument mutation", "let action=one;action<T>((action=two,value),value);return value;"),
        ("conditional branch replacement", "let action=one;(flag?(action=two):action);return action<T>(value);"),
        ("short-circuit replacement", "let action=one;flag&&(action=two);return action<T>(value);"),
        ("logical-or replacement", "let action=one;flag||(action=two);return action<T>(value);"),
        ("condition replacement before branch call", "let action=one;if((action=two,flag)){action<T>(value);}return value;"),
        ("for update sequence", "let action=one;for(;flag;(action=two,action<T>(value))){}return value;"),
        ("partial evaluation before failure", "let action=one;try{(action=two,[1][99],action=one);}catch(error){return action<T>(value);}return value;"),
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
fn expression_source_contracts_preserve_snapshots_lazy_paths_and_replacements() {
    for (label, body) in [
        ("callee snapshot", "let action=one;action<T>((action=two,value));return value;"),
        ("callee snapshot with larger arity", "let action=two;action<T>((action=one,value),value);return value;"),
        ("sequence restores target", "let action=one;(action=two,action=one);return action<T>(value);"),
        ("saved initializer snapshot", "let action=one;const saved=(action=action,action);(action=two);return saved<T>(value);"),
        ("conditional both arms restore", "let action=two;(flag?(action=one):(action=one));return action<T>(value);"),
        ("coalescing skips dead write", "let action=one;0??(action=two);return action<T>(value);"),
        ("and skips dead write", "let action=one;false&&(action=two);return action<T>(value);"),
        ("or skips dead write", "let action=one;true||(action=two);return action<T>(value);"),
        ("assignment returns right snapshot", "let action=one;return (action=(action=two,one))<T>(value);"),
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
fn expression_contracts_execute_formatted_assignment_callees_and_lazy_order() {
    let directory = support::TempDirectory::new();
    let source = include_str!("fixtures/source-call-expression-order/contracts.tsr");
    let formatted = tsrust_compiler::format(source).unwrap();
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted);
    let output = support::cli().arg("run").arg(directory.source(&formatted)).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(output.stdout.is_empty());
}
