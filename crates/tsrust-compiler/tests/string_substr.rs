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
    let path = directory.0.join("string-substr.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-substr-{level}"));
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
fn substr_relative_starts_lengths_defaults_extremes_and_contexts_are_exact() {
    native(
        r#"
        type Row={text:string;};
        function cut(text:string,start:int,length:int):string{return text.substr(start,length);}
        function whole(text:string):string{return text.substr();}
        function suffix(text:string,start:int):string{return text.substr(start);}
        function main(){
            console.log("abcdef".substr());console.log("abcdef".substr(2));console.log("abcdef".substr(-2));
            console.log("abcdef".substr(-99,3));console.log("abcdef".substr(2,2));console.log("abcdef".substr(4,9));
            console.log("abcdef".substr(4,-1));console.log("abcdef".substr(9223372036854775807));
            console.log("abcdef".substr(-9223372036854775808,9223372036854775807));console.log("abcdef".substr(3,-9223372036854775808));console.log("".substr());
            const inferred=clone("abc").substr(1)+"!";console.log(inferred);
            const words=["abc".substr(1),"xy".substr(0,1)];console.log(words[0]);console.log(words[1]);
            const row:Row={text:"field".substr(-3)};console.log(row.text);
            console.log((true?"abc".substr(1):"z").length);console.log("abcdef".substr(1,4).substr(-2,1));
            console.log("abcdef".substring(4,1));console.log("abcdef".slice(1,4));
        }"#,
        "abcdef\ncdef\nef\nabc\ncd\nef\n\n\nabcdef\n\n\nbc!\nbc\nx\neld\n2\nd\nbcd\nbcd\n",
        "",
        r#"
        for (start,length,expected) in [(0,0,""),(0,1,"a"),(2,2,"cd"),(-2,1,"e"),(-9,3,"abc"),(6,9,""),(i64::MIN,i64::MAX,"abcdef"),(i64::MAX,i64::MAX,""),(1,i64::MIN,""),(1,i64::MAX,"bcdef")]{
            for input in [String::Borrowed("abcdef"),String::Owned("abcdef".into())]{let result=function0(std::hint::black_box(&input),std::hint::black_box(start),std::hint::black_box(length));assert_eq!(result.as_ref(),expected);assert_eq!(input.as_ref(),"abcdef");}
        }
        for (start,expected) in [(i64::MIN,"abcdef"),(-6,"abcdef"),(-2,"ef"),(0,"abcdef"),(1,"bcdef"),(6,""),(i64::MAX,"")]{assert_eq!(function2(&String::Borrowed("abcdef"),start).as_ref(),expected);}
        assert_eq!(function1(&String::Borrowed("abcdef")).as_ref(),"abcdef");
        "#,
    );
}

#[test]
fn substr_receiver_arguments_loans_captures_and_temporaries_preserve_order() {
    let analysis = native(
        r#"
        type Box={text:string;};
        function receiver(trace:int[]):string{trace.push(1);return "abcdef".repeat(1);}
        function start(trace:int[]):int{trace.push(2);return 2;}
        function length(trace:int[]):int{trace.push(3);return 2;}
        function zero(trace:int[]):int{trace.push(4);return 0;}
        function view(text:string):string{return text.substr(1,2);}
        function main(){
            const trace:int[]=[];console.log(receiver(trace).substr(start(trace),length(trace)));console.log(receiver(trace).substr(start(trace),zero(trace)));
            const input="owned".repeat(1);const copy=input.substr();const partial=input.substr(1,2);console.log(copy);console.log(partial);console.log(input);
            const box:Box={text:"before".repeat(1)};const saved=box.text.substr(1,3);box.text="changed";console.log(saved);console.log(box.text);
            const read=():string=>{return input.substr(-2);};console.log(read());console.log(input);
            const values=["abc","def"];console.log(values[0].substr(1));console.log(values[1]);
            const mapped=values.map((text:string):string=>{return text.substr(1);});console.log(mapped[1]);console.log(view("abcd"));
            for(const flag of [true,false]){console.log((flag?"abc".repeat(1):"def".repeat(1)).substr(1));for(const text of [(flag?"abcd".repeat(1):"efgh".repeat(1)).substr(1,2)]){console.log(text);}}
            console.log(`${input.substr(1,2)}`.concat("!"));
            for(const item of trace){console.log(item);}
        }"#,
        "cd\n\nowned\nwn\nowned\nefo\nchanged\ned\nowned\nbc\ndef\nef\nbc\nbc\nbc\nef\nfg\nwn!\n1\n2\n3\n1\n2\n4\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("view.text"), Some(&Mode::Read));
    for name in [
        "receiver.trace",
        "start.trace",
        "length.trace",
        "zero.trace",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Mutate));
    }
    for source in [
        "function take(text:string):int{const moved=text;return moved.length;}function main(){const text=\"owned\";text.substr(take(text));}",
        "type Box={text:string;};function change(box:Box):int{box.text=\"changed\";return 0;}function main(){const box:Box={text:\"owned\"};box.text.substr(0,change(box));}",
        "function take(text:string):int{const moved=text;return moved.length;}function main(){const a=\"a\";const b=\"b\";(true?a:b).substr(0,take(b));}",
    ] {
        assert_eq!(
            check(source).err().unwrap().code,
            "conflicting-access",
            "{source}"
        );
    }
}

#[test]
fn substr_utf8_empty_ranges_strict_types_and_boundary_traps_are_exact() {
    native(
        r#"
        function cut(text:string,start:int,length:int):string{return text.substr(start,length);}
        function suffix(text:string,start:int):string{return text.substr(start);}
        function main(){
            const text="é中😀e\u0301\0\uFEFF";
            console.log(text.substr(0,2));console.log(text.substr(2,3));console.log(text.substr(5,4));console.log(text.substr(9,3));console.log(text.substr(12,1));console.log(text.substr(-3));
            console.log(text.substr(1,0));console.log(text.substr(3,-1));console.log(text.substr(7,-9223372036854775808));console.log(text.substr(9223372036854775807,1));
            console.log("hello".substr(-3,2));console.log("é😀x".substr(2,4));
        }"#,
        "é\n中\n😀\né\n\0\n\u{FEFF}\n\n\n\n\nll\n😀\n",
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        for input in [String::Borrowed("é😀x"),String::Owned("é😀x".into())]{
            for (start,length,expected) in [(0,2,"é"),(2,4,"😀"),(-1,1,"x"),(-5,4,"😀"),(1,0,""),(3,-1,""),(3,i64::MIN,""),(i64::MAX,i64::MAX,"")]{assert_eq!(function0(&input,start,length).as_ref(),expected);}
            for (start,length) in [(1,1),(0,1),(2,1),(3,1),(-4,1),(-2,2)]{
                let error=std::panic::catch_unwind(||function0(&input,start,length)).unwrap_err();
                let message=error.downcast_ref::<std::string::String>().map(|s|s.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();assert!(message.contains("string substr is not on UTF-8 boundaries at source bytes"));drop(error);
            }
            let error=std::panic::catch_unwind(||function1(&input,1)).unwrap_err();drop(error);
        }
        for text in [String::Borrowed(""),String::Owned(std::string::String::new())]{assert!(function0(&text,i64::MIN,i64::MAX).is_empty());assert!(function1(&text,1).is_empty());}
        std::panic::set_hook(hook);
        "#,
    );
    for (source, code) in [
        ("function main(){\"abc\".substr(0,1,2);}", "arity"),
        ("function main(){\"abc\".substr(0.0);}", "type"),
        ("function main(){\"abc\".substr(0,1.0);}", "type"),
        ("function main(){\"abc\".substr(true);}", "type"),
        ("function main(){[1].substr(0);}", "type"),
        ("function main(){const value:int=\"abc\".substr();}", "type"),
        (
            "function main(){const text=\"abc\";const moved=text;console.log(text.substr());}",
            "transferred",
        ),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
        assert_ne!(error.code, "internal");
    }
}

#[test]
fn substr_standard_cow_allocations_and_normal_unwind_disposal_are_observed() {
    native(
        r#"
        function cut(text:string,start:int,length:int):string{return text.substr(start,length);}
        function whole(text:string):string{return text.substr();}
        function fail():int{const bad=1/0;return bad;}
        function argumentTrap():string{return "temporary".repeat(2).substr(0,fail());}
        function boundaryTrap():string{return "é😀".repeat(1).substr(1,1);}
        function aggregateTrap():int{const values=["owned".repeat(1).substr(1,2),"other".repeat(1).substr()];const bad=1/0;return bad;}
        function main(){}
        "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Allocations;
        unsafe impl GlobalAlloc for Allocations{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){LIVE.fetch_add(layout.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
            unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe{System.dealloc(p,layout)};LIVE.fetch_sub(layout.size() as isize,Ordering::SeqCst);}
            unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8{let next=unsafe{System.realloc(p,layout,size)};if !next.is_null(){LIVE.fetch_add(size as isize-layout.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}next}
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        let borrowed=String::Borrowed("é😀x");ALLOCS.store(0,Ordering::SeqCst);let whole=function1(std::hint::black_box(&borrowed));assert!(matches!(whole,String::Borrowed(_)));assert_eq!(whole.as_ptr(),borrowed.as_ptr());assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(whole);
        ALLOCS.store(0,Ordering::SeqCst);let partial=function0(std::hint::black_box(&borrowed),2,4);assert_eq!(partial.as_ref(),"😀");assert!(matches!(partial,String::Borrowed(_)));assert_eq!(partial.as_ptr(),borrowed.as_ref().get(2..6).unwrap().as_ptr());assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(partial);
        let input=String::Owned("é😀x".into());let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);let whole=function1(std::hint::black_box(&input));assert!(matches!(whole,String::Owned(_)));assert_eq!(whole.as_ref(),"é😀x");assert_ne!(whole.as_ptr(),input.as_ptr());assert_eq!(ALLOCS.load(Ordering::SeqCst),1);drop(whole);assert_eq!(LIVE.load(Ordering::SeqCst),before);
        ALLOCS.store(0,Ordering::SeqCst);let partial=function0(std::hint::black_box(&input),2,4);assert_eq!(partial.as_ref(),"😀");assert!(matches!(partial,String::Owned(_)));assert_ne!(partial.as_ptr(),input.as_ptr());assert_eq!(ALLOCS.load(Ordering::SeqCst),1);drop(input);assert_eq!(partial.as_ref(),"😀");drop(partial);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for (start,length) in [(1,0),(3,-1),(3,i64::MIN),(i64::MAX,i64::MAX)]{ALLOCS.store(0,Ordering::SeqCst);let empty=function0(&borrowed,start,length);assert!(empty.is_empty());assert!(matches!(empty,String::Borrowed(_)));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(empty);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        for call in [function3 as fn()->String,function4]{let error=std::panic::catch_unwind(call).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function5).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
