mod support;

fn execute(source: &str) {
    let source = support::format_round_trip(source);
    let directory = support::TempDirectory::new();
    support::success(&support::run(
        support::cli().arg("run").arg(directory.source(&source)),
        60,
    ));
}

#[test]
fn independent_factory_instances_release_the_overwritten_callback() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function counter(start: int): () => int {
            let value = start;
            return (): int => { value += 1; return value; };
        }
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void { holder.run = action; }
        function main() {
            const first = counter(3); const second = counter(10);
            const holder: Holder = { run: undefined };
            install(holder, first); strictEqual(holder.run!(), 4);
            install(holder, second); strictEqual(first(), 5);
            strictEqual(holder.run!(), 11);
        }
    "#;
    execute(source);
    let nonnullable = source
        .replace(" | undefined", "")
        .replace("run: undefined", "run: (): int => 0")
        .replace("run!()", "run()");
    execute(&nonnullable);
}

#[test]
fn direct_array_replacement_releases_the_overwritten_owner() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: Mutable<int[]>): void {
            holder.run = (): int => { source[0] += 1; return source[0]; };
        }
        function main() {
            const first = [3]; const second = [10];
            const holder: Holder = { run: undefined };
            install(holder, first); strictEqual(holder.run!(), 4);
            install(holder, second); first[0] += 1;
            strictEqual(first[0], 5); strictEqual(holder.run!(), 11);
        }
    "#,
    );
}

#[test]
fn installer_argument_effects_are_evaluated_once_before_capture() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function next(calls: Mutable<int[]>): int { calls[0] += 1; return calls[0]; }
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, offset: int): void {
            holder.run = (): int => source[0] + offset;
        }
        function main() {
            const calls = [0]; const source = [7];
            const holder: Holder = { run: undefined };
            install(holder, source, next(calls));
            strictEqual(calls[0], 1); strictEqual(holder.run!(), 8);
            strictEqual(holder.run!(), 8); strictEqual(calls[0], 1);
        }
    "#,
    );
}

#[test]
fn installer_body_effects_are_not_skipped_by_direct_capture_lowering() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = action;
        }
        function main() {
            const a = [3]; const b = [10]; const calls = [0];
            const first = (): int => { a[0] += 1; return a[0]; };
            const second = (): int => { b[0] += 2; return b[0]; };
            const holder: Holder = { run: undefined };
            install(holder, first, calls); install(holder, second, calls);
            strictEqual(calls[0], 2); strictEqual(first(), 4); strictEqual(holder.run!(), 12);
        }
    "#,
    );
}

#[test]
fn direct_record_replacement_releases_the_overwritten_owner() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Counter = { value: int; };
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: Mutable<Counter>): void {
            holder.run = (): int => { source.value += 1; return source.value; };
        }
        function main() {
            const first: Counter = { value: 3 }; const second: Counter = { value: 10 };
            const holder: Holder = { run: undefined };
            install(holder, first); strictEqual(holder.run!(), 4);
            install(holder, second); first.value += 1;
            strictEqual(first.value, 5); strictEqual(holder.run!(), 11);
        }
    "#,
    );
}

#[test]
fn effectful_installer_releases_an_independent_same_origin_factory() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function counter(start: int): () => int {
            let value = start;
            return (): int => { value += 1; return value; };
        }
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = action;
        }
        function main() {
            const first = counter(3); const second = counter(10); const calls = [0];
            const holder: Holder = { run: undefined };
            install(holder, first, calls); strictEqual(holder.run!(), 4);
            install(holder, second, calls);
            strictEqual(calls[0], 2); strictEqual(first(), 5); strictEqual(holder.run!(), 11);
        }
    "#,
    );
}

#[test]
fn effectful_relay_preserves_independent_factory_owners() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function counter(start: int): () => int {
            let value = start; return (): int => { value += 1; return value; };
        }
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>, calls: Mutable<int[]>): void {
            calls[0] += 1; holder.run = action;
        }
        function relay(holder: Mutable<Holder>, action: Mutable<Function<int>>, calls: Mutable<int[]>): void {
            const alias = action; install(holder, alias, calls);
        }
        function main() {
            const first = counter(3); const second = counter(10); const calls = [0];
            const holder: Holder = { run: undefined };
            relay(holder, first, calls); strictEqual(holder.run!(), 4);
            relay(holder, second, calls); strictEqual(first(), 5); strictEqual(holder.run!(), 11);
            relay(holder, first, calls); strictEqual(second(), 12); strictEqual(holder.run!(), 6);
            strictEqual(calls[0], 3);
            const observe = (): int => holder.run!(); strictEqual(observe(), 7);
        }
    "#,
    );
}

#[test]
fn local_forwarding_environment_keeps_its_source_type_before_storage() {
    execute(
        r#"import { strictEqual } from "std:testing";
type Holder = { run: Mutable<Function<int>> | undefined; };
function counter(start: int): () => int { let value = start; return (): int => { value += 1; return value; }; }
function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
    const wrapped = (): int => action();
    holder.run = wrapped;
}
function main() {
    const first = counter(3); const second = counter(10); const holder: Holder = { run: undefined };
    install(holder, first); install(holder, second); strictEqual(first(), 4); strictEqual(holder.run!(), 11);
}
"#,
    );
}

#[test]
fn effectful_data_installer_releases_equal_type_array_owners() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: Mutable<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => { source[0] += 1; return source[0]; };
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined };
            install(holder, first, calls); strictEqual(holder.run!(), 4);
            install(holder, second, calls); first[0] += 1;
            strictEqual(first[0], 5); strictEqual(holder.run!(), 11); strictEqual(calls[0], 2);
        }
    "#;
    execute(source);
    let readonly = source
        .replace("source: Mutable<int[]>", "source: ReadOnly<int[]>")
        .replace("source[0] += 1; return source[0];", "return source[0];")
        .replace("holder.run!(), 4", "holder.run!(), 3")
        .replace("first[0], 5", "first[0], 4")
        .replace("holder.run!(), 11", "holder.run!(), 10");
    execute(&readonly);
    let relay=source.replace("install(holder, first, calls)","relay(holder, first, calls)")
        .replace("install(holder, second, calls)","relay(holder, second, calls)")
        .replace("strictEqual(calls[0], 2);","strictEqual(calls[0], 6); const observe = (): int => holder.run!(); strictEqual(observe(), 12);");
    execute(
        &(relay
            + "\nfunction relay(holder: Mutable<Holder>, source: Mutable<int[]>, calls: Mutable<int[]>): void { calls[0] += 2; install(holder, source, calls); }"),
    );
    execute(
        &source
            .replace(" | undefined", "")
            .replace("run: undefined", "run: (): int => 0")
            .replace("run!()", "run()"),
    );
}

#[test]
fn effectful_data_installer_releases_equal_type_record_owners() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Counter = { value: int; };
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: Mutable<Counter>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => { source.value += 1; return source.value; };
        }
        function main() {
            const first: Counter = { value: 3 }; const second: Counter = { value: 10 }; const calls = [0];
            const holder: Holder = { run: undefined };
            install(holder, first, calls); strictEqual(holder.run!(), 4);
            install(holder, second, calls); first.value += 1;
            strictEqual(first.value, 5); strictEqual(holder.run!(), 11); strictEqual(calls[0], 2);
        }
    "#,
    );
}

#[test]
fn conditional_data_installer_preserves_selected_behavior_and_releases_old_owner() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, plus: boolean, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = plus ? (): int => source[0] + 1 : (): int => source[0] + 2;
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined };
            install(holder, first, true, calls); strictEqual(holder.run!(), 4);
            install(holder, second, false, calls); first[0] += 1;
            strictEqual(first[0], 4); strictEqual(holder.run!(), 12); strictEqual(calls[0], 2);
        }
    "#,
    );
}

#[test]
fn conditional_installer_cannot_retain_a_local_owner() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, plus: boolean): void {
            const local = [3];
            holder.run = plus ? (): int => local[0] + 1 : (): int => local[0] + 2;
        }
        function main() { const holder: Holder = { run: undefined }; install(holder,true); console.log(holder.run!()); }
    "#;
    let source = support::format_round_trip(source);
    let error = tsrust_compiler::check(&source)
        .err()
        .expect("A conditional callback cannot outlive its local owner");
    assert_eq!(error.code, "callback-escape", "{error:?}");
    let saved = source
        .replace("holder.run = plus ?", "const selected = plus ?")
        .replace("local[0] + 2;", "local[0] + 2; holder.run = selected;");
    assert_ne!(source, saved);
    let saved = support::format_round_trip(&saved);
    let error = tsrust_compiler::check(&saved)
        .err()
        .expect("A saved family cannot outlive its local owner");
    assert_eq!(error.code, "callback-escape", "{error:?}");
}

#[test]
fn conditional_plain_data_installer_releases_old_owner() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, plus: boolean, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = plus ? (): int => source[0] + 1 : (): int => source[0] + 2;
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: (): int => 0 };
            install(holder, first, true, calls); strictEqual(holder.run(), 4);
            install(holder, second, false, calls); first[0] += 1;
            strictEqual(first[0], 4); strictEqual(holder.run(), 12); strictEqual(calls[0], 2);
        }
"#,
    );
}

#[test]
fn conditional_installer_evaluates_predicate_and_only_selected_branch_once() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, plus: boolean, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = choose(plus, calls) ? (calls[0] += 100, (): int => source[0] + 1) : (calls[0] += 200, (): int => source[0] + 2);
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined };
            install(holder, first, true, calls); strictEqual(holder.run!(), 4);
            install(holder, second, false, calls); first[0] += 1;
            strictEqual(first[0], 4); strictEqual(holder.run!(), 12); strictEqual(calls[0], 322);
        }

function choose(plus: boolean, calls: Mutable<int[]>): boolean { calls[0] += 10; return plus; }"#,
    );
}

#[test]
fn conditional_local_family_keeps_its_value_before_record_storage() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, plus: boolean, calls: Mutable<int[]>): void {
            calls[0] += 1;
            const selected = plus ? (): int => source[0] + 1 : (): int => source[0] + 2;
            strictEqual(selected(), source[0] + (plus ? 1 : 2));
            holder.run = selected;
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0]; const holder: Holder = { run: undefined };
            install(holder, first, true, calls); strictEqual(holder.run!(), 4);
            install(holder, second, false, calls); first[0] += 1;
            strictEqual(first[0], 4); strictEqual(holder.run!(), 12); strictEqual(calls[0], 2);
        }
    "#;
    execute(source);
    execute(
        &source
            .replace(" | undefined", "")
            .replace("run: undefined", "run: (): int => 0")
            .replace("run!()", "run()"),
    );
}

#[test]
fn saved_conditional_family_retains_new_owner_and_releases_old_owner() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, plus: boolean): void {
            const selected = plus ? (): int => source[0] + 1 : (): int => source[0] + 2;
            console.log(selected());
            holder.run = selected;
        }
        function main() {
            const first = [3]; const second = [10];
            const holder: Holder = { run: undefined };
            install(holder, first, true);
            install(holder, second, false);
            first[0] += 1;
            console.log(holder.run!());
        }
    "#;
    let source = support::format_round_trip(source);
    tsrust_compiler::check(&source).expect("Replacement releases the old owner");
    let conflict = source.replace("first[0] += 1", "second[0] += 1");
    let error = tsrust_compiler::check(&conflict)
        .err()
        .expect("The saved family retains its new owner while the holder is live");
    assert_eq!(error.code, "capture-conflict", "{error:?}");
}

#[test]
fn saved_mutable_family_moves_selected_payload_after_local_invocation() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: Mutable<int[]>, plus: boolean, calls: Mutable<int[]>): void {
            const selected = plus
                ? (calls[0] += 1, (): int => { source[0] += 1; return source[0]; })
                : (calls[0] += 10, (): int => { source[0] += 2; return source[0]; });
            strictEqual(selected(), plus ? 4 : 12);
            holder.run = selected;
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined };
            install(holder, first, true, calls); strictEqual(holder.run!(), 5);
            install(holder, second, false, calls); first[0] += 1;
            strictEqual(first[0], 6); strictEqual(holder.run!(), 14); strictEqual(calls[0], 11);
        }
    "#;
    execute(source);
    execute(
        &source
            .replace(" | undefined", "")
            .replace("run: undefined", "run: (): int => 0")
            .replace("run!()", "run()"),
    );
}

#[test]
fn saved_callback_family_preserves_generic_input_and_local_move() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>, plus: boolean): void {
            const selected = plus ? (): int => action() + 1 : (): int => action() + 2;
            strictEqual(selected(), plus ? 4 : 12);
            const moved = selected;
            holder.run = moved;
        }
        function main() {
            const first = (): int => 3; const second = (): int => 10;
            const holder: Holder = { run: undefined };
            install(holder, first, true); strictEqual(holder.run!(), 4);
            install(holder, second, false); strictEqual(first(), 3);
            strictEqual(holder.run!(), 12);
        }
    "#,
    );
}

#[test]
fn closure_body_installer_releases_independent_input_owners() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => source[0];
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined };
            const perform = (target: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void => {
                install(target, source, calls);
            };
            perform(holder, first, calls); strictEqual(holder.run!(), 3);
            perform(holder, second, calls); first[0] += 1;
            strictEqual(first[0], 4); strictEqual(holder.run!(), 10); strictEqual(calls[0], 2);
        }
    "#;
    for source in [
        source.to_owned(),
        source.replace(
            "install(target, source, calls);",
            "const input = source; install(target, input, calls);",
        ),
    ] {
        execute(&source);
        execute(
            &source
                .replace(" | undefined", "")
                .replace("run: undefined", "run: (): int => 0")
                .replace("run!()", "run()"),
        );
    }
    let mutable = source
        .replace("ReadOnly<Function<int>>", "Mutable<Function<int>>")
        .replace("source: ReadOnly<int[]>", "source: Mutable<int[]>")
        .replace(
            "holder.run = (): int => source[0];",
            "holder.run = (): int => { source[0] += 1; return source[0]; };",
        )
        .replace("holder.run!(), 3", "holder.run!(), 4")
        .replace("first[0], 4", "first[0], 5")
        .replace("holder.run!(), 10", "holder.run!(), 11");
    execute(&mutable);
    execute(
        &mutable
            .replace(" | undefined", "")
            .replace("run: undefined", "run: (): int => 0")
            .replace("run!()", "run()"),
    );
}

#[test]
fn closure_body_installer_retains_new_owner_in_analysis() {
    let source = support::format_round_trip(
        r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function main() {
            const first = [3]; const second = [10];
            const holder: Holder = { run: undefined };
            const perform = (target: Mutable<Holder>, source: ReadOnly<int[]>): void => {
                install(target, source);
            };
            perform(holder, first);
            perform(holder, second);
            first[0] += 1;
            console.log(holder.run!());
        }
    "#,
    );
    let relay = r#"
        function relay(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            const perform = (target: Mutable<Holder>, input: ReadOnly<int[]>): void => {
                install(target, input);
            };
            perform(holder, source);
        }
    "#;
    for source in [
        source.clone(),
        format!("{source}\n{relay}").replace("perform(holder, second);", "relay(holder, second);"),
        source.replace(
            "install(target, source);",
            "if (source.length > 0) { install(target, source); }",
        ),
    ] {
        let source = support::format_round_trip(&source);
        if !source.contains("if (") {
            tsrust_compiler::check(&source)
                .expect("Complete closure installation releases the previous owner");
        } else {
            let error = tsrust_compiler::check(&source)
                .err()
                .expect("Conditional replacement must retain the old owner");
            assert_eq!(error.code, "capture-conflict", "{error:?}");
        }
        let conflict = source.replace("first[0] += 1", "second[0] += 1");
        assert_ne!(source, conflict);
        let error = tsrust_compiler::check(&conflict)
            .err()
            .expect("Closure installation retains the new owner");
        assert_eq!(error.code, "capture-conflict", "{error:?}");
    }
}

#[test]
fn installer_function_value_releases_independent_input_owners() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => source[0];
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined };
            const perform = install;
            perform(holder, first, calls); strictEqual(holder.run!(), 3);
            perform(holder, second, calls); first[0] += 1;
            strictEqual(first[0], 4); strictEqual(holder.run!(), 10); strictEqual(calls[0], 2);
        }
    "#;
    execute(source);
    execute(
        &source
            .replace(" | undefined", "")
            .replace("run: undefined", "run: (): int => 0")
            .replace("run!()", "run()"),
    );
}

#[test]
fn installer_function_value_retains_new_owner_and_releases_old_owner_in_analysis() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => source[0];
        }
        function relay(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            const perform = install;
            perform(holder, source, calls);
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined };
            const perform = install;
            perform(holder, first, calls);
            relay(holder, second, calls);
            first[0] += 1;
            console.log(holder.run!());
        }
    "#;
    let source = support::format_round_trip(source);
    for source in [
        source.clone(),
        source.replace(
            "relay(holder, second, calls);",
            "perform(holder, second, calls);",
        ),
    ] {
        tsrust_compiler::check(&source)
            .expect("A complete forwarded replacement releases the old owner");
        let conflict = source.replace("first[0] += 1", "second[0] += 1");
        assert_ne!(source, conflict);
        let error = tsrust_compiler::check(&conflict)
            .err()
            .expect("Forwarded installation retains the new owner");
        assert_eq!(error.code, "capture-conflict", "{error:?}");
    }
}

#[test]
fn installer_function_value_carries_replacement_bounds_for_one_input() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => source[0];
        }
        function main() {
            const source = [7]; const calls = [0]; const holder: Holder = { run: undefined };
            const perform = install;
            perform(holder, source, calls);
            strictEqual(holder.run!(), 7); strictEqual(calls[0], 1);
        }
    "#,
    );
}

#[test]
fn forwarded_mutable_installer_preserves_invocation_and_owner_release() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: Mutable<int[]>): void {
            holder.run = (): int => { source[0] += 1; return source[0]; };
        }
        function main() {
            const first = [3]; const second = [10]; const holder: Holder = { run: undefined };
            const perform = install;
            perform(holder, first); strictEqual(holder.run!(), 4);
            perform(holder, second); first[0] += 1;
            strictEqual(first[0], 5); strictEqual(holder.run!(), 11);
        }
    "#;
    execute(source);
    execute(
        &source
            .replace(" | undefined", "")
            .replace("run: undefined", "run: (): int => 0")
            .replace("run!()", "run()"),
    );
}

#[test]
fn forwarded_installer_preserves_typed_failure_after_record_storage() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1; holder.run = (): int => source[0]; throw "installed";
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0];
            const holder: Holder = { run: undefined }; const perform = install;
            let caught = 0;
            try { perform(holder, first, calls); } catch(error) { strictEqual(error, "installed"); caught += 1; }
            strictEqual(holder.run!(), 3);
            try { perform(holder, second, calls); } catch(error) { strictEqual(error, "installed"); caught += 1; }
            strictEqual(holder.run!(), 10); strictEqual(caught, 2); strictEqual(calls[0], 2);
        }
    "#,
    );
}

#[test]
fn closure_body_installer_preserves_typed_failure_after_storage() {
    execute(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => source[0];
            throw "installed";
        }
        function main() {
            const source = [7]; const calls = [0]; const holder: Holder = { run: undefined };
            const perform = (target: Mutable<Holder>, input: ReadOnly<int[]>, effects: Mutable<int[]>): void => {
                install(target, input, effects);
            };
            let caught = false;
            try { perform(holder, source, calls); } catch (error) { strictEqual(error, "installed"); caught = true; }
            strictEqual(caught, true); strictEqual(holder.run!(), 7); strictEqual(calls[0], 1);
        }
    "#,
    );
}

#[test]
fn captured_installer_effect_state_preserves_input_owner_replacement() {
    execute(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>, calls: Mutable<int[]>): void {
            calls[0] += 1;
            holder.run = (): int => source[0];
        }
        function main() {
            const first = [3]; const second = [10]; const calls = [0]; const offset = 2;
            const holder: Holder = { run: undefined };
            {
                const perform = (target: Mutable<Holder>, source: ReadOnly<int[]>): void => {
                    install(target, source, calls);
                    calls[0] += offset;
                };
                perform(holder, first); strictEqual(holder.run!(), 3);
                perform(holder, second); first[0] += 1;
                strictEqual(first[0], 4); strictEqual(holder.run!(), 10);
            }
            strictEqual(calls[0], 6);
        }
    "#);
}

#[test]
fn captured_installer_retains_stored_owner_after_installer_disposal() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function clear(holder: Mutable<Holder>): void { holder.run = undefined; }
        function main() {
            const source = [3]; const holder: Holder = { run: undefined };
            {
                const perform = (target: Mutable<Holder>): void => { install(target, source); };
                perform(holder);
            }
            console.log(holder.run!());
        }
    "#);
    tsrust_compiler::check(&source).expect("A stored capture outlives the installer that created it");
    let conflict = source.replace("console.log(holder.run!());", "source[0] += 1; console.log(holder.run!());");
    assert_ne!(source,conflict);
    let error = tsrust_compiler::check(&conflict).err().expect("The holder still borrows the captured source after installer disposal");
    assert_eq!(error.code,"capture-conflict","{error:?}");
    let released = source.replace("console.log(holder.run!());", "clear(holder); source[0] += 1;");
    assert_ne!(source,released);
    tsrust_compiler::check(&released).expect("Clearing the stored callback releases the captured input loan");
}

#[test]
fn captured_source_installer_keeps_native_input_alive_after_disposal() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function clear(holder: Mutable<Holder>): void { holder.run = undefined; }
        function main() {
            const source = [3]; const holder: Holder = { run: undefined };
            {
                const perform = (target: Mutable<Holder>): void => { install(target, source); };
                perform(holder);
            }
            strictEqual(holder.run!(), 3);
            clear(holder); source[0] += 1; strictEqual(source[0], 4);
        }
    "#;
    execute(source);
    let plain = source.replace(" | undefined", "").replace("run: undefined", "run: (): int => 0").replace("run!()", "run()").replace("            clear(holder); source[0] += 1; strictEqual(source[0], 4);", "").replace("        function clear(holder: Mutable<Holder>): void { holder.run = undefined; }", "");
    execute(&plain);
    let pair = source.replace("source: ReadOnly<int[]>", "source: ReadOnly<int[]>, other: ReadOnly<int[]>")
        .replace("(): int => source[0]", "(): int => source[0] + other[0]")
        .replace("const source = [3];", "const source = [3]; const other = [11];")
        .replace("install(target, source);", "install(target, source, other);")
        .replace("strictEqual(holder.run!(), 3);", "strictEqual(holder.run!(), 14);");
    execute(&pair);
    let alias = source.replace("const perform = (target", "const alias = source; const perform = (target").replace("install(target, source);", "install(target, alias);");
    let alias = alias.replace("import { strictEqual } from \"std:testing\";", "").replace("strictEqual(holder.run!(), 3);", "console.log(holder.run!());").replace("strictEqual(source[0], 4);", "console.log(source[0]);");
    let error = tsrust_compiler::check(&alias).err().expect("A plain local alias transfers ownership into the shorter block");
    assert_eq!(error.code, "callback-escape", "{error:?}");
}

#[test]
fn independent_captured_installers_release_the_previous_source_owner() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function main() {
            const first = [3]; const second = [10];
            const holder: Holder = { run: undefined };
            {
                const perform = (target: Mutable<Holder>): void => { install(target, first); };
                perform(holder);
            }
            strictEqual(holder.run!(), 3);
            {
                const perform = (target: Mutable<Holder>): void => { install(target, second); };
                perform(holder);
            }
            first[0] += 1;
            strictEqual(first[0], 4); strictEqual(holder.run!(), 10);
        }
    "#;
    execute(source);
    let plain = source.replace(" | undefined", "").replace("run: undefined", "run: (): int => 0").replace("run!()", "run()");
    execute(&plain);
    let checkable=source.replace("import { strictEqual } from \"std:testing\";", "").replace("strictEqual(holder.run!(), 3);", "console.log(holder.run!());").replace("strictEqual(first[0], 4); strictEqual(holder.run!(), 10);", "console.log(first[0], holder.run!());");
    let conflict=checkable.replace("first[0] += 1;", "second[0] += 1;");
    let error=tsrust_compiler::check(&conflict).err().expect("The replacement captured source remains borrowed");
    assert_eq!(error.code,"capture-conflict","{error:?}");
}
