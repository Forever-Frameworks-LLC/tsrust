mod support;

#[test]
fn nullable_store_cannot_escape_a_callee_local_capture() {
    support::reject(
        r#"
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function invalid(holder: Mutable<Holder>): void {
            const local = [3];
            holder.run = (): int => local[0];
        }
        function main() {}
    "#,
        "callback-escape",
    );
}

#[test]
fn conditional_clear_does_not_release_the_uncleared_path() {
    support::reject(
        r#"
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = action;
        }
        function maybeClear(holder: Mutable<Holder>, clear: boolean): void {
            if (clear) { holder.run = undefined; }
        }
        function main() {
            const values = [3];
            const action = (): int => { values[0] += 1; return values[0]; };
            const holder: Holder = { run: undefined };
            install(holder, action);
            maybeClear(holder, false);
            console.log(action(), holder.run!());
        }
    "#,
        "capture-conflict",
    );
}

#[test]
fn nullable_callback_installs_invokes_and_clears() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = action;
        }
        function clear(holder: Mutable<Holder>): void { holder.run = undefined; }
        function main() {
            const values = [3];
            const action = (): int => values[0];
            const holder: Holder = { run: undefined };
            strictEqual(holder.run === undefined, true);
            install(holder, action);
            strictEqual(holder.run!(), 3);
            clear(holder);
            strictEqual(holder.run === undefined, true);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    support::success(&support::run(
        support::cli().arg("run").arg(directory.source(&source)),
        60,
    ));
}

#[test]
fn clearing_nullable_mutable_callback_releases_the_retained_input() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = action;
        }
        function clear(holder: Mutable<Holder>): void { holder.run = undefined; }
        function main() {
            const values = [3];
            const action = (): int => { values[0] += 1; return values[0]; };
            const holder: Holder = { run: undefined };
            install(holder, action);
            strictEqual(holder.run!(), 4);
            clear(holder);
            strictEqual(holder.run === undefined, true);
            strictEqual(action(), 5);
            strictEqual(holder.run === undefined, true);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    support::success(&support::run(
        support::cli().arg("run").arg(directory.source(&source)),
        60,
    ));
}

#[test]
fn nullable_clear_relays_and_normal_exits_release_the_input() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void {
            holder.run = action;
        }
        function clear(holder: Mutable<Holder>): void { holder.run = undefined; }
        function relay(holder: Mutable<Holder>, early: boolean): void {
            if (early) { clear(holder); return; }
            clear(holder);
        }
        function main() {
            const values = [3];
            const action = (): int => { values[0] += 1; return values[0]; };
            const holder: Holder = { run: undefined };
            install(holder, action);
            relay(holder, true);
            strictEqual(action(), 4);
            strictEqual(holder.run === undefined, true);
            install(holder, action);
            relay(holder, false);
            strictEqual(action(), 5);
            strictEqual(holder.run === undefined, true);
        }
    "#,
    );
    let directory = support::TempDirectory::new();
    support::success(&support::run(
        support::cli().arg("run").arg(directory.source(&source)),
        60,
    ));
}

#[test]
fn nullable_stored_callback_preserves_typed_failure_contract() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void { holder.run = action; }
        function main() {
            const fail = true;
            const action = (): int => { if (fail) { throw "failed"; } return 0; };
            const holder: Holder = { run: undefined };
            install(holder, action);
            let caught = false;
            try { console.log(holder.run!()); } catch (error) { caught = error === "failed"; }
            strictEqual(caught, true);
        }
    "#);
    let directory = support::TempDirectory::new();
    support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)), 60));
}
