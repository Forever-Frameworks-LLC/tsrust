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
