//! B7 witness: one generic stored origin must accept distinct input environments.
mod support;

#[test]
fn replacing_a_borrowed_callback_accepts_a_different_environment_type() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function install(holder: Mutable<Holder>, action: ReadOnly<Function<int>>): void {
            holder.run = (): int => action();
        }
        function main() {
            const values = [3];
            const first = (): int => values[0];
            const second = (): int => 9;
            const holder: Holder = { run: (): int => 0 };
            install(holder, first);
            strictEqual(holder.run(), 3);
            install(holder, second);
            strictEqual(holder.run(), 9);
        }
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
}
