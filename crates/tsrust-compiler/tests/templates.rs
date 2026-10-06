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
    let path = directory.0.join("templates.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("templates-{level}"));
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
fn template_chunks_nested_scalars_and_expression_contexts_are_exact() {
    native(
        r#"
        type Row={text:string;};
        function describe(text:string,n:int,value:number,flag:boolean):string{return `α:${text}:${n}:${value}:${flag}ω`;}
        function make(prefix:string):(n:int)=>string{return (n:int):string=>{return `${prefix}:${n}`;};}
        function inferred(n:int):string{return `value=${n}`;}
        function main(){
            console.log(`a\`b\${literal}\\c\x41\u{1F600}\0`);
            console.log(`line1
line2`);
            console.log(`before${`inner${2}`}after`);
            console.log(`${-0.0}:${Number.NaN}:${Number.POSITIVE_INFINITY}:${Number.NEGATIVE_INFINITY}`);
            const local=`${true}:${false}`;console.log(local);
            const row:Row={text:`row${3}`};console.log(row.text);
            const values=[`${1}`,`${2}`];console.log(values[0]+values[1]);
            const f=make("captured");console.log(f(7));console.log(inferred(9));
            console.log(describe("é中😀",-2,1.25,true));
            for(const flag of [true,false]){
                console.log(`${flag?"left".repeat(1):"right".repeat(1)}`);
                for(const text of [`item${flag}`]){console.log(text);}
            }
            let condition=true;console.log(condition?`yes${1}`:`no${2}`);
            const parse=():string=>{return `${("a".toUpperCase())}`;};console.log(parse());
        }"#,
        "a`b${literal}\\cA😀\0\nline1\nline2\nbeforeinner2after\n0:NaN:Infinity:-Infinity\ntrue:false\nrow3\n12\ncaptured:7\nvalue=9\nα:é中😀:-2:1.25:trueω\nleft\nitemtrue\nright\nitemfalse\nyes1\nA\n",
        "",
        r#"
        // Number text checked against Node.
        for (value,number_text) in [(f64::NEG_INFINITY,"-Infinity"),(-0.0,"0"),(0.0,"0"),(1.0,"1"),(1e-7,"1e-7"),(1e21,"1e+21"),(f64::INFINITY,"Infinity"),(f64::NAN,"NaN")]{
            let text=String::Borrowed("é\0😀");let result=function0(&text,std::hint::black_box(i64::MIN),std::hint::black_box(value),false);
            let expected=format!("α:é\0😀:{}:{}:falseω",i64::MIN,number_text);assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Owned(_)));assert_eq!(text.as_ref(),"é\0😀");
        }
        let text=String::Owned("owned".into());let result=function0(&text,i64::MAX,2.0,true);drop(text);assert_eq!(result.as_ref(),"α:owned:9223372036854775807:2:trueω");
        "#,
    );
    native(
        "function main(){console.log(`a\\\nb\\\r\nc\\\rd\\\u{2028}e\\\u{2029}f`);console.log(`a\r\nb\rc\u{2028}d\u{2029}e`);console.log(`\\uD83D\\\n\\uDE00${1}`);}",
        "abcdef\na\nb\nc\u{2028}d\u{2029}e\n😀1\n",
        "",
        "",
    );
}

#[test]
fn interpolations_append_before_later_effects_and_preserve_loans_and_captures() {
    let analysis = native(
        r#"
        type Box={text:string;n:int;};
        function read(text:string):string{return `${text}`;}
        function grow(values:string[],trace:int[]):int{trace.push(2);values[0]="after";values.push("new");return values.length;}
        function mark(trace:int[]):string{trace.push(1);return "first".repeat(1);}
        function index(trace:int[]):int{trace.push(3);return 0;}
        function take(text:string):string{return text;}
        function fixed(flag:boolean):int{const values=[1,2,3];const text=`${flag?values[0]:values[2]}`;return values.length+text.length;}
        function main(){
            const trace:int[]=[];const values=["before".repeat(1)];
            console.log(`${mark(trace)}:${values[index(trace)]}:${grow(values,trace)}:${values[0]}`);
            console.log(values[1]);console.log(values[0]);
            const text="owned".repeat(1);console.log(`${text}|${take(text)}`);
            let n=1;console.log(`${n++}:${++n}:${(n=7)}:${n}`);console.log(n);
            console.log(`${false&&(++n>0)}:${true||(++n>0)}:${n}`);
            const box:Box={text:"moved".repeat(1),n:8};const moved=box.text;console.log(`${box.n}:${moved}`);box.text=moved;console.log(box.text);
            const captured="capture".repeat(1);const closure=():string=>{return `${captured}`;};console.log(closure());console.log(captured);
            console.log(read("parameter"));console.log(fixed(true));console.log(fixed(false));
            for(const item of trace){console.log(item);}
        }"#,
        "first:before:2:after\nnew\nafter\nowned|owned\n1:3:7:7\n7\nfalse:true:7\n8:moved\nmoved\ncapture\ncapture\nparameter\n4\n4\n1\n3\n2\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("read.text"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("grow.values"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("take.text"), Some(&Mode::Consume));
    assert!(analysis.emit_rust().contains(": [i64; 3] = ["));
}

#[test]
fn template_types_transfers_incoming_loans_and_normal_stack_are_diagnosed() {
    eprintln!("stage: diagnostics entry");
    for (source, code) in [
        ("function main(){console.log(`${[1,2]}`);}", "type"),
        ("function main(){console.log(`${[1.0,2.0]}`);}", "type"),
        (
            "type R={n:int;};function main(){const r:R={n:1};console.log(`${r}`);}",
            "type",
        ),
        (
            "function f():int{return 1;}function main(){console.log(`${f}`);}",
            "type",
        ),
        (
            "function f():void{}function main(){console.log(`${f()}`);}",
            "type",
        ),
        ("function main(){const n:int=`${1}`;}", "type"),
        ("function main(){const b:boolean=`${true}`;}", "type"),
        (
            "function take(s:string):string{return s;}function main(){const s=\"x\".repeat(1);console.log(`${take(s)}${s}`);}",
            "transferred",
        ),
        (
            "type R={s:string;n:int;};function main(){const r:R={s:\"x\",n:1};const s=r.s;console.log(`${r.s}`);}",
            "partial-transfer",
        ),
        (
            "function read(s:string,t:string):int{return s.length+t.length;}function take(s:string):string{return s;}function main(){const s=\"x\".repeat(1);console.log(read(s,`${take(s)}`));}",
            "conflicting-access",
        ),
        (
            "function take(s:string):string{return s;}function main(){const s=\"x\".repeat(1);console.log(s+`${take(s)}`);}",
            "conflicting-access",
        ),
        ("function main(){console.log(`${}`);}", "syntax"),
        ("function main(){console.log(`${/*empty*/}`);}", "syntax"),
        ("function main(){console.log(`unterminated);}", "syntax"),
        ("function main(){console.log(`\\xG0`);}", "syntax"),
        (
            "function main(){console.log(`\\uD800${\"\"}\\uDC00`);}",
            "syntax",
        ),
        (
            "function tag(s:string):string{return s;}function main(){console.log(tag`text`);}",
            "unsupported",
        ),
    ] {
        eprintln!("stage: reject {code}: {source}");
        let error = support::reject(source, code);
        assert_ne!(error.code, "internal");
        assert!(error.span.end > error.span.start);
    }
    // The original worker stack and depth128 contract stay unchanged.
    for count in [48, 200] {
        eprintln!("stage: nested template {count} before");
        let expression = format!("{}\"x\"{}", "`${".repeat(count), "}`".repeat(count));
        let source = format!("function main(){{console.log({expression});}}");
        if count == 48 {
            assert!(check(&source).is_ok());
        } else {
            let error = check(&source).err().unwrap();
            assert_eq!(error.code, "unsupported");
            assert!(error.message.contains("128"));
        }
    }
    eprintln!("stage: nested templates complete; wide template before");
    let wide = format!("function main(){{console.log(`{}`);}}", "${1}".repeat(256));
    assert!(check(&wide).is_ok());
    eprintln!("stage: wide template complete");
    // Existing hostile indexed-array parsing remains bounded on this same stack.
    for count in [100, 200] {
        eprintln!("stage: nested index {count} before");
        let source = format!(
            "function main(){{const a=[1];console.log({}0{});}}",
            "a[".repeat(count),
            "]".repeat(count)
        );
        if count == 100 {
            let analysis = check(&source).unwrap();
            eprintln!("stage: nested index {count} check complete; emit before");
            analysis.emit_rust();
            eprintln!("stage: nested index {count} emit complete");
        } else {
            let error = check(&source).err().unwrap();
            assert_eq!(error.code, "unsupported");
            assert!(error.message.contains("128"));
        }
    }
}

#[test]
fn template_results_and_selected_temporaries_dispose_normally_and_on_unwind() {
    native(
        r#"
        function literal():string{return `é😀`;}
        function owned(text:string):string{return `${text}`;}
        function scalar(value:int):string{return `v=${value}`;}
        function empty():string{return `${""}`;}
        function selected(flag:boolean):string{return `${flag?"left".repeat(1):"right".repeat(1)}:${1}`;}
        function fail():int{const bad=1/0;return bad;}
        function trap():string{return `${"temporary".repeat(1)}:${fail()}`;}
        function bounds():string{const values=["held".repeat(1)];return `${values[9]}`;}
        function aggregate():string[]{return [`first${1}`,`${"second".repeat(1)}`,`third${true}`];}
        function aggregateTrap():string{const values=aggregate();return `${values[0]}:${fail()}`;}
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
        ALLOCS.store(0,Ordering::SeqCst);let literal=std::hint::black_box(function0());assert_eq!(literal.as_ref(),"é😀");assert!(matches!(literal,String::Borrowed(_)));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(literal);
        ALLOCS.store(0,Ordering::SeqCst);let empty=std::hint::black_box(function3());assert!(empty.is_empty());assert!(matches!(empty,String::Owned(_)));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(empty);
        for text in ["","ASCII","é中😀e\u{301}\0"]{
            let input=String::Owned(text.into());let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);let output=std::hint::black_box(function1(std::hint::black_box(&input)));
            assert_eq!(output.as_ref(),text);assert!(matches!(output,String::Owned(_)));if !text.is_empty(){assert_ne!(output.as_ptr(),input.as_ptr());}assert_eq!(input.as_ref(),text);assert_eq!(ALLOCS.load(Ordering::SeqCst),usize::from(!text.is_empty()));drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),before);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        let input=String::Owned("independent".into());let output=function1(&input);drop(input);assert_eq!(output.as_ref(),"independent");drop(output);
        for value in [i64::MIN,-1,0,7,i64::MAX]{let output=std::hint::black_box(function2(std::hint::black_box(value)));assert_eq!(output.as_ref(),format!("v={value}"));drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        for flag in [true,false]{let output=function4(std::hint::black_box(flag));assert_eq!(output.as_ref(),if flag{"left:1"}else{"right:1"});drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let values=function8();assert_eq!(values.len(),3);assert_eq!(values.get(0).unwrap().as_ref(),"first1");assert_eq!(values.get(1).unwrap().as_ref(),"second");assert_eq!(values.get(2).unwrap().as_ref(),"thirdtrue");drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for call in [function6 as fn()->String,function7,function9]{let error=std::panic::catch_unwind(call).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}std::panic::set_hook(hook);
        "#,
    );
}
