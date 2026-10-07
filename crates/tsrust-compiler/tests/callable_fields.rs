mod support;

#[test]
fn callable_reductions_transfer_owned_accumulators_and_keep_receiver_loans() {
    execute("type Total={value:int;};function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];const values=actions.reduce((acc:int[],action:Mutable<Function<int>>):int[]=>{acc.push(action());return acc;},[]);console.log(values.join('|'));const total=actions.reduceRight((acc:Total,action:Mutable<Function<int>>):Total=>{acc.value=acc.value+action();return acc;},{value:0});console.log(total.value);const text=actions.reduce((acc:string,action:Mutable<Function<int>>):string=>{return acc+String(action());},'');console.log(text,actions[0]());}", "7|9\n16\n79 7\n");
    for method in ["reduce", "reduceRight"] {
        support::reject(&format!("function seed(actions:(()=>int)[]):int{{const removed=actions.remove(0);return removed();}}function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];actions.{method}((sum:int,action:Mutable<Function<int>>):int=>{{return sum+action();}},seed(actions));}}"), "conflicting-access");
        support::reject(&format!("function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];actions.{method}((sum:int,action:Mutable<Function<int>>,index:int,source:(()=>int)[]):int=>{{return sum+action();}},0);}}"), "conflicting-access");
        support::reject(&format!("function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];actions.{method}((sum:int,action:Mutable<Function<int>>):int=>{{return sum+action();}},actions.push(():int=>{{return 9;}}));}}"), "conflicting-access");
    }
}

#[test]
fn callable_reductions_lend_environments_with_independent_accumulators() {
    execute("function visitor():(sum:int,action:Mutable<Function<int>>)=>int{console.log('visitor');return (sum:int,action:Mutable<Function<int>>):int=>{return sum+action();};}function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];const local=(sum:int,action:Mutable<Function<int>>):int=>{return sum+action();};console.log(actions.reduce(local,0),actions.reduceRight(local,0));console.log(actions.reduce(visitor(),0));}", "16 16\nvisitor\n16\n");
    execute("type State={value:int;};function main(){const left:State={value:0};const right:State={value:10};const actions:(()=>int)[]=[():int=>{left.value=left.value+1;return left.value;},():int=>{right.value=right.value+2;return right.value;}];console.log(actions.reduce((sum:int,action:Mutable<Function<int>>,index:int):int=>{return sum+action()+index;},0));console.log(actions.reduceRight((sum:int,action:Mutable<Function<int>>,index:int):int=>{console.log(index);return sum+action();},0));console.log(actions[0](),actions[1]());const empty:(()=>int)[]=[];console.log(empty.reduce((sum:int,action:Mutable<Function<int>>):int=>{return sum+action();},7));}", "14\n1\n0\n16\n3 16\n7\n");
    for method in ["reduce", "reduceRight"] {
        support::reject(&format!("function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];actions.{method}((sum:int,action:Mutable<Function<int>>):int=>{{actions.reverse();return sum+action();}},0);}}"), "conflicting-access");
        support::reject(&format!("function main(){{const actions:(Once<Function<int>>)[]=[():int=>{{return 7;}}];actions.{method}((sum:int,action:Mutable<Function<int>>):int=>{{return sum+action();}},0);}}"), "callback-contract");
    }
}

#[test]
fn callable_flat_map_lends_environments_and_transfers_independent_rows() {
    execute("function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];const values=actions.flatMap((action:Mutable<Function<int>>,index:int):int[]=>{return [action(),index];});console.log(values.join('|'),actions[0](),actions[1]());}", "7|0|9|1 7 9\n");
    execute("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];const results=actions.flatMap((action:Mutable<Function<int>>):(()=>int)[]=>{console.log(action());return [():int=>{return 42;}];});console.log(results[0](),actions[0]());const empty:(()=>int)[]=[];console.log(empty.flatMap((action:Mutable<Function<int>>):int[]=>{return [action()];}).length);}", "7\n42 7\n0\n");
    for (source, code) in [
        ("function main(){const actions:(Once<Function<int>>)[]=[():int=>{return 7;}];actions.flatMap((action:Mutable<Function<int>>):int[]=>{return [action()];});}", "callback-contract"),
        ("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.flatMap((action:Mutable<Function<int>>):int[]=>{actions.reverse();return [action()];});}", "conflicting-access"),
    ] { support::reject(source, code); }
}

#[test]
fn callable_flat_map_reuses_local_visitors_and_evaluates_factories_once() {
    execute("function visitor():(action:Mutable<Function<int>>)=>int[]{console.log('visitor');return (action:Mutable<Function<int>>):int[]=>{return [action()];};}function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];let visits=0;{const local=(action:Mutable<Function<int>>,index:int):int[]=>{visits=visits+1;return [action(),index];};console.log(actions.flatMap(local).join('|'));console.log(actions.flatMap(local).join('|'));}console.log(visits);console.log(actions.flatMap(visitor()).join('|'));const empty:(()=>int)[]=[];console.log(empty.flatMap(visitor()).length);}", "7|0|9|1\n7|0|9|1\n4\nvisitor\n7|9\nvisitor\n0\n");
    support::reject("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.flatMap((action:Mutable<Function<int>>):(()=>int)[]=>{return [action];});}", "callback-contract");
}

#[test]
fn empty_callable_families_emit_for_nested_fields_and_function_signatures() {
    execute("type Box={actions:(Once<Function<string>>)[][];};function empty():((input:ReadOnly<string>)=>int)[]{return [];}function count(actions:((input:ReadOnly<string>)=>int)[]):int{return actions.length;}function main(){const box:Box={actions:[]};const actions=empty();console.log(count(actions),box.actions.length,actions.length);}", "0 0 0\n");
}

#[test]
fn callable_predicates_borrow_mutably_and_preserve_short_circuit_order() {
    execute("type State={value:int;};function main(){const first:State={value:0};const second:State={value:10};const actions:(()=>int)[]=[():int=>{first.value=first.value+1;return first.value;},():int=>{second.value=second.value+1;return second.value;}];console.log(actions.some((action:Mutable<Function<int>>):boolean=>{return action()>5;}));console.log(actions.every((action:Mutable<Function<int>>):boolean=>{return action()>5;}));console.log(actions.findIndex((action:Mutable<Function<int>>,index:int):boolean=>{return action()>5;}));console.log(actions.findLastIndex((action:Mutable<Function<int>>,index:int):boolean=>{console.log(index);return action()>5;}));console.log(actions[0](),actions[1]());}", "true\nfalse\n1\n1\n1\n4 14\n");
    execute("function visitor():(action:Mutable<Function<int>>)=>boolean{console.log('visitor');return (action:Mutable<Function<int>>):boolean=>{return action()>5;};}function main(){const actions:(()=>int)[]=[];console.log(actions.some(visitor()));console.log(actions.every((action:Mutable<Function<int>>):boolean=>{return action()>5;}),actions.findIndex((action:Mutable<Function<int>>):boolean=>{return action()>5;}),actions.findLastIndex((action:Mutable<Function<int>>):boolean=>{return action()>5;}));}", "visitor\nfalse\ntrue -1 -1\n");
}

#[test]
fn mutable_callable_predicates_keep_once_and_receiver_loan_guards() {
    for method in ["some", "every", "findIndex", "findLastIndex"] {
        support::reject(&format!("function main(){{const actions:(Once<Function<int>>)[]=[():int=>{{return 7;}}];actions.{method}((action:Mutable<Function<int>>):boolean=>{{return action()>5;}});}}"), "callback-contract");
        support::reject(&format!("function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];actions.{method}((action:Mutable<Function<int>>):boolean=>{{actions.reverse();return action()>5;}});}}"), "conflicting-access");
        support::reject(&format!("function main(){{const actions:(()=>int)[]=[():int=>{{return 7;}}];actions.{method}((action:Mutable<Function<int>>,index:int,source:(()=>int)[]):boolean=>{{return action()>5;}});}}"), "conflicting-access");
    }
}

#[test]
fn callable_array_iteration_lends_mutable_environments_and_preserves_state() {
    execute("type State={value:int;};function main(){const left:State={value:0};const right:State={value:10};const actions:(()=>int)[]=[():int=>{left.value=left.value+1;return left.value;},():int=>{right.value=right.value+2;return right.value;}];actions.forEach((action:Mutable<Function<int>>):void=>{console.log(action());});const values=actions.map((action:Mutable<Function<int>>,index:int):int=>{return action()+index;});console.log(values.join('|'),actions[0](),actions[1]());}", "1\n12\n2|15 3 16\n");
    execute("function visitor():(action:Mutable<Function<int>>)=>int{console.log('visitor');return (action:Mutable<Function<int>>):int=>{return action();};}function main(){const actions:(()=>int)[]=[():int=>{return 7;}];console.log(actions.map(visitor()).join('|'));}", "visitor\n7\n");
    execute("function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];let visits=0;{const visitor=(action:Mutable<Function<int>>,index:int):int=>{visits=visits+1;return action()+index;};console.log(actions.map(visitor).join('|'));console.log(actions.map(visitor).join('|'));}console.log(visits,actions[0]());}", "7|10\n7|10\n4 7\n");
}

#[test]
fn mutable_callable_iteration_rejects_once_elements_receiver_aliases_and_escape() {
    for (source, code) in [
        ("function main(){const actions:(Once<Function<int>>)[]=[():int=>{return 7;}];actions.forEach((action:Mutable<Function<int>>):void=>{action();});}", "callback-contract"),
        ("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.forEach((action:Mutable<Function<int>>):void=>{actions.reverse();action();});}", "conflicting-access"),
        ("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.forEach((action:Mutable<Function<int>>,index:int,source:(()=>int)[]):void=>{action();console.log(source.length);});}", "conflicting-access"),
        ("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.map((action:Mutable<Function<int>>):()=>int=>{return action;});}", "callback-escape"),
        ("function visit(action:Mutable<Function<int>>):int{return action();}function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.map(visit);}", "unsupported"),
    ] {
        support::reject(source, code);
    }
}

#[test]
fn stored_callable_environments_lend_to_mutate_function_parameters() {
    execute("type State={value:int;};function invoke(action:Mutable<Function<int>>):int{return action();}function main(){const state:State={value:0};const actions:(()=>int)[]=[():int=>{state.value=state.value+1;return state.value;}];console.log(invoke(actions[0]),invoke(actions[0]),actions[0]());const moved=actions.remove(0);console.log(invoke(moved),moved());}", "1 2 3\n4 5\n");
    execute("type State={value:int;};function measure(action:Mutable<Function<int, [text:ReadOnly<string>]>>,text:string):int{return action(text);}function main(){const state:State={value:0};const actions:((input:ReadOnly<string>)=>int)[]=[(text:string):int=>{state.value=state.value+1;return state.value+text.length;}];const text='Pat';console.log(measure(actions[0],text),measure(actions[0],text),text);}", "4 5 Pat\n");
}

#[test]
fn stored_callable_borrowing_preserves_access_identity_and_exclusive_loans() {
    for (source, code) in [
        ("function invoke(action:ReadOnly<Function<int>>):int{return action();}function main(){const actions:(()=>int)[]=[():int=>{return 7;}];invoke(actions[0]);}", "callback-contract"),
        ("function invoke(action:Mutable<Function<int>>):int{return action();}function main(){const actions:(Once<Function<int>>)[]=[():int=>{return 7;}];invoke(actions[0]);}", "callback-contract"),
        ("function pair(first:Mutable<Function<int>>,second:Mutable<Function<int>>):int{return first()+second();}function main(){const actions:(()=>int)[]=[():int=>{return 7;}];pair(actions[0],actions[0]);}", "conflicting-access"),
        ("function invoke(action:Mutable<Function<int>>,value:int):int{return action()+value;}function main(){const actions:(()=>int)[]=[():int=>{return 7;}];invoke(actions[0],actions.unshift(():int=>{return 9;}));}", "conflicting-access"),
        ("type User={name:string;};type Other={name:string;};function invoke(action:Mutable<Function<int, [input:ReadOnly<Other>]>>):void{}function main(){const actions:((input:ReadOnly<User>)=>int)[]=[(value:User):int=>{return value.name.length;}];invoke(actions[0]);}", "type"),
        ("function invoke(action:Mutable<Function<int, [input:ReadOnly<string>]>>):void{}function main(){const actions:((input:Owned<string>)=>int)[]=[(value:string):int=>{const moved=value;return moved.length;}];invoke(actions[0]);}", "callback-access"),
    ] {
        support::reject(source, code);
    }
}

#[test]
fn callable_array_zero_parameter_iteration_observes_without_copying_environments() {
    execute("function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];let visits=0;actions.forEach(():void=>{visits=visits+1;});console.log(visits,actions.some(():boolean=>{return true;}),actions.every(():boolean=>{return true;}));let next=0;const values=actions.map(():int=>{next=next+1;return next;});console.log(values.join('|'),actions[0](),actions[1]());const text='Pat';const onceActions:(Once<Function<string>>)[]=[():string=>{return text;}];console.log(onceActions.some(():boolean=>{return true;}));const finish=onceActions.remove(0);console.log(finish());}", "2 true true\n1|2 7 9\ntrue\nPat\n");
}

#[test]
fn callable_array_sort_requires_explicit_comparison_and_preserves_equal_owners() {
    execute("function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];actions.sort(():int=>{return 0;});console.log(actions[0](),actions[1]());const text='Pat';const finishers:(Once<Function<string>>)[]=[():string=>{return text;},():string=>{return 'Sam';}];finishers.sort(():number=>{return Number.NaN;});const first=finishers.remove(0);const second=finishers.remove(0);console.log(first(),second());}", "7 9\nPat Sam\n");
    let error=tsrust_compiler::check("function main(){const actions:(()=>int)[]=[];actions.sort();}").err().expect("default callable sorting");
    assert_eq!(error.code,"type");
    assert!(error.message.contains("explicit sorting comparator"));
    support::reject("function main(){const actions:(()=>int)[]=[];actions.toSorted(():int=>{return 0;});}", "clone-type");
}

#[test]
fn reversing_callable_arrays_moves_environments_without_resetting_state() {
    execute("type State={value:int;};function main(){const left:State={value:0};const right:State={value:10};const actions:(()=>int)[]=[():int=>{left.value=left.value+1;return left.value;},():int=>{right.value=right.value+2;return right.value;}];console.log(actions[0](),actions[1]());actions.reverse();console.log(actions[0](),actions[1]());const text='Pat';const finishers:(Once<Function<string>>)[]=[():string=>{return text;},():string=>{return 'Sam';}];finishers.reverse();const first=finishers.remove(0);const second=finishers.remove(0);console.log(first(),second());}", "1 12\n14 2\nSam Pat\n");
    support::reject("function main(){const actions:((value:int)=>int)[]=[(value:int):int=>{return value;}];actions[0](actions.reverse());}", "conflicting-access");
}

#[test]
fn callable_array_splice_transfers_inserted_and_removed_environments() {
    execute("function main(){const actions:(()=>int)[]=[():int=>{return 7;},():int=>{return 9;}];const removed=actions.splice(0,1,():int=>{return 11;},():int=>{return 13;});console.log(removed[0](),actions[0](),actions[1](),actions[2]());const text='Pat';const onceActions:(Once<Function<string>>)[]=[];onceActions.splice(0,0,():string=>{return text;});const extracted=onceActions.splice(0,1);const finish=extracted.remove(0);console.log(finish(),onceActions.length,extracted.length);}", "7 11 13 9\nPat 0 0\n");
}

#[test]
fn callable_array_splice_keeps_environment_transfer_guards() {
    for (source, code) in [
        ("function main(){let value=0;const action=():int=>{value=value+1;return value;};const actions:(()=>int)[]=[];actions.splice(0,0,action);}", "callback-escape"),
        ("function main(){const action=():int=>{return 7;};const actions:(()=>int)[]=[];actions.splice(0,0,action);action();}", "closure-consumed"),
        ("function main(){const actions:(()=>int)[]=[():int=>{return 7;}];actions.splice(0,0,actions[0]);}", "borrowed-element"),
    ] {
        let error = tsrust_compiler::check(source).err().expect("invalid callable splice");
        assert_eq!(error.code, code, "{error:?}");
    }
}

fn execute(source: &str, expected: &str) -> String {
    let rust = support::execute(source, expected, None);
    let directory = support::TempDirectory::new();
    let path = directory.0.join("optimized.rs");
    std::fs::write(&path, &rust).unwrap();
    let binary = directory.0.join("optimized");
    support::success(&support::run(
        support::rustc()
            .arg("--edition=2024")
            .arg("-Copt-level=3")
            .arg(&path)
            .arg("-o")
            .arg(&binary),
        60,
    ));
    let output = support::run(&mut std::process::Command::new(binary), 10);
    support::success(&output);
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
    rust
}

#[test]
fn source_owned_fields_construct_invoke_and_replace_distinct_origins() {
    execute(
        "type Handler={run:Function<int, [delta:int]>;};function main(){const handler:Handler={run:(delta:int):int=>{return delta+1;}};console.log(handler.run(2));handler.run=(delta:int):int=>{return delta*2;};console.log(handler.run(3));}",
        "3\n6\n",
    );
}

#[test]
fn source_once_fields_own_capture_values_and_restore_after_consumption() {
    execute(
        "type Handler={run:Once<Function<string>>;};function main(){const text=\"Pat\";const handler:Handler={run:():string=>{return text;}};console.log(handler.run());handler.run=():string=>{return \"Sam\";};console.log(handler.run());}",
        "Pat\nSam\n",
    );
}

#[test]
fn generic_callable_fields_resolve_nominal_read_parameters() {
    execute(
        "type User={name:string;};type Handler<T>={run:Function<int, [value:ReadOnly<T>]>;};function main(){const handler:Handler<User>={run:(user:User):int=>{return user.name.length;}};console.log(handler.run({name:\"Pat\"}));}",
        "3\n",
    );
}

#[test]
fn unused_source_callable_fields_need_no_environment_origin() {
    let rust = execute(
        "type Handler={first:Function<int, [count:int]>;second:Once<Function<string>>;};function main(){}",
        "",
    );
    assert!(rust.contains("enum StoredCallable0"));
    assert!(rust.contains("enum StoredCallable1"));
}

#[test]
fn source_field_contract_and_ownership_errors_precede_native_emission() {
    for (source, code) in [
        // A borrowing field is declared freely; a record that holds a borrowing
        // callback still cannot leave the function that owns the borrowed variable.
        (
            "type Handler={run:Mutable<Function<int>>;};function make():Handler{const a=[0];return {run:():int=>{a[0]+=1;return a[0];}};}function main(){}",
            "callback-escape",
        ),
        (
            "type Handler={run:Function<int>;};function main(){const text=\"Pat\";const read=():int=>{return text.length;};const handler:Handler={run:read};}",
            "callback-escape",
        ),
        (
            "type Handler={run:Once<Function<int>>;};function main(){const handler:Handler={run:():int=>{return 7;}};handler.run();handler.run();}",
            "partial-transfer",
        ),
        (
            "type Handler={run:Function<string>;};function main(){const handler:Handler={run:():int=>{return 7;}};}",
            "type",
        ),
        (
            "type Handler={run:Function<int>;};function main(){const handler:Handler={run:():int=>{return 7;}};const copied=clone(handler);}",
            "clone-type",
        ),
        (
            "type Handler={run:Function<int, [action:Function<int>]>;};function main(){}",
            "callable-layout",
        ),
        (
            "type State={action:Function<int>;};function make(state:State):()=>int{return ():int=>{return state.action();};}function main(){const first:State={action:():int=>{return 7;}};const second:State={action:make(first)};}",
            "callable-layout",
        ),
    ] {
        support::reject(source, code);
    }
}

#[test]
fn ownership_quick_fixes_exclude_callable_records_and_nested_containers() {
    for source in [
        "type Handler={run:Function<int>;};function main(){const handler:Handler={run:():int=>{return 7;}};const moved=handler;console.log(handler.run());}",
        "type Handler={run:Function<int>;};type Box={handler:Handler;};function main(){const box:Box={handler:{run:():int=>{return 7;}}};const moved=box;console.log(box.handler.run());}",
        "type Handler={run:Function<int>;};function main(){const handlers:Handler[]=[{run:():int=>{return 7;}}];const moved=handlers;console.log(handlers.length);}",
        "type Handler={run:Function<int>;};function main(){const action=():int=>{const handler:Handler={run:():int=>{return 7;}};const moved=handler;return handler.run();};console.log(action());}",
    ] {
        let error = tsrust_compiler::check(source).err().expect("ownership error");
        assert_eq!(error.code, "transferred", "{source}: {error:?}");
        assert!(error.fixes.is_empty(), "{source}: {error:?}");
        assert!(!error.related.is_empty());
    }
}

#[test]
fn conditional_field_construction_stores_only_the_selected_environment() {
    execute(
        "type Handler={run:Function<int, [value:int]>;};function choose(flag:boolean):Handler{const base=10;return {run:flag?(value:int):int=>{return base+value;}:(value:int):int=>{return value*2;}};}function main(){const first=choose(true);const second=choose(false);console.log(first.run(3),second.run(3));first.run=false?(value:int):int=>{return value+100;}:(value:int):int=>{return value-1;};console.log(first.run(3));}",
        "13 6\n2\n",
    );
}

#[test]
fn conditional_once_fields_join_capture_moves_and_reject_invalid_branches() {
    let source = "type Handler={run:Once<Function<string>>;};function choose(flag:boolean):Handler{const text='Pat';return {run:flag?():string=>{return text;}:false?():string=>{return 'unused';}:():string=>{return text.toUpperCase();}};}function main(){const first=choose(true);const second=choose(false);console.log(first.run(),second.run());}";
    execute(source, "Pat PAT\n");
    for (source, code) in [
        ("type Handler={run:Once<Function<string>>;};function main(){const text='Pat';const handler:Handler={run:true?():string=>{return text;}:():string=>{return 'Sam';}};console.log(text);}", "transferred"),
        ("type Handler={run:Function<int>;};function main(){const handler:Handler={run:true?():int=>{return 1;}:():string=>{return 'bad';}};}", "type"),
        ("type Handler={run:Function<int>;};function main(){const text='Pat';const borrowed=():int=>{return text.length;};const handler:Handler={run:true?():int=>{return 1;}:borrowed};}", "callback-escape"),
    ] {
        let error = tsrust_compiler::check(source).err().expect("conditional ownership error");
        assert_eq!(error.code, code, "{source}: {error:?}");
    }
}

#[test]
fn annotated_callable_arrays_construct_distinct_owned_families_without_clones() {
    let source = "function main() { const first = 7; const second = 9; const actions: (() => int)[] = [(): int => { return first; }, (): int => { return second; }]; console.log(actions.length); }";
    let rust = execute(source, "2\n");
    assert!(rust.contains("Vec<StoredCallable"));
    support::reject(&source.replace("console.log(actions.length);", "const copied = clone(actions);"), "clone-type");
}

#[test]
fn callable_array_index_calls_borrow_independent_mutable_environments() {
    execute(
        "type State = { value: int; }; function main() { const left: State = { value: 0 }; const right: State = { value: 10 }; const actions: (() => int)[] = [(): int => { left.value = left.value + 1; return left.value; }, (): int => { right.value = right.value + 2; return right.value; }]; console.log(actions[0]()); console.log(actions[1]()); console.log(actions[0]()); console.log(actions[1]()); }",
        "1\n12\n2\n14\n",
    );
    execute(
        "function main() { const actions: (() => int)[][] = [[(): int => { return 7; }], [(): int => { return 9; }]]; console.log(actions[1][0]()); console.log(actions[0][0]()); }",
        "9\n7\n",
    );
}

#[test]
fn callable_array_replacement_and_removal_transfer_owned_environments() {
    execute(
        "function main() { const actions: (() => int)[] = [(): int => { return 7; }, (): int => { return 9; }]; actions[0] = (): int => { return 11; }; console.log(actions[0]()); const moved = actions.remove(0); console.log(moved()); console.log(actions[0]()); console.log(actions.length); }",
        "11\n11\n9\n1\n",
    );
    execute(
        "function main() { const text = \"Pat\"; const actions: (Once<Function<string>>)[] = [(): string => { return text; }]; const moved = actions.remove(0); console.log(moved()); console.log(actions.length); }",
        "Pat\n0\n",
    );
}

#[test]
fn callable_array_consuming_index_requires_removal_and_removed_once_is_single_use() {
    let source = "function main() { const actions: (Once<Function<int>>)[] = [(): int => { return 7; }]; console.log(actions[0]()); }";
    let error = support::reject(source, "borrowed-element");
    assert!(error.message.contains("remove()"));
    assert!(!error.message.contains("clone()"));
    support::reject(
        "function main() { const actions: (Once<Function<int>>)[] = [(): int => { return 7; }]; const moved = actions.remove(0); console.log(moved()); console.log(moved()); }",
        "closure-consumed",
    );
}

#[test]
fn callable_array_push_and_unshift_convert_eligible_owned_origins() {
    execute(
        "function main() { const actions: (() => int)[] = []; actions.push((): int => { return 7; }); console.log(actions.unshift((): int => { return 9; }, (): int => { return 11; })); console.log(actions[0]()); console.log(actions[1]()); console.log(actions[2]()); }",
        "3\n9\n11\n7\n",
    );
    execute(
        "function main() { const actions: (Once<Function<string>>)[] = []; const text = \"Pat\"; actions.push((): string => { return text; }); const moved = actions.remove(0); console.log(moved()); }",
        "Pat\n",
    );
}

#[test]
fn callable_array_insertion_and_copying_preserve_ownership_guards() {
    let borrowed = "type State = { value: int; }; function main() { const actions: (() => int)[] = []; const state: State = { value: 0 }; const next = (): int => { state.value = state.value + 1; return state.value; }; actions.push(next); }";
    support::reject(borrowed, "callback-escape");
    support::reject(&borrowed.replace("actions.push(next)", "actions.unshift(next)"), "callback-escape");
    let conflict = "function main() { const actions: (() => int)[] = [(): int => { return 7; }]; actions[0](actions.remove(0)); }";
    // Receiver signature is checked before argument expressions.
    support::reject(conflict, "arity");
    support::reject(
        "function main() { const actions: ((value: int) => int)[] = [(value: int): int => { return value; }]; console.log(actions[0](actions.unshift((value: int): int => { return value + 1; }))); }",
        "conflicting-access",
    );
    for operation in ["clone(actions)", "actions.slice()", "actions.toReversed()", "actions.concat(actions)", "actions.filter((): boolean => { return true; })"] {
        let source = format!("function main() {{ const actions: (() => int)[] = [(): int => {{ return 7; }}]; const copied = {operation}; }}");
        support::reject(&source, "clone-type");
    }
}

#[test]
fn callable_array_owned_captures_drop_on_replacement_and_failed_insertion() {
    let observer = r#"
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicIsize, Ordering};
static LIVE: AtomicIsize = AtomicIsize::new(0);
struct Allocations;
unsafe impl GlobalAlloc for Allocations {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() { LIVE.fetch_add(1, Ordering::SeqCst); }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) };
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}
#[global_allocator] static ALLOCATOR: Allocations = Allocations;
"#;
    for (ending, fails) in [
        ("actions.push((): string => { return third.trim(); }); const removed = actions.remove(0); removed();", false),
        ("actions.unshift((): string => { return third.trim(); }, failure());", true),
        ("const removed = actions.splice(0,1,(): string => { return third.trim(); }); removed[0](); actions[0]();", false),
        ("actions.splice(0,1,(): string => { return third.trim(); }, failure());", true),
        ("const values = actions.map((action: Mutable<Function<string>>): string => { return action(); }); values.length;", false),
        ("const values = actions.flatMap((action: Mutable<Function<string>>): string[] => { return [action(), action()]; }); values.length;", false),
        ("const values = actions.flatMap((action: Mutable<Function<string>>): string[] => { const value = action(); const unused = failure(); return [value]; }); values.length;", true),
        ("const value = actions.reduce((acc: string[], action: Mutable<Function<string>>): string[] => { acc.push(action()); return acc; }, []); value.length;", false),
        ("const value = actions.reduceRight((acc: string[], action: Mutable<Function<string>>): string[] => { acc.push(action()); const unused = failure(); return acc; }, []); value.length;", true),
        ("actions.forEach((action: Mutable<Function<string>>): void => { action(); const unused = failure(); });", true),
        ("actions.some((action: Mutable<Function<string>>): boolean => { return action().length > 0; });", false),
        ("actions.some((action: Mutable<Function<string>>): boolean => { action(); const unused = failure(); return true; });", true),
    ] {
        let source = format!("function failure(): () => string {{ const values = [1]; values.remove(-1); return (): string => {{ return \"unused\"; }}; }} function main() {{ const first = \"first\".repeat(64); const second = \"second\".repeat(64); const third = \"third\".repeat(64); const actions: (() => string)[] = [(): string => {{ return first.trim(); }}]; actions[0] = (): string => {{ return second.trim(); }}; {ending} }}");
        let analysis = tsrust_compiler::check(&source).unwrap();
        let rust = analysis.emit_rust().replace("fn main()", "fn source_main()") + observer
            + &format!(r#"
fn main() {{
    let hook = std::panic::take_hook(); std::panic::set_hook(Box::new(|_| {{}}));
    drop(std::panic::catch_unwind(source_main));
    let baseline = LIVE.load(Ordering::SeqCst);
    for _ in 0..32 {{
        let outcome = std::panic::catch_unwind(source_main);
        assert_eq!(outcome.is_err(), {fails}); drop(outcome);
        assert_eq!(LIVE.load(Ordering::SeqCst), baseline);
    }}
    std::panic::set_hook(hook);
}}
"#);
        let directory = support::TempDirectory::new();
        let file = directory.0.join("array-cleanup.rs");
        std::fs::write(&file, rust).unwrap();
        for level in ["0", "3"] {
            let binary = directory.0.join(format!("cleanup-{level}"));
            support::success(&support::run(support::rustc().arg("--edition=2024").arg(format!("-Copt-level={level}")).arg(&file).arg("-o").arg(&binary), 60));
            support::success(&support::run(&mut std::process::Command::new(binary), 10));
        }
    }
}

#[test]
fn callable_arrays_in_record_fields_preserve_construction_and_indexed_mutation() {
    execute(
        "type Collection = { actions: (() => int)[]; }; function main() { const collection: Collection = { actions: [(): int => { return 7; }] }; console.log(collection.actions[0]()); collection.actions.push((): int => { return 9; }); collection.actions[0] = (): int => { return 11; }; console.log(collection.actions[0]()); const moved = collection.actions.remove(1); console.log(moved()); console.log(collection.actions.length); }",
        "7\n11\n9\n1\n",
    );
}

#[test]
fn generic_record_callable_arrays_keep_nominal_argument_contracts() {
    let source = "type User = { name: string; }; type Collection<T> = { actions: ((value: ReadOnly<T>) => int)[]; }; function main() { const collection: Collection<User> = { actions: [(value: User): int => { return value.name.length; }] }; const user: User = { name: \"Pat\" }; console.log(collection.actions[0](user)); console.log(user.name); }";
    execute(source, "3\nPat\n");
    support::reject(&source.replace("console.log(user.name);", "const copied = clone(collection);"), "clone-type");
}

#[test]
fn callable_arrays_forward_through_function_parameters_and_results() {
    execute(
        "function make(): (() => int)[] { return [(): int => { return 7; }]; } function invoke(actions: (() => int)[]): int { return actions[0](); } function forward(actions: (() => int)[]): (() => int)[] { return actions; } function main() { const actions = make(); console.log(invoke(actions)); const moved = forward(actions); console.log(moved[0]()); }",
        "7\n7\n",
    );
}

#[test]
fn returned_once_callable_arrays_preserve_owned_payload_transfer() {
    execute(
        "function make(text: string): (Once<Function<string>>)[] { return [(): string => { return text; }]; } function take(actions: (Once<Function<string>>)[]): string { const action = actions.remove(0); return action(); } function main() { const actions = make(\"Pat\"); console.log(take(actions)); console.log(actions.length); }",
        "Pat\n0\n",
    );
    support::reject(
        "function make(text: string): (Once<Function<string>>)[] { return [(): string => { return text; }]; } function forward(actions: (Once<Function<string>>)[]): (Once<Function<string>>)[] { return actions; } function main() { const actions = make(\"Pat\"); const moved = forward(actions); console.log(actions.length); }",
        "transferred",
    );
}
