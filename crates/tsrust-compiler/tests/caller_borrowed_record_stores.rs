mod support;

#[test]
fn repeated_origin_in_one_relay_keeps_both_inputs() {
    let source = support::format_round_trip(r#"
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
    "#);
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
