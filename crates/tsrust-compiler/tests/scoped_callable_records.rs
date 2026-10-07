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
    project(
        r#"
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
    "#,
    );
}

#[test]
fn named_scoped_mutable_field_updates_original_capture_and_releases_loan() {
    project(
        r#"
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
    "#,
    );
}

#[test]
fn scoped_record_instances_keep_distinct_environments_through_record_and_field_moves() {
    project(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        function main() {
            const firstOffsets = [3, 7];
            const secondOffsets = [11, 13];
            const first = firstOffsets.length === 2
                ? (value: int): int => value + firstOffsets[0]
                : (value: int): int => value + firstOffsets[1];
            const second = secondOffsets.length === 2
                ? (value: int): int => value + secondOffsets[0]
                : (value: int): int => value + secondOffsets[1];
            const firstHolder: Holder = { run: first };
            const secondHolder: Holder = { run: second };
            strictEqual(firstHolder.run(2), 5);
            strictEqual(secondHolder.run(2), 13);
            const moved = firstHolder;
            strictEqual(moved.run(4), 7);
            const run = moved.run;
            strictEqual(run(5), 8);
            strictEqual(secondHolder.run(4), 15);
        }
    "#,
    );
}

#[test]
fn scoped_record_fields_reject_escape_and_conflicting_owner_access() {
    for (source, code) in [
        (
            r#"
            type Holder = { run: ReadOnly<Function<int>>; };
            function make(): Holder {
                const values = [3, 7];
                const run = values.length === 2 ? (): int => values[0] : (): int => values[1];
                return { run: run };
            }
            function main() {}
        "#,
            "callback-escape",
        ),
        (
            r#"
            type Holder = { run: ReadOnly<Function<int>>; };
            function main() {
                const values = [3, 7];
                const run = values.length === 2 ? (): int => values[0] : (): int => values[1];
                const holder: Holder = { run: run };
                values[0] = 9;
                console.log(holder.run());
            }
        "#,
            "capture-conflict",
        ),
        (
            r#"
            type Holder = { run: ReadOnly<Function<int>>; };
            function main() {
                const values = [3, 7];
                const run = values.length === 2 ? (): int => values[0] : (): int => values[1];
                const holder: Holder = { run: run };
                throw holder;
            }
        "#,
            "closure-escape",
        ),
    ] {
        let error = match tsrust_compiler::check(source) {
            Ok(_) => panic!("accepted scoped escape/conflict"),
            Err(error) => error,
        };
        assert_eq!(error.code, code, "{error:?}");
    }
}

#[test]
fn nullable_scoped_record_field_preserves_optional_invocation() {
    project(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>> | null; };
        function main() {
            const offsets = [3, 7];
            const selected = offsets.length === 2
                ? (value: int): int => value + offsets[0]
                : (value: int): int => value + offsets[1];
            const optional = offsets.length === 2 ? selected : null;
            const holder: Holder = { run: optional };
            strictEqual(holder.run?.(2) ?? -1, 5);
        }
    "#,
    );
}

#[test]
fn direct_scoped_record_arrow_keeps_concrete_environment_and_contextual_parameter() {
    project(
        r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        function main() {
            const offsets = [3, 7];
            const holder: Holder = { run: (value): int => value + offsets[0] };
            strictEqual(holder.run(2), 5);
            strictEqual(holder.run(4), 7);
        }
    "#,
    );
}
