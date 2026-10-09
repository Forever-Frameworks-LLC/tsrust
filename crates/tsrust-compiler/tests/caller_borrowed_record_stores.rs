mod support;

#[test]
fn record_parameter_store_retains_the_callers_array_borrow() {
    let source = support::format_round_trip(r#"
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
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
