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
    let path = directory.0.join("string-well-formed.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-well-formed-{level}"));
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
fn valid_utf8_content_unicode_boundaries_and_independent_results_are_exact() {
    native(
        r#"
        function predicate(text:string):boolean{return text.isWellFormed();}
        function repair(text:string):string{return text.toWellFormed();}
        function main(){
            for(const text of ["","ASCII","é中😀e\u0301\u0000\uFEFF","\uFDD0\uFFFE\uFFFF","\uFFFD","\uDBFF\uDFFF","\uD800\uDC00"]){console.log(text.isWellFormed());}
            console.log("a😀b".charAt(1).isWellFormed());console.log("a😀b".split("")[1].toWellFormed().isWellFormed());
            console.log("é中😀e\u0301\u0000\uFEFF".toWellFormed());
            const text="é中😀".repeat(1);const copy=text.toWellFormed();console.log(copy);console.log(text);
            const empty="".repeat(1);console.log(empty.toWellFormed().isWellFormed());
        }"#,
        "true\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\né中😀é\0\u{FEFF}\né中😀\né中😀\ntrue\n",
        "",
        r#"
        for text in ["","ASCII","\0","\u{feff}","\u{7f}\u{80}\u{7ff}\u{800}\u{d7ff}\u{e000}\u{ffff}\u{10000}\u{10ffff}","é中😀e\u{301}","\u{fdd0}\u{fffe}\u{ffff}","\u{fffd}"]{
            let input=String::Borrowed(text);assert!(function0(std::hint::black_box(&input)));
            let result=function1(std::hint::black_box(&input));assert_eq!(result.as_ref(),text);assert!(matches!(result,String::Borrowed(_)));
            let owned=String::Owned(text.to_owned());assert!(function0(std::hint::black_box(&owned)));
            let copied=function1(std::hint::black_box(&owned));assert_eq!(copied.as_ref(),text);assert!(matches!(copied,String::Owned(_)));
            if !text.is_empty(){assert_ne!(copied.as_ptr(),owned.as_ptr());}drop(owned);assert_eq!(copied.as_ref(),text);
        }
        "#,
    );
}

#[test]
fn receiver_effects_contexts_loans_captures_and_temporaries_are_preserved() {
    let analysis = native(
        r#"
        type Box={text:string;flag:boolean;};
        function predicate(text:string):boolean{return text.isWellFormed();}
        function repair(text:string):string{return text.toWellFormed();}
        function receiver(trace:int[]):string{trace.push(1);return "abc".repeat(1);}
        function index(trace:int[]):int{trace.push(2);return 0;}
        function take(text:string):string{return text;}
        function fixed():boolean{const values=["a","b"];return values[0].isWellFormed();}
        function main(){
            const trace:int[]=[];console.log(receiver(trace).isWellFormed());receiver(trace).isWellFormed();
            console.log(receiver(trace).toWellFormed());
            const values=["abc","def"];console.log(values[index(trace)].isWellFormed());console.log(values[0]);
            const box:Box={text:"é中😀",flag:false};box.flag=box.text.isWellFormed();console.log(box.flag);console.log(box.text.toWellFormed());
            const predicates=["x".isWellFormed(),"y".toWellFormed().isWellFormed()];console.log(predicates[1]);
            const joined=clone("a").toWellFormed()+clone("b").toWellFormed();console.log(joined);
            const text="owned".repeat(1);const action=():boolean=>{return text.isWellFormed();};console.log(action());console.log(text);
            const consumable="owned".repeat(1);const copy=consumable.toWellFormed();console.log(take(consumable));console.log(copy);
            console.log(predicate("x"));console.log(repair("y"));console.log(fixed());
            for(const flag of [true,false]){
                console.log((flag?"abc".repeat(1):"def".repeat(1)).isWellFormed());
                for(const item of [(flag?"abc".repeat(1):"def".repeat(1)).toWellFormed()]){console.log(item);}
                console.log("x".concat((flag?"a".repeat(1):"b".repeat(1)).toWellFormed()));
            }
            for(const n of trace){console.log(n);}
        }"#,
        "true\nabc\ntrue\nabc\ntrue\né中😀\ntrue\nab\ntrue\nowned\nowned\nowned\ntrue\ny\ntrue\ntrue\nabc\nxa\ntrue\ndef\nxb\n1\n1\n1\n2\n",
        "",
        "",
    );
    assert_eq!(
        analysis.contracts().get("predicate.text"),
        Some(&Mode::Read)
    );
    assert_eq!(analysis.contracts().get("repair.text"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("take.text"), Some(&Mode::Consume));
    assert!(analysis.emit_rust().contains("LiteralStringArray"));
    for source in [
        "function take(text:string):string{return text;}function read(text:string,flag:boolean):boolean{return flag;}function main(){const text=\"x\".repeat(1);read(text,take(text).isWellFormed());}",
        "function take(text:string):string{return text;}function read(text:string,copy:string):int{return copy.length;}function main(){const text=\"x\".repeat(1);read(text,take(text).toWellFormed());}",
    ] {
        assert_eq!(check(source).err().unwrap().code, "conflicting-access");
    }
}

#[test]
fn strict_zero_arity_types_unicode_and_normal_stack_diagnostics_are_bounded() {
    for (source, code) in [
        ("function main(){\"x\".isWellFormed(1);}", "arity"),
        ("function main(){\"x\".toWellFormed(1);}", "arity"),
        ("function main(){1.isWellFormed();}", "type"),
        ("function main(){1.0.toWellFormed();}", "type"),
        ("function main(){true.isWellFormed();}", "type"),
        ("function main(){[\"x\"].toWellFormed();}", "type"),
        (
            "type Box={text:string;};function main(){const box:Box={text:\"x\"};box.isWellFormed();}",
            "type",
        ),
        (
            "function main(){const value:string=\"x\".isWellFormed();}",
            "type",
        ),
        (
            "function main(){const value:boolean=\"x\".toWellFormed();}",
            "type",
        ),
        ("function main(){\"x\".toWellFormed(,);}", "syntax"),
    ] {
        let error = support::reject(source, code);
        assert_ne!(error.code, "internal");
        assert!(error.span.end > error.span.start);
    }
    for method in ["isWellFormed", "toWellFormed"] {
        let source = format!("function main(){{1.{method}(missing);}}");
        assert_eq!(check(&source).err().unwrap().code, "arity");
    }
    for escape in [r"\uD800", r"\uDC00", r"\uD800\u0000"] {
        let source = format!("function main(){{console.log(\"{escape}\".isWellFormed());}}");
        let error = check(&source).err().unwrap();
        assert_eq!(error.code, "syntax");
        assert!(error.message.contains("Unpaired surrogate"));
    }
    for depth in [100, 200] {
        let source = format!(
            "function main(){{console.log({}\"x\".isWellFormed());}}",
            "!".repeat(depth)
        );
        let directory = support::TempDirectory::new();
        let output = support::run(
            support::cli().arg("emit").arg(directory.source(&source)),
            15,
        );
        // Deep chains compile through the CLI, which has no nesting limit.
        support::success(&output);
    }
    for count in [48, 200] {
        let source = format!(
            "function main(){{console.log(\"x\"{}.isWellFormed());}}",
            ".toWellFormed()".repeat(count)
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
fn predicate_and_cow_allocations_inputs_and_normal_unwind_disposal_are_observed() {
    native(
        r#"
        function predicate(text:string):boolean{return text.isWellFormed();}
        function repair(text:string):string{return text.toWellFormed();}
        function selected(flag:boolean):boolean{return (flag?"abc".repeat(1):"def".repeat(1)).isWellFormed();}
        function selectedRepair(flag:boolean):string{return (flag?"abc".repeat(1):"def".repeat(1)).toWellFormed();}
        function fail():int{const bad=1/0;return bad;}
        function trap():string{return "owned".repeat(2).toWellFormed().substring(fail());}
        function trapPredicate():boolean{const values=["owned".repeat(2)];return values[9].isWellFormed();}
        function trapUnusedPredicate():int{const values=["owned".repeat(2)];values[9].isWellFormed();return 7;}
        function main(){}
        "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Allocations;
        unsafe impl GlobalAlloc for Allocations{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let pointer=unsafe{System.alloc(layout)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}pointer}
            unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
            unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8{ALLOCS.fetch_add(1,Ordering::SeqCst);unsafe{System.realloc(pointer,layout,size)}}
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;
        fn measured(input:&String,owned:bool,allocations:usize){
            let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
            assert!(std::hint::black_box(function0(std::hint::black_box(input))));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);
            ALLOCS.store(0,Ordering::SeqCst);let output=std::hint::black_box(function1(std::hint::black_box(input)));
            assert_eq!(output.as_ref(),input.as_ref());assert_eq!(matches!(output,String::Owned(_)),owned);assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);
            if owned&&!output.is_empty(){assert_ne!(output.as_ptr(),input.as_ptr());}drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),before);
        }
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for text in ["","ASCII","é中😀e\u{301}\0\u{feff}","\u{fdd0}\u{fffe}\u{10ffff}"]{
            let borrowed=String::Borrowed(text);measured(std::hint::black_box(&borrowed),false,0);
            let owned=String::Owned(text.to_owned());let pointer=owned.as_ptr();measured(std::hint::black_box(&owned),true,usize::from(!text.is_empty()));assert_eq!(owned.as_ptr(),pointer);assert_eq!(owned.as_ref(),text);drop(owned);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        for flag in [true,false]{assert!(std::hint::black_box(function2(std::hint::black_box(flag))));assert_eq!(LIVE.load(Ordering::SeqCst),baseline);let output=function3(std::hint::black_box(flag));assert_eq!(output.as_ref(),if flag{"abc"}else{"def"});drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let owned=String::Owned("é中😀".into());let output=function1(std::hint::black_box(&owned));assert_ne!(output.as_ptr(),owned.as_ptr());drop(owned);assert_eq!(output.as_ref(),"é中😀");drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let error=std::panic::catch_unwind(function5).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let error=std::panic::catch_unwind(function6).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let error=std::panic::catch_unwind(function7).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
