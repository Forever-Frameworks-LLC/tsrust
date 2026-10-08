mod support;

#[test]
fn removed_array_preserves_dispatch_after_source_widens() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            const removed = holders.splice(0, 1);
            holders.push({ run: (): int => values[1] });
            strictEqual(removed[0].run(), 3);
            strictEqual(holders[0].run(), 7);
        }
    "#);
}

#[test]
fn branching_insertions_share_one_array_environment_layout() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function exercise(flag: boolean) {
            const values = [3, 7, 11];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            if (flag) {
                holders.push({ run: (): int => values[1] });
            } else {
                holders.push({ run: (): int => values[2] });
            }
            strictEqual(holders[0].run(), 3);
            strictEqual(holders[1].run(), flag ? 7 : 11);
        }
        function main() { exercise(true); exercise(false); }
    "#);
}

#[test]
fn moved_array_can_widen_to_another_callback_origin() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const initial: Holder[] = [{ run: (): int => values[0] }];
            const moved = initial;
            moved.push({ run: (): int => values[1] });
            strictEqual(moved[0].run(), 3);
            strictEqual(moved[1].run(), 7);
        }
    "#);
}

#[test]
fn removed_array_moves_into_outer_storage_with_its_environment() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3];
            let removed: Holder[];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                removed = holders.splice(0, 1);
            }
            strictEqual(removed[0].run(), 3);
        }
    "#);
}

#[test]
fn removed_array_retains_capture_conflict_after_source_container_ends() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];let removed:Holder[];{const holders:Holder[]=[{run:():int=>values[0]}];removed=holders.splice(0,1);}values[0]=9;console.log(removed[0].run());}";
    let error = tsrust_compiler::check(source).err().expect("removed payload must retain its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn splice_keeps_removed_and_inserted_record_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                const removed = holders.splice(0, 1, { run: (): int => values[1] });
                strictEqual(removed[0].run(), 3);
                strictEqual(holders[0].run(), 7);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn unshift_and_indexed_replacement_keep_distinct_record_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                holders.unshift({ run: (): int => values[1] });
                strictEqual(holders[0].run(), 7);
                strictEqual(holders[1].run(), 3);
                holders[1] = { run: (): int => values[2] };
                strictEqual(holders[0].run(), 7);
                strictEqual(holders[1].run(), 11);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

fn project(source: &str) {
    let directory = support::TempDirectory::new();
    let source = support::format_round_trip(source);
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn returned_array_cannot_hide_local_capture_borrows() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function bad():Holder[]{const values=[3];return [{run:():int=>values[0]}];}function main(){}";
    let error = tsrust_compiler::check(source).err().expect("aggregate escape must be rejected");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn mutable_record_elements_update_original_capture_and_release_at_scope_exit() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Counter = { run: Mutable<Function<int>>; };
        function main() {
            let count = 0;
            {
                const counters: Counter[] = [{ run: (): int => { count += 1; return count; } }];
                strictEqual(counters[0].run(), 1);
                strictEqual(counters[0].run(), 2);
            }
            count += 3;
            strictEqual(count, 5);
        }
    "#);
}

#[test]
fn inserting_another_record_origin_preserves_both_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                strictEqual(holders[0].run(), 3);
                holders.push({ run: (): int => values[1] });
                strictEqual(holders[0].run(), 3);
                strictEqual(holders[1].run(), 7);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn extracted_record_keeps_capture_conflicts_after_container_scope_ends() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3];
            let escaped: Holder;
            {
                const holders: Holder[] = [{ run: (): int => values[0] }];
                escaped = holders.pop()!;
            }
            values[0] = 9;
            console.log(escaped.run());
        }
    "#;
    let error = tsrust_compiler::check(&support::format_round_trip(source)).err().expect("capture conflict must be rejected");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn dense_arrays_of_borrowing_records_keep_capture_owners_live() {
    let source = support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const holders: Holder[] = [
                    { run: (): int => values[0] },
                    { run: (): int => values[1] }
                ];
                strictEqual(holders[0].run(), 3);
                strictEqual(holders[1].run(), 7);
                for (const holder of holders) strictEqual(holder.run() > 0, true);
                const last = holders.pop();
                if (last !== undefined) strictEqual(last.run(), 7);
                const first = holders.shift();
                if (first !== undefined) strictEqual(first.run(), 3);
                strictEqual(holders.length, 0);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}
