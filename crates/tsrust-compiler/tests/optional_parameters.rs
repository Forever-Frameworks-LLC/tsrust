mod support;
use tsrust_compiler::check;

fn native(source: &str, expected: &str) {
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let persistent = std::env::var_os("TSR_GATE98_NATIVE_DIR");
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
    for level in ["0", "3"] {
        let path = directory.join(format!("native-O{level}"));
        let build = support::run(
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
        let output = support::run(&mut std::process::Command::new(path), 15);
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
fn optional_and_default_data_parameters_initialize_lazily_with_native_ownership() {
    native(
        r#"type Item={value:int;};
function optional(value?:int):int{return value??9;}
function initialized(value:int=7):int{return value;}
function front(first:int=5,required:int):int{return first*10+required;}
function number(value:number=1):number{return value;}
function nullable(value:boolean|null=false):string{return String(value);}
function text(value:string="é".repeat(2)):int{return value.length;}
function array(value:int[]=[1,2]):int{return value.length;}
function record(value:Item={value:8}):int{return value.value;}
function main(){
console.log(optional(),optional(undefined),optional(0));
console.log(initialized(),initialized(undefined),initialized(3));
console.log(front(undefined,2),front(3,2));
console.log(number(),number(2));
console.log(nullable(),nullable(null),nullable(true));
const source="x".repeat(3);console.log(text(),text(source),source);
const values=[4,5,6];console.log(array(),array(values),values.length);
const item:Item={value:4};console.log(record(),record(item),item.value);
const action=(value=7):int=>{return value+1;};console.log(action(),action(2));
}"#,
        "9 9 0\n7 7 3\n52 32\n1 2\nfalse null true\n4 3 xxx\n2 3 3\n8 4 4\n8 3\n",
    );
}

#[test]
fn optional_default_callable_parameters_preserve_read_mutate_once_and_owned_access() {
    native(
        r#"type Trace={calls:int;};
function step(value:int,trace:Trace):int{trace.calls++;return value+1;}
function useRead(trace:Trace,fallback:ReadOnly<Function<int, [value:int,trace:Mutable<Trace>]>>,action:ReadOnly<Function<int, [value:int,trace:Mutable<Trace>]>>=fallback):int{return action(4,trace);}
function optionalRead(trace:Trace,action?:ReadOnly<Function<int, [value:int,trace:Mutable<Trace>]>>):int{return action?.(5,trace)??-1;}
function useMutate(trace:Trace,fallback:Mutable<Function<int, [value:int,trace:Mutable<Trace>]>>,action:Mutable<Function<int, [value:int,trace:Mutable<Trace>]>>=fallback):int{return action(6,trace);}
function make(): (value:int)=>int {return (value:int):int=>{return value+10;};}
function useOwned(action:Function<int, [value:int]>=make()):int{return action(3);}
function useOnce(action:Once<Function<int, [value:int]>>=make()):int{return action(2);}
function useOwnedOnce(action:Once<Function<int, [value:int]>>=make()):int{return action(1);}
function main(){const trace:Trace={calls:0};const read=(value:int,target:Trace):int=>{target.calls++;return value+1;};
console.log(useRead(trace,read),useRead(trace,read,undefined),optionalRead(trace),optionalRead(trace,read),read(1,trace));
let n=0;const changing=(value:int,target:Trace):int=>{n++;target.calls++;return value+n;};
console.log(useMutate(trace,changing),changing(3,trace));
console.log(useOwned(),useOwned(make()),useOnce(),useOwnedOnce());console.log(trace.calls);
}"#,
        "5 5 -1 6 2\n7 5\n13 13 12 11\n6\n",
    );
}

#[test]
fn optional_mutable_data_slots_borrow_payloads_and_default_owners() {
    native(
        r#"type Item={value:int;};
function array(value:int[]=[1]):int{value.push(7);return value.length;}
function record(value:Item={value:2}):int{value.value++;return value.value;}
function optional(value?:int[]):int{if(value!==undefined){value.push(8);return value.length;}return -1;}
function front(fallback:int[],value:int[]=fallback):int{value.push(9);return value.length;}
function main(){const values=[4,5];console.log(array(),array(values),values.length);const item:Item={value:3};console.log(record(),record(item),item.value);console.log(optional(),optional(values),values.length);const fallback=[2];console.log(front(fallback),fallback.length);}
"#,
        "2 3 3\n3 4 4\n-1 4 4\n2 2\n",
    );
}

#[test]
fn owning_default_callbacks_can_return_and_capture_selected_owners() {
    native(
        r#"function make():()=>int{const text="é".repeat(2);return ():int=>{return text.length;};}
function choose(action:Function<int>=make()):()=>int{return action;}
function capture(action:Function<int>=make()):()=>int{return ():int=>{return action();};}
function other():()=>int{return ():int=>{return 9;};}
function main(){const first=choose();const second=choose(other());console.log(first(),second());const a=capture();const b=capture(other());console.log(a(),b());}
"#,
        "4 9\n4 9\n",
    );
}

#[test]
fn default_callable_forwarding_preserves_factories_once_results_and_field_arity() {
    native(
        r#"type Handler={run:Function<int, [value?:int]>;};
function make():()=>int{const text="row".repeat(1);return ():int=>{return text.length;};}
function choose(action:Function<int>=make()):()=>int{return action;}
function forward(action:Function<int>):()=>int{return choose(action);}
function consumeText(value:string):string{return value;}
function makeOnce():Once<Function<int>>{const text="é".repeat(2);return ():int=>{return consumeText(text).length;};}
function chooseOnce(action:Once<Function<int>>=makeOnce()):Once<Function<int>>{return action;}
function initialized(value:int=7):int{return value+1;}
function main(){const first=forward(make());const second=choose(undefined);console.log(first(),second());const third=chooseOnce();const fourth=chooseOnce(makeOnce());console.log(third(),fourth());const item:Handler={run:initialized};console.log(item.run(),item.run(2),item.run?.(undefined));}
"#,
        "3 3\n4 4\n8 3 8\n",
    );
}

#[test]
fn owning_arrow_defaults_specialize_supplied_slots_without_losing_selected_owners() {
    native(
        r#"function make():()=>int{const text="x".repeat(3);return ():int=>{return text.length;};}
function other():()=>int{return ():int=>{return 8;};}
function main(){const choose=(action:Function<int>=make()):()=>int=>{return action;};const first=choose();const second=choose(other());console.log(first(),second());const selected=(action:Function<int>=make()):int=>{return action();};console.log(selected(undefined),selected(other()));}
"#,
        "3 8\n3 8\n",
    );
}

#[test]
fn nullable_default_data_arguments_preserve_present_null_and_caller_tags() {
    native(
        r#"type Item={value:int;};
function text(value:string|null="default"):string{return String(value);}
function array(value:int[]|null=[1]):int{return value?.length??-1;}
function record(value:Item|null={value:9}):int{return value?.value??-1;}
function main(){const label:string|null="é".repeat(1);console.log(text(label),String(label),text(null),text(undefined));const values:int[]|null=[2,3];console.log(array(values),String(values),array(null),array());const item:Item|null={value:4};console.log(record(item),String(item),record(null),record());}
"#,
        "é é null default\n2 2,3 -1 1\n4 [object Object] -1 9\n",
    );
}

#[test]
fn nullable_owning_defaults_keep_null_and_move_selected_callable_family() {
    native(
        r#"function make():(Function<int>)|null{return ():int=>{return 6;};}
function other():()=>int{return ():int=>{return 9;};}
function use(action:(Function<int>)|null=make()):int{return action?.()??-1;}
function choose(action:(Function<int>)|null=make()):(Function<int>)|null{return action;}
function main(){console.log(use(),use(undefined),use(null),use(other()));const first=choose();const second=choose(other());const absent=choose(null);console.log(first?.()??-1,second?.()??-1,typeof absent);}
"#,
        "6 6 -1 9\n6 9 object\n",
    );
}

#[test]
fn optional_mutate_absent_arguments_and_nested_optional_callable_types() {
    native(
        r#"function apply(action?:Mutable<Function<int>>):int{return action?.()??-1;}
function step(value:int=4):int{return value+1;}
function main(){let state=0;const callback=():int=>{state++;return state;};console.log(apply(undefined),apply(),apply(callback),callback());const actions:((value?:int)=>int)[]=[step];console.log(actions[0](),actions[0](2));const selected:((value?:int)=>int)|null=step;if(selected!==null){console.log(selected());}}
"#,
        "-1 -1 1 2\n5 3\n5\n",
    );
}

#[test]
fn optional_parameter_normal_depth_keeps_existing_128_limit() {
    let source = format!(
        "function id(value:int):int{{return value;}}function use(value:int={}1{}):int{{return value;}}function main(){{console.log(use(),use(undefined),use(2));}}",
        "id(".repeat(100),
        ")".repeat(100)
    );
    native(&source, "1 1 2\n");
    let source = format!(
        "function id(value:int):int{{return value;}}function use(value:int={}1{}):int{{return value;}}function main(){{use();}}",
        "id(".repeat(200),
        ")".repeat(200)
    );
    support::accept_deep(&source);
}

#[test]
fn optional_parameter_scope_arity_and_once_diagnostics_are_precise() {
    for (source, code, message, start, end) in [
        (
            "function use(first?:int,second:int):int{return second;}function main(){}",
            "parameter-order",
            "required parameter cannot follow",
            24,
            30,
        ),
        (
            "function use(value?:int=7):int{return value;}function main(){}",
            "parameter-initializer",
            "cannot also have a default",
            23,
            24,
        ),
        (
            "function use(first:int=second,second:int=7):int{return first;}function main(){}",
            "name",
            "Unknown value",
            23,
            29,
        ),
        (
            "function use(value:int=local):int{const local=7;return value;}function main(){}",
            "name",
            "Unknown value",
            23,
            28,
        ),
        (
            "function front(first:int=5,required:int):int{return first+required;}function main(){front();}",
            "arity",
            "Expected 2..=2",
            84,
            91,
        ),
        (
            "function use<T>(value:T=7):T{return value;}function main(){use<int>();}",
            "type",
            "unconstrained generic parameter",
            24,
            25,
        ),
        (
            "function use(value:int=7):int{return value;}function main(){use(null);}",
            "type",
            "Expected int | undefined, found null",
            64,
            68,
        ),
        (
            "function use(action:Once<Function<int>>=():int=>{return 1;}):int{return action()+action();}function main(){use();}",
            "callback-consumed",
            "no longer available",
            81,
            87,
        ),
    ] {
        let error = check(source).err().expect("precise expected diagnostic");
        assert_eq!(error.code, code, "{}", error.render("negative.tsr", source));
        assert!(
            error.message.contains(message),
            "{}",
            error.render("negative.tsr", source)
        );
        assert_eq!(
            (error.span.start, error.span.end),
            (start, end),
            "{}",
            error.render("negative.tsr", source)
        );
    }
}

#[test]
fn generic_defaults_reject_known_incompatible_expression_types() {
    for (prefix, expression, annotation) in [
        ("", "7", "T"),
        ("function seed():int{return 7;}", "seed()", "T"),
        ("", "1+2", "T"),
        ("", "String(7)", "T"),
        ("", "Number(7)", "T"),
        ("", "Math.abs(7)", "T"),
        ("", "\"row\".length", "T"),
        ("", "[7]", "T[]"),
        ("function seed():int{return 7;}", "seed", "()=>T"),
    ] {
        let source = format!(
            "{prefix}function use<T>(value:{annotation}={expression}):int{{return 1;}}function main(){{}}"
        );
        let error = check(&source)
            .err()
            .expect("declaration rejects unconstrained concrete default even unused");
        assert_eq!(
            error.code,
            "type",
            "{}",
            error.render("generic.tsr", &source)
        );
        assert!(
            error.message.contains("unconstrained generic parameter"),
            "{}",
            error.message
        );
        let start = source.find(&format!("={expression}")).unwrap() + 1;
        assert_eq!(
            (error.span.start, error.span.end),
            (start, start + expression.len())
        );
    }
    native(
        r#"type Item={value:int;};type Box<T>={value:T;};
function id<T>(value:T):T{return value;}
function choose<T>(fallback:T,value:T=clone(fallback)):T{return value;}
function relay<T>(fallback:T,value:T=id<T>(fallback)):T{return value;}
function record(values:Box<int>[]=Array.of<Box<int>>({value:7})):int{return values[0].value;}
function main(){console.log(relay<int>(4),relay<int>(5));const text="é".repeat(1);console.log(choose<string>(text),text);const values=[2,3];console.log(choose<int[]>(values).length,values.length);const item:Item={value:9};console.log(choose<Item>(item).value,item.value);const arrow=(boxes:Box<string>[]=Array.of<Box<string>>({value:"é".repeat(1)})):int=>{return boxes[0].value.length;};console.log(record(),arrow());}
"#,
        "4 5\né é\n2 2\n9 9\n7 2\n",
    );
}

#[test]
fn contextual_optional_arrows_and_default_created_captures_preserve_ownership() {
    native(
        r#"function other():()=>int{return ():int=>{return 9;};}
function consume(text:string):string{return text;}
function capture(text:string,action:Function<int>=():int=>{return text.length;}):()=>int{return action;}
function captureOnce(text:string,action:Once<Function<int>>=():int=>{return consume(text).length;}):Once<Function<int>>{return action;}
function main(){const optional:Function<int, [value?:int]>=(value?):int=>{return value??5;};console.log(optional(),optional(2));const number:Function<number, [value?:number]>=(value=1):number=>{return value;};console.log(number(),number(2));const first=capture("é".repeat(2));const second=capture("discard".repeat(1),other());console.log(first(),second());const once=captureOnce("é".repeat(3));console.log(once());}
"#,
        "5 2\n1 2\n4 9\n6\n",
    );
}

#[test]
fn generic_default_closed_record_field_and_nullish_results_are_checked_before_specialization() {
    for (prefix, parameters, expression) in [
        (
            "type Item={value:int;};function seed():Item{return {value:7};}",
            "",
            "seed().value",
        ),
        (
            "type Box<T>={value:T;};function seed():Box<int>{return {value:7};}",
            "",
            "seed().value",
        ),
        ("function seed():int|null{return null;}", "", "seed()??7"),
        ("", "earlier:T|undefined,", "earlier??7"),
        ("", "", "Number.isFinite(7)"),
        ("", "", "Array.isArray([7])"),
        ("", "", "\"row\".repeat(2).length"),
    ] {
        let source = format!(
            "{prefix}function use<T>({parameters}value:T={expression}):int{{return 1;}}function main(){{}}"
        );
        let error = check(&source)
            .err()
            .expect("known incompatible declaration default");
        assert_eq!(
            error.code,
            "type",
            "{}",
            error.render("closed-default.tsr", &source)
        );
        assert!(
            error.message.contains("unconstrained generic parameter"),
            "{}",
            error.render("closed-default.tsr", &source)
        );
        let start = source.find(&format!("={expression}")).unwrap() + 1;
        assert_eq!(
            (error.span.start, error.span.end),
            (start, start + expression.len())
        );
    }
    let source = "function relay<T>(earlier:T|undefined,fallback:T,value:T=earlier??fallback):T{return value;}function main(){console.log(relay<int>(undefined,7));}";
    check(source)
        .expect("symbolic same-T default retains ordinary specialization checking")
        .emit_rust();
}

#[test]
fn generic_default_earlier_callable_results_preserve_symbolic_types() {
    let source = "function bad<T>(f:ReadOnly<Function<int>>,value:T=f()):T{return value;}function main(){}";
    let error = check(source)
        .err()
        .expect("known concrete earlier callback result is not arbitrary T");
    assert_eq!(error.code, "type");
    assert!(error.message.contains("unconstrained generic parameter"));
    let start = source.find("=f()").unwrap() + 1;
    assert_eq!((error.span.start, error.span.end), (start, start + 3));
    let source = "function choose<T>(f:ReadOnly<Function<T, [input:T]>>,fallback:T,value:T=f(fallback)):T{return value;}function step(value:int):int{return value+1;}function main(){console.log(choose<int>(step,7));}";
    check(source)
        .expect("same symbolic T earlier callable result remains valid")
        .emit_rust();
}

#[test]
fn nullable_callable_default_absence_preserves_each_invocation_contract() {
    native(
        r#"function read(action:(ReadOnly<Function<int>>)|null=null):int{return action?.()??-1;}
function mutate(action:(Mutable<Function<int>>)|null=null):int{return action?.()??-1;}
function owned(action:(Function<int>)|null=null):int{return action?.()??-1;}
function once(action:(Once<Function<int>>)|null=null):int{if(action!==null){return action();}return -1;}
function ownedOnce(action:(Once<Function<int>>)|null=null):int{if(action!==null){return action();}return -1;}
function make():()=>int{return ():int=>{return 9;};}
function main(){const stable=():int=>{return 7;};let n=0;const changing=():int=>{n++;return n;};console.log(read(),read(undefined),read(null),read(stable),stable());console.log(mutate(),mutate(undefined),mutate(null),mutate(changing),changing());console.log(owned(),owned(undefined),owned(null),owned(make()));console.log(once(),once(undefined),once(null),once(make()));console.log(ownedOnce(),ownedOnce(undefined),ownedOnce(null),ownedOnce(make()));}
"#,
        "-1 -1 -1 7 7\n-1 -1 -1 1 2\n-1 -1 -1 9\n-1 -1 -1 9\n-1 -1 -1 9\n",
    );
}

#[test]
fn nullable_borrowed_data_absent_defaults_preserve_payload_access_and_reuse() {
    native(r#"type Item={value:int;};
function textreadnull(value:string|null|undefined=null):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
function textreadundefined(value:string|null|undefined=undefined):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
function textmutatenull(value:string|null|undefined=null):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
function textmutateundefined(value:string|null|undefined=undefined):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
function arrayreadnull(value:int[]|null|undefined=null):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
function arrayreadundefined(value:int[]|null|undefined=undefined):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
function arraymutatenull(value:int[]|null|undefined=null):int{if(value!==null&&value!==undefined){value.push(7);return value.length;}return -1;}
function arraymutateundefined(value:int[]|null|undefined=undefined):int{if(value!==null&&value!==undefined){value.push(7);return value.length;}return -1;}
function recordreadnull(value:Item|null|undefined=null):int{if(value!==null&&value!==undefined){return value.value;}return -1;}
function recordreadundefined(value:Item|null|undefined=undefined):int{if(value!==null&&value!==undefined){return value.value;}return -1;}
function recordmutatenull(value:Item|null|undefined=null):int{if(value!==null&&value!==undefined){value.value++;return value.value;}return -1;}
function recordmutateundefined(value:Item|null|undefined=undefined):int{if(value!==null&&value!==undefined){value.value++;return value.value;}return -1;}
function main(){const x0:string|null|undefined="é".repeat(1);console.log(textreadnull(),textreadnull(undefined),textreadnull(null),textreadnull(x0),x0?.length??-1);const x1:string|null|undefined="é".repeat(1);console.log(textreadundefined(),textreadundefined(undefined),textreadundefined(null),textreadundefined(x1),x1?.length??-1);const x2:string|null|undefined="é".repeat(1);console.log(textmutatenull(),textmutatenull(undefined),textmutatenull(null),textmutatenull(x2),x2?.length??-1);const x3:string|null|undefined="é".repeat(1);console.log(textmutateundefined(),textmutateundefined(undefined),textmutateundefined(null),textmutateundefined(x3),x3?.length??-1);const x4:int[]|null|undefined=[1,2];console.log(arrayreadnull(),arrayreadnull(undefined),arrayreadnull(null),arrayreadnull(x4),x4?.length??-1);const x5:int[]|null|undefined=[1,2];console.log(arrayreadundefined(),arrayreadundefined(undefined),arrayreadundefined(null),arrayreadundefined(x5),x5?.length??-1);const x6:int[]|null|undefined=[1,2];console.log(arraymutatenull(),arraymutatenull(undefined),arraymutatenull(null),arraymutatenull(x6),x6?.length??-1);const x7:int[]|null|undefined=[1,2];console.log(arraymutateundefined(),arraymutateundefined(undefined),arraymutateundefined(null),arraymutateundefined(x7),x7?.length??-1);const x8:Item|null|undefined={value:1};console.log(recordreadnull(),recordreadnull(undefined),recordreadnull(null),recordreadnull(x8),x8?.value??-1);const x9:Item|null|undefined={value:1};console.log(recordreadundefined(),recordreadundefined(undefined),recordreadundefined(null),recordreadundefined(x9),x9?.value??-1);const x10:Item|null|undefined={value:1};console.log(recordmutatenull(),recordmutatenull(undefined),recordmutatenull(null),recordmutatenull(x10),x10?.value??-1);const x11:Item|null|undefined={value:1};console.log(recordmutateundefined(),recordmutateundefined(undefined),recordmutateundefined(null),recordmutateundefined(x11),x11?.value??-1);}
"#, "-1 -1 -1 2 2\n-1 -1 -1 2 2\n-1 -1 -1 2 2\n-1 -1 -1 2 2\n-1 -1 -1 2 2\n-1 -1 -1 2 2\n-1 -1 -1 3 3\n-1 -1 -1 3 3\n-1 -1 -1 1 1\n-1 -1 -1 1 1\n-1 -1 -1 2 2\n-1 -1 -1 2 2\n");
}

#[test]
fn nullable_callable_undefined_defaults_and_named_once_optional_restriction() {
    if std::env::var_os("TSR_GATE98_RETAIN_UNDEFINED_NATIVE").is_none() { native(r#"function read(action:(ReadOnly<Function<int>>)|null|undefined=undefined):int{return action?.()??-1;}
function mutate(action:(Mutable<Function<int>>)|null|undefined=undefined):int{return action?.()??-1;}
function owned(action:(Function<int>)|null|undefined=undefined):int{return action?.()??-1;}
function once(action:(Once<Function<int>>)|null|undefined=undefined):int{if(action!==null&&action!==undefined){return action();}return -1;}
function ownedOnce(action:(Once<Function<int>>)|null|undefined=undefined):int{if(action!==null&&action!==undefined){return action();}return -1;}
function make():()=>int{return ():int=>{return 9;};}
function main(){const stable=():int=>{return 7;};let n=0;const changing=():int=>{n++;return n;};console.log(read(),read(undefined),read(null),read(stable),stable());console.log(mutate(),mutate(undefined),mutate(null),mutate(changing),changing());console.log(owned(),owned(undefined),owned(null),owned(make()));console.log(once(),once(undefined),once(null),once(make()));console.log(ownedOnce(),ownedOnce(undefined),ownedOnce(null),ownedOnce(make()));}
"#, "-1 -1 -1 7 7\n-1 -1 -1 1 2\n-1 -1 -1 9\n-1 -1 -1 9\n-1 -1 -1 9\n"); }
    let source="function use(action:(Once<Function<int>>)|null=null):int{return action?.()??-1;}function main(){console.log(use());}";
    let error=check(source).err().expect("named once optional receiver remains a scoped readonly view");
    assert_eq!(error.code,"borrowed-place");
    assert!(error.message.contains("optional selection borrows its source"),"{}",error.message);
    let start=source.find("action?.").unwrap();
    assert_eq!((error.span.start,error.span.end),(start,start+6));
}

#[test]
fn nullable_borrowed_callable_defaults_select_earlier_callback_without_escape() {
    native(r#"function read(fallback:ReadOnly<Function<int>>,action:(ReadOnly<Function<int>>)|null=fallback):int{return action?.()??-1;}
function mutate(fallback:Mutable<Function<int>>,action:(Mutable<Function<int>>)|null=fallback):int{return action?.()??-1;}
function main(){const stable=():int=>{return 7;};const second=():int=>{return 9;};console.log(read(stable),read(stable,undefined),read(stable,null),read(stable,second),stable(),second());let n=0;const changing=():int=>{n++;return n;};let m=10;const other=():int=>{m++;return m;};console.log(mutate(changing),mutate(changing,undefined),mutate(changing,null),mutate(changing,other),changing(),other());}
"#, "7 7 -1 9 7 9\n1 2 -1 11 3 12\n");
}

#[test]
fn nullable_data_defaults_select_earlier_and_fresh_borrowed_owners() {
    native(r#"type Item={value:int;};
function text(fallback:string,value:string|null=fallback):int{return value?.length??-1;}
function arrayRead(fallback:int[],value:int[]|null=fallback):int{return value?.length??-1;}
function arrayMutate(fallback:int[],value:int[]|null=fallback):int{if(value!==null){value.push(7);return value.length;}return -1;}
function recordRead(fallback:Item,value:Item|null=fallback):int{return value?.value??-1;}
function recordMutate(fallback:Item,value:Item|null=fallback):int{if(value!==null){value.value++;return value.value;}return -1;}
function freshArray(value:int[]|null=[1]):int{if(value!==null){value.push(7);return value.length;}return -1;}
function freshRecord(value:Item|null={value:4}):int{if(value!==null){value.value++;return value.value;}return -1;}
function main(){const label="é".repeat(1);console.log(text(label),text(label,null),label.length);const a=[1];const b=[2,3];console.log(arrayRead(a),arrayRead(a,b),a.length,b.length);console.log(arrayMutate(a),arrayMutate(a,undefined),arrayMutate(a,null),arrayMutate(a,b),a.length,b.length);const first:Item={value:1};const second:Item={value:8};console.log(recordRead(first),recordRead(first,second),first.value,second.value);console.log(recordMutate(first),recordMutate(first,undefined),recordMutate(first,null),recordMutate(first,second),first.value,second.value);console.log(freshArray(),freshArray(undefined),freshArray(null),freshArray(b),b.length);console.log(freshRecord(),freshRecord(undefined),freshRecord(null),freshRecord(second),second.value);}
"#, "2 -1 2\n1 2 1 2\n2 3 -1 3 3 3\n1 8 1 8\n2 3 -1 9 3 9\n2 2 -1 4 4\n5 5 -1 10 10\n");
    native(r#"function make():()=>int{return ():int=>{return 4;};}
function read(action:(ReadOnly<Function<int>>)|null=make()):int{return action?.()??-1;}
function mutate(action:(Mutable<Function<int>>)|null=make()):int{return action?.()??-1;}
function main(){const stable=():int=>{return 7;};let n=0;const changing=():int=>{n++;return n;};console.log(read(),read(undefined),read(null),read(stable),stable());console.log(mutate(),mutate(undefined),mutate(null),mutate(changing),changing());}
"#, "4 4 -1 7 7\n4 4 -1 1 2\n");
}

#[test]
fn nullable_borrowed_defaults_keep_source_loan_and_escape_restrictions() {
    let sources=[
        "function bad(fallback:Mutable<Function<int>>,action:(Mutable<Function<int>>)|null=fallback):(Function<int>)|null{return action;}function main(){let n=0;const f=():int=>{n++;return n;};bad(f);}",
    ];
    for source in sources {
        let error=check(source).err().expect("borrowed selected default cannot escape as an owner");
        assert_eq!(error.code,"type");
        assert_eq!(error.message,"Expected closure | null, found closure | null.");
        let start=source.find("return action").unwrap()+7;
        assert_eq!((error.span.start,error.span.end),(start,start+6));
    }
}

#[test]
fn benchmark_source_order_witness_uses_current_callable_contracts() {
    support::execute(
        include_str!("../../../benchmarks/runtime/optional-default-parameter-source-order.tsr"),
        include_str!("../../../benchmarks/runtime/optional-default-parameter-source-order-expected.txt"),
        None,
    );
}
