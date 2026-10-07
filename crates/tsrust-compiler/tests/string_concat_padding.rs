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
    let path = directory.0.join("string-concat-padding.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-concat-padding-{level}"));
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
fn concat_and_padding_preserve_native_byte_semantics_and_existing_array_contexts() {
    native(
        r#"
        function numbers():int[]{return [].concat([1],2);}
        function main(){
            console.log("".concat());console.log("a".concat());
            console.log("a".concat("","b","é","😀","\u0000","e\u0301"));
            console.log("x".padStart(-9));console.log("x".padEnd(0));
            console.log("abc".padStart(2,"é"));console.log("abc".padEnd(3,"😀"));
            console.log("x".padStart(4));console.log("x".padEnd(4));
            console.log("x".padStart(6,"ab"));console.log("x".padEnd(6,"ab"));
            console.log("x".padStart(7,"é😀"));console.log("x".padEnd(7,"é😀"));
            console.log("x".padStart(3,"é😀"));console.log("x".padEnd(3,"é😀"));
            console.log("x".padStart(5,"😀"));console.log("x".padEnd(5,"😀"));
            console.log("x".padStart(4,"e\u0301"));console.log("x".padEnd(4,"e\u0301"));
            console.log("x".padStart(2,"e\u0301"));console.log("x".padEnd(2,"e\u0301"));
            console.log("".padStart(3,"éx"));console.log("".padEnd(10,"😀ab"));
            console.log("é".padStart(4));console.log("😀".padEnd(5));
            console.log("a\u0000".concat("😀"));console.log("a\u0000".padEnd(4,"é"));
            console.log("x".padStart(9223372036854775807,""));
            console.log("x".padEnd(-9223372036854775808,"é"));
            console.log("".padEnd(9223372036854775807,"").length);
            console.log("x".concat("y","z").padStart(5,"0").padEnd(6,"!"));
            const values=numbers();console.log(values.length);console.log(values[0]);console.log(values[1]);
            const words=["a".concat("b"),"c"].concat(["d"]);console.log(words[0]);console.log(words[2]);
            const fixed=[1,2,3];console.log(fixed[2]);
        }
        "#,
        "\na\nabé😀\0é\nx\nx\nabc\nabc\n   x\nx   \nababax\nxababa\né😀x\nxé😀\néx\nxé\n😀x\nx😀\néx\nxé\nex\nxe\néx\n😀ab😀\n  é\n😀 \na\0😀\na\0é\nx\nx\n0\n00xyz!\n2\n1\n2\nab\nd\n3\n",
        "",
        "",
    );
}

#[test]
fn operands_run_once_in_order_and_selected_temporaries_survive_borrowed_consumers() {
    let analysis = native(
        r#"
        type Box={text:string;};
        function receiver():string{console.log("receiver");return "x".repeat(1);}
        function first():string{console.log("first");return "A".repeat(1);}
        function last():string{console.log("last");return "B".repeat(1);}
        function target():int{console.log("target");return 0;}
        function filler():string{console.log("filler");return "é".repeat(1);}
        function compose(input:string,other:string):string{return input.concat(other).padEnd(5,"!");}
        function invoke(make:Mutable<Function<string>>):string{return make().padStart(4,"0");}
        function factory():()=>string{return ():string=>{return "factory";};}
        function empty():string{console.log("empty");return "";}
        function main(){
            console.log(receiver().concat(first(),last()));
            console.log(receiver().padStart(target(),filler()));
            console.log(receiver().padEnd(target(),filler()));console.log(receiver().concat());
            const box:Box={text:"old".repeat(1)};const saved=box.text.padEnd(5,"z");box.text="changed";console.log(saved);
            const words=["a".repeat(1),"é".repeat(1)];const joined=words[1].concat(words[0]);words[0]="X";console.log(joined);console.log(words[1]);
            const captured="xy".repeat(1);const make=():string=>{return captured.concat("!");};console.log(invoke(make));console.log(captured);
            console.log(compose("a","b"));
            const produced=factory();console.log(produced().concat("!").padEnd(9,"."));
            const inferred=clone("x").concat()+clone("y").concat();console.log(inferred);
            console.log("static".concat(empty(),empty()));
            console.log("x".padStart(4,"\uFEFFa"));console.log("x".padEnd(4,"中a"));
            const left="l".repeat(1);const right="r".repeat(1);
            console.log((true?left:right).concat(false?left:right));console.log(left);console.log(right);
            for(const flag of [true,false]){
                console.log((flag?"x".repeat(1):"y".repeat(1)).padStart(3,flag?"a".repeat(1):"b".repeat(1)).concat(flag?"!":"."));
                console.log((flag?"x".repeat(1):"y".repeat(1)).concat(flag?"1".repeat(1):"2".repeat(1),flag?"3".repeat(1):"4".repeat(1)));
                for(const text of [flag?"q".repeat(1):"r".repeat(1)]){console.log(text.padEnd(2,flag?"!".repeat(1):".".repeat(1)));}
            }
        }
        "#,
        "receiver\nfirst\nlast\nxAB\nreceiver\ntarget\nfiller\nx\nreceiver\ntarget\nfiller\nx\nreceiver\nx\noldzz\néa\né\n0xy!\nxy\nab!!!\nfactory!.\nxy\nempty\nempty\nstatic\n\u{FEFF}x\nx中\nlr\nl\nr\naax!\nx13\nq!\nbby.\ny24\nr.\n",
        "",
        "",
    );
    for source in [
        "function main(){const values=clone([1]);values.concat(\"x\");}",
    ] {
        assert_eq!(check(source).err().unwrap().code, "type", "{source}");
    }
    for name in ["compose.input", "compose.other"] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
}

#[test]
fn strict_types_arity_and_all_prior_operand_loans_reject_invalid_calls() {
    for (source, code) in [
        ("function main(){\"x\".concat(1);}", "type"),
        ("function main(){\"x\".concat(true);}", "type"),
        ("function main(){\"x\".concat([\"x\"]);}", "type"),
        ("function main(){(1).concat(\"x\");}", "type"),
        ("function main(){\"x\".padStart();}", "arity"),
        ("function main(){\"x\".padEnd(2,\"x\",\"y\");}", "arity"),
        ("function main(){\"x\".padStart(2.0);}", "type"),
        ("function main(){\"x\".padEnd(true);}", "type"),
        ("function main(){\"x\".padEnd(3,1);}", "type"),
        ("function main(){[1].padStart(3);}", "type"),
        ("function main(){\"x\".padStart(,\"x\");}", "syntax"),
        (
            "function consume(value:string):string{return value;}function main(){const text=\"x\";text.concat(consume(text));}",
            "conflicting-access",
        ),
        (
            "function consume(value:string):string{return value;}function main(){const text=\"x\";\"a\".concat(text,consume(text));}",
            "conflicting-access",
        ),
        (
            "function consume(value:string):string{return value;}function main(){const left=\"x\";const right=\"y\";\"a\".concat(true?left:right,consume(right));}",
            "conflicting-access",
        ),
        (
            "type Box={text:string;};function change(box:Box):string{box.text=\"new\";return \"x\";}function main(){const box:Box={text:\"x\"};\"a\".concat(box.text,change(box));}",
            "conflicting-access",
        ),
        (
            "type Box={text:string;};function size(box:Box):int{box.text=\"new\";return 0;}function main(){const box:Box={text:\"x\"};box.text.padStart(size(box),\"\");}",
            "conflicting-access",
        ),
        (
            "type Box={text:string;};function change(box:Box):string{box.text=\"new\";return \"\";}function main(){const box:Box={text:\"x\"};(true?box.text:\"y\").padEnd(0,change(box));}",
            "conflicting-access",
        ),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
        assert_ne!(error.code, "internal");
    }
    // Use ordinary-stack CLI processes for both accepted and rejected chains.
    for method in ["concat(\"\")", "padStart(1)", "padEnd(1,\"x\")"] {
        let directory = support::TempDirectory::new();
        let source = format!(
            "function main(){{console.log(\"x\"{});}}",
            format!(".{method}").repeat(48)
        );
        let path = directory.source(&source);
        let output = support::run(support::cli().arg("emit").arg(path), 15);
        support::success(&output);
        assert!(output.stderr.is_empty());
        let source = format!(
            "function main(){{console.log(\"x\"{});}}",
            format!(".{method}").repeat(200)
        );
        let error = support::reject(&source, "unsupported");
        assert!(error.message.contains("128"));
    }
}

#[test]
fn independent_owned_results_single_reservation_and_checked_failures_dispose_cleanly() {
    native(
        r#"
        function concat(value:string,other:string):string{return value.concat(other);}
        function copy(value:string):string{return value.concat();}
        function start(value:string,target:int,filler:string):string{return value.padStart(target,filler);}
        function end(value:string,target:int,filler:string):string{return value.padEnd(target,filler);}
        function bad():string{return "temp".repeat(4).padStart(17,"é");}
        function huge():string{return "temp".repeat(4).padEnd(9223372036854775807,"x");}
        function fail():string{const bad=1/0;return "bad";}
        function rhsTrap():string{return "temp".repeat(4).concat(fail());}
        function lazy(flag:boolean):string{return (flag?"x".repeat(1):"y".repeat(1)).padEnd(3,flag?"a".repeat(1):"b".repeat(1));}
        function main(){}
        "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);static REALLOCS:AtomicUsize=AtomicUsize::new(0);struct Allocations;
        unsafe impl GlobalAlloc for Allocations{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let pointer=unsafe{System.alloc(layout)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}pointer}
            unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
            unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8{REALLOCS.fetch_add(1,Ordering::SeqCst);unsafe{System.realloc(pointer,layout,size)}}
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;
        fn measured(make:impl FnOnce()->String,input:&String,expected:&str,owned:bool){
            let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);REALLOCS.store(0,Ordering::SeqCst);
            let output=make();assert_eq!(matches!(output,String::Owned(_)),owned);assert_eq!(output.as_ref(),expected);
            if owned && !output.is_empty(){assert_ne!(output.as_ptr(),input.as_ptr());assert_eq!(ALLOCS.load(Ordering::SeqCst),1);}else{assert_eq!(ALLOCS.load(Ordering::SeqCst),0);}
            assert_eq!(REALLOCS.load(Ordering::SeqCst),0);drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),before);
        }
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        let input=String::Owned("éx".into());let live=LIVE.load(Ordering::SeqCst);let pointer=input.as_ptr();
        measured(||function0(&input,&input),&input,"éxéx",true);measured(||function1(&input),&input,"éx",true);
        measured(||function2(&input,7,&String::Borrowed("ab")),&input,"ababéx",true);
        measured(||function3(&input,9,&String::Borrowed("é")),&input,"éxééé",true);
        measured(||function2(&input,-1,&String::Borrowed("é")),&input,"éx",true);
        measured(||function3(&input,3,&String::Borrowed("😀")),&input,"éx",true);
        measured(||function2(&input,i64::MAX,&String::Borrowed("")),&input,"éx",true);
        measured(||function3(&String::Borrowed(""),i64::MAX,&String::Borrowed("")),&input,"",false);
        measured(||function1(&String::Borrowed("literal")),&input,"literal",false);
        measured(||function0(&String::Borrowed("literal"),&String::Borrowed("")),&input,"literal",false);
        measured(||function2(&String::Borrowed("literal"),0,&String::Borrowed("é")),&input,"literal",false);
        measured(||function3(&String::Borrowed("literal"),i64::MAX,&String::Borrowed("")),&input,"literal",false);
        assert_eq!(input.as_ptr(),pointer);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        for (target,filler) in [(2,"é"),(5,"é😀"),(9,"é😀e\u{301}"),(3,"😀"),(3,"e\u{301}")]{
            for pad in [function2 as fn(&String,i64,&String)->String,function3]{
                let error=std::panic::catch_unwind(||pad(&String::Borrowed("x"),target,&String::Borrowed(filler))).unwrap_err();
                let text=error.downcast_ref::<std::string::String>().map(|s|s.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();
                assert!(text.contains("UTF-8 boundary")&&text.contains("source bytes"));drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),live);
            }
        }
        for run in [function4 as fn()->String,function5,function7]{
            let error=std::panic::catch_unwind(run).unwrap_err();
            let text=error.downcast_ref::<std::string::String>().map(|s|s.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();
            assert!(text.contains("source bytes"));drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        }
        let error=std::panic::catch_unwind(||string_owned_capacity(usize::MAX,"capacity witness")).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        for flag in [true,false]{let output=function8(flag);assert_eq!(output.as_ref(),if flag{"xaa"}else{"ybb"});drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),live);}
        let copy=function1(&input);drop(input);assert_eq!(copy.as_ref(),"éx");drop(copy);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        std::panic::set_hook(hook);
        "#,
    );
    let analysis = check("function main(){\"x\".concat(\"y\").padEnd(4);}").unwrap();
    assert!(!analysis.emit_rust().contains("enum Array<"));
}
