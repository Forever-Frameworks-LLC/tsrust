mod support;

#[test]
fn absent_concrete_callback_origin_does_not_invent_a_catch_type() {
    support::reject(r#"
        function guarded(action: ReadOnly<Function<int>>): int {
            try { return action(); } catch (error) { return error.length; }
        }
        function main() {}
    "#, "error-effect");
}

#[test]
fn captured_holder_handler_retains_outer_installation_error_origins() {
    native(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = action;
        }
        function main() {
            const values = [3];
            const action = (): int => { throw values[0]; };
            const holder: Holder = { run: (): int => 0 };
            install(holder, action);
            const guarded = (): int => {
                try { return holder.run(); } catch (error) {
                    return typeof error === "number" ? error : 0;
                }
            };
            strictEqual(guarded(), 3);
        }
    "#,
    );
}

#[test]
fn conditional_installation_retains_both_typed_failure_origins() {
    native(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = action;
        }
        function choose(flag: boolean): int {
            const values = [3];
            const number = (): int => { throw values[0]; };
            const text = (): int => { throw "four"; };
            const holder: Holder = { run: (): int => 0 };
            if (flag) { install(holder, number); } else { install(holder, text); }
            try { return holder.run(); } catch (error) {
                return typeof error === "number" ? error : typeof error === "string" ? error.length : 0;
            }
        }
        function main() {
            strictEqual(choose(true), 3);
            strictEqual(choose(false), 4);
        }
    "#,
    );
}

#[test]
fn directly_stored_callback_preserves_typed_failure_and_success() {
    native(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [fail: boolean]>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int, [fail: boolean]>>): void {
            holder.run = action;
        }
        function main() {
            const values = [3];
            const action = (fail: boolean): int => {
                if (fail) { throw "callback"; }
                return values[0];
            };
            const holder: Holder = { run: (fail: boolean): int => 0 };
            install(holder, action);
            let caught = false;
            try { holder.run(true); } catch (error) {
                caught = typeof error === "string" && error === "callback";
            }
            strictEqual(caught, true);
            strictEqual(holder.run(false), 3);
        }
    "#,
    );
}

#[test]
fn installed_once_callback_cannot_be_called_twice() {
    support::reject(
        r#"
        type Holder = { run: Once<Function<string>>; };
        function install(holder: Mutable<Holder>, action: Once<Function<string>>): void {
            holder.run = action;
        }
        function main() {
            const text = "Pat";
            const action = (): string => text;
            const holder: Holder = { run: (): string => "initial" };
            install(holder, action);
            console.log(holder.run());
            console.log(holder.run());
        }
    "#,
        "partial-transfer",
    );
}

#[test]
fn failure_after_mutable_store_preserves_state_and_releases_on_disposal() {
    native(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = action;
            throw "installed";
        }
        function main() {
            const values = [3];
            const action = (): int => { values[0] += 1; return values[0]; };
            {
                const holder: Holder = { run: (): int => 0 };
                let caught = false;
                try { install(holder, action); } catch (error) { caught = error === "installed"; }
                strictEqual(caught, true);
                strictEqual(holder.run(), 4);
                strictEqual(holder.run(), 5);
            }
            strictEqual(action(), 6);
        }
    "#,
    );
}

#[test]
fn caught_installation_failure_does_not_release_retained_mutable_input() {
    support::reject(
        r#"
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = action;
            throw "installed";
        }
        function main() {
            const values = [3];
            const action = (): int => { values[0] += 1; return values[0]; };
            const holder: Holder = { run: (): int => 0 };
            try { install(holder, action); } catch (error) {}
            console.log(action(), holder.run());
        }
    "#,
        "capture-conflict",
    );
}

fn native(source: &str) {
    let source = support::format_round_trip(source);
    let directory = support::TempDirectory::new();
    support::success(&support::run(
        support::cli().arg("run").arg(directory.source(&source)),
        60,
    ));
}

#[test]
fn direct_once_store_preserves_single_consuming_invocation() {
    native(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Once<Function<string>>; };
        function install(holder: Mutable<Holder>, action: Once<Function<string>>): void {
            holder.run = action;
        }
        function main() {
            const text = "Pat";
            const action = (): string => text;
            const holder: Holder = { run: (): string => "initial" };
            install(holder, action);
            strictEqual(holder.run(), "Pat");
        }
    "#,
    );
}

#[test]
fn failure_after_direct_store_keeps_installed_callback() {
    native(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = action;
            throw "installed";
        }
        function main() {
            const values = [3];
            const action = (): int => values[0];
            const holder: Holder = { run: (): int => 0 };
            let caught = false;
            try { install(holder, action); } catch (error) { caught = error === "installed"; }
            strictEqual(caught, true);
            strictEqual(holder.run(), 3);
        }
    "#,
    );
}

#[test]
fn failure_before_direct_store_preserves_previous_callback() {
    native(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>, fail: boolean): void {
            if (fail) { throw "before"; }
            holder.run = action;
        }
        function main() {
            const values = [3];
            const action = (): int => values[0];
            const holder: Holder = { run: (): int => 9 };
            let caught = false;
            try { install(holder, action, true); } catch (error) { caught = error === "before"; }
            strictEqual(caught, true);
            strictEqual(holder.run(), 9);
        }
    "#,
    );
}
