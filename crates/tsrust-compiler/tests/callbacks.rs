mod support;

use support::{execute, reject};
use tsrust_compiler::{Mode, check};

#[test]
fn named_read_callbacks_are_reusable_and_statically_dispatched() {
    let rust = execute(
        r#"function twice(action: Mutable<Function<void>>) { action(); action(); }
function main() {
    const text = "hello";
    const read = (): void => { console.log(text); };
    twice(read); read(); twice(read);
    twice((): void => { console.log("inline"); });
}"#,
        "hello\nhello\nhello\nhello\nhello\ninline\ninline\n",
        Some(&[("twice.action", Mode::Mutate)]),
    );
    assert!(rust.contains("F0: Callback<(), ()>"));
    assert!(rust.contains("Callback::invoke(value0, ())"));
    for hidden in ["Box<", "dyn ", "Rc<", "Clone::clone("] {
        assert!(!rust.contains(hidden), "{hidden}");
    }
}

#[test]
fn mutable_inline_captures_release_after_each_call() {
    execute(
        include_str!("fixtures/callbacks/inline_mutation.tsr"),
        "true\nfalse\nfalse\ntrue\ntrue\ntrue\ntrue\n",
        Some(&[("twice.action", Mode::Mutate), ("use.state", Mode::Mutate)]),
    );
}

#[test]
fn returned_environments_work_as_named_and_temporary_callbacks() {
    execute(
        r#"type State = { on: boolean; };
function make(): () => boolean {
    const state: State = { on: false };
    return (): boolean => {
        if (state.on) { state.on = false; } else { state.on = true; }
        return state.on;
    };
}
function use(action: Mutable<Function<boolean>>) { console.log(action()); console.log(action()); }
function main() {
    const action = make(); use(action); console.log(action()); use(action);
    use(make());
}"#,
        "true\nfalse\ntrue\nfalse\ntrue\ntrue\nfalse\n",
        Some(&[("use.action", Mode::Mutate)]),
    );
}

#[test]
fn callback_results_cover_all_existing_owned_and_copy_types() {
    execute(
        r#"type User = { name: string; };
function integer(action: Mutable<Function<int>>): int { return action(); }
function text(action: Mutable<Function<string>>): string { return action(); }
function record(action: Mutable<Function<User>>): User { return action(); }
function flag(action: Mutable<Function<boolean>>): boolean { return action(); }
function main() {
    console.log(integer((): int => { return 7; }));
    const original = " Pat ";
    const read = (): string => { return original.trim(); };
    console.log(text(read)); console.log(text(read));
    const user = record((): User => { return { name: "Sam" }; }); console.log(user.name);
    console.log(flag((): boolean => { return true; }));
}"#,
        "7\nPat\nPat\nSam\ntrue\n",
        Some(&[
            ("integer.action", Mode::Mutate),
            ("text.action", Mode::Mutate),
            ("record.action", Mode::Mutate),
            ("flag.action", Mode::Mutate),
        ]),
    );
}

#[test]
fn forwarding_and_branch_contracts_propagate_before_callers_are_checked() {
    execute(
        r#"function outer(action: Mutable<Function<string>>, flag: boolean): string { return branch(action, flag); }
function branch(action: Mutable<Function<string>>, flag: boolean): string {
    if (flag) { return invoke(action); } else { return "skipped"; }
}
function invoke(action: Mutable<Function<string>>): string { return action(); }
function unused(action: Mutable<Function<string>>) {}
function unusedForward(action: Mutable<Function<string>>) { unused(action); }
function main() {
    const text = " read "; const action = (): string => { return text.trim(); };
    unusedForward(action); console.log(outer(action, true)); console.log(outer(action, false));
    console.log(action());
}"#,
        "read\nskipped\nread\n",
        Some(&[
            ("outer.action", Mode::Mutate),
            ("outer.flag", Mode::Copy),
            ("branch.action", Mode::Mutate),
            ("branch.flag", Mode::Copy),
            ("invoke.action", Mode::Mutate),
            ("unused.action", Mode::Read),
            ("unusedForward.action", Mode::Read),
        ]),
    );
}

#[test]
fn named_mutable_captures_keep_block_lifetime_after_callback_calls() {
    let source = r#"type User = { name: string; };
function use(action: Mutable<Function<void>>) { action(); }
function main() {
    const user: User = { name: "Pat" };
    if (true) {
        const action = (): void => { user.name = "Sam"; };
        use(action); action();
    }
    console.log(user.name);
}"#;
    execute(source, "Sam\n", None);
    let bad = source.replace(
        "use(action); action();",
        "use(action); action(); console.log(user.name);",
    );
    let error = reject(&bad, "capture-conflict");
    assert!(!error.related.is_empty());
}

#[test]
fn active_callback_arguments_require_exclusive_access() {
    for source in [
        "function both(a: Mutable<Function<void>>, b: Mutable<Function<void>>) { a(); b(); } function main() { const f = (): void => {}; both(f, f); }",
        "function both(a: Mutable<Function<void>>, b: Mutable<Function<void>>) { a(); b(); } function forward(f: Mutable<Function<void>>) { both(f, f); } function main() {}",
        "function both(a: Mutable<Function<void>>, b: Mutable<Function<void>>) { b(); } function main() { const f = (): void => {}; both(f, f); }",
        "function both(a: Mutable<Function<void>>, b: Mutable<Function<void>>) { a(); } function main() { const f = (): void => {}; both(f, f); }",
        "function later(f: Mutable<Function<int>>, value: int) { console.log(f()); } function main() { const f = (): int => { return 1; }; later(f, f()); }",
    ] {
        let error = reject(source, "conflicting-access");
        assert!(!error.related.is_empty());
    }
    execute(
        "function unused(a: Mutable<Function<void>>, b: Mutable<Function<void>>) {} function main() { const f = (): void => {}; unused(f, f); f(); }",
        "",
        Some(&[("unused.a", Mode::Read), ("unused.b", Mode::Read)]),
    );
}

#[test]
fn inline_capture_loans_cover_later_arguments_and_respect_earlier_arguments() {
    let prefix = r#"type User = { name: string; };
function first(action: Mutable<Function<void>>, user: User) { action(); console.log(user.name); }
function last(user: User, action: Mutable<Function<void>>) { console.log(user.name); action(); }
function mutate(user: User): int { user.name = "changed"; return 1; }
function nested(action: Mutable<Function<void>>, value: int) { action(); }
function main() { const user: User = { name: "Pat" }; "#;
    for (body, code) in [
        (
            "first((): void => { user.name = \"Sam\"; }, user);",
            "capture-conflict",
        ),
        (
            "last(user, (): void => { user.name = \"Sam\"; });",
            "conflicting-access",
        ),
        (
            "nested((): void => { console.log(user.name); }, mutate(user));",
            "capture-conflict",
        ),
    ] {
        let error = reject(&format!("{prefix}{body}}}"), code);
        assert!(!error.related.is_empty());
    }
    execute(
        &format!(
            "{prefix}first((): void => {{ console.log(user.name); }}, user); user.name = \"Sam\"; console.log(user.name);}}"
        ),
        "Pat\nPat\nSam\n",
        None,
    );
}

#[test]
fn nested_call_cleanup_preserves_outer_and_named_capture_loans() {
    let prefix = r#"type User = { name: string; };
function invoke(action: Mutable<Function<int>>): int { return action(); }
function outer(action: Mutable<Function<void>>, value: int, user: User) { action(); user.name = "changed"; }
function main() { const user: User = { name: "Pat" }; "#;
    reject(
        &format!(
            "{prefix}outer((): void => {{ console.log(user.name); }}, invoke((): int => {{ return 1; }}), user);}}"
        ),
        "capture-conflict",
    );
    reject(
        &format!(
            "{prefix}const read = (): void => {{ console.log(user.name); }}; invoke((): int => {{ return 1; }}); user.name = \"Sam\";}}"
        ),
        "capture-conflict",
    );
    execute(
        r#"type User = { name: string; };
function read(action: Mutable<Function<string>>): string { return action(); }
function write(user: User, value: string) { user.name = value; }
function main() {
    const user: User = { name: " Pat " };
    const text = read((): string => { return user.name.trim(); });
    write(user, text); console.log(user.name);
}"#,
        "Pat\n",
        None,
    );
}

#[test]
fn callback_parameters_cannot_escape_through_return_or_capture() {
    for body in [
        "return action;",
        "const captured = (): void => { action(); };",
        "const captured = (): void => { forward(action); };",
        "console.log(action);",
    ] {
        reject(
            &format!(
                "function forward(f: Mutable<Function<void>>) {{ f(); }} function invalid(action: Mutable<Function<void>>) {{ {body} }} function main() {{}}"
            ),
            "callback-escape",
        );
    }
    reject(
        "function factory(action: Mutable<Function<void>>): () => void { return action; } function main() {}",
        "callback-escape",
    );
    reject(
        "function factory(action: Mutable<Function<void>>): () => void { return (): void => { action(); }; } function main() {}",
        "callback-escape",
    );
}

#[test]
fn consuming_closures_never_satisfy_reusable_callback_annotations() {
    for body in ["action();", ""] {
        for argument in ["once", "(): string => { return text; }"] {
            let source = format!(
                r#"function use(action: Mutable<Function<string>>) {{ {body} }}
function main() {{ const text = "Pat"; {} use({argument}); }}"#,
                if argument == "once" {
                    "const once = (): string => { return text; };"
                } else {
                    ""
                }
            );
            let error = reject(&source, "callback-once");
            assert!(!error.related.is_empty());
        }
    }
    reject(
        r#"function make(text: string): () => string { return (): string => { return text; }; }
function use(action: Mutable<Function<string>>) { action(); }
function main() { use(make("Pat")); }"#,
        "callback-once",
    );
}

#[test]
fn result_mismatches_and_unsupported_callback_shapes_are_source_errors() {
    for (source, code) in [
        (
            "function use(f: Mutable<Function<int>>) {} function main() { use((): boolean => { return true; }); }",
            "type",
        ),
        (
            "function use(f: Mutable<Function<int>>) {} function forward(f: Mutable<Function<string>>) { use(f); } function main() {}",
            "type",
        ),
        (
            "function use(f: Mutable<Function<void>>) {} function main() { use(1); }",
            "type",
        ),
        (
            "function use(f: Mutable<Function<void>>) {} function named() {} function main() { use(main); }",
            "unsupported",
        ),
        (
            "function use(f: Mutable<Function<void>>) { f(1); } function main() {}",
            "arity",
        ),
        (
            "function use(f: Mutable<Function<Missing>>) {} function main() {}",
            "type",
        ),
        (
            "function use(f: Mutable<Function<void, [x: string]>>) {} function main() {}",
            "callback-access",
        ),
        (
            "function use(f: Mutable<Function<() =>>>) {} function main() {}",
            "syntax",
        ),
        ("function use(f: () void) {} function main() {}", "syntax"),
        (
            "function use(f: Mutable<() =>>) {} function main() {}",
            "syntax",
        ),
        (
            "function use(f: Function<void>): () => int { return f; } function main() {}",
            "type",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn callbacks_check_and_emit_without_backend_and_reject_before_build() {
    let valid = "function use(f: Mutable<Function<int>>): int { return f(); } function main() { console.log(use((): int => { return 7; })); }";
    let invalid = "function use(f: Mutable<Function<int>>) {} function main() { use((): boolean => { return true; }); }";
    for (source, command, status) in [
        (valid, "check", 0),
        (valid, "emit", 0),
        (invalid, "check", 1),
        (invalid, "build", 1),
    ] {
        let dir = support::TempDirectory::new();
        let path = dir.source(source);
        let mut invocation = support::cli();
        invocation.arg(command).arg(path).env("PATH", "");
        if command == "build" {
            invocation.arg("--output").arg(dir.0.join("program"));
        }
        let result = support::run(&mut invocation, 10);
        assert_eq!(result.status.code(), Some(status));
        if status == 1 {
            let error = String::from_utf8(result.stderr).unwrap();
            assert!(error.contains("type"), "{error}");
            assert!(!error.contains("rustup"));
        }
    }
    assert_eq!(check(valid).unwrap().contracts()["use.f"], Mode::Mutate);
}

#[test]
fn grouped_structured_callbacks_preserve_owned_and_nested_contracts() {
    execute(
        r#"type User = { name: string; };
function apply(action: ((Function<User, [value: Owned<User>]>)), value: User): User {
    return action(value);
}
function factory(): (((value: int) => int)) {
    return (value: int): int => { return value + 1; };
}
function nested(action: ((Function<int, [inner: ((Function<int, [value: int]>)), value: int]>)), value: int): int {
    return action(factory(), value);
}
function main() {
    const user: User = { name: "Pat" };
    const result = apply((value: User): User => { return value; }, user);
    console.log(result.name);
    console.log(nested((inner: Function<int, [value: int]>, value: int): int => { return inner(value); }, 4));
}"#,
        "Pat\n5\n",
        None,
    );
}

#[test]
fn grouped_callbacks_allow_nullable_data_and_nullable_callable_storage() {
    execute(
        r#"function apply(action: (((value: int | null) => int | null)), value: int | null): int | null { return action(value); }
function main() { console.log(String(apply((value: int | null): int | null => { return value; }, null))); }"#,
        "null\n",
        None,
    );
    for (annotation, absent) in [
        ("((() => int) | null)", "null"),
        ("(((Function<int>) | undefined))", "undefined"),
    ] {
        let source = format!(
            "function apply(action: {annotation}):int{{if(action==={absent}){{return -1;}}return action();}}function main(){{console.log(apply(():int=>{{return 7;}}),apply({absent}));}}"
        );
        execute(&source, "7 -1\n", None);
    }
}

#[test]
fn explicit_mutate_callable_parameters_borrow_and_release_owned_environments() {
    execute(
        "function counter():()=>int{let value=0;return ():int=>{value=value+1;return value;};}function twice(action:Mutable<Function<int>>):int{return action()+action();}function grouped(action:((Mutable<Function<int>>))):int{return action();}function main(){const action=counter();console.log(twice(action),action(),grouped(action),action());}",
        "3 3 4 5\n",
        Some(&[
            ("twice.action", Mode::Mutate),
            ("grouped.action", Mode::Mutate),
        ]),
    );
    reject(
        "function escape(action:Mutable<Function<int>>):()=>int{return action;}function main(){}",
        "callback-escape",
    );
    reject(
        "type Handler={run:Mutable<Function<int>>;};function main(){}",
        "callback-escape",
    );
}

#[test]
fn explicit_read_callbacks_borrow_without_environment_mutation() {
    let rust = execute(
        "function twice(action:ReadOnly<Function<int>>):int{return action()+action();}function main(){const text='Pat';const action=():int=>{return text.length;};console.log(twice(action),action(),text);}",
        "6 3 Pat\n",
        Some(&[("twice.action", Mode::Read)]),
    );
    assert!(rust.contains("ReadCallback::invoke_read"));
    for source in [
        "function use(action:ReadOnly<Function<int>>):int{return action();}function main(){let value=0;use(():int=>{value=value+1;return value;});}",
        "function use(action:ReadOnly<Function<string>>):string{return action();}function main(){const text='Pat';use(():string=>{return text;});}",
    ] {
        reject(source, "callback-access");
    }
    reject(
        "function escape(action:ReadOnly<Function<int>>):()=>int{return action;}function main(){}",
        "callback-escape",
    );
}

#[test]
fn read_callback_forwarding_and_conditional_invocation_preserve_shared_borrows() {
    execute(
        "function one(action:ReadOnly<Function<int>>):int{return action();}function choose(first:ReadOnly<Function<int>>):int{return (true?first:first)();}function nested(action:Mutable<Function<int, [inner:ReadOnly<Function<int>>]>>,inner:ReadOnly<Function<int>>):int{return action(inner);}function main(){const text='Pat';const action=():int=>{return text.length;};console.log(one(action),choose(action),nested((inner:ReadOnly<Function<int>>):int=>{return inner();},action));}",
        "3 3 3\n",
        None,
    );
}
