mod support;

fn project(source: &str) {
    let source = support::format_round_trip(source);
    let directory = support::TempDirectory::new();
    let path = directory.source(&source);
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn named_scoped_readonly_field_keeps_original_owner_and_repeated_invocation() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        function main() {
            const offsets = [3, 7];
            {
                const selected = offsets.length === 2
                    ? (value: int): int => value + offsets[0]
                    : (value: int): int => value + offsets[1];
                const holder: Holder = { run: selected };
                strictEqual(holder.run(2), 5);
                strictEqual(holder.run(4), 7);
            }
            offsets[0] = 9;
            strictEqual(offsets[0], 9);
        }
    "#);
}

#[test]
fn named_scoped_mutable_field_updates_original_capture_and_releases_loan() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Counter = { run: Mutable<Function<int, [value: int]>>; };
        function main() {
            let left = 0;
            let right = 0;
            {
                const selected = left === 0
                    ? (value: int): int => { left += value; return left; }
                    : (value: int): int => { right += value; return right; };
                const holder: Counter = { run: selected };
                strictEqual(holder.run(2), 2);
                strictEqual(holder.run(4), 6);
            }
            strictEqual(left, 6);
            strictEqual(right, 0);
            left += 1;
            strictEqual(left, 7);
        }
    "#);
}
