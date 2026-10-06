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
    let path = directory.0.join("array-join.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-join-{level}"));
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
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn join_all_leaf_types_nested_internal_commas_and_number_text_are_exact() {
    let analysis = native(
        r#"
        type Row={text:string;values:int[];};
        type Node={value:int;children:Node[];};
        function integers(values:int[],separator:string):string{return values.join(separator);}
        function numbers(values:number[],separator:string):string{return values.join(separator);}
        function strings(values:string[],separator:string):string{return values.join(separator);}
        function main(){
            console.log([1,2,3].join());console.log([1,2,3].join(""));console.log([1,2,3].join("::"));
            console.log([true,false,true].join("|"));console.log([null,null].join("|"));console.log([undefined,undefined,undefined].join());
            console.log(["é😀","","e\u0301\0"].join("\"&<"));console.log([""].join("ignored"));console.log([].join());console.log([].join("ignored"));
            console.log([[-1,2],[],[3]].join("|"));console.log([[[1,2],[]],[[3]]].join("|"));
            console.log([[true,false],[],[false]].join("|"));console.log([["a","b"],[],["😀"]].join("|"));
            console.log([[null,null],[],[null]].join("|"));console.log([[undefined,undefined],[],[undefined]].join("|"));
            const rows:Row[]=[{text:"owned".repeat(1),values:[1,2]},{text:"second",values:[]}];console.log(rows.join("|"));console.log([rows].join("!"));
            const nodes:Node[]=[{value:1,children:[]}];console.log(nodes.join());console.log([nodes].join("!"));
            const values:number[]=[-0.0,Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY,1e-7,1e-6,1e20,1e21];console.log(values.join("|"));console.log([values].join("!"));
            console.log([-9223372036854775808,9223372036854775807].join("|"));
        }"#,
        "1,2,3\n123\n1::2::3\ntrue|false|true\n|\n,,\né😀\"&<\"&<é\0\n\n\n\n-1,2||3\n1,2,|3\ntrue,false||false\na,b||😀\n,||\n,||\n[object Object]|[object Object]\n[object Object],[object Object]\n[object Object]\n[object Object]\n0|NaN|Infinity|-Infinity|1e-7|0.000001|100000000000000000000|1e+21\n0,NaN,Infinity,-Infinity,1e-7,0.000001,100000000000000000000,1e+21\n-9223372036854775808|9223372036854775807\n",
        "",
        r#"
        for size in [0,1,4,5,17]{
            let mut values=NumericArray::from([]);for n in 0..size{values.push(n as i64-2);}
            let expected=(0..size).map(|n|(n as i64-2).to_string()).collect::<Vec<_>>().join("😀");
            let result=function0(std::hint::black_box(&values),&String::Borrowed("😀"));assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Owned(_)));assert_eq!(values.len(),size);
        }
        let values=vec![f64::from_bits(0x8000000000000000),f64::NAN,f64::INFINITY,f64::NEG_INFINITY,f64::from_bits(1)];
        let result=function1(std::hint::black_box(&values),&String::Borrowed("|"));assert_eq!(result.as_ref(),"0|NaN|Infinity|-Infinity|5e-324");assert_eq!(values[0].to_bits(),0x8000000000000000);assert!(values[1].is_nan());
        for owned in [false,true]{let value=if owned{String::Owned("é😀\0".to_owned())}else{String::Borrowed("é😀\0")};let values=Array::from([value]);let result=function2(std::hint::black_box(&values),&String::Borrowed("|"));assert!(matches!(result,String::Owned(_)));assert_ne!(result.as_ptr(),values.get(0).unwrap().as_ptr());drop(values);assert_eq!(result.as_ref(),"é😀\0");}
        "#,
    );
    for name in [
        "integers.values",
        "numbers.values",
        "strings.values",
        "strings.separator",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
}

#[test]
fn join_order_lazy_temporaries_captures_reuse_and_fixed_storage_are_preserved() {
    native(
        r#"
        type Box={values:string[];};
        function receiver(trace:int[]):string[]{trace.push(1);return ["a".repeat(1),"b".repeat(1)];}
        function separator(trace:int[]):string{trace.push(2);return "|".repeat(1);}
        function empty(trace:int[]):string[]{trace.push(3);return [];}
        function index(trace:int[]):int{trace.push(4);return 0;}
        function main(){
            const trace:int[]=[];console.log(receiver(trace).join(separator(trace)));receiver(trace).join(separator(trace));console.log(empty(trace).join(separator(trace)));
            const rows=[["x","y"],["z"]];console.log(rows[index(trace)].join(separator(trace)));console.log(rows[0][0]);
            const values=["owned".repeat(1),"other".repeat(1)];const sep="/".repeat(1);const result=values.join(sep);values[0]="changed";console.log(result);console.log(values[0]);console.log(sep);
            const box:Box={values:["field".repeat(1)]};const read=():string=>{return box.values.join("!");};console.log(read());console.log(box.values[0]);
            const output:Box={values:[values.join("!"),"tail"]};console.log(output.values[0]);const mapped=rows.map((row:string[]):string=>{return row.join("+");});console.log(mapped[1]);
            for(const flag of [true,false]){console.log((flag?receiver(trace):empty(trace)).join(flag?separator(trace):"?".repeat(1)));for(const text of [(flag?receiver(trace):empty(trace)).join(flag?separator(trace):"?".repeat(1))]){console.log(text);}}
            console.log(`${values.join("+")}`);for(const value of trace){console.log(value);}
        }"#,
        "a|b\n\nx|y\nx\nowned/other\nchanged\n/\nfield\nfield\nchanged!other\nz\na|b\na|b\n\n\nchanged+other\n1\n2\n1\n2\n3\n2\n4\n2\n1\n2\n1\n2\n3\n3\n",
        "",
        "",
    );
    let fixed=check("function fixed():string{const numbers=[1,2,3];const words=[\"a\",\"b\"];return numbers.join(\"/\")+words.join(\"|\");}function main(){console.log(fixed());}").unwrap().emit_rust();
    assert!(fixed.contains(": [i64; 3] = ["));
    assert!(fixed.contains("LiteralStringArray"));
    assert!(
        !fixed
            .split("// Generated strings")
            .next()
            .unwrap()
            .contains("TsrConsoleValue::Array")
    );
    // Returning joined text needs no console support or diagnostic value tree.
    let isolated=check("function join(values:number[]):string{return values.join();}function main(){join([1.0,2.0]);}").unwrap().emit_rust();
    assert!(!isolated.contains("enum TsrConsoleValue"));
    assert!(!isolated.contains("tsr_console_number"));
}

#[test]
fn join_strict_arguments_loans_transfers_and_normal_depth_limits_are_checked() {
    for (source, code) in [
        ("function main(){[1].join(\"a\",\"b\");}", "arity"),
        ("function main(){[1].join(1);}", "type"),
        ("function main(){[1].join(true);}", "type"),
        ("function main(){[1].join(undefined);}", "type"),
        ("function main(){[1].join(null);}", "type"),
        ("function main(){[1].join([\"x\"]);}", "type"),
        ("function main(){\"x\".join();}", "type"),
        ("function main(){const value:int=[1].join();}", "type"),
        (
            "function main(){const values=[1,2];const moved=values;console.log(values.join());}",
            "transferred",
        ),
        (
            "function grow(values:int[]):string{values.push(3);return \"|\";}function main(){const values=[1,2];values.join(grow(values));}",
            "conflicting-access",
        ),
        (
            "function grow(values:int[]):string{values.push(3);return \"|\";}function main(){const a=[1];const b=[2];(true?a:b).join(grow(b));}",
            "conflicting-access",
        ),
        (
            "function take(values:string[]):string{const moved=values;return \"|\";}function main(){const values=[\"a\"];values.join(take(values));}",
            "conflicting-access",
        ),
        (
            "type Box={values:int[];};function grow(box:Box):string{box.values.push(3);return \"|\";}function main(){const box:Box={values:[1,2]};box.values.join(grow(box));}",
            "conflicting-access",
        ),
        (
            "function take(text:string):int{return text.length;}function main(){const separator=\"x\";const moved=separator;[1].join(separator);}",
            "transferred",
        ),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
    }
    // The established index witness is checked independently of runtime output.
    let valid = format!(
        "function main(){{const a=[0];console.log({}0{});console.log([1].join());}}",
        "a[".repeat(100),
        "]".repeat(100)
    );
    check(&valid).unwrap().emit_rust();
    let hostile = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
        "a[".repeat(200),
        "]".repeat(200)
    );
    support::reject(&hostile, "unsupported");
    let deep = format!(
        "function main(){{console.log({}1{}.join());}}",
        "[".repeat(40),
        "]".repeat(40)
    );
    check(&deep).unwrap().emit_rust();
}

#[test]
fn join_direct_buffer_allocation_and_normal_unwind_disposal_are_observed() {
    native(
        r#"
        function strings(values:string[],separator:string):string{return values.join(separator);}
        function numbers(values:number[],separator:string):string{return values.join(separator);}
        function integers(values:int[],separator:string):string{return values.join(separator);}
        function fail():string{const bad=1/0;return "unused";}
        function argumentTrap():string{return ["receiver".repeat(3)].join(fail());}
        function aggregateTrap():int{const values=[["owned".repeat(1)].join(),["other".repeat(1)].join("!")];const bad=1/0;return bad;}
        function indexTrap():string{const values=[["owned".repeat(1)]];return values[2].join();}
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
        for count in [0,1,4,5,256]{
            let mut inputs=Array::from([]);for _ in 0..count{inputs.push(String::Owned("é😀\0".to_owned()));}
            let expected=(0..count).map(|_|"é😀\0").collect::<Vec<_>>().join("|");let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
            let result=function0(std::hint::black_box(&inputs),&String::Borrowed("|"));let allocations=ALLOCS.load(Ordering::SeqCst);assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Owned(_)));assert!(allocations<=16,"no per-element clone or diagnostic allocation: {allocations}");if count==0{assert_eq!(allocations,0);}drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),before);
            let result=function0(&inputs,&String::Borrowed("|"));drop(inputs);assert_eq!(result.as_ref(),expected);drop(result);drop(expected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        let inputs=(0..256).map(|n|n as f64+0.25).collect::<Vec<_>>();let expected=(0..256).map(|n|format!("{}.25",n)).collect::<Vec<_>>().join("|");let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
        let result=function1(std::hint::black_box(&inputs),&String::Borrowed("|"));assert_eq!(result.as_ref(),expected);assert!(ALLOCS.load(Ordering::SeqCst)<=16,"finite Number leaves append without per-element strings");drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),before);drop(inputs);drop(expected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let inputs=NumericArray::Heap((0..256).map(|n|n-128).collect::<Vec<i64>>());let expected=(-128..128).map(|n|n.to_string()).collect::<Vec<_>>().join("|");ALLOCS.store(0,Ordering::SeqCst);
        let result=function2(std::hint::black_box(&inputs),&String::Borrowed("|"));assert_eq!(result.as_ref(),expected);assert!(ALLOCS.load(Ordering::SeqCst)<=16);drop(inputs);drop(result);drop(expected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        // With capacity already available, the actual append path allocates
        // nothing while preserving all values. No method result is discarded.
        let input=[1_i64,-2,3];let mut output=std::string::String::with_capacity(16);ALLOCS.store(0,Ordering::SeqCst);tsr_append_join(&mut output,input.iter(),"|");assert_eq!(output,"1|-2|3");assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for call in [function4 as fn()->String,function6]{let error=std::panic::catch_unwind(call).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function5).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
