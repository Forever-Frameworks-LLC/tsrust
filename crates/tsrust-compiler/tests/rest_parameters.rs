mod support;
use tsrust_compiler::check;

fn recorded(
    directory: &std::path::Path,
    label: &str,
    command: &mut std::process::Command,
    timeout: u64,
) -> std::process::Output {
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    let stamp = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    };
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|value| format!("{:?}", value.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(",");
    let start = stamp();
    let elapsed = Instant::now();
    let stdout = directory.join(format!("{label}.stdout"));
    let stderr = directory.join(format!("{label}.stderr"));
    command
        .stdout(std::fs::File::create(&stdout).unwrap())
        .stderr(std::fs::File::create(&stderr).unwrap());
    let mut child = command.spawn().unwrap();
    let pid = child.id();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if elapsed.elapsed() > Duration::from_secs(timeout) {
            child.kill().unwrap();
            panic!("native child timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    std::fs::write(directory.join(format!("{label}.json")),format!("{{\"argv\":[{argv}],\"pid\":{pid},\"startUtcEpochMillis\":{start},\"endUtcEpochMillis\":{},\"exitCode\":{}}}\n",stamp(),status.code().map_or("null".into(),|value|value.to_string()))).unwrap();
    std::process::Output {
        status,
        stdout: std::fs::read(stdout).unwrap(),
        stderr: std::fs::read(stderr).unwrap(),
    }
}

fn native(source: &str, expected: &str) {
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let persistent = std::env::var_os("TSR_GATE99_NATIVE_DIR");
    let temporary = persistent.is_none().then(support::TempDirectory::new);
    let directory = if let Some(root) = persistent {
        let root = std::path::PathBuf::from(root);
        let case = CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if case == 0 {
            root
        } else {
            root.join(format!("case-{case}"))
        }
    } else {
        temporary.as_ref().unwrap().0.clone()
    };
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("input.tsr"), source).unwrap();
    std::fs::write(directory.join("expected.txt"), expected).unwrap();
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    std::fs::write(directory.join("generated.rs"), &rust).unwrap();
    assert!(!rust.contains("unsafe"));
    if std::env::var_os("TSR_GATE99_EMIT_ONLY").is_some() {
        return;
    }
    for level in ["0", "3"] {
        let path = directory.join(format!("native-O{level}"));
        let build = recorded(
            &directory,
            &format!("build-O{level}"),
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(directory.join("generated.rs"))
                .arg("-o")
                .arg(&path),
            60,
        );
        std::fs::write(
            directory.join(format!("build-O{level}-stdout.txt")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-O{level}-stderr.txt")),
            &build.stderr,
        )
        .unwrap();
        support::success(&build);
        let output = recorded(
            &directory,
            &format!("run-O{level}"),
            &mut std::process::Command::new(path),
            15,
        );
        std::fs::write(
            directory.join(format!("stdout-O{level}.txt")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("stderr-O{level}.txt")),
            &output.stderr,
        )
        .unwrap();
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn rest_data_packs_are_owned_ordered_and_typed() {
    native(
        r#"type Item={value:int;};
function ints(...values:int[]):int{return values.reduce((total:int,value:int):int=>{return total+value;},0);}
function text(...values:string[]):int{return values[0].length+values[1].length;}
function nested(...values:int[][]):int{return values[0].length+values[1][0];}
function records(...values:Item[]):int{return values[0].value+values[1].value;}
function number(...values:number[]):number{return values[0]+values[1];}
function boolean(...values:boolean[]):boolean{return values[0]&&values[1];}
function nil(...values:null[]):int{return values.length;}
function absent(...values:undefined[]):int{return values.length;}
function main(){console.log(ints(),ints(4),ints(1,2,3));const a="é".repeat(1);const b="x".repeat(2);console.log(text(a,b));console.log(nested([1,2],[7]));const first:Item={value:3};const second:Item={value:5};console.log(records(first,second));console.log(number(1,2),boolean(true,false),nil(null,null),absent(undefined));}"#,
        "0 4 6\n4\n9\n8\n3 false 2 1\n",
    );
}

#[test]
fn rest_generic_and_callable_dispatch_keeps_one_packed_contract() {
    native(
        r#"type Item<T>={value:T;};
function first<T>(...values:T[]):T{return values.remove(0);}
function apply(action:ReadOnly<Function<int, [...values:int[]]>>):int{return action()+action(2,3);}
function change(action:Mutable<Function<int, [...values:int[]]>>):int{return action(4)+action(1,2);}
function once(action:Once<Function<int, [...values:int[]]>>):int{return action(5,6);}
function factory():(...values:int[])=>int{return (...values:int[]):int=>{return values.length;};}
type Handler={run:(...values:int[])=>int;};
function main(){console.log(first<int>(3,4),first("row","other"));const item:Item<int>={value:8};console.log(first<Item<int>>(item).value);
const action=(...values:int[]):int=>{return values.length;};console.log(action(),action(1,2,3),apply(action));
let state=0;const changing=(...values:int[]):int=>{state++;return values.length+state;};console.log(change(changing),changing());
const returned=factory();console.log(once((...values:int[]):int=>{return values.length;}),returned(1,2,3));
const holder:Handler={run:(...values:int[]):int=>{return values.length+4;}};console.log(holder.run(),holder.run(1,2));
const nullable:(Function<int, [...values:int[]]>)|null=(...values:int[]):int=>{return values.length;};console.log(nullable?.(1,2)??-1);
}"#,
        "3 row\n8\n0 3 2\n6 3\n2 3\n4 6\n2\n",
    );
}

#[test]
fn rest_to_fixed_adapters_preserve_environment_and_element_access() {
    native(
        r#"function size(...values:int[]):int{return values.length;}
function front(first:int=10,...values:int[]):int{return first+values.length;}
function apply(action:ReadOnly<Function<int, [left:int,right:int]>>):int{return action(2,3);}
function consume(action:Function<int, [left:Owned<string>,right:Owned<string>]>):int{return action("é".repeat(1),"x".repeat(3));}
type Holder={run:(left:int,right:int)=>int;};
function main(){console.log(apply(size));const action:Function<int, [left:int,right:int]>=size;console.log(action(4,5));
const holder:Holder={run:size};console.log(holder.run(1,2));
let state=0;const source=(...values:int[]):int=>{state++;return values.length+state;};
const changing:Mutable<Function<int, [left:int,right:int]>>=source;console.log(changing(1,2),changing(3,4));
console.log(consume((...values:string[]):int=>{return values[0].length+values[1].length;}));
console.log(apply(front));}"#,
        "2\n2\n2\n3 4\n5\n3\n",
    );
}

fn rejected(source: &str, code: &str, message: &str) {
    let error = check(source).err().expect("source must reject");
    assert_eq!(error.code, code, "{}", error.render("input.tsr", source));
    assert!(
        error.message.contains(message),
        "{}",
        error.render("input.tsr", source)
    );
    assert!(error.span.end <= source.len());
    assert!(error.span.start < error.span.end);
}

#[test]
fn rest_grammar_arity_and_transfers_preserve_existing_rejections() {
    rejected(
        "function use(...values:int):int{return values;}function main(){}",
        "rest-type",
        "homogeneous array",
    );
    rejected(
        "function use(...values?:int[]):int{return values.length;}function main(){}",
        "rest-parameter",
        "optional",
    );
    rejected(
        "function use(...values:int[]=[]):int{return values.length;}function main(){}",
        "rest-parameter",
        "default",
    );
    rejected(
        "function use(...values:int[],other:int):int{return other;}function main(){}",
        "rest-parameter",
        "last",
    );
    rejected(
        "function use(action:ReadOnly<Function<int, [...values:ReadOnly<int[]>]>>):int{return 1;}function main(){}",
        "rest-access",
        "owns its pack",
    );
    rejected(
        "function use(first:int,...values:int[]):int{return first;}function main(){use();}",
        "arity",
        "arguments",
    );
    rejected(
        "function use(...values:int[]):int{return values.length;}function main(){use(1,undefined);}",
        "type",
        "Expected int",
    );
    rejected(
        "function use(...values:string[]):int{return values.length;}function main(){const text=\"x\".repeat(2);use(text);console.log(text);}",
        "transferred",
        "transferred",
    );
    rejected(
        "function use(first:string,...values:string[]):int{return first.length+values.length;}function main(){const text=\"x\".repeat(2);use(text,text);}",
        "conflicting-access",
        "access",
    );
    rejected(
        "function use(values:int[]):int{return values.length;}function main(){const action:Function<int, [...values:int[]]>=use;}",
        "callback-contract",
        "rest contracts",
    );
    rejected(
        "function use(...values:string[]):int{return values.length;}function main(){const action:ReadOnly<Function<int, [value:ReadOnly<string>]>>=use;}",
        "callback-access",
        "transfer owned",
    );
    rejected(
        "function use<T>(...values:T[]):int{return values.length;}function main(){use();}",
        "generic-inference",
        "infer",
    );
}

#[test]
fn rest_normal_depth_and_over_limit_keep_the_original_worker_stack() {
    let expression = format!("{}1{}", "head(".repeat(100), ")".repeat(100));
    let source = format!(
        "function head(...values:int[]):int{{return values[0];}}function main(){{console.log({expression});}}"
    );
    native(&source, "1\n");
    let expression = format!("{}1{}", "head(".repeat(200), ")".repeat(200));
    let source = format!(
        "function head(...values:int[]):int{{return values[0];}}function main(){{console.log({expression});}}"
    );
    support::accept_deep(&source);
}

#[test]
fn rest_fixed_optional_adapters_preserve_omission_and_explicit_undefined() {
    native(
        r#"function size(...values:undefined[]):int{return values.length;}
function read(action:ReadOnly<Function<int, [value?:undefined]>>):int{return action()*10+action(undefined);}
function mutate(action:Mutable<Function<int, [value?:undefined]>>):int{return action()*10+action(undefined);}
function once(action:Once<Function<int, [value?:undefined]>>):int{return action();}
function defaulted(action:ReadOnly<Function<int, [value?:undefined]>>=size):int{return action()*10+action(undefined);}
type Holder={run:(value?:undefined)=>int;};
function main(){console.log(size(),size(undefined),read(size));
const direct:Function<int, [value?:undefined]>=size;console.log(direct(),direct(undefined));
const holder:Holder={run:size};console.log(holder.run(),holder.run(undefined));
let state=0;const source=(...values:undefined[]):int=>{state++;return values.length+state;};console.log(mutate(source));
console.log(once((...values:undefined[]):int=>{return values.length;}),defaulted());
const nullable:(Function<int, [value?:undefined]>)|null=size;console.log(nullable?.()??-1,nullable?.(undefined)??-1);
}"#,
        "0 1 1\n0 1\n0 1\n13\n0 1\n0 1\n",
    );
}

#[test]
fn rest_optional_adapter_array_callbacks_use_the_real_supplied_tuple() {
    native(
        r#"function mapped(value:undefined,index:int,source:undefined[],...tail:undefined[]):int{return tail.length;}
function reduced(total:int,value:undefined,index:int,source:undefined[],...tail:undefined[]):int{return total+tail.length;}
function main(){const values:undefined[]=[undefined,undefined];
const mapper:Function<int, [value:undefined,index:int,source:ReadOnly<undefined[]>,extra?:undefined]>=mapped;
const reducer:Function<int, [total:int,value:undefined,index:int,source:ReadOnly<undefined[]>,extra?:undefined]>=reduced;
console.log(values.map(mapper).join(","),values.reduce(reducer,0));
const source=(left:int,right:int,...tail:undefined[]):int=>{return left-right+tail.length*100;};
const comparator:Function<int, [left:int,right:int,extra?:undefined]>=source;
const items=[3,1,2];const copy=items.toSorted(comparator);items.sort(comparator);console.log(copy.join(","),items.join(","));
console.log(comparator(1,2,undefined));
}"#,
        "0,0 0\n1,2,3 1,2,3\n99\n",
    );
}

#[test]
fn rest_array_callbacks_pack_only_the_real_method_tuple() {
    native(
        r#"function main() {
    const values = [1, 2, 3];
    const mapped = values.map((value: int, index: int, source: int[], ...extra: int[]): int => {
        return value + index + source.length + extra.length;
    });
    const flattened = values.flatMap((value: int, index: int, source: int[], ...extra: int[]): int[] => {
        return [value, extra.length];
    });
    const filtered = values.filter((value: int, index: int, source: int[], ...extra: int[]): boolean => {
        return value > 1 && extra.length === 0;
    });
    console.log(mapped.join(","), flattened.join(","), filtered.join(","));
    console.log(values.every((value: int, index: int, source: int[], ...extra: int[]): boolean => {
        return value > 0 && extra.length === 0;
    }), values.some((value: int, index: int, source: int[], ...extra: int[]): boolean => {
        return value === 2 && extra.length === 0;
    }));
    console.log(String(values.find((value: int, index: int, source: int[], ...extra: int[]): boolean => {
        return value > 1 && extra.length === 0;
    })), String(values.findLast((value: int, index: int, source: int[], ...extra: int[]): boolean => {
        return value > 1 && extra.length === 0;
    })));
    console.log(values.findIndex((value: int, index: int, source: int[], ...extra: int[]): boolean => {
        return value > 1 && extra.length === 0;
    }), values.findLastIndex((value: int, index: int, source: int[], ...extra: int[]): boolean => {
        return value > 1 && extra.length === 0;
    }));
    console.log(values.reduce((total: int, value: int, index: int, source: int[], ...extra: int[]): int => {
        return total * 10 + value + index + extra.length;
    }, 0), values.reduceRight((total: int, value: int, index: int, source: int[], ...extra: int[]): int => {
        return total * 10 + value + index + extra.length;
    }, 0));
    let visited = 0;
    values.forEach((value: int, index: int, source: int[], ...extra: int[]): int => {
        visited += value + extra.length;
        return 0;
    });
    console.log(visited);
    const copied = [3, 1, 2].toSorted((...pair: int[]): int => {
        return pair[0] - pair[1];
    });
    let mutable = [3, 1, 2];
    mutable.sort((left: int, ...right: int[]): int => {
        return left - right[0];
    });
    console.log(copied.join(","), mutable.join(","));
    const padded = values.map((value: int, index: int, source: int[], fallback: int = 7, ...extra: int[]): int => { return value + fallback + extra.length; });
    console.log(padded.join(","));
}
"#,
        "4,6,8 1,0,2,0,3,0 2,3\ntrue true\n2 3\n1 2\n135 531\n6\n1,2,3 1,2,3\n8,9,10\n",
    );
}

#[test]
fn rest_callable_elements_preserve_the_protected_array_visitor_tuple() {
    native(
        r#"function counter(initial: int): () => int {
    let count = initial;
    return (): int => { count += 1; return count; };
}
function main() {
    const actions: (Function<int>)[] = [counter(2), counter(5)];
    const result = actions.map((action: Mutable<Function<int>>, ...indices: int[]): int => {
        return action() + indices[0];
    });
    console.log(result[0], result[1]);
    console.log(actions.reduce((total: int, action: Mutable<Function<int>>, ...indices: int[]): int => {
        return total + action() + indices[0];
    }, 0));
}
"#,
        "3 7\n12\n",
    );
}

#[test]
fn rest_array_tuple_type_and_borrow_rejections_remain_precise() {
    rejected(
        "function main(){const values=[1,2];values.map((value:int,index:int,...source:int[][]):int=>{return source.length;});}\n",
        "callback-access",
        "borrowed element or source array",
    );
    rejected(
        "function main(){const values=[\"b\",\"a\"];values.toSorted((...parts:string[]):int=>{return 0;});}\n",
        "callback-access",
        "borrowed element or source array",
    );
    rejected(
        "function main(){const values=[1,2];values.map((...parts:int[]):int=>{return parts.length;});}\n",
        "type",
        "Expected int, found int[]",
    );
    rejected(
        "function main(){const values=[1,2];values.map((value:int,index:int,source:int[],required:int,...extra:int[]):int=>{return required;});}\n",
        "arity",
        "required callback position",
    );
}

#[test]
fn rest_returned_packs_captures_and_owned_callable_elements_keep_ownership() {
    native(
        r#"type Item={text:string;};
function returning<T>(...values:T[]):T[]{return values;}
function capture(...values:string[]):()=>int{return ():int=>{return values[0].length;};}
function factory():(Function<int, [...values:string[]]>)|null{return (...values:string[]):int=>{return values[0].length;};}
function once(action:Once<Function<int, [...values:string[]]>>):int{return action("é".repeat(2));}
function stored(...actions:(Function<int, [value:int]>)[]):int{return actions[0](2)+actions[1](3);}
function main(){console.log(returning<string>("x".repeat(1),"é").join(","));
const first:Item={text:"a".repeat(2)};const second:Item={text:"b".repeat(3)};const records=returning<Item>(first,second);console.log(records[0].text.length+records[1].text.length);
const action=capture("é".repeat(2));console.log(action());
const nullable=factory();console.log(nullable?.("z".repeat(3))??-1);
console.log(once((...values:string[]):int=>{return values[0].length;}));
console.log(stored((value:int):int=>{return value+1;},(value:int):int=>{return value+2;}));}
"#,
        "x,é\n5\n4\n3\n4\n8\n",
    );
}

#[test]
fn rest_abstract_forwarding_keeps_native_generic_capture_access() {
    native(
        r#"function size(...values:int[]):int{return values.length;}
function use(action:ReadOnly<Function<int, [value:int]>>):int{return action(3);}
function forward(action:ReadOnly<Function<int, [...values:int[]]>>):int{return use(action);}
function mutateUse(action:Mutable<Function<int, [value:int]>>):int{return action(4);}
function mutateForward(action:Mutable<Function<int, [...values:int[]]>>):int{return mutateUse(action);}
function onceUse(action:Once<Function<int, [value:int]>>):int{return action(5);}
function onceForward(action:Once<Function<int, [...values:int[]]>>):int{return onceUse(action);}
function main(){console.log(forward(size));let state=0;const change=(...values:int[]):int=>{state++;return state+values.length;};console.log(mutateForward(change));console.log(onceForward((...values:int[]):int=>{return values.length;}));const empty:Function<int>=size;console.log(empty());}
"#,
        "1\n2\n1\n0\n",
    );
}

#[test]
fn rest_abstract_owned_adapters_return_store_and_preserve_absent_counts() {
    native(
        r#"
type Handler={run:Function<int, [value:int]>;};
function adapt(action:Function<int, [...values:int[]]>):Function<int, [value:int]>{return action;}
function make(n:int):Handler{return {run:adapt((...values:int[]):int=>{return n+values.length;})};}
function size(...values:undefined[]):int{return values.length;}
function use(action:ReadOnly<Function<int, [value?:undefined]>>):int{return action()+action(undefined)*10;}
function forward(action:ReadOnly<Function<int, [...values:undefined[]]>>):int{return use(action);}
function choose(flag:boolean):Function<int, [value:int]>{return adapt((...values:int[]):int=>{return values.length+(flag?0:4);});}
function main(){const returned=adapt((...values:int[]):int=>{return values.length;});console.log(returned(2));const handler=make(7);console.log(handler.run(3));console.log(forward(size));const selected=choose(false);console.log(selected(1));}
"#,
        "1\n8\n10\n5\n",
    );
}

#[test]
fn rest_abstract_forwarding_does_not_widen_ownership_or_escape() {
    let source = "function onceUse(action:Once<Function<int, [value:int]>>):int{return action(5);} function forward(action:Function<int, [...values:int[]]>):int{return onceUse(action);} function main(){}";
    let error = check(source).err().expect("source must reject");
    assert_eq!(error.code, "callback-contract");
    assert!(error.message.contains("cannot be interchanged"));
    let source = "function forward(action:ReadOnly<Function<int, [...values:int[]]>>):Function<int, [value:int]>{return action;} function main(){}";
    let error = check(source).err().expect("source must reject");
    assert_eq!(error.code, "callback-escape");
    assert!(error.message.contains("owned callback"));
}

#[test]
fn rest_nested_callable_prefix_adaptation_preserves_checked_signature() {
    native(
        r#"
function use(action:ReadOnly<Function<int, [inner:ReadOnly<Function<int>>,value:int]>>,inner:ReadOnly<Function<int>>):int{return action(inner,2);}
function forward(action:ReadOnly<Function<int, [inner:ReadOnly<Function<int>>,...values:int[]]>>,inner:ReadOnly<Function<int>>):int{return use(action,inner);}
function main(){console.log(forward((inner:ReadOnly<Function<int>>,...values:int[]):int=>{return inner()+values.length;},():int=>{return 7;}));}
"#,
        "8\n",
    );
}

#[test]
fn rest_nested_callable_prefix_preserves_optional_undefined_count() {
    native(
        r#"
function use(action:ReadOnly<Function<int, [inner:ReadOnly<Function<int>>,value?:undefined]>>,inner:ReadOnly<Function<int>>):int{return action(inner)+action(inner,undefined)*10;}
function forward(action:ReadOnly<Function<int, [inner:ReadOnly<Function<int>>,...values:undefined[]]>>,inner:ReadOnly<Function<int>>):int{return use(action,inner);}
function main(){console.log(forward((inner:ReadOnly<Function<int>>,...values:undefined[]):int=>{return inner()+values.length;},():int=>{return 2;}));}
"#,
        "32\n",
    );
}

#[test]
fn rest_abstract_read_prefix_preserves_borrow_lifetimes() {
    native(
        r#"
function use(action:ReadOnly<Function<int, [text:ReadOnly<string>,value:int]>>,text:string):int{return action(text,2);}
function forward(action:ReadOnly<Function<int, [text:ReadOnly<string>,...values:int[]]>>,text:string):int{return use(action,text);}
function main(){console.log(forward((text:string,...values:int[]):int=>{return text.length+values.length;},"x"));}
"#,
        "2\n",
    );
}

#[test]
fn rest_materialized_callable_array_visitor_preserves_actual_count() {
    native(
        r#"
function counter(initial:int):()=>int{let count=initial;return ():int=>{count++;return count;};}
function main(){const actions:(Function<int>)[]=[counter(2),counter(5)];const visitor:ReadOnly<Function<int, [action:Mutable<Function<int>>,index:int,extra?:undefined]>>=(action:Mutable<Function<int>>,index:int,...tail:undefined[]):int=>{return action()+index+tail.length*100;};const result=actions.map(visitor);console.log(result[0],result[1]);console.log(visitor(actions[0],0,undefined));}
"#,
        "3 7\n104\n",
    );
}

#[test]
fn rest_callable_visitors_still_reject_required_extra_positions() {
    let prefix = "function counter(initial:int):()=>int{let count=initial;return ():int=>{count++;return count;};} function main(){const actions:(Function<int>)[]=[counter(2),counter(5)];";
    rejected(
        &format!(
            "{prefix}actions.map((action:Mutable<Function<int>>,index:int,required:undefined):int=>{{return action()+index;}});}}"
        ),
        "conflicting-access",
        "mutable callable element",
    );
    rejected(
        &format!(
            "{prefix}actions.reduce((total:int,action:Mutable<Function<int>>,index:int,required:undefined):int=>{{return total+action()+index;}},0);}}"
        ),
        "conflicting-access",
        "mutable callable element",
    );
}
