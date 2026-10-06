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
    let path = directory.0.join("string-html.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-html-{level}"));
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
fn all_thirteen_wrappers_raw_bodies_and_quote_only_attributes_are_exact() {
    let analysis = native(
        r#"
        function all(text:string,attribute:string):string[]{return [text.anchor(attribute),text.big(),text.blink(),text.bold(),text.fixed(),text.fontcolor(attribute),text.fontsize(attribute),text.italics(),text.link(attribute),text.small(),text.strike(),text.sub(),text.sup()];}
        function main(){const values=all("x","a\"&<");for(const value of values){console.log(value);}console.log("é😀e\u0301\0<>&\"".link("\"'&<>é😀\0"));console.log("".bold());console.log("".anchor(""));}
        "#,
        "<a name=\"a&quot;&<\">x</a>\n<big>x</big>\n<blink>x</blink>\n<b>x</b>\n<tt>x</tt>\n<font color=\"a&quot;&<\">x</font>\n<font size=\"a&quot;&<\">x</font>\n<i>x</i>\n<a href=\"a&quot;&<\">x</a>\n<small>x</small>\n<strike>x</strike>\n<sub>x</sub>\n<sup>x</sup>\n<a href=\"&quot;'&<>é😀\0\">é😀é\0<>&\"</a>\n<b></b>\n<a name=\"\"></a>\n",
        "",
        r#"
        let golden=["<a name=\"a&quot;&<\">x</a>","<big>x</big>","<blink>x</blink>","<b>x</b>","<tt>x</tt>","<font color=\"a&quot;&<\">x</font>","<font size=\"a&quot;&<\">x</font>","<i>x</i>","<a href=\"a&quot;&<\">x</a>","<small>x</small>","<strike>x</strike>","<sub>x</sub>","<sup>x</sup>"];
        for owned in [false,true]{
            let text=if owned{String::Owned("x".to_owned())}else{String::Borrowed("x")};
            let attribute=if owned{String::Owned("a\"&<".to_owned())}else{String::Borrowed("a\"&<")};
            let outputs=function0(std::hint::black_box(&text),std::hint::black_box(&attribute));
            assert_eq!(outputs.len(),13);assert_eq!(text.as_ref(),"x");assert_eq!(attribute.as_ref(),"a\"&<");drop(text);drop(attribute);
            for (index,expected) in golden.into_iter().enumerate(){let output=outputs.get(index).unwrap();assert_eq!(output.as_ref(),expected);assert!(matches!(output,String::Owned(_)));}
        }
        "#,
    );
    assert_eq!(analysis.contracts().get("all.text"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("all.attribute"), Some(&Mode::Read));
}

#[test]
fn wrapper_receiver_attribute_order_temporary_contexts_and_reuse_are_preserved() {
    native(
        r#"
        type Box={text:string;};
        function receiver(trace:int[]):string{trace.push(1);return "x".repeat(1);}
        function attribute(trace:int[]):string{trace.push(2);return "q\"&".repeat(1);}
        function index(trace:int[]):int{trace.push(3);return 0;}
        function view(text:string):string{return text.bold();}
        function main(){
            const trace:int[]=[];console.log(receiver(trace).link(attribute(trace)));receiver(trace).bold();
            const words=["a","b"];console.log(words[index(trace)].anchor(attribute(trace)));console.log(words[0]);
            const text="owned".repeat(1);const wrapped=text.bold();const read=():string=>{return text.italics();};console.log(read());console.log(wrapped);console.log(text);
            const box:Box={text:"field".repeat(1)};const saved=box.text.fontcolor("red");box.text="changed";console.log(saved);console.log(box.text);
            const result:Box={text:"x".bold()};console.log(result.text);const mapped=words.map((value:string):string=>{return value.small();});console.log(mapped[1]);
            console.log("x".bold().italics());console.log(view("y"));console.log(`${text.sub()}`);
            for(const flag of [true,false]){console.log((flag?"a".repeat(1):"b".repeat(1)).link(flag?"c".repeat(1):"d".repeat(1)));for(const value of [(flag?"a".repeat(1):"b".repeat(1)).sup()]){console.log(value);}}
            for(const value of trace){console.log(value);}
        }"#,
        "<a href=\"q&quot;&\">x</a>\n<a name=\"q&quot;&\">a</a>\na\n<i>owned</i>\n<b>owned</b>\nowned\n<font color=\"red\">field</font>\nchanged\n<b>x</b>\n<small>b</small>\n<i><b>x</b></i>\n<b>y</b>\n<sub>owned</sub>\n<a href=\"c\">a</a>\n<sup>a</sup>\n<a href=\"d\">b</a>\n<sup>b</sup>\n1\n2\n1\n3\n2\n",
        "",
        "",
    );
    let analysis = check("function fixed():string{const words=[\"a\",\"b\"];return words[0].bold();}function main(){console.log(fixed());}").unwrap();
    assert!(analysis.emit_rust().contains("LiteralStringArray"));
}

#[test]
fn wrappers_strict_signatures_transfers_and_incoming_loans_are_checked() {
    for method in [
        "big", "blink", "bold", "fixed", "italics", "small", "strike", "sub", "sup",
    ] {
        support::reject(
            &format!("function main(){{\"x\".{method}(\"unexpected\");}}"),
            "arity",
        );
        support::reject(&format!("function main(){{[1].{method}();}}"), "type");
        support::reject(
            &format!("function main(){{const value:int=\"x\".{method}();}}"),
            "type",
        );
    }
    for method in ["anchor", "fontcolor", "fontsize", "link"] {
        for args in ["", "\"a\",\"b\""] {
            support::reject(
                &format!("function main(){{\"x\".{method}({args});}}"),
                "arity",
            );
        }
        for arg in ["1", "1.0", "true", "null", "undefined", "[\"a\"]"] {
            support::reject(
                &format!("function main(){{\"x\".{method}({arg});}}"),
                "type",
            );
        }
        let source = format!(
            "function take(text:string):string{{return text;}}function main(){{const text=\"x\".repeat(1);text.{method}(take(text));}}"
        );
        let error = support::reject(&source, "conflicting-access");
        assert!(error.span.end > error.span.start);
    }
    for (source, code) in [
        (
            "function main(){const text=\"x\";const moved=text;console.log(text.bold());}",
            "transferred",
        ),
        (
            "type Box={text:string;};function mutate(box:Box):string{box.text=\"new\";return \"a\";}function main(){const box:Box={text:\"old\"};box.text.link(mutate(box));}",
            "conflicting-access",
        ),
        (
            "function take(text:string):string{return text;}function main(){const a=\"a\";const b=\"b\";(true?a:b).anchor(take(b));}",
            "conflicting-access",
        ),
        (
            "function read(text:string,n:int):int{return text.length+n;}function take(text:string):int{const moved=text;return moved.length;}function main(){const text=\"x\";read(text,text.bold().length+take(text));}",
            "conflicting-access",
        ),
    ] {
        support::reject(source, code);
    }
    check("function take(text:string):string{return text;}function main(){const text=\"x\";console.log(clone(text).link(take(text)));}").unwrap();
}

#[test]
fn owned_wrapper_outputs_allocate_and_dispose_on_normal_and_unwind_paths() {
    native(
        r#"
        function wrap(text:string):string{return text.bold();}
        function attributed(text:string,value:string):string{return text.link(value);}
        function fail():string{const value=1/0;return "unused";}
        function argumentTrap():string{return "receiver".repeat(2).link(fail());}
        function aggregateTrap():int{const outputs=["owned".repeat(1).bold(),"other".repeat(1).fontcolor("q\"&".repeat(1))];const value=1/0;return value;}
        function nestedTrap():string{return "owned".repeat(1).bold().link(fail());}
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
        for owned in [false,true]{
            let input=if owned{String::Owned("é😀\0<&\"".to_owned())}else{String::Borrowed("é😀\0<&\"")};
            let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
            let result=function0(std::hint::black_box(&input));assert_eq!(result.as_ref(),"<b>é😀\0<&\"</b>");assert!(matches!(result,String::Owned(_)));assert!(ALLOCS.load(Ordering::SeqCst)>0);assert_ne!(result.as_ptr(),input.as_ptr());drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),before);
            let attribute=String::Owned("q\"&\0😀".to_owned());let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
            let result=function1(std::hint::black_box(&input),std::hint::black_box(&attribute));assert_eq!(result.as_ref(),"<a href=\"q&quot;&\0😀\">é😀\0<&\"</a>");assert!(matches!(result,String::Owned(_)));assert!(ALLOCS.load(Ordering::SeqCst)>0);assert_eq!(input.as_ref(),"é😀\0<&\"");assert_eq!(attribute.as_ref(),"q\"&\0😀");drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),before);drop(input);drop(attribute);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        let empty=String::Borrowed("");let result=function0(std::hint::black_box(&empty));assert_eq!(result.as_ref(),"<b></b>");assert!(matches!(result,String::Owned(_)));drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for size in [1,4,17,4096]{let input=String::Owned("é".repeat(size));let result=function1(std::hint::black_box(&input),&String::Borrowed("\"\""));assert_eq!(result.len(),input.len()+27);drop(input);assert!(result.starts_with("<a href=\"&quot;&quot;\">"));drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        for call in [function3 as fn()->String,function5]{let error=std::panic::catch_unwind(call).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function4).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
