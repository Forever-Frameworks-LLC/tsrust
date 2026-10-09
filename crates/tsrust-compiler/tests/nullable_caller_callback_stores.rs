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
    let source = support::format_round_trip(
        r#"
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
    "#,
    );
    let directory = support::TempDirectory::new();
    support::success(&support::run(
        support::cli().arg("run").arg(directory.source(&source)),
        60,
    ));
}

#[test]
fn nullable_replacement_releases_only_the_overwritten_mutable_input() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void { holder.run = action; }
        function main() {
            const firstValues = [3];
            const secondValues = [10];
            const first = (): int => { firstValues[0] += 1; return firstValues[0]; };
            const second = (): int => { secondValues[0] += 2; return secondValues[0]; };
            const holder: Holder = { run: undefined };
            install(holder, first);
            strictEqual(holder.run!(), 4);
            install(holder, second);
            strictEqual(first(), 5);
            strictEqual(holder.run!(), 12);
            strictEqual(holder.run!(), 14);
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
fn nullable_replacement_keeps_the_new_mutable_input_borrowed() {
    support::reject(
        r#"
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void { holder.run = action; }
        function main() {
            const firstValues = [3]; const secondValues = [10];
            const first = (): int => { firstValues[0] += 1; return firstValues[0]; };
            const second = (): int => { secondValues[0] += 1; return secondValues[0]; };
            const holder: Holder = { run: undefined };
            install(holder, first); install(holder, second);
            console.log(second(), holder.run!());
        }
    "#,
        "capture-conflict",
    );
}

#[test]
fn nullable_branch_replacement_retains_borrowed_and_capture_free_alternatives() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void { holder.run = action; }
        function replace(holder: Mutable<Holder>, action: Mutable<Function<int>>, useAction: boolean): void {
            if (useAction) { holder.run = action; } else { holder.run = (): int => 99; }
        }
        function scenario(useAction: boolean) {
            const firstValues = [3]; const secondValues = [10];
            const first = (): int => { firstValues[0] += 1; return firstValues[0]; };
            const second = (): int => { secondValues[0] += 2; return secondValues[0]; };
            const holder: Holder = { run: undefined };
            install(holder, first); replace(holder, second, useAction);
            strictEqual(first(), 4); strictEqual(holder.run!(), useAction ? 12 : 99);
        }
        function main() { scenario(false); scenario(true); }
    "#,
    );
    let directory = support::TempDirectory::new();
    support::success(&support::run(
        support::cli().arg("run").arg(directory.source(&source)),
        60,
    ));
}

#[test]
fn nested_nullable_replacement_relays_preserve_input_aliases() {
    let source = support::format_round_trip(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>> | undefined; };
        type Box = { slot: Holder; };
        function install(box: Mutable<Box>, action: Mutable<Function<int>>): void {
            const alias = action;
            box.slot.run = alias;
        }
        function relay(box: Mutable<Box>, action: Mutable<Function<int>>): void { install(box, action); }
        function main() {
            const firstValues = [3]; const secondValues = [10];
            const first = (): int => { firstValues[0] += 1; return firstValues[0]; };
            const second = (): int => { secondValues[0] += 2; return secondValues[0]; };
            const box: Box = { slot: { run: undefined } };
            relay(box, first); strictEqual(box.slot.run!(), 4);
            relay(box, second); strictEqual(first(), 5);
            strictEqual(box.slot.run!(), 12);
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
fn conditional_nullable_replacement_keeps_the_old_input_on_the_skipped_path() {
    support::reject(
        r#"
        type Holder = { run: Mutable<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void { holder.run = action; }
        function maybeReplace(holder: Mutable<Holder>, action: Mutable<Function<int>>, replace: boolean): void {
            if (replace) { holder.run = action; }
        }
        function main() {
            const firstValues = [3]; const secondValues = [10];
            const first = (): int => { firstValues[0] += 1; return firstValues[0]; };
            const second = (): int => { secondValues[0] += 1; return secondValues[0]; };
            const holder: Holder = { run: undefined };
            install(holder, first); maybeReplace(holder, second, false);
            console.log(first(), holder.run!());
        }
    "#,
        "capture-conflict",
    );
}

#[test]
fn readonly_nullable_replacement_preserves_distinct_callback_environments() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>> | undefined; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void { holder.run = action; }
        function main() {
            const firstValues = [3]; const secondValues = [10];
            const first = (): int => firstValues[0];
            const second = (): int => secondValues[0];
            const holder: Holder = { run: undefined };
            install(holder, first); strictEqual(holder.run!(), 3);
            install(holder, second); strictEqual(first(), 3); strictEqual(holder.run!(), 10);
        }
    "#);
    let directory = support::TempDirectory::new();
    support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)), 60));
}

#[test]
fn nonnullable_callback_replacement_also_releases_the_overwritten_input() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: Mutable<Function<int>>; };
        function install(holder: Mutable<Holder>, action: Mutable<Function<int>>): void { holder.run = action; }
        function main() {
            const firstValues = [3]; const secondValues = [10];
            const first = (): int => { firstValues[0] += 1; return firstValues[0]; };
            const second = (): int => { secondValues[0] += 2; return secondValues[0]; };
            const holder: Holder = { run: (): int => 0 };
            install(holder, first); install(holder, second);
            strictEqual(first(), 4); strictEqual(holder.run(), 12);
        }
    "#);
    let directory = support::TempDirectory::new();
    support::success(&support::run(support::cli().arg("run").arg(directory.source(&source)), 60));
}
