mod support;
use tsrust_compiler::check;

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let assertions = if assertions.contains("__NUMBER_FUNCTION__") {
        let functions = emitted
            .lines()
            .filter(|line| {
                line.starts_with("fn function") && line.contains("(value0: f64) -> Vec<f64>")
            })
            .collect::<Vec<_>>();
        assert_eq!(functions.len(), 1, "unique Number constructor witness");
        let name = functions[0]
            .split_once('(')
            .unwrap()
            .0
            .strip_prefix("fn ")
            .unwrap();
        assertions.replace("__NUMBER_FUNCTION__", name)
    } else {
        assertions.to_owned()
    };
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let evidence = std::env::var_os("TSR_GATE97_EVIDENCE_DIR").map(|root| {
        std::path::PathBuf::from(root).join(format!(
            "case-{}",
            CASE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ))
    });
    if let Some(path) = &evidence {
        std::fs::create_dir_all(path).unwrap();
        std::fs::write(path.join("input.tsr"), source).unwrap();
        std::fs::write(path.join("expected.txt"), expected).unwrap();
        std::fs::write(path.join("generated.rs"), &rust).unwrap();
    }
    let directory = support::TempDirectory::new();
    let path = directory.0.join("nullable-callables.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("nullable-callables-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(executable), 15);
        if let Some(path) = &evidence {
            std::fs::write(path.join(format!("stdout-O{level}.txt")), &output.stdout).unwrap();
            std::fs::write(path.join(format!("stderr-O{level}.txt")), &output.stderr).unwrap();
        }
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn nullable_callable_owned_transports_invocations_and_observers() {
    native(
        r#"type Handler={run:(Function<int, [input:int]>)|null|undefined;};function inc(input:int):int{return input+1;}function pass(action:(Function<int, [input:int]>)|null):(Function<int, [input:int]>)|null{return action;}function main(){const a:(Function<int, [input:int]>)|null=inc;console.log(typeof a,Boolean(a),Array.isArray(a));const b=pass(a);if(b!==null){console.log(b(4));}const holder:Handler={run:(input:int):int=>{return input*2;}};console.log(holder.run?.(3)??-1);let nil:(Function<int, [input:int]>)|undefined=undefined;console.log(typeof nil,Boolean(nil),Array.isArray(nil));nil??=inc;if(nil!==undefined){console.log(nil(8));}}"#,
        "function true false\n5\n6\nundefined false false\n9\n",
        "",
        "",
    );
}
#[test]
fn nullable_callable_borrowed_generic_nested_callbacks_and_captures() {
    native(
        r#"function apply(action:(ReadOnly<Function<int, [input:int]>>)|null,input:int):int{if(action!==null){return action(input);}return -1;}function main(){const a:(ReadOnly<Function<int, [input:int]>>)|null=(input:int):int=>{return input+3;};console.log(apply(a,4),apply(a,5));}"#,
        "7 8\n",
        "",
        "",
    );
}
fn error_at(source: &str, code: &str, message: &str, marker: &str) {
    let error = check(source)
        .err()
        .unwrap_or_else(|| panic!("specific rejected contract accepted: {source}"));
    assert_eq!(error.code, code, "{}", error.render("negative.tsr", source));
    assert!(
        error.message.contains(message),
        "{}",
        error.render("negative.tsr", source)
    );
    let start = source.rfind(marker).expect("marker exists");
    assert!(
        error.span.start < start + marker.len() && error.span.end > start,
        "range {:?} must identify {marker}: {}",
        error.span,
        error.render("negative.tsr", source)
    );
}
#[test]
fn nullable_callable_strict_contracts_cloneability_and_normal_depth() {
    for (source, code, message, marker) in [
        (
            "function main(){const a:(Function<int>)|null=():int=>{return 1;};clone(a);}",
            "clone-type",
            "cannot be cloned",
            "clone(a)",
        ),
        (
            "function main(){const a:(Function<int>)|null=():int=>{return 1;};String(a);}",
            "unsupported",
            "function",
            "String(a)",
        ),
        (
            "function main(){const a:(Function<int>)|null=null;a();}",
            "unsupported",
            "Only closure bindings",
            "a()",
        ),
        (
            "function bad(action:(ReadOnly<Function<int>>)|null):(ReadOnly<Function<int>>)|null{return action;}function main(){}",
            "callback-escape",
            "must own",
            "(ReadOnly<Function<int>>)|null{return",
        ),
        (
            "function bad(action:(Mutable<Function<int>>)|null):(Mutable<Function<int>>)|null{return action;}function main(){}",
            "callback-escape",
            "must own",
            "(Mutable<Function<int>>)|null{return",
        ),
        (
            "function main(){let action:(Function<int>)|null=null;(action??=():int=>{return 1;})();}",
            "borrowed-place",
            "readonly",
            "??=",
        ),
        (
            "function main(){const a:(Once<Function<int>>)|null=():int=>{return 1;};if(a!==null){a();if(a!==null){a();}}}",
            "transferred",
            "transferred",
            "a!==null",
        ),
        (
            "type Holder={run:(Function<int>)|null;};function main(){const holder:Holder={run:():int=>{return 1;}};clone(holder);}",
            "clone-type",
            "cannot be cloned",
            "clone(holder)",
        ),
        (
            "function apply(action:(ReadOnly<Function<int, [value:int]>>)|null,fallback:ReadOnly<Function<int, [value:int]>>):int{let local=action;const selected=local??=fallback;return selected(5);}function main(){}",
            "callable-origin",
            "parameter origin",
            "fallback;return",
        ),
    ] {
        error_at(source, code, message, marker);
    }
    let source = format!(
        "function id(value:int):int{{return value;}}function main(){{const action:(Function<int, [value:int]>)|null=id;if(action!==null){{console.log(action({}1{}));}}}}",
        "id(".repeat(100),
        ")".repeat(100)
    );
    native(&source, "1\n", "", "");
    let too_deep = format!(
        "function id(value:int):int{{return value;}}function main(){{const action:(Function<int, [value:int]>)|null=id;action?.({}1{});}}",
        "id(".repeat(200),
        ")".repeat(200)
    );
    let error = check(&too_deep)
        .err()
        .expect("bounded normal-stack diagnostic");
    assert_eq!(error.code, "unsupported");
    assert!(
        error.message.contains("nesting") && error.message.contains("128"),
        "{}",
        error.message
    );
}

#[test]
fn nullable_callable_once_owned_origins_and_disposal_are_native() {
    native(
        r#"function main(){const text="value".repeat(1);const a:(Once<Function<string>>)|null=():string=>{return text;};if(a!==null){console.log(a());}}"#,
        "value\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_nested_bounds_and_owned_captures() {
    native(
        r#"function use(action:(ReadOnly<Function<int, [inner:ReadOnly<Function<int, [input:int]>>,input:int]>>)|null,inner:ReadOnly<Function<int, [input:int]>>):int{if(action!==null){return action(inner,4);}return -1;}function main(){const action:(ReadOnly<Function<int, [inner:ReadOnly<Function<int, [input:int]>>,input:int]>>)|null=(inner:ReadOnly<Function<int, [input:int]>>,input:int):int=>{return inner(input);};const inner=(input:int):int=>{return input+7;};console.log(use(action,inner));let owned:(Function<int>)|null=():int=>{return 9;};const captured=():int=>{if(owned!==null){return owned();}return -1;};console.log(captured(),captured());}"#,
        "11\n9 9\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_inferred_borrowed_capture_and_once_transport() {
    native(
        r#"function main(){const text="é".repeat(1);const a=true?():int=>{return text.length;}:null;const see=():int=>{if(a!==null){return a();}return -1;};console.log(see(),see(),text.length);const once:(Once<Function<int>>)|null=():int=>{return 7;};if(once!==null){console.log(once());}}"#,
        "2 2 2\n7\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_conditional_declared_forwarding_and_generic_owned_relay() {
    native(
        r#"function inc(input:int):int{return input+1;}function relay<T>(action:(Function<T, [input:T]>)|null|undefined):(Function<T, [input:T]>)|null|undefined{return action;}function main(){const action:(ReadOnly<Function<int, [input:int]>>)|null|undefined=true?inc:false?null:undefined;if(action!==null&&action!==undefined){console.log(action(4));}const owned:(Function<int, [input:int]>)|null=inc;const other=relay<int>(owned);if(other!==null&&other!==undefined){console.log(other(8));}}"#,
        "5\n9\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_generic_record_data_arguments_keep_owned_field_contracts() {
    native(
        r#"function step(input:int):int{return input+2;}type Holder<T>={run:(Function<T, [input:T]>)|null;};function main(){const holder:Holder<int>={run:step};if(holder.run!==null){console.log(holder.run(7));}}"#,
        "9\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_absent_read_tags_and_existing_origin_policy() {
    error_at(
        "function f(a:ReadOnly<Function<int>>,b:ReadOnly<Function<int>>):int{let c=a;c=b;return c();}function main(){}",
        "callable-origin",
        "same parameter",
        "c=b",
    );
    error_at(
        "function apply(action:(ReadOnly<Function<int, [value:int]>>)|null,fallback:ReadOnly<Function<int, [value:int]>>):int{let local=action;const selected=local??=fallback;return selected(5);}function main(){}",
        "callable-origin",
        "parameter origin",
        "fallback;return",
    );
    native(
        r#"function main(){const empty:(ReadOnly<Function<int>>)|null=null;console.log(typeof empty,Boolean(empty),Array.isArray(empty));}"#,
        "object false false\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_initially_absent_read_storage_can_commit_one_origin() {
    native(
        r#"function step(input:int):int{return input+2;}function fallback():(input:int)=>int{return step;}function main(){let empty:(ReadOnly<Function<int, [input:int]>>)|null=null;const selected=empty??=fallback();console.log(selected(5));if(empty!==null){console.log(empty(7));}}"#,
        "7\n9\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_native_transport_has_no_hidden_clone_and_unwinds_owned_captures() {
    let observer = r#"
use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicBool,AtomicUsize,Ordering};
struct Observe;static RECORD:AtomicBool=AtomicBool::new(false);static ALLOCS:AtomicUsize=AtomicUsize::new(0);static REALLOCS:AtomicUsize=AtomicUsize::new(0);static WATCH:AtomicUsize=AtomicUsize::new(0);
unsafe impl GlobalAlloc for Observe{unsafe fn alloc(&self,l:Layout)->*mut u8{if RECORD.load(Ordering::Relaxed){ALLOCS.fetch_add(1,Ordering::Relaxed);}unsafe{System.alloc(l)}}unsafe fn dealloc(&self,p:*mut u8,l:Layout){if p as usize==WATCH.load(Ordering::Relaxed){WATCH.store(0,Ordering::Relaxed);}unsafe{System.dealloc(p,l)}}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{if RECORD.load(Ordering::Relaxed){REALLOCS.fetch_add(1,Ordering::Relaxed);}unsafe{System.realloc(p,l,n)}}}
#[global_allocator]static GLOBAL:Observe=Observe;
"#;
    native(
        r#"function transfer(value:string):string{const action:(Once<Function<string>>)|null=():string=>{return value;};if(action!==null){return action();}return "";}function fail(value:string){const action:(Once<Function<string>>)|null=():string=>{const bad="é".substr(1,1);return value;};if(action!==null){action();}}function main(){}"#,
        "",
        observer,
        r#"
 let input:String=std::borrow::Cow::Owned(std::string::String::from("owned pointer transfer"));let address=input.as_ptr() as usize;WATCH.store(address,Ordering::Relaxed);ALLOCS.store(0,Ordering::Relaxed);REALLOCS.store(0,Ordering::Relaxed);RECORD.store(true,Ordering::Relaxed);let output=function0(std::hint::black_box(input));RECORD.store(false,Ordering::Relaxed);assert_eq!(output.as_ptr() as usize,address);assert_eq!(ALLOCS.load(Ordering::Relaxed),0);assert_eq!(REALLOCS.load(Ordering::Relaxed),0);drop(output);assert_eq!(WATCH.load(Ordering::Relaxed),0);
 std::panic::set_hook(Box::new(|_|{}));let input:String=std::borrow::Cow::Owned(std::string::String::from("unwind capture owner"));WATCH.store(input.as_ptr() as usize,Ordering::Relaxed);let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(move||function1(std::hint::black_box(input))));assert!(result.is_err());drop(result);assert_eq!(WATCH.load(Ordering::Relaxed),0);
 "#,
    );
}

#[test]
fn nullable_callable_nested_results_distinct_abstract_paths_and_builtin_named_fields() {
    native(
        r#"function use(action:(ReadOnly<Function<()=>int>>)|null):int{if(action!==null){const next=action();return next();}return -1;}function pair(first:(ReadOnly<Function<int>>)|null,second:(ReadOnly<Function<int>>)|null):int{if(first!==null&&second!==null){return first()+second();}return -1;}type Handler={concat:(Function<int, [value:int]>)|null;repeat:Function<int, [value:int]>;};function main(){const action:(ReadOnly<Function<()=>int>>)|null=():()=>int=>{return ():int=>{return 7;};};console.log(use(action));const first:(ReadOnly<Function<int>>)|null=():int=>{return 2;};const second:(ReadOnly<Function<int>>)|null=():int=>{return 5;};console.log(pair(first,second));const holder:Handler={concat:(value:int):int=>{return value+1;},repeat:(value:int):int=>{return value+2;}};console.log(holder.concat?.(3)??-1,holder.repeat(3));}"#,
        "7\n7\n4 5\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_invocation_kinds_and_recursive_contracts_remain_distinct() {
    native(
        r#"function mutable(action:(Mutable<Function<int>>)|null):int{if(action!==null){const a=action();const b=action?.()??-1;return a+b;}return -1;}function forward(action:(Mutable<Function<int>>)|null):int{return mutable(action);}function make():(Function<int>)|null{let count=10;return ():int=>{count=count+1;return count;};}function once():(Once<Function<int>>)|null{return ():int=>{return 17;};}function step(value:int):int{return value+1;}function main(){let count=0;const action:(Mutable<Function<int>>)|null=():int=>{count=count+1;return count;};console.log(forward(action));const owner=make();if(owner!==null){console.log(owner(),owner?.()??-1);}const chosen=make()??():int=>{return 0;};console.log(chosen());let stored:(Function<int>)|null=null;stored??=make();if(stored!==null){console.log(stored());}const selected=once()??():int=>{return 3;};console.log(selected());let single:(Once<Function<int>>)|null=null;single??=once();if(single!==null){console.log(single());}const inner=():int=>{let empty:(ReadOnly<Function<int, [value:int]>>)|null=null;empty??=step;if(empty!==null){return empty(4);}return 0;};console.log(inner());}"#,
        "3\n11 12\n11\n11\n17\n17\n5\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_nested_nullable_parameters_recursive_calls_and_access_modes() {
    native(
        r#"function recur(action:(ReadOnly<Function<int, [input:int]>>)|null,depth:int):int{if(depth===0){if(action!==null){return action(4);}return -1;}return recur(action,depth-1);}function nested(action:(ReadOnly<Function<int, [inner:(ReadOnly<Function<int>>)|null]>>)|null,inner:(ReadOnly<Function<int>>)|null):int{if(action!==null){return action(inner);}return -1;}function main(){const value:(ReadOnly<Function<int, [input:int]>>)|null=(input:int):int=>{return input+3;};console.log(recur(value,5));const outer:(ReadOnly<Function<int, [inner:(ReadOnly<Function<int>>)|null]>>)|null=(inner:(ReadOnly<Function<int>>)|null):int=>{if(inner!==null){return inner();}return -1;};const inner:(ReadOnly<Function<int>>)|null=():int=>{return 8;};console.log(nested(outer,inner),nested(outer,null));}"#,
        "7\n8 -1\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_owning_nested_parameters_use_existing_concrete_specialization() {
    native(
        r#"function invoke(action:(Function<int, [inner:ReadOnly<Function<int>>]>)|null,inner:ReadOnly<Function<int>>):int{if(action!==null){return action(inner);}return -1;}function main(){const action:(Function<int, [inner:ReadOnly<Function<int>>]>)|null=(inner:ReadOnly<Function<int>>):int=>{return inner();};const inner=():int=>{return 12;};console.log(invoke(action,inner));}"#,
        "12\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_nested_owning_factory_results_retain_concrete_origin() {
    native(
        r#"function make():(Function<int, [inner:ReadOnly<Function<int>>]>)|null{return (inner:ReadOnly<Function<int>>):int=>{return inner();};}function main(){const action=make();const inner=():int=>{return 13;};if(action!==null){console.log(action(inner));}}"#,
        "13\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_nested_factory_absence_and_recursive_return_keep_tags() {
    native(
        r#"function empty():(Function<int, [inner:ReadOnly<Function<int>>]>)|null{return null;}function make(depth:int):(Function<int, [inner:ReadOnly<Function<int>>]>)|null{if(depth>0){return make(depth-1);}return (inner:ReadOnly<Function<int>>):int=>{return inner();};}function main(){const emptyResult=empty();console.log(typeof emptyResult,Boolean(emptyResult));const action=make(3);const inner=():int=>{return 14;};if(action!==null){console.log(action(inner));}}"#,
        "object false\n14\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_absent_factory_calls_are_typed_lazy_and_forwardable() {
    native(
        r#"type Trace={n:int;};function inner():int{return 14;}function argument(trace:Trace):()=>int{trace.n=trace.n+1;return inner;}function empty():(Function<int, [inner:ReadOnly<Function<int>>]>)|null{return null;}function forward():(Function<int, [inner:ReadOnly<Function<int>>]>)|null{return empty();}function apply(action:(Function<int, [inner:ReadOnly<Function<int>>]>)|null,inner:ReadOnly<Function<int>>):int{if(action!==null){return action(inner);}return -1;}function mixed(values:int[]):(Function<int, [inner:ReadOnly<Function<int>>]>)|null{for(const value of values){if(value===0){return null;}}return (inner:ReadOnly<Function<int>>):int=>{return inner();};}function main(){const trace:Trace={n:0};const absent=empty();if(absent!==null){console.log(absent(argument(trace)));}console.log(forward()?.(argument(trace))??-1,trace.n);console.log(apply(empty(),inner));const nil=mixed([0]);console.log(typeof nil);const present=mixed([1]);if(present!==null){console.log(present(inner));}}"#,
        "-1 0\n-1\nobject\n14\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_nested_owning_result_and_absence_preserve_signature() {
    native(
        r#"function make():(Function<()=>int>)|null{return ():()=>int=>{return ():int=>{return 15;};};}function empty():(Function<()=>int>)|null{return null;}function main(){const action=make();if(action!==null){const next=action();console.log(next());}const absent=empty();if(absent!==null){const next=absent();console.log(next());}console.log(typeof empty());}"#,
        "15\nobject\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_read_arguments_borrow_existing_environments_per_iteration() {
    error_at(
        "function main(){let n=0;const callback:ReadOnly<Function<int>>=():int=>{return 7;};while(n<2){const action:ReadOnly<Function<int>>=callback;console.log(action());n=n+1;}}",
        "loop-transfer",
        "Cannot transfer",
        "callback;console",
    );
    error_at(
        "function main(){let n=0;const callback:ReadOnly<Function<int>>=():int=>{return 7;};while(n<2){const action:(ReadOnly<Function<int>>)|null=callback;if(action!==null){console.log(action());}n=n+1;}}",
        "loop-transfer",
        "Cannot transfer",
        "callback;if",
    );
    native(
        r#"function apply(action:(ReadOnly<Function<int, [value:int]>>)|null,value:int):int{if(action!==null){return action(value);}return -1;}function main(){const bias=3;const callback:ReadOnly<Function<int, [value:int]>>=(value:int):int=>{return value+bias;};let n=0;while(n<3){console.log(apply(n<2?callback:null,n));n=n+1;}console.log(callback(4));}"#,
        "3\n4\n-1\n7\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_guarded_absence_string_and_absent_coalesce_keep_effects() {
    error_at(
        "function main(){const action:(Function<int>)|null=null;String(action);}",
        "unsupported",
        "function-source",
        "String(action)",
    );
    native(
        r#"type Trace={n:int;};function empty(trace:Trace):(Function<int, [inner:ReadOnly<Function<int>>]>)|null|undefined{trace.n=trace.n+1;return null;}function present(trace:Trace):(inner:ReadOnly<Function<int>>)=>int{trace.n=trace.n+2;return (inner:ReadOnly<Function<int>>):int=>{return inner();};}function main(){const action:(Function<int, [value:int]>)|null=null;if(action===null){console.log(String(action));}const missing:(Function<int>)|undefined=undefined;if(missing===undefined){console.log(String(missing));}const trace:Trace={n:0};const selected=empty(trace)??present(trace);const inner=():int=>{return 16;};console.log(selected(inner),trace.n);}"#,
        "null\nundefined\n16 3\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_conditional_read_carriers_keep_absent_and_nested_argument_types() {
    native(
        r#"function apply(action:(ReadOnly<Function<int>>)|null|undefined):int{if(action!==null&&action!==undefined){return action();}return -1;}function make(inner:(ReadOnly<Function<int>>)|null):()=>int{const observed=inner?.()??0;return ():int=>{return observed+5;};}function main(){console.log(apply(true?null:undefined),apply(false?null:undefined));console.log(apply(true?make(null):null),apply(false?make(null):null));}"#,
        "-1 -1\n5 -1\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_conditional_factory_initializer_is_native() {
    native(
        r#"
function make(text:string):Once<Function<string, [...values:int[]]>>{return (...values:int[]):string=>{return text+String(values.length);};}
function reader(text:string):(value:int)=>string{return (value:int):string=>{return text+String(value);};}
function main(){let i=0;while(i<2){const present=i===0;
 const first:(Once<Function<string, [...values:int[]]>>)|undefined=present?make("a"):undefined;if(first!==undefined){console.log(first(1,2));}else{console.log("absent");}
 const second:(Once<Function<string, [...values:int[]]>>)|undefined=present?undefined:make("b");if(second!==undefined){console.log(second(...[1]));}else{console.log("absent");}
 const third:((value:int)=>string)|undefined=present?reader("c"):undefined;if(third!==undefined){console.log(third(7));}else{console.log("absent");}
 i=i+1;}}
"#,
        "a2\nabsent\nc7\nabsent\nb1\nabsent\n",
        "",
        "",
    );
}

#[test]
fn nullable_callable_nested_callback_arguments_are_caller_specialized() {
    native(
        r#"
function one():int{return 1;}
function named(inner:(ReadOnly<Function<int>>)|null,value:int):int{if(inner===null){return value;}return inner();}
function literal(action:ReadOnly<Function<int, [inner:(ReadOnly<Function<int>>)|null,value:int]>>):int{return action(null,2);}
function present(action:ReadOnly<Function<int, [inner:(ReadOnly<Function<int>>)|null,value:int]>>):int{return action(one,2);}
function chosen(action:ReadOnly<Function<int, [inner:(ReadOnly<Function<int>>)|null,value:int]>>,flag:boolean):int{return action(flag?one:null,2);}
function main(){const arrow=(inner:(ReadOnly<Function<int>>)|null,value:int):int=>{return inner===null?value:inner();};
 console.log(literal(arrow),present(arrow),chosen(arrow,true),chosen(arrow,false),literal(named),present(named));}
"#,
        "2 1 1 2 2 1\n",
        "",
        "",
    );
}
