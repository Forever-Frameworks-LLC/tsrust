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
    let path = directory.0.join("string-split.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-split-{level}"));
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
fn split_content_nonoverlap_scalars_defaults_and_u32_limits_are_exact() {
    native(
        r#"
        function pieces(text:string,separator:string,limit:int):string[]{return text.split(separator,limit);}
        function unlimited(text:string,separator:string):string[]{return text.split(separator);}
        function whole(text:string):string[]{return text.split();}
        function main(){
            const pieces="a,,b,".split(",",2);console.log(pieces.length);console.log(pieces[0]);console.log(pieces[1]);
            console.log("aaaaa".split("aa")[2]);console.log("x".split("longer")[0]);
            console.log("".split().length);console.log("".split(",").length);console.log("".split("").length);
            const scalars="é😀e\u0301\0".split("");console.log(scalars.length);for(const scalar of scalars){console.log(scalar);}
            for(const limit of [0,1,2,-1,4294967296,4294967297,-4294967296,-9223372036854775808,9223372036854775807]){console.log("a,b,c".split(",",limit).length);}
        }"#,
        "2\na\n\na\nx\n1\n1\n0\n5\né\n😀\ne\ń\n\0\n0\n1\n2\n3\n0\n1\n0\n0\n3\n",
        "",
        r#"
        let cases:[(&'static str,&str,&[&str]);11]=[
            ("",",",&[""]),("","",&[]),("a,,b,",",",&["a","","b",""]),
            (",a,",",",&["","a",""]),("aaaaa","aa",&["","","a"]),("x","longer",&["x"]),
            ("😀é😀","😀",&["","é",""]),("a\0b\0","\0",&["a","b",""]),
            ("é😀e\u{301}\0","",&["é","😀","e","\u{301}","\0"]),("e\u{301}","é",&["e\u{301}"]),
            ("a|b|c|d|e","|",&["a","b","c","d","e"])
        ];
        for (text,separator,expected) in cases{
            for input in [String::Borrowed(text),String::Owned(text.to_owned())]{
                let separator=String::Owned(separator.to_owned());
                let result=function1(std::hint::black_box(&input),std::hint::black_box(&separator));
                assert_eq!(result.len(),expected.len());for (actual,expected) in result.iter().zip(expected){assert_eq!(actual.as_ref(),*expected);}
                for limit in [i64::MIN,-4294967296,-1,0,1,2,4,4294967296,4294967297,i64::MAX]{
                    let result=function0(std::hint::black_box(&input),std::hint::black_box(&separator),std::hint::black_box(limit));
                    let count=expected.len().min((limit as u32) as usize);assert_eq!(result.len(),count);
                    for (actual,expected) in result.iter().zip(expected.iter().take(count)){assert_eq!(actual.as_ref(),*expected);}
                }
                let result=function2(&input);assert_eq!(result.len(),1);assert_eq!(result.get(0).unwrap().as_ref(),text);
            }
        }
        "#,
    );
}

#[test]
fn split_evaluates_receiver_and_all_arguments_once_and_reuses_owners_in_contexts() {
    let analysis = native(
        r#"
        type Box={text:string;};
        function receiver(trace:int[]):string{trace.push(1);return "a,b".repeat(1);}
        function separator(trace:int[]):string{trace.push(2);return ",".repeat(1);}
        function limit(trace:int[]):int{trace.push(3);return 0;}
        function selected(text:string):string[]{return text.split(",");}
        function main(){
            const trace:int[]=[];const empty=receiver(trace).split(separator(trace),limit(trace));console.log(empty.length);for(const value of trace){console.log(value);}
            const box:Box={text:"before,after".repeat(1)};const saved=box.text.split(",");box.text="changed";console.log(saved[0]);console.log(saved[1]);console.log(box.text);
            const text="captured,value".repeat(1);const read=():string[]=>{return text.split(",",1);};console.log(read()[0]);console.log(text);
            const staticValues=["a,b","x,y"];console.log(staticValues[0].split(",")[1]);console.log(staticValues[1]);
            const rows=staticValues.map((value:string):string[]=>{return value.split(",");});console.log(rows[1][0]);
            for(const flag of [true,false]){const result=(flag?"left,tail".repeat(1):"right,tail".repeat(1)).split(",",1);console.log(result[0]);}
            console.log("é,z".split(",")[0].toUpperCase().concat("!"));
            for(const part of selected("p,q")){console.log(part);}
            const whole="original".repeat(1);const copy=whole.split();console.log(whole);console.log(copy[0]);
        }"#,
        "0\n1\n2\n3\nbefore\nafter\nchanged\ncaptured\ncaptured,value\nb\nx,y\nx\nleft\nright\nÉ!\np\nq\noriginal\noriginal\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("selected.text"), Some(&Mode::Read));
    assert_eq!(
        analysis.contracts().get("receiver.trace"),
        Some(&Mode::Mutate)
    );
    assert_eq!(
        analysis.contracts().get("separator.trace"),
        Some(&Mode::Mutate)
    );
    assert_eq!(analysis.contracts().get("limit.trace"), Some(&Mode::Mutate));
}

#[test]
fn split_strict_types_arity_spans_and_borrowed_argument_loans_are_preserved() {
    for (source, code) in [
        ("function main(){\"x\".split(\",\",1,2);}", "arity"),
        ("function main(){\"x\".split(1);}", "type"),
        ("function main(){\"x\".split(true);}", "type"),
        ("function main(){\"x\".split(\",\",1.0);}", "type"),
        ("function main(){\"x\".split(\",\",false);}", "type"),
        ("function main(){[1].split(\",\");}", "type"),
        ("function main(){const n:int=\"x\".split(\",\");}", "type"),
        (
            "function main(){const s=\"x\".repeat(1);const moved=s;console.log(s.split());}",
            "transferred",
        ),
        (
            "type Box={text:string;};function change(box:Box):string{box.text=\"changed\";return \",\";}function main(){const box:Box={text:\"a,b\"};box.text.split(change(box));}",
            "conflicting-access",
        ),
        (
            "type Box={text:string;};function limit(box:Box):int{box.text=\"changed\";return 0;}function main(){const box:Box={text:\",\"};\"a,b\".split(box.text,limit(box));}",
            "conflicting-access",
        ),
        (
            "function read(text:string,parts:string[]):int{return text.length+parts.length;}function take(text:string):string{return text;}function main(){const text=\"owned\".repeat(1);console.log(read(text,take(text).split()));}",
            "conflicting-access",
        ),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
        assert_ne!(error.code, "internal");
    }
}

#[test]
fn split_static_pieces_owned_independence_and_normal_unwind_disposal_are_observed() {
    native(
        r#"
        function pieces(text:string,separator:string,limit:int):string[]{return text.split(separator,limit);}
        function whole(text:string):string[]{return text.split();}
        function fail():int{const bad=1/0;return bad;}
        function argumentTrap():string[]{return "temporary,held".repeat(1).split(",".repeat(1),fail());}
        function bounds(text:string):string{return text.split(",")[99];}
        function resultTrap():int{const parts="held,another".repeat(1).split(",");const bad=1/0;return bad;}
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
        let staticInput=String::Borrowed("é😀,middle,tail");let separator=String::Borrowed(",");
        ALLOCS.store(0,Ordering::SeqCst);let staticPieces=std::hint::black_box(function0(std::hint::black_box(&staticInput),&separator,-1));
        assert_eq!(staticPieces.len(),3);assert_eq!(staticPieces.get(0).unwrap().as_ref(),"é😀");assert_eq!(staticPieces.get(1).unwrap().as_ref(),"middle");assert_eq!(staticPieces.get(2).unwrap().as_ref(),"tail");
        assert!(staticPieces.iter().all(|piece|matches!(piece,String::Borrowed(_))));assert_eq!(ALLOCS.load(Ordering::SeqCst),1);drop(staticPieces);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        ALLOCS.store(0,Ordering::SeqCst);let whole=function1(&staticInput);assert!(matches!(whole.get(0).unwrap(),String::Borrowed(_)));assert_eq!(whole.get(0).unwrap().as_ptr(),staticInput.as_ptr());assert_eq!(ALLOCS.load(Ordering::SeqCst),1);drop(whole);
        let scalars=function0(&String::Borrowed("é😀e\u{301}\0"),&String::Borrowed(""),-1);assert_eq!(scalars.len(),5);assert!(scalars.iter().all(|piece|matches!(piece,String::Borrowed(_))));assert_eq!(scalars.get(1).unwrap().as_ref(),"😀");assert_eq!(scalars.get(4).unwrap().as_ref(),"\0");drop(scalars);
        let owned=String::Owned("first,second,third".into());let pointer=owned.as_ptr();let pieces=function0(std::hint::black_box(&owned),&separator,-1);let whole=function1(&owned);
        assert!(pieces.iter().all(|piece|matches!(piece,String::Owned(_))));assert!(matches!(whole.get(0).unwrap(),String::Owned(_)));assert_ne!(whole.get(0).unwrap().as_ptr(),pointer);assert_ne!(pieces.get(0).unwrap().as_ptr(),pointer);assert_eq!(owned.as_ref(),"first,second,third");drop(owned);
        assert_eq!(pieces.get(0).unwrap().as_ref(),"first");assert_eq!(pieces.get(2).unwrap().as_ref(),"third");assert_eq!(whole.get(0).unwrap().as_ref(),"first,second,third");drop(pieces);drop(whole);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for limit in [0,4294967296,i64::MIN]{ALLOCS.store(0,Ordering::SeqCst);let empty=function0(&staticInput,&separator,std::hint::black_box(limit));assert_eq!(empty.len(),0);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(empty);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        for input in [String::Borrowed(""),String::Owned(std::string::String::new())]{let empty=function0(&input,&String::Borrowed(""),-1);assert_eq!(empty.len(),0);drop(empty);let whole=function1(&input);assert_eq!(whole.len(),1);assert_eq!(whole.get(0).unwrap().as_ref(),"");drop(whole);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function3).err().expect("expected argument panic");drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let input=String::Owned("held,another".into());let before=LIVE.load(Ordering::SeqCst);let error=std::panic::catch_unwind(||function4(&input)).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),before);assert_eq!(input.as_ref(),"held,another");drop(input);
        let error=std::panic::catch_unwind(function5).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
