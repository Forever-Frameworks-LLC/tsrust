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
    let path = directory.0.join("string-substring.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-substring-{level}"));
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
fn clamp_swap_default_and_expression_contexts_preserve_existing_string_and_array_methods() {
    native(
        r#"
        function make(value:string):string{return value.substring(3,1);}
        function main(){
            console.log("abc".substring(1));console.log("abc".substring(-4));
            console.log("abc".substring(-9,1));console.log("abc".substring(2,-7));
            console.log("abc".substring(3,1));console.log("abc".substring(9));
            console.log("abc".substring(1,1));console.log("abc".substring(9223372036854775807,-9223372036854775808));
            console.log("abc".substring(-9223372036854775808,9223372036854775807));
            console.log("".substring(0));console.log(make("abcd"));
            const inferred=clone("abc").substring(1)+"!";console.log(inferred);
            const words=["abc".substring(1),"xy".substring(0,1)];console.log(words[0]);console.log(words[1]);
            const text="abc".substring(1).concat("!").padStart(5,"0").padEnd(6,"?");console.log(text);
            console.log("abc".slice(-2));console.log("abc".charAt(1));
            const values=[1,2,3].slice(1,3);console.log(values[0]);console.log(values[1]);
            console.log("abcdef".substring(5,2).substring(1));
        }
    "#,
        "bc\nabc\na\nab\nbc\n\n\nabc\nabc\n\nbc\nbc!\nbc\nx\n00bc!?\nbc\nb\n2\n3\nde\n",
        "",
        "",
    );
}

#[test]
fn once_only_order_borrowed_roots_and_selected_temporaries_cover_all_consumers() {
    let analysis = native(
        r#"
        type Box={text:string;};
        function receiver():string{console.log("receiver");return "abcdef".repeat(1);}
        function start():int{console.log("start");return 4;}
        function end():int{console.log("end");return 2;}
        function empty():int{console.log("empty");return 4;}
        function view(value:string):string{return value.substring(1,3);}
        function invoke(action:Mutable<Function<string>>):string{return action().substring(0,2);}
        function factory():()=>string{return ():string=>{return "abc".repeat(1);};}
        function main(){
            console.log(receiver().substring(start(),end()));
            console.log(receiver().substring(start(),empty()));
            const box:Box={text:"abcd".repeat(1)};const result=box.text.substring(1,3);box.text="changed";console.log(result);
            const texts=["abc".repeat(1),"def".repeat(1)];console.log(texts[0].substring(1));console.log(texts[0]);
            const captured="abcd".repeat(1);const closure=():string=>{return captured.substring(1,4);};console.log(invoke(closure));console.log(captured);
            const action=factory();console.log(action().substring(1));console.log(view("abcdef"));
            const left="abc".repeat(1);const right="def".repeat(1);console.log((true?left:right).substring(1));console.log(left);console.log(right);
            for(const flag of [true,false]){
                console.log((flag?"abc".repeat(1):"def".repeat(1)).substring(1,3).padEnd(3,"!"));
                for(const text of [(flag?"abc".repeat(1):"def".repeat(1)).substring(1)]){console.log(text);}
            }
        }
    "#,
        "receiver\nstart\nend\ncd\nreceiver\nstart\nempty\n\nbc\nbc\nabc\nbc\nabcd\nbc\nbc\nbc\nabc\ndef\nbc!\nbc\nef!\nef\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("view.value"), Some(&Mode::Read));
    for source in [
        "function take(value:string):int{const owner=value;return owner.length;}function main(){const text=\"abc\";text.substring(take(text));}",
        "type Box={text:string;};function size(box:Box):int{box.text=\"changed\";return 0;}function main(){const box:Box={text:\"abc\"};box.text.substring(size(box));}",
        "function take(value:string):int{const owner=value;return owner.length;}function main(){const left=\"a\";const right=\"b\";(true?left:right).substring(0,take(right));}",
    ] {
        assert_eq!(
            check(source).err().unwrap().code,
            "conflicting-access",
            "{source}"
        );
    }
}

#[test]
fn unicode_boundaries_empty_interior_ranges_strict_types_and_normal_stack_limits() {
    native(
        r#"
        function main(){
            const text="é中😀e\u0301\u0000\uFEFF";
            console.log(text.substring(0,2));console.log(text.substring(5,2));console.log(text.substring(5,9));
            console.log(text.substring(9,12));console.log(text.substring(12,13));console.log(text.substring(16,13));
            console.log(text.substring(1,1));console.log(text.substring(3,3));console.log(text.substring(7,7));
            console.log(text.substring(10,12));console.log(text.substring(-9,2));console.log(text.substring(13));
        }
    "#,
        "é\n中\n😀\né\n\0\n\u{FEFF}\n\n\n\ń\né\n\u{FEFF}\n",
        "",
        "",
    );
    for (source, code) in [
        ("function main(){\"x\".substring();}", "arity"),
        ("function main(){\"x\".substring(0,1,2);}", "arity"),
        ("function main(){\"x\".substring(0.0);}", "type"),
        ("function main(){\"x\".substring(0,1.0);}", "type"),
        ("function main(){\"x\".substring(true);}", "type"),
        ("function main(){[1].substring(0);}", "type"),
        ("function main(){\"x\".substring(,1);}", "syntax"),
    ] {
        let error = support::reject(source, code);
        assert_ne!(error.code, "internal");
        assert!(error.span.end > error.span.start);
    }
    for count in [48, 200] {
        let source = format!(
            "function main(){{console.log(\"abc\"{});}}",
            ".substring(0)".repeat(count)
        );
        if count == 48 {
            let directory = support::TempDirectory::new();
            support::success(&support::run(
                support::cli().arg("emit").arg(directory.source(&source)),
                15,
            ));
        } else {
            let error = support::reject(&source, "unsupported");
            assert!(error.message.contains("128"));
        }
    }
}

#[test]
fn standard_cow_storage_independence_allocations_and_boundary_unwind_are_exact() {
    native(
        r#"
        function part(value:string,start:int,end:int):string{return value.substring(start,end);}
        function bad():string{return "é中😀".repeat(2).substring(1,2);}
        function fail():int{const bad=1/0;return bad;}
        function rhsTrap():string{return "temporary".repeat(2).substring(fail());}
        function selected(flag:boolean):string{return (flag?"abc".repeat(1):"def".repeat(1)).substring(1);}
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
        fn measured(input:&String,start:i64,end:i64,expected:&str,owned:bool,allocations:usize){
            let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);let output=function0(input,start,end);
            assert_eq!(output.as_ref(),expected);assert_eq!(matches!(output,String::Owned(_)),owned);
            assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);
            if owned && !output.is_empty(){assert_ne!(output.as_ptr(),input.as_ptr());}
            drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),before);
        }
    "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        let input=String::Owned("é中😀e\u{301}\0\u{FEFF}".into());let live=LIVE.load(Ordering::SeqCst);let pointer=input.as_ptr();
        measured(&input,2,5,"中",true,1);measured(&input,5,2,"中",true,1);
        measured(&input,i64::MIN,i64::MAX,input.as_ref(),true,1);
        for index in [1,3,7,11,14]{measured(&input,index,index,"",false,0);}
        measured(&String::Borrowed("é中😀"),2,5,"中",false,0);
        measured(&String::Borrowed("é中😀"),i64::MIN,i64::MAX,"é中😀",false,0);
        measured(&String::Borrowed(""),0,0,"",false,0);
        for (start,end) in [(1,2),(2,4),(5,8),(9,11),(13,14),(4,2),(8,5),(14,13)]{
            let error=std::panic::catch_unwind(||function0(&input,start,end)).unwrap_err();
            let message=error.downcast_ref::<std::string::String>().map(|s|s.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();
            assert!(message.contains("UTF-8 boundaries")&&message.contains("source bytes"));drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        }
        for run in [function1 as fn()->String,function3]{let error=std::panic::catch_unwind(run).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),live);}
        for flag in [true,false]{let output=function4(flag);assert_eq!(output.as_ref(),if flag{"bc"}else{"ef"});drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),live);}
        assert_eq!(input.as_ptr(),pointer);let output=function0(&input,2,5);drop(input);assert_eq!(output.as_ref(),"中");drop(output);
        assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
    "#,
    );
}
