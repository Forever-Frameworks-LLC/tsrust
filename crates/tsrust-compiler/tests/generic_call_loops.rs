mod support;

const DECLARATIONS: &str =
    "function one<T>(value:T):T{return value;}function two<T>(value:T,other:T):T{return other;}";

fn source(body: &str) -> String {
    format!("{DECLARATIONS}function unused<T>(flag:boolean,value:T):T{{{body}}}function main(){{}}")
}

#[test]
fn loop_carried_source_contracts_reject_invalid_reachable_calls() {
    let cases = [
        (
            "numeric for update",
            "let action=one;for(let i=0;i<2;i++){action<T>(value);action=two;}return value;",
        ),
        (
            "ordered update calls",
            "let action=one;for(;flag;action=two,action<T>(value)){continue;}return value;",
        ),
        (
            "post-test condition",
            "let action=one;do{action=two;}while(action<T>(value));return value;",
        ),
        (
            "while back edge",
            "let action=one;while(flag){action<T>(value);action=two;}return value;",
        ),
        (
            "for update",
            "let action=one;for(;flag;action=two){action<T>(value);}return value;",
        ),
        (
            "continue update",
            "let action=one;for(;flag;action=two){action<T>(value);continue;}return value;",
        ),
        (
            "post-test assignment",
            "let action=one;do{action=two;}while(flag);return action<T>(value);",
        ),
        (
            "break exit",
            "let action=one;while(flag){action=two;break;}return action<T>(value);",
        ),
        (
            "for-of back edge",
            "let action=one;for(const item of [1,2]){action<T>(value);action=two;}return value;",
        ),
        (
            "labeled continue",
            "let action=one;outer:for(;flag;action=two){while(flag){action<T>(value);continue outer;}}return value;",
        ),
    ];
    let failures = cases
        .into_iter()
        .filter_map(|(label, body)| {
            let source = source(body);
            match tsrust_compiler::check(&source) {
                Ok(_) => Some(format!("{label}: invalid call accepted")),
                Err(error) if error.code == "arity" && !error.related.is_empty() => None,
                Err(error) => Some(format!("{label}: {}", error.render("input.tsr", &source))),
            }
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn loop_contract_precision_preserves_overwrites_snapshots_and_completion_order() {
    let cases = [
        (
            "empty iteration",
            "let action=one;for(const item of []){action=two;}return action<T>(value);",
        ),
        (
            "single iteration",
            "let action=one;for(const item of [1]){action<T>(value);action=two;}return action<T>(value,value);",
        ),
        (
            "zero iterations",
            "let action=one;while(false){action=two;}return action<T>(value);",
        ),
        (
            "body overwrite",
            "let action=one;while(flag){action=one;action<T>(value);action=two;}return value;",
        ),
        (
            "saved snapshot",
            "let action=one;const saved=action;while(flag){action=two;}return saved<T>(value);",
        ),
        (
            "post-test overwrite",
            "let action=one;do{action=two;}while(flag);return action<T>(value,value);",
        ),
        (
            "break skips update",
            "let action=one;for(;flag;action=two){break;}return action<T>(value);",
        ),
        (
            "labeled break skips update",
            "let action=one;outer:for(;flag;action=two){while(flag){break outer;}break;}return action<T>(value);",
        ),
        (
            "loop shadowing",
            "let action=one;while(flag){let action=two;action<T>(value,value);}return action<T>(value);",
        ),
        (
            "block break",
            "let action=one;done:{action=one;break done;action=two;}return action<T>(value);",
        ),
    ];
    let failures = cases
        .into_iter()
        .filter_map(|(label, body)| {
            let source = support::format_round_trip(&source(body));
            tsrust_compiler::check(&source)
                .err()
                .map(|error| format!("{label}: {}", error.render("input.tsr", &source)))
        })
        .collect::<Vec<_>>();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn loop_source_contracts_preserve_native_ownership_and_typescript_results() {
    let source=include_str!("fixtures/source-call-loops/contracts.tsr");
    let formatted=support::format_round_trip(source);
    let directory=support::TempDirectory::new();
    let path=directory.source(&formatted);
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
