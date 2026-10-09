mod support;

#[test]
fn callback_input_alias_retains_original_owner() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            const alias = action;
            holder.run = (): int => alias();
        }
        function main() {
            const values = [3];
            const action = (): int => values[0];
            const holder: Holder = { run: (): int => 0 };
            install(holder, action);
            strictEqual(holder.run(), 3);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn owned_escape_cannot_consume_a_borrowed_callback_input() {
    support::reject(
        r#"
        function escape(action: ReadOnly<Function<int>>): Function<int> {
            return (): int => action();
        }
        function main() {}
    "#,
        "callback-escape",
    );
}

#[test]
fn retained_mutable_callback_prevents_independent_invocation() {
    support::reject(
        r#"
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = (): int => action();
        }
        function main() {
            const values = [3];
            const action = (): int => { values[0] += 1; return values[0]; };
            const holder: Holder = { run: (): int => 0 };
            install(holder, action);
            console.log(action(), holder.run());
        }
    "#,
        "capture-conflict",
    );
}

#[test]
fn mutable_callback_input_keeps_state_after_install() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = (): int => action();
        }
        function main() {
            const values = [3];
            const action = (): int => { values[0] += 1; return values[0]; };
            {
                const holder: Holder = { run: (): int => 0 };
                install(holder, action);
                strictEqual(holder.run(), 4);
                strictEqual(holder.run(), 5);
            }
            strictEqual(action(), 6);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn identical_callback_contracts_keep_distinct_native_capture_types() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, left: ReadOnly<Function<int>>, right: ReadOnly<Function<int>>): void {
            holder.run = (): int => left() + right();
        }
        function main() {
            const values = [3];
            const left = (): int => values[0];
            const right = (): int => 8;
            const holder: Holder = { run: (): int => 0 };
            install(holder, left, right);
            strictEqual(holder.run(), 11);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn input_alias_crosses_a_named_relay() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function relay(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            const first = source;
            const alias = first;
            install(holder, alias);
        }
        function main() {
            const values = [3];
            const holder: Holder = { run: (): int => 0 };
            relay(holder, values);
            strictEqual(holder.run(), 3);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn stored_wrapper_borrows_a_callback_input() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = (): int => action();
        }
        function main() {
            const values = [3];
            const action = (): int => values[0];
            const holder: Holder = { run: (): int => 0 };
            install(holder, action);
            strictEqual(holder.run(), 3);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn readonly_input_alias_keeps_caller_provenance() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            const alias = source;
            holder.run = (): int => alias[0];
        }
        function main() {
            const values = [3];
            {
                const holder: Holder = { run: (): int => 0 };
                install(holder, values);
                strictEqual(holder.run(), 3);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn repeated_store_to_one_destination_accepts_distinct_inputs() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function replace(holder: Mutable<Holder>, left: ReadOnly<int[]>, right: ReadOnly<int[]>): void {
            install(holder, left);
            strictEqual(holder.run(), left[0]);
            install(holder, right);
        }
        function main() {
            const left = [3];
            const right = [8];
            const holder: Holder = { run: (): int => 0 };
            replace(holder, left, right);
            strictEqual(holder.run(), 8);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn nested_destination_retains_multiple_input_captures() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handler: Holder; };
        function install(box: Mutable<Box>, left: ReadOnly<int[]>, right: ReadOnly<int[]>): void {
            box.handler.run = (): int => left[0] + right[0];
        }
        function main() {
            const left = [3];
            const right = [8];
            {
                const box: Box = { handler: { run: (): int => 0 } };
                install(box, left, right);
                strictEqual(box.handler.run(), 11);
            }
            left[0] = 4;
            right[0] = 9;
            strictEqual(left[0] + right[0], 13);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn repeated_origin_in_one_relay_keeps_both_inputs() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function both(first: Mutable<Holder>, second: Mutable<Holder>, left: ReadOnly<int[]>, right: ReadOnly<int[]>): void {
            install(first, left);
            install(second, right);
        }
        function main() {
            const left = [3];
            const right = [8];
            const first: Holder = { run: (): int => 0 };
            const second: Holder = { run: (): int => 0 };
            both(first, second, left, right);
            strictEqual(first.run(), 3);
            strictEqual(second.run(), 8);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn mutable_input_store_updates_the_caller_owner() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, source: Mutable<int[]>): void {
            holder.run = (): int => { source[0] += 1; return source[0]; };
        }
        function main() {
            const values = [3];
            {
                const holder: Holder = { run: (): int => 0 };
                install(holder, values);
                strictEqual(holder.run(), 4);
                strictEqual(holder.run(), 5);
            }
            strictEqual(values[0], 5);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn shorter_lived_input_cannot_be_stored_in_outer_holder() {
    support::reject(
        r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function main() {
            const holder: Holder = { run: (): int => 0 };
            { const values = [3]; install(holder, values); }
            console.log(holder.run());
        }
    "#,
        "callback-escape",
    );
}

#[test]
fn retained_input_borrows_reject_mutation_and_transfer() {
    for operation in [
        "values[0] = 9;",
        "const moved = values; console.log(moved[0]);",
    ] {
        let source = format!(
            r#"
            type Holder = {{ run: ReadOnly<Function<int>>; }};
            function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {{
                holder.run = (): int => source[0];
            }}
            function main() {{
                const values = [3];
                const holder: Holder = {{ run: (): int => 0 }};
                install(holder, values);
                {operation}
                console.log(holder.run());
            }}
        "#
        );
        support::reject(&support::format_round_trip(&source), "capture-conflict");
    }
}

#[test]
fn callee_local_borrows_still_cannot_escape() {
    support::reject(
        r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>): void {
            const local = [3];
            holder.run = (): int => local[0];
        }
        function main() {
            const holder: Holder = { run: (): int => 0 };
            install(holder);
            console.log(holder.run());
        }
    "#,
        "callback-escape",
    );
}

#[test]
fn relayed_stores_keep_independent_input_lifetimes() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function relay(source: ReadOnly<int[]>, holder: Mutable<Holder>): void {
            install(holder, source);
        }
        function main() {
            const left = [3];
            const right = [8];
            {
                const first: Holder = { run: (): int => 0 };
                const second: Holder = { run: (): int => 0 };
                relay(left, first);
                relay(right, second);
                strictEqual(first.run(), 3);
                strictEqual(second.run(), 8);
            }
            left[0] = 4;
            right[0] = 9;
            strictEqual(left[0] + right[0], 13);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn record_parameter_store_retains_the_callers_array_borrow() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, source: ReadOnly<int[]>): void {
            holder.run = (): int => source[0];
        }
        function main() {
            const values = [3];
            {
                const holder: Holder = { run: (): int => 0 };
                install(holder, values);
                strictEqual(holder.run(), 3);
                strictEqual(holder.run(), 3);
            }
            values[0] = 7;
            strictEqual(values[0], 7);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
