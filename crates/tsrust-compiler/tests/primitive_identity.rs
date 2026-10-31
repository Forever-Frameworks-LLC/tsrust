mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("primitive-identity.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("primitive-identity-{level}"));
        support::success(&support::run(
            support::rustc()
                .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(executable), 15);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn primitive_types_contexts_and_existing_methods_have_exact_results() {
    native(
        r#"
    type Box={text:string;flag:boolean;count:int;value:number;};
    function text(s:string):string{return s.valueOf().toString();}
    function integer(n:int):int{return n.valueOf();}
    function number(n:number):number{return n.valueOf();}
    function boolean(n:boolean):boolean{return n.valueOf();}
    function main(){
        console.log("x".valueOf());console.log("x".toString());
        console.log(true.valueOf());console.log(false.valueOf());console.log(true.toString());console.log(false.toString());
        console.log(integer(-7));console.log(number(1.5));console.log(boolean(false));
        const inferred=clone("a").valueOf()+clone("b").toString();console.log(inferred);
        const bools=[true.valueOf(),false.valueOf()];console.log(bools[1]);
        const numbers=[1.5.valueOf(),2.5.valueOf()];console.log(numbers[0]+numbers[1]);
        const box:Box={text:"abc".toString(),flag:true.valueOf(),count:7.valueOf(),value:1.5.valueOf()};
        console.log(box.text);console.log(box.flag.toString());console.log(box.count.valueOf());console.log(box.value.valueOf());
        console.log(text("abc"));console.log("ab".valueOf().substring(1).concat("!").padStart(3,"0"));
        const selected=(true?1:2).valueOf();console.log(selected);
        const values=[1,2].concat(3);console.log(values[2]);console.log("abc".slice(1));console.log("abc".charAt(1));
    }"#,
        "x\nx\ntrue\nfalse\ntrue\nfalse\n-7\n1.5\nfalse\nab\nfalse\n4\nabc\ntrue\n7\n1.5\nabc\n0b!\n1\n3\nbc\nb\n",
        "",
        "",
    );
}

#[test]
fn receiver_once_contracts_captures_temporaries_and_fixed_storage_are_preserved() {
    let analysis = native(
        r#"
    type Box={text:string;};
    function receiver(trace:int[]):string{trace.push(1);return "owned".repeat(1);}
    function flag(trace:int[]):boolean{trace.push(2);return true;}
    function integer(trace:int[]):int{trace.push(3);return 7;}
    function number(trace:int[]):number{trace.push(4);return 1.5;}
    function view(value:string):string{return value.valueOf();}
    function invoke(action:Mutable<Function<string>>):string{return action().toString();}
    function factory():()=>string{return ():string=>{return "factory".repeat(1);};}
    function fixed():int{const values=[1,2,3];return values[1].valueOf();}
    function main(){
        const trace:int[]=[];console.log(receiver(trace).valueOf());console.log(receiver(trace).toString());
        console.log(flag(trace).valueOf());console.log(flag(trace).toString());console.log(integer(trace).valueOf());console.log(number(trace).valueOf());
        for(const n of trace){console.log(n);}
        const input="abc".repeat(1);const result=view(input);console.log(result);console.log(input);
        const captured="capture".repeat(1);const action=():string=>{return captured.toString();};console.log(invoke(action));console.log(captured);
        const made=factory();console.log(made().valueOf());
        const box:Box={text:"before".repeat(1)};const copied=box.text.toString();box.text="after";console.log(copied);
        const texts=["left".repeat(1),"right".repeat(1)];console.log(texts[0].valueOf());console.log(texts[0]);
        for(const condition of [true,false]){
            const length=(condition?"abc".repeat(1):"def".repeat(1)).valueOf().length;console.log(length);
            console.log((condition?"a".repeat(1):"b".repeat(1)).toString());
            for(const s of [(condition?"x".repeat(1):"y".repeat(1)).valueOf()]){console.log(s);}
        }
        console.log(fixed());
    }"#,
        "owned\nowned\ntrue\ntrue\n7\n1.5\n1\n1\n2\n2\n3\n4\nabc\nabc\ncapture\ncapture\nfactory\nbefore\nleft\nleft\n3\na\nx\n3\nb\ny\n2\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("view.value"), Some(&Mode::Read));
    assert_eq!(
        analysis.contracts().get("invoke.action"),
        Some(&Mode::Mutate)
    );
    let emitted = analysis.emit_rust();
    assert!(emitted.contains("[1i64, 2i64, 3i64]"), "{emitted}");
    let source = "function take(s:string):string{const moved=s;return moved;}function main(){const s=\"abc\";s.concat(take(s).valueOf());}";
    assert_eq!(check(source).err().unwrap().code, "conflicting-access");
    native(
        "function take(s:string):string{const moved=s;return moved;}function main(){const s=\"abc\".repeat(1);console.log(s.valueOf().concat(take(s)));}",
        "abcabc\n",
        "",
        "",
    );
}

#[test]
fn scalar_bits_unicode_strict_zero_arity_and_normal_stack_limits_are_exact() {
    native(
        r#"
    function number(value:number):number{return value.valueOf();}
    function integer(value:int):int{return value.valueOf();}
    function main(){console.log("é中😀e\u0301\u0000\uFEFF".valueOf().toString());console.log(false.toString());}
    "#,
        "é中😀é\0\u{FEFF}\nfalse\n",
        "",
        r#"
    for bits in [0u64,1,0x8000000000000000,0x8000000000000001,0x0010000000000000,0x7fefffffffffffff,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000001234,0x7ff0000000001234,0xfff8000000000042]{
        assert_eq!(function0(f64::from_bits(bits)).to_bits(),bits);
    }
    for value in [i64::MIN,i64::MAX,0,-1,1]{assert_eq!(function1(value),value);}
    "#,
    );
    for (source, code) in [
        ("function main(){\"x\".valueOf(1);}", "arity"),
        ("function main(){true.toString(1);}", "arity"),
        ("function main(){false.valueOf(0);}", "arity"),
        ("function main(){1.0.valueOf(0);}", "arity"),
        ("function main(){1.valueOf(0);}", "arity"),
        ("function main(){\"x\".toString(0);}", "arity"),
        ("function main(){[1].valueOf();}", "type"),
        (
            "type Box={n:int;};function main(){const box:Box={n:1};box.toString();}",
            "type",
        ),
        ("function main(){1.toString(10, 16);}", "arity"),
        ("function main(){1.0.toString(2,3);}", "arity"),
        ("function main(){const n:number=true.valueOf();}", "type"),
        ("function main(){const n:int=1.0.valueOf();}", "type"),
        ("function main(){\"x\".valueOf(,);}", "syntax"),
    ] {
        let error = support::reject(source, code);
        assert_ne!(error.code, "internal");
        assert!(error.span.end > error.span.start);
    }
    for count in [48, 200] {
        let source = format!(
            "function main(){{console.log(\"x\"{});}}",
            ".valueOf()".repeat(count)
        );
        let directory = support::TempDirectory::new();
        let output = support::run(
            support::cli().arg("emit").arg(directory.source(&source)),
            15,
        );
        // Deep chains compile through the CLI, which has no nesting limit.
        support::success(&output);
    }
}

#[test]
fn standard_cow_clones_boolean_text_and_normal_unwind_disposal_are_exact() {
    native(
        r#"
    function value(s:string):string{return s.valueOf();}
    function text(s:string):string{return s.toString();}
    function boolean(b:boolean):string{return b.toString();}
    function selected(flag:boolean):string{return (flag?"abc".repeat(1):"def".repeat(1)).valueOf();}
    function fail():int{const bad=1/0;return bad;}
    function trap():string{return "owned".repeat(2).toString().substring(fail());}
    function main(){}
    "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Allocations;
        unsafe impl GlobalAlloc for Allocations{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let pointer=unsafe{System.alloc(layout)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}pointer}
            unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
            unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8{unsafe{System.realloc(pointer,layout,size)}}
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;

        fn measured(input:&String,method:fn(&String)->String,owned:bool,allocations:usize){
            let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);let output=method(input);
            assert_eq!(output.as_ref(),input.as_ref());assert_eq!(matches!(output,String::Owned(_)),owned);
            assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);
            if owned && !output.is_empty(){assert_ne!(output.as_ptr(),input.as_ptr());}
            drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),before);
        }
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        let input=String::Owned("é中😀e\u{301}\0\u{FEFF}".into());let live=LIVE.load(Ordering::SeqCst);let pointer=input.as_ptr();
        for method in [function0 as fn(&String)->String,function1]{
            measured(&input,method,true,1);measured(&String::Borrowed("é中😀"),method,false,0);measured(&String::Borrowed(""),method,false,0);measured(&String::Owned(std::string::String::new()),method,true,0);
        }
        for flag in [true,false]{ALLOCS.store(0,Ordering::SeqCst);let text=function2(flag);assert_eq!(text.as_ref(),if flag{"true"}else{"false"});assert!(matches!(text,String::Borrowed(_)));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(text);assert_eq!(LIVE.load(Ordering::SeqCst),live);let output=function3(flag);assert_eq!(output.as_ref(),if flag{"abc"}else{"def"});drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),live);}
        let error=std::panic::catch_unwind(function5).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        assert_eq!(input.as_ptr(),pointer);let output=function0(&input);drop(input);assert_eq!(output.as_ref(),"é中😀e\u{301}\0\u{FEFF}");drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
