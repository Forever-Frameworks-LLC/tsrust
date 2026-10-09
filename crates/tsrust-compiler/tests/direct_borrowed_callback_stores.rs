mod support;

#[test]
fn direct_callback_store_retains_caller_environment() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = action;
        }
        function main() {
            const values = [3];
            {
                const action = (): int => values[0];
                const holder: Holder = { run: (): int => 0 };
                install(holder, action);
                strictEqual(holder.run(), 3);
            }
            values[0] = 8;
            strictEqual(values[0], 8);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}

#[test]
fn direct_mutable_callback_store_preserves_state_and_releases_input() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = action;
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
fn direct_alias_store_crosses_named_relay() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            const alias = action;
            holder.run = alias;
        }
        function relay(action: ReadOnly<Function<int>>, holder: Mutable<Holder>): void {
            install(holder, action);
        }
        function main() {
            const values = [7];
            const action = (): int => values[0];
            const holder: Holder = { run: (): int => 0 };
            relay(action, holder);
            strictEqual(holder.run(), 7);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}
