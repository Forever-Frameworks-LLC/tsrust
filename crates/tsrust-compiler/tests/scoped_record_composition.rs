mod support;

#[test]
fn separate_map_elements_keep_key_contracts_and_both_capture_owners() {
    let prefix = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const a=[3];const b=[7];const first=new Map<string,Holder>();first.set('value',{run:():int=>a[0]});const second=new Map<string,Holder>();second.set('value',{run:():int=>b[0]});const boxes:Box[]=[{handlers:first},{handlers:second}];";
    for owner in ["a", "b"] {
        let source = format!("{prefix}{owner}[0]=9;console.log(boxes[0].handlers.size,boxes[1].handlers.size);}}");
        let error = tsrust_compiler::check(&source).err().expect("each Map retains its own capture owner");
        assert_eq!(error.code, "capture-conflict", "{}", error.message);
    }
    let wrong_key = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const values=[3];const first=new Map<string,Holder>();first.set('value',{run:():int=>values[0]});const second=new Map<int,Holder>();second.set(1,{run:():int=>values[0]});const boxes:Box[]=[{handlers:first},{handlers:second}];console.log(boxes.length);}";
    let error = tsrust_compiler::check(wrong_key).err().expect("array composition cannot change Map key contracts");
    assert_eq!(error.code, "type", "{}", error.message);
}

#[test]
fn separate_record_map_elements_join_callback_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7, 11];
            const first = new Map<string, Holder>();
            first.set("value", { run: (): int => values[0] });
            const second = new Map<string, Holder>();
            second.set("value", { run: (): int => values[1] });
            const boxes: Box[] = [{ handlers: first }, { handlers: second }];
            boxes[0].handlers.set("later", { run: (): int => values[2] });
            const left = boxes[0].handlers.get("value");
            const right = boxes[1].handlers.get("value");
            const later = boxes[0].handlers.get("later");
            if (left !== undefined) strictEqual(left.run(), 3);
            if (right !== undefined) strictEqual(right.run(), 7);
            if (later !== undefined) strictEqual(later.run(), 11);
        }
    "#);
}

#[test]
fn indexed_map_replacement_releases_capture_loans_at_container_scope_exit() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3];
            {
                const first = new Map<string, Holder>();
                first.set("first", { run: (): int => values[0] });
                const boxes: Box[] = [{ handlers: first }];
                const replacement = new Map<string, Holder>();
                replacement.set("next", { run: (): int => values[0] + 1 });
                boxes[0].handlers = replacement;
                const next = boxes[0].handlers.get("next");
                if (next !== undefined) strictEqual(next.run(), 4);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn indexed_record_map_fields_keep_callback_environments() {
    let source = r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            const boxes: Box[] = [{ handlers: handlers }];
            boxes[0].handlers.set("second", { run: (): int => values[1] });
            const first = boxes[0].handlers.get("first");
            const second = boxes[0].handlers.get("second");
            if (first !== undefined) strictEqual(first.run(), 3);
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#;
    project(source);
    let nested = source.replace("const boxes: Box[] = [{ handlers: handlers }];", "const boxes: Box[][] = [[{ handlers: handlers }]];")
        .replace("boxes[0].handlers", "boxes[0][0].handlers");
    project(&nested);
}

#[test]
fn indexed_map_field_insertion_preserves_new_capture_loans() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const first=[3];const second=[7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});const boxes:Box[]=[{handlers:handlers}];boxes[0].handlers.set('second',{run:():int=>second[0]});second[0]=9;console.log(boxes[0].handlers.size);}";
    let error = tsrust_compiler::check(source).err().expect("indexed field insertion keeps the new capture owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn scoped_map_for_each_propagates_callback_failures_and_rejects_mutation() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            handlers.set("second", { run: (): int => { throw values[1]; } });
            let sum = 0;
            let caught = 0;
            try {
                handlers.forEach((holder: ReadOnly<Holder>): void => { sum += holder.run(); });
            } catch (error) {
                caught = 1;
            }
            strictEqual(sum, 3);
            strictEqual(caught, 1);
            strictEqual(handlers.size, 2);
        }
    "#);
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});handlers.forEach((holder:ReadOnly<Holder>):void=>{handlers.clear();console.log(holder.run());});}";
    let error = tsrust_compiler::check(source).err().expect("Map iteration holds a receiver read loan");
    assert!(error.code.contains("conflict") || error.message.to_lowercase().contains("borrow"), "{}: {}", error.code, error.message);
}

#[test]
fn map_for_each_invokes_borrowing_record_callbacks_from_distinct_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            let initial = 0;
            handlers.forEach((holder: ReadOnly<Holder>): void => { initial += holder.run(); });
            strictEqual(initial, 3);
            handlers.set("second", { run: (): int => values[1] });
            let sum = 0;
            handlers.forEach((holder: ReadOnly<Holder>, key: string): void => {
                sum += holder.run();
            });
            strictEqual(sum, 10);
        }
    "#);
}

#[test]
fn map_field_replacement_retains_capture_owners_and_rejects_live_views() {
    let prefix = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const firstValues=[3];const secondValues=[7];const first=new Map<string,Holder>();first.set('value',{run:():int=>firstValues[0]});let box:Box={handlers:first};const second=new Map<string,Holder>();second.set('value',{run:():int=>secondValues[0]});";
    let conflict = format!("{prefix}box.handlers=second;secondValues[0]=9;console.log(box.handlers.size);}}");
    let error = tsrust_compiler::check(&conflict).err().expect("replacement retains its callback owners");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let view = format!("{prefix}const old=box.handlers.get('value');box.handlers=second;if(old!==undefined)console.log(old.run());}}");
    let error = tsrust_compiler::check(&view).err().expect("replacement cannot invalidate an entry view");
    assert_eq!(error.code, "view-conflict", "{}", error.message);
}

#[test]
fn record_map_field_replacement_preserves_distinct_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const first = new Map<string, Holder>();
            first.set("value", { run: (): int => values[0] });
            let box: Box = { handlers: first };
            {
                const old = box.handlers.get("value");
                if (old !== undefined) strictEqual(old.run(), 3);
            }
            const second = new Map<string, Holder>();
            second.set("value", { run: (): int => values[1] });
            box.handlers = second;
            const replacement = box.handlers.get("value");
            if (replacement !== undefined) strictEqual(replacement.run(), 7);
        }
    "#);
}

#[test]
fn scoped_aggregate_callback_environments_drop_once_on_each_storage_boundary() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3];
            {
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            console.log("replace");
            handlers.set("first", { run: (): int => values[0] + 1 });
            console.log("delete");
            handlers.delete("first");
            handlers.set("clear", { run: (): int => values[0] + 2 });
            console.log("clear");
            handlers.clear();
            handlers.set("scope", { run: (): int => values[0] + 3 });
            console.log("scope");
            }
            console.log("array");
            {
                const items: Holder[] = [
                    { run: (): int => values[0] + 4 },
                    { run: (): int => values[0] + 5 }
                ];
                {
                    const removed = items.splice(0, 1);
                    console.log("removed");
                }
                const last = items.pop();
                console.log("pop");
            }
            console.log("record");
            {
                const handlers = new Map<string, Holder>();
                handlers.set("record", { run: (): int => values[0] + 6 });
                const box: Box = { handlers: handlers };
                const moved = box;
                console.log("record scope");
            }
        }
    "#);
    verify_disposal(&source, 7, "replace\ndrop 0\ndelete\ndrop 1\nclear\ndrop 2\nscope\ndrop 3\narray\nremoved\ndrop 4\npop\ndrop 5\nrecord\nrecord scope\ndrop 6\n");
}

#[test]
fn scoped_aggregate_failure_disposes_environments_before_catch_and_finally() {
    let source = support::format_round_trip(r#"
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3];
            try {
                const items: Holder[] = [{ run: (): int => { throw values[0]; } }];
                items[0].run();
            } catch (error) {
                console.log("caught array");
            } finally {
                console.log("finally array");
            }
            try {
                const handlers = new Map<string, Holder>();
                handlers.set("fail", { run: (): int => { throw values[0]; } });
                const handler = handlers.get("fail");
                if (handler !== undefined) handler.run();
            } catch (error) {
                console.log("caught map");
            } finally {
                console.log("finally map");
            }
        }
    "#);
    verify_disposal(&source, 2, "drop 0\ncaught array\nfinally array\ndrop 1\ncaught map\nfinally map\n");
}

fn verify_disposal(source: &str, environments: usize, expected: &str) {
    let analysis = tsrust_compiler::check(&source).unwrap();
    let mut rust = analysis.emit_rust();
    // Attach an observer to generated borrowed environments. The checked source
    // controls disposal boundaries; each observer runs only when Rust drops one.
    let closures = analysis.closures();
    assert_eq!(closures.len(), environments);
    for closure in closures {
        assert!(!closure.captures.is_empty());
        rust.push_str(&format!("\nimpl<'a> Drop for Closure{}<'a> {{ fn drop(&mut self) {{ println!(\"drop {}\"); }} }}\n", closure.id, closure.id));
    }
    let directory = support::TempDirectory::new();
    let path = directory.0.join("disposal.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("disposal-{level}"));
        support::success(&support::run(support::rustc().arg(&path).arg("--edition=2024").arg("-C").arg(format!("opt-level={level}")).arg("-o").arg(&binary), 60));
        let output = support::run(std::process::Command::new(&binary).args(std::iter::empty::<&str>()), 60);
        support::success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
    }
}

#[test]
fn record_map_fields_preserve_new_capture_conflicts_and_reject_escape() {
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function main(){const first=[3];const second=[7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>first[0]});const box:Box={handlers:handlers};box.handlers.set('second',{run:():int=>second[0]});second[0]=9;console.log(box.handlers.size);}";
    let error = tsrust_compiler::check(conflict).err().expect("field insertion retains the new callback owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let escape = "type Holder={run:ReadOnly<Function<int>>;};type Box={handlers:Map<string,Holder>;};function bad():Box{const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const box:Box={handlers:handlers};return box;}function main(){}";
    let error = tsrust_compiler::check(escape).err().expect("record Map cannot conceal a local borrowed capture");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn record_fields_hold_maps_with_borrowing_callback_values() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { handlers: Map<string, Holder>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            const box: Box = { handlers: handlers };
            box.handlers.set("second", { run: (): int => values[1] });
            const first = box.handlers.get("first");
            const second = box.handlers.get("second");
            if (first !== undefined) strictEqual(first.run(), 3);
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#);
}

#[test]
fn moved_map_can_widen_to_another_callback_origin() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            const moved = handlers;
            moved.set("second", { run: (): int => values[1] });
            const finalMap = moved;
            const first = finalMap.get("first");
            const second = finalMap.get("second");
            if (first !== undefined) strictEqual(first.run(), 3);
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#);
}

#[test]
fn moved_map_keeps_capture_loans_and_transfer_diagnostics() {
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const moved=handlers;values[0]=9;console.log(moved.size);}";
    let error = tsrust_compiler::check(conflict).err().expect("moving a Map retains its capture owners");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let transferred = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const moved=handlers;console.log(handlers.size,moved.size);}";
    let error = tsrust_compiler::check(transferred).err().expect("Map move consumes the source binding");
    assert_eq!(error.code, "transferred", "{}", error.message);
}

#[test]
fn map_get_before_later_insertion_keeps_callback_environment() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            {
                const first = handlers.get("first");
                if (first !== undefined) strictEqual(first.run(), 3);
            }
            handlers.set("second", { run: (): int => values[1] });
            const second = handlers.get("second");
            if (second !== undefined) strictEqual(second.run(), 7);
        }
    "#);
}

#[test]
fn scoped_map_values_keep_capture_conflicts_and_reject_local_escape() {
    let live_view = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3,7];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});const first=handlers.get('first');handlers.set('second',{run:():int=>values[1]});if(first!==undefined)console.log(first.run());}";
    let error = tsrust_compiler::check(live_view).err().expect("Map mutation cannot invalidate a live entry view");
    assert_eq!(error.code, "view-conflict", "{}", error.message);
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});values[0]=9;console.log(handlers.size);}";
    let error = tsrust_compiler::check(conflict).err().expect("Map payload retains its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let escape = "type Holder={run:ReadOnly<Function<int>>;};function bad():Map<string,Holder>{const values=[3];const handlers=new Map<string,Holder>();handlers.set('first',{run:():int=>values[0]});return handlers;}function main(){}";
    let error = tsrust_compiler::check(escape).err().expect("Map cannot conceal local borrowed payload");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn map_values_keep_borrowing_record_callback_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const handlers = new Map<string, Holder>();
            handlers.set("first", { run: (): int => values[0] });
            handlers.set("second", { run: (): int => values[1] });
            const handler = handlers.get("first");
            if (handler !== undefined) strictEqual(handler.run(), 3);
            const other = handlers.get("second");
            if (other !== undefined) strictEqual(other.run(), 7);
        }
    "#);
}

#[test]
fn alternating_path_removal_keeps_inserted_capture_after_scope_exit() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={items:Holder[];};function main(){const first=[3];const second=[7];let removed:Holder|undefined;{const boxes:Box[]=[{items:[{run:():int=>first[0]}]}];boxes[0].items.push({run:():int=>second[0]});removed=boxes[0].items.pop();}second[0]=9;if(removed!==undefined)console.log(removed.run());}";
    let error = tsrust_compiler::check(source).err().expect("removed callback must retain the inserted owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn indexed_records_keep_array_field_callback_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { items: Holder[]; };
        function main() {
            const values = [3, 7, 11];
            const boxes: Box[] = [{ items: [{ run: (): int => values[0] }] }];
            boxes[0].items.push({ run: (): int => values[1] });
            const removed = boxes[0].items.pop();
            boxes[0].items[0] = { run: (): int => values[2] };
            if (removed !== undefined) strictEqual(removed.run(), 7);
            strictEqual(boxes[0].items[0].run(), 11);
        }
    "#);
}

#[test]
fn mixed_path_insertions_keep_new_capture_conflicts() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={groups:Holder[][];};function main(){const first=[3];const second=[7];const box:Box={groups:[[{run:():int=>first[0]}]]};box.groups[0].push({run:():int=>second[0]});second[0]=9;console.log(box.groups[0][1].run());}";
    let error = tsrust_compiler::check(source).err().expect("mixed path insertion retains its owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn mixed_field_index_paths_preserve_callback_record_arrays() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { groups: Holder[][]; };
        function main() {
            const values = [3, 7, 11];
            const box: Box = { groups: [[{ run: (): int => values[0] }]] };
            box.groups[0].push({ run: (): int => values[1] });
            const removed = box.groups[0].pop();
            box.groups[0][0] = { run: (): int => values[2] };
            if (removed !== undefined) strictEqual(removed.run(), 7);
            strictEqual(box.groups[0][0].run(), 11);
        }
    "#);
}

#[test]
fn record_field_removals_keep_capture_conflicts_after_record_scope() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};type Box={items:Holder[];};function main(){const first=[3];const second=[7];let removed:Holder|undefined;{const box:Box={items:[{run:():int=>first[0]}]};box.items.push({run:():int=>second[0]});removed=box.items.pop();}second[0]=9;if(removed!==undefined)console.log(removed.run());}";
    let error = tsrust_compiler::check(source).err().expect("removed field payload retains its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn record_fields_hold_and_mutate_callback_record_arrays() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        type Box = { items: Holder[]; };
        function main() {
            const values = [3, 7];
            const box: Box = { items: [{ run: (): int => values[0] }] };
            box.items.push({ run: (): int => values[1] });
            strictEqual(box.items[0].run(), 3);
            strictEqual(box.items[1].run(), 7);
            const last = box.items.pop();
            if (last !== undefined) strictEqual(last.run(), 7);
        }
    "#);
}

#[test]
fn indexed_nested_removal_retains_capture_after_container_scope() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];let removed:Holder|undefined;{const groups:Holder[][]=[[{run:():int=>values[0]}]];removed=groups[0].pop();}values[0]=9;if(removed!==undefined)console.log(removed.run());}";
    let error = tsrust_compiler::check(source).err().expect("indexed removal retains borrowed owner");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn indexed_nested_removals_keep_dispatch_after_later_widening() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            const groups: Holder[][] = [[{ run: (): int => values[0] }]];
            const last = groups[0].pop();
            groups[0].push({ run: (): int => values[1] });
            const removed = groups[0].splice(0, 1);
            groups[0].push({ run: (): int => values[2] });
            if (last !== undefined) strictEqual(last.run(), 3);
            strictEqual(removed[0].run(), 7);
            strictEqual(groups[0][0].run(), 11);
        }
    "#);
}

#[test]
fn nested_array_valued_writes_join_record_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            const groups: Holder[][] = [[{ run: (): int => values[0] }]];
            groups.push([{ run: (): int => values[1] }]);
            strictEqual(groups[0][0].run(), 3);
            strictEqual(groups[1][0].run(), 7);
            groups[0] = [{ run: (): int => values[2] }];
            strictEqual(groups[0][0].run(), 11);
            strictEqual(groups[1][0].run(), 7);
        }
    "#);
}

#[test]
fn nested_insertions_protect_new_capture_owners() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const first=[3];const second=[7];const groups:Holder[][]=[[{run:():int=>first[0]}]];groups[0].push({run:():int=>second[0]});second[0]=9;console.log(groups[0][1].run());}";
    let error = tsrust_compiler::check(source).err().expect("inserted callback retains its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn nested_indexed_arrays_accept_new_record_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7, 11];
            const groups: Holder[][] = [[{ run: (): int => values[0] }]];
            groups[0].push({ run: (): int => values[1] });
            strictEqual(groups[0][0].run(), 3);
            strictEqual(groups[0][1].run(), 7);
            groups[0][0] = { run: (): int => values[2] };
            strictEqual(groups[0][0].run(), 11);
            strictEqual(groups[0][1].run(), 7);
        }
    "#);
}

#[test]
fn nested_array_conversion_consumes_its_sources() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3,7];const first:Holder[]=[{run:():int=>values[0]}];const second:Holder[]=[{run:():int=>values[1]}];const groups:Holder[][]=[first,second];console.log(first.length,groups[0][0].run());}";
    let error = tsrust_compiler::check(source).err().expect("conversion moves existing arrays");
    assert_eq!(error.code, "transferred", "{}", error.message);
}

#[test]
fn nested_storage_moves_existing_arrays_with_distinct_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const first: Holder[] = [{ run: (): int => values[0] }];
            const second: Holder[] = [{ run: (): int => values[1] }];
            const groups: Holder[][] = [first, second];
            strictEqual(groups[0][0].run(), 3);
            strictEqual(groups[1][0].run(), 7);
        }
    "#);
}

#[test]
fn nested_record_arrays_keep_capture_conflicts_and_reject_escape() {
    let conflict = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];const groups:Holder[][]=[[{run:():int=>values[0]}]];values[0]=9;console.log(groups[0][0].run());}";
    let error = tsrust_compiler::check(conflict).err().expect("nested storage retains captures");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
    let escape = "type Holder={run:ReadOnly<Function<int>>;};function bad():Holder[][]{const values=[3];return [[{run:():int=>values[0]}]];}function main(){}";
    let error = tsrust_compiler::check(escape).err().expect("nested storage cannot hide local captures");
    assert_eq!(error.code, "callback-escape", "{}", error.message);
}

#[test]
fn nested_arrays_preserve_distinct_record_environments() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            {
                const groups: Holder[][] = [
                    [{ run: (): int => values[0] }],
                    [{ run: (): int => values[1] }]
                ];
                strictEqual(groups[0][0].run(), 3);
                strictEqual(groups[1][0].run(), 7);
            }
            values[0] = 9;
            strictEqual(values[0], 9);
        }
    "#);
}

#[test]
fn reassigned_nullable_removal_retains_capture_conflicts() {
    let source = "type Holder={run:ReadOnly<Function<int>>;};function main(){const values=[3];let outer:Holder|undefined;{const holders:Holder[]=[{run:():int=>values[0]}];outer=holders.pop();}values[0]=9;if(outer!==undefined)console.log(outer.run());}";
    let error = tsrust_compiler::check(source).err().expect("nullable storage must retain its capture");
    assert_eq!(error.code, "capture-conflict", "{}", error.message);
}

#[test]
fn nullable_removal_moves_and_reassignment_preserve_environment_layout() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            const removed = holders.pop();
            const saved = removed;
            holders.push({ run: (): int => values[1] });
            if (saved !== undefined) strictEqual(saved.run(), 3);
            let outer: Holder | undefined;
            {
                const queue: Holder[] = [{ run: (): int => values[0] }];
                outer = queue.shift();
            }
            if (outer !== undefined) strictEqual(outer.run(), 3);
        }
    "#);
}

#[test]
fn loop_insertions_keep_initial_and_repeated_callback_origins() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            for (let i = 0; i < 2; i += 1) {
                holders.push({ run: (): int => values[1] });
            }
            strictEqual(holders[0].run(), 3);
            strictEqual(holders[1].run(), 7);
            strictEqual(holders[2].run(), 7);
        }
    "#);
}

#[test]
fn nullable_removed_record_keeps_dispatch_after_array_widens() {
    project(r#"
        import { strictEqual } from "std:testing";
        type Holder = { run: ReadOnly<Function<int>>; };
        function main() {
            const values = [3, 7];
            const holders: Holder[] = [{ run: (): int => values[0] }];
            const removed = holders.pop();
            holders.push({ run: (): int => values[1] });
            if (removed !== undefined) strictEqual(removed.run(), 3);
            strictEqual(holders[0].run(), 7);
            const queue: Holder[] = [{ run: (): int => values[0] }];
            const first = queue.shift();
            queue.push({ run: (): int => values[1] });
            if (first !== undefined) strictEqual(first.run(), 3);
            strictEqual(queue[0].run(), 7);
        }
    "#);
}

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
