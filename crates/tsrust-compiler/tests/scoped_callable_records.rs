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

#[test]
fn scoped_record_absence_and_omitted_optional_fields_keep_call_signatures() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Nullable = { run: ReadOnly<Function<int, [value: int]>> | null; };
        type Optional = { run?: ReadOnly<Function<int, [value: int]>>; };
        function main() {
            const nullable: Nullable = { run: null };
            const missing: Optional = {};
            const explicit: Optional = { run: undefined };
            let effects = 0;
            strictEqual(nullable.run?.(effects++) ?? -1, -1);
            strictEqual(missing.run?.(effects++) ?? -2, -2);
            strictEqual(explicit.run?.(effects++) ?? -3, -3);
            strictEqual(effects, 0);
            strictEqual(typeof nullable.run, "object");
            strictEqual(typeof missing.run, "undefined");
        }
    "#);
}

#[test]
fn nested_scoped_records_preserve_distinct_layouts_and_original_captures() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        type Envelope = { holder: Holder; label: string; };
        type Outer = { envelope: Envelope; };
        function main() {
            const offsets = [3, 7];
            {
                const holder: Holder = { run: (value): int => value + offsets[0] };
                const envelope: Envelope = { holder: holder, label: "nested" };
                const outer: Outer = { envelope: envelope };
                strictEqual(outer.envelope.holder.run(2), 5);
                strictEqual(outer.envelope.holder.run(4), 7);
                strictEqual(outer.envelope.label, "nested");
            }
            offsets[0] = 9;
            strictEqual(offsets[0], 9);
        }
    "#);
}

#[test]
fn closure_capturing_nested_scoped_record_preserves_environment_lifetimes() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        type Envelope = { holder: Holder; };
        function main() {
            const offsets = [3, 7];
            {
                const holder: Holder = { run: (value): int => value + offsets[0] };
                const envelope: Envelope = { holder: holder };
                const invoke = (value: int): int => envelope.holder.run(value);
                strictEqual(invoke(2), 5);
                strictEqual(invoke(4), 7);
            }
            offsets[0] = 9;
            strictEqual(offsets[0], 9);
        }
    "#);
}

#[test]
fn escaping_closure_cannot_hide_borrowed_storage_inside_a_record_capture() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        function make(): () => int {
            const offsets = [3, 7];
            const holder: Holder = { run: (value): int => value + offsets[0] };
            return (): int => holder.run(2);
        }
        function main() {}
    "#;
    let error = match tsrust_compiler::check(source) { Ok(_) => panic!("accepted borrowed record environment escape"), Err(error) => error };
    assert_eq!(error.code, "closure-escape", "{error:?}");
}

#[test]
fn record_capture_names_borrowed_family_lifetimes_and_keeps_empty_factory_owned() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Optional = { run?: ReadOnly<Function<int, [value: int]>>; };
        function empty(): () => int {
            const missing: Optional = {};
            return (): int => missing.run?.(3) ?? 0;
        }
        function main() {
            const offsets = [3, 7];
            {
                const selected = offsets.length === 2
                    ? (value: int): int => value + offsets[0]
                    : (value: int): int => value + offsets[1];
                const optional = offsets.length === 2 ? selected : undefined;
                const holder: Optional = { run: optional };
                const invoke = (value: int): int => holder.run?.(value) ?? 0;
                strictEqual(invoke(2), 5);
                strictEqual(invoke(4), 7);
                const standaloneSelected = offsets.length === 2
                    ? (value: int): int => value + offsets[0]
                    : (value: int): int => value + offsets[1];
                const standalone = offsets.length === 2 ? standaloneSelected : undefined;
                const invokeStandalone = (value: int): int => standalone?.(value) ?? 0;
                strictEqual(invokeStandalone(2), 5);
                strictEqual(invokeStandalone(4), 7);
            }
            offsets[0] = 9;
            strictEqual(offsets[0], 9);
            const ownedEmpty = empty();
            strictEqual(ownedEmpty(), 0);
            strictEqual(ownedEmpty(), 0);
        }
    "#);
}

#[test]
fn scoped_record_parameter_invokes_without_retaining_owner() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        function invoke(holder: ReadOnly<Holder>, value: int): int {
            return holder.run(value);
        }
        function main() {
            const offsets = [3, 7];
            {
                const holder: Holder = { run: (value: int): int => value + offsets[0] };
                strictEqual(invoke(holder, 2), 5);
                strictEqual(invoke(holder, 4), 7);
            }
            offsets[0] = 9;
            strictEqual(offsets[0], 9);
        }
    "#);
}

#[test]
fn nested_scoped_record_parameter_preserves_callback_environment() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        type Envelope = { holder: Holder; };
        function invoke(envelope: ReadOnly<Envelope>, value: int): int {
            return envelope.holder.run(value);
        }
        function main() {
            const offsets = [3, 7];
            {
                const holder: Holder = { run: (value: int): int => value + offsets[0] };
                const envelope: Envelope = { holder: holder };
                strictEqual(invoke(envelope, 2), 5);
                strictEqual(invoke(envelope, 4), 7);
            }
            offsets[0] = 9;
            strictEqual(offsets[0], 9);
        }
    "#);
}

#[test]
fn retaining_record_parameter_cannot_return_scoped_callback_storage() {
    let source = r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        type Envelope = { holder: Holder; };
        function retain(envelope: Envelope): Envelope { return envelope; }
        function main() {
            const values = [3, 7];
            const holder: Holder = { run: (): int => values[0] };
            const envelope: Envelope = { holder: holder };
            const retained = retain(envelope);
            console.log(retained.holder.run());
        }
    "#;
    let error = match tsrust_compiler::check(source) {
        Ok(_) => panic!("accepted retaining scoped record parameter"),
        Err(error) => error,
    };
    assert_eq!(error.code, "callback-escape", "{error:?}");
}

#[test]
fn owned_record_factory_result_preserves_distinct_callback_instances() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        function make(offset: int): Holder {
            return { run: (value: int): int => value + offset };
        }
        function main() {
            const first = make(3);
            const second = make(11);
            strictEqual(first.run(2), 5);
            strictEqual(second.run(2), 13);
            strictEqual(first.run(4), 7);
        }
    "#);
}

#[test]
fn nested_owned_record_result_keeps_environment_through_forwarding() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        type Envelope = { holder: Holder; };
        function make(offset: int): Envelope {
            const holder: Holder = { run: (value: int): int => value + offset };
            return { holder: holder };
        }
        function forward(offset: int): Envelope { return make(offset); }
        function main() {
            const first = forward(3);
            const second = forward(11);
            strictEqual(first.holder.run(2), 5);
            strictEqual(second.holder.run(2), 13);
            strictEqual(first.holder.run(4), 7);
        }
    "#);
}

#[test]
fn scoped_record_partial_field_moves_release_independent_capture_loans() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Pair = {
            left: ReadOnly<Function<int>>;
            right: ReadOnly<Function<int>>;
        };
        function main() {
            const leftValues = [3];
            const rightValues = [11];
            {
                const pair: Pair = {
                    left: (): int => leftValues[0],
                    right: (): int => rightValues[0]
                };
                {
                    const left = pair.left;
                    strictEqual(left(), 3);
                    strictEqual(pair.right(), 11);
                }
                leftValues[0] = 5;
                strictEqual(leftValues[0], 5);
                strictEqual(pair.right(), 11);
            }
            rightValues[0] = 13;
            strictEqual(rightValues[0], 13);
        }
    "#);
}

#[test]
fn moved_and_remaining_record_fields_keep_live_capture_conflicts() {
    for (owner, observer) in [("leftValues", "left()"), ("rightValues", "pair.right()")] {
        let source = format!(r#"
            type Pair = {{ left: ReadOnly<Function<int>>; right: ReadOnly<Function<int>>; }};
            function main() {{
                const leftValues = [3];
                const rightValues = [11];
                const pair: Pair = {{ left: (): int => leftValues[0], right: (): int => rightValues[0] }};
                const left = pair.left;
                {owner}[0] = 5;
                console.log({observer});
            }}
        "#);
        let error = match tsrust_compiler::check(&source) {
            Ok(_) => panic!("accepted conflicting live record field capture"),
            Err(error) => error,
        };
        assert_eq!(error.code, "capture-conflict", "{error:?}");
    }
}

#[test]
fn scoped_record_assignment_restores_field_and_whole_record_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3];
            {
                let holder: Holder = { run: (): int => values[0] };
                holder = holder;
                {
                    const temporary = holder.run;
                    holder.run = temporary;
                }
                strictEqual(holder.run(), 3);
                {
                    const temporary = holder;
                    holder = temporary;
                }
                strictEqual(holder.run(), 3);
            }
            values[0] = 5;
            strictEqual(values[0], 5);
        }
    "#);
}

#[test]
fn scoped_record_destructuring_preserves_nested_and_readonly_parameter_callbacks() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int, [value: int]>>; };
        type Envelope = { holder: Holder; tag: int; };
        function invoke({ run }: ReadOnly<Holder>, amount: int): int {
            return run(amount);
        }
        function main() {
            const values = [3];
            {
                const holder: Holder = { run: (value: int): int => value + values[0] };
                strictEqual(invoke(holder, 2), 5);
                const envelope: Envelope = { holder: holder, tag: 1 };
                const { holder: { run: extracted } } = envelope;
                strictEqual(extracted(4), 7);
                strictEqual(envelope.tag, 1);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn deferred_scoped_record_fields_release_independent_capture_loans() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Pair = { left: ReadOnly<Function<int>>; right: ReadOnly<Function<int>>; };
        function main() {
            const a = [3]; const b = [11];
            {
                let pair: Pair;
                pair = { left: (): int => a[0], right: (): int => b[0] };
                { const left = pair.left; strictEqual(left(), 3); }
                a[0] = 5;
                strictEqual(pair.right(), 11);
            }
            b[0] = 13; strictEqual(b[0], 13);
        }
    "#);
}

#[test]
fn owned_record_parameter_entry_uses_independent_leaf_storage() {
    let source=support::format_round_trip(r#"
        import { strictEqual } from "std:testing";
        type Pair = { left: ReadOnly<Function<int>>; right: ReadOnly<Function<int>>; };
        function inspect(pair: Owned<Pair>): int {
            const left = pair.left;
            return left() + pair.right();
        }
        function main() {
            const pair: Pair = { left: (): int => 3, right: (): int => 11 };
            strictEqual(inspect(pair), 14);
        }
    "#);
    let standalone=source.replace("import { strictEqual } from \"std:testing\";", "");
    let declaration_only=format!("{} function main() {{}}",standalone.split("function main").next().unwrap());
    let checked=tsrust_compiler::check(&declaration_only).unwrap();
    let native=checked.emit_rust();
    assert!(native.contains("field0:value0_field0"),"owned parameter lacks independent entry storage");
    project(&source);
}

#[test]
fn mutable_record_capture_updates_independent_leaves_and_releases_owner() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Counter = { run: Mutable<Function<int>>; label: string; };
        function main() {
            let total = 0;
            {
                let holder: Counter = { run: (): int => { total += 1; return total; }, label: "start" };
                {
                    const update = (): int => { holder.label = "changed"; return holder.run(); };
                    strictEqual(update(), 1); strictEqual(update(), 2);
                }
                strictEqual(holder.label, "changed"); strictEqual(holder.run(), 3);
            }
            total += 4; strictEqual(total, 7);
        }
    "#);
}

#[test]
fn deferred_record_branch_and_loop_restoration_preserve_capture_release() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; label: string; };
        function main() {
            const values = [7];
            {
                let holder: Holder;
                if (values.length === 1) {
                    holder = { run: (): int => values[0], label: "yes" };
                } else {
                    holder = { run: (): int => values[0], label: "no" };
                }
                let count = 0;
                while (count < 2) {
                    strictEqual(holder.run(), 7);
                    holder.label = "loop"; count += 1;
                }
                strictEqual(holder.label, "loop");
            }
            values[0] = 9; strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn nested_record_branch_assignments_preserve_three_actual_origins() {
    let source=r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            {
                let holder: Holder;
                if (values.length === 3) {
                    if (values[0] === 3) {
                        holder = { run: (): int => values[0] };
                    } else {
                        holder = { run: (): int => values[1] };
                    }
                } else {
                    holder = { run: (): int => values[2] };
                }
                strictEqual(holder.run(), 3);
            }
            values[0] = 13; strictEqual(values[0], 13);
        }
    "#;
    for (values,expected) in [("[3, 7, 11]",3),("[9, 7, 11]",7),("[3, 7, 11, 99]",11)] {
        let selected=source.replace("[3, 7, 11]",values).replace("strictEqual(holder.run(), 3);",&format!("strictEqual(holder.run(), {expected});"));
        project(&selected);
    }
}
