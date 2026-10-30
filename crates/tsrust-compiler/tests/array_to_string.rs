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
    let path = directory.0.join("array-string.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-string-{level}"));
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
fn array_text_all_leaves_ranks_records_and_scalar_conversion_preservation() {
    for id in ["int", "boolean", "string", "nested", "number", "nullish"] {
        let source = std::fs::read_to_string(format!(
            "../../benchmarks/runtime/workloads/array_to_string_{id}.tsr"
        ))
        .unwrap();
        check(&source).unwrap().emit_rust();
    }
    native(
        include_str!("../../../benchmarks/runtime/array-to-string-pool-hands.tsr"),
        include_str!("../../../benchmarks/runtime/array-to-string-pool-hands-expected.txt"),
        "",
        "",
    );
    native(
        r#"
    type Row={text:string;values:int[];};type Node={value:int;children:Node[];};
    function ints(values:int[]):string{return values.toString();}
    function numbers(values:number[]):string{return String(values);}
    function strings(values:string[]):string{return values.toString();}
    function main(){
        console.log([1,2,3].toString());console.log(String([1,2,3]));console.log([].toString());console.log(String([]));
        console.log([true,false].toString());console.log(String([true,false]));console.log([null,null].toString());console.log(String([undefined,undefined,undefined]));
        console.log(["é😀","","e\u0301\0"].toString());console.log(String(["é😀","","e\u0301\0"]));
        console.log([[-1,2],[],[3]].toString());console.log(String([[-1,2],[],[3]]));console.log([[],[]].toString());
        console.log([[[1,2],[]],[[3]]].toString());console.log(String([[[1,2],[]],[[3]]]));
        const rows:Row[]=[{text:"owned".repeat(1),values:[1,2]},{text:"second",values:[]}];console.log(rows.toString());console.log(String([rows]));
        const nodes:Node[]=[{value:1,children:[]}];console.log(nodes.toString());console.log(String(nodes));
        const values:number[]=[-0.0,Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY,1e-7,1e-6,1e20,1e21];console.log(values.toString());console.log(String([values]));
        console.log([-9223372036854775808,9223372036854775807].toString());
        console.log("é".toString());console.log(true.toString());console.log((15).toString(16));
        console.log(String(-0.0));console.log(String(Number.NaN));console.log(String(null));console.log(String(undefined));console.log(String());console.log(String({text:"field"}));
    }"#,
        "1,2,3\n1,2,3\n\n\ntrue,false\ntrue,false\n,\n,,\né😀,,é\0\né😀,,é\0\n-1,2,,3\n-1,2,,3\n,\n1,2,,3\n1,2,,3\n[object Object],[object Object]\n[object Object],[object Object]\n[object Object]\n[object Object]\n0,NaN,Infinity,-Infinity,1e-7,0.000001,100000000000000000000,1e+21\n0,NaN,Infinity,-Infinity,1e-7,0.000001,100000000000000000000,1e+21\n-9223372036854775808,9223372036854775807\né\ntrue\nf\n0\nNaN\nnull\nundefined\n\n[object Object]\n",
        "",
        r#"
    for size in [0,1,4,5,17]{let mut values=NumericArray::from([]);for n in 0..size{values.push(n as i64-2);}let expected=(0..size).map(|n|(n as i64-2).to_string()).collect::<Vec<_>>().join(",");let output=function0(std::hint::black_box(&values));assert_eq!(output.as_ref(),expected);assert!(matches!(output,String::Owned(_)));assert_eq!(values.len(),size);}
    let values=vec![f64::from_bits(0x8000000000000000),f64::NAN,f64::INFINITY,f64::NEG_INFINITY,f64::from_bits(1)];let output=function1(std::hint::black_box(&values));assert_eq!(output.as_ref(),"0,NaN,Infinity,-Infinity,5e-324");assert_eq!(values[0].to_bits(),0x8000000000000000);
    for owned in [false,true]{let values=Array::from([if owned{String::Owned("é😀\0".to_owned())}else{String::Borrowed("é😀\0")}]);let output=function2(std::hint::black_box(&values));assert!(matches!(output,String::Owned(_)));assert_ne!(output.as_ptr(),values.get(0).unwrap().as_ptr());drop(values);assert_eq!(output.as_ref(),"é😀\0");}
    "#,
    );
}

#[test]
fn array_text_order_extra_arguments_lazy_temporaries_and_native_layouts() {
    native(
        include_str!("../../../benchmarks/runtime/array-to-string-source-order.tsr"),
        include_str!("../../../benchmarks/runtime/array-to-string-source-order-expected.txt"),
        "",
        "",
    );
    let analysis = native(
        r#"
    type Box={values:string[];};
    function receiver(trace:int[]):string[]{trace.push(1);return ["a".repeat(1),"b".repeat(1)];}
    function effect(trace:int[]):int{trace.push(2);return 0;}
    function empty(trace:int[]):string[]{trace.push(3);return [];}
    function at(trace:int[]):int{trace.push(4);return 0;}
    function read(values:int[]):string{return String(values);}
    function main(){
        const trace:int[]=[];console.log(String(receiver(trace),effect(trace)));String(receiver(trace),effect(trace));console.log(String(empty(trace),effect(trace)));
        const rows=[["x","y"],["z"]];console.log(rows[at(trace)].toString());console.log(String(rows[0]));console.log(rows[0][0]);
        const values=["owned".repeat(1),"other".repeat(1)];const output=String(values);values[0]="changed";console.log(output);console.log(values.toString());
        const box:Box={values:["field".repeat(1)]};const capture=():string=>{return String(box.values);};console.log(capture());console.log(box.values[0]);
        for(const flag of [true,false]){console.log(String(flag?receiver(trace):empty(trace),effect(trace)));for(const value of [(flag?receiver(trace):empty(trace)).toString()]){console.log(value);}}
        const mapped=rows.map((row:string[]):string=>{return row.toString();});console.log(mapped[1]);console.log(`${String(values)}`);
        for(const value of trace){console.log(value);}
    }"#,
        "a,b\n\nx,y\nx,y\nx\nowned,other\nchanged,other\nfield\nfield\na,b\na,b\n\n\nz\nchanged,other\n1\n2\n1\n2\n3\n2\n4\n1\n2\n1\n3\n2\n3\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("read.values"), Some(&Mode::Read));
    let emitted=check(r#"type Row={values:int[];};function fixed():string{const ints=[1,2,3];const words=["a","b"];const nested=[[1,2],[3,4]];const rows:Row[]=[{values:[1,2]},{values:[3,4]}];return String(ints)+words.toString()+String(nested)+String(rows);}function main(){fixed();}"#).unwrap().emit_rust();
    assert!(emitted.contains(": [i64; 3] = ["));
    assert!(emitted.contains("LiteralStringArray"));
    assert!(!emitted.contains("enum TsrConsoleValue"));
    assert!(!emitted.contains("tsr_console_record_"));
    assert!(!emitted.contains("TsrConsoleValue::Array"));
}

#[test]
fn array_text_strict_diagnostics_loans_and_supported_depth_are_preserved() {
    for (source, code) in [
        ("function main(){[1].toString(\"x\");}", "arity"),
        ("function main(){[1].toString(1);}", "arity"),
        ("function main(){[1].valueOf();}", "type"),
        ("function main(){const n:int=[1].toString();}", "type"),
        ("function main(){const n:int=String([1]);}", "type"),
        (
            "function main(){const a=[1];const b=a;String(a);}",
            "transferred",
        ),
        (
            "function main(){const a=[1];const b=a;a.toString();}",
            "transferred",
        ),
        (
            "function grow(a:int[]):int{a.push(3);return 0;}function main(){const a=[1,2];String(a,grow(a));}",
            "conflicting-access",
        ),
        (
            "function grow(a:int[]):int{a.push(3);return 0;}function main(){const a=[1];const b=[2];String(true?a:b,grow(b));}",
            "conflicting-access",
        ),
        (
            "function take(a:string[]):int{const b=a;return 0;}function main(){const a=[\"a\"];String(a,take(a));}",
            "conflicting-access",
        ),
        (
            "type Box={values:int[];};function grow(b:Box):int{b.values.push(3);return 0;}function main(){const b:Box={values:[1]};String(b.values,grow(b));}",
            "conflicting-access",
        ),
        ("function main(){true.toString(1);}", "arity"),
        ("function main(){\"x\".toString(1);}", "arity"),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
    }
    let valid = format!(
        "function main(){{const a=[0];console.log({}0{});console.log([].toString());}}",
        "a[".repeat(100),
        "]".repeat(100)
    );
    check(&valid).unwrap().emit_rust();
    let hostile = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
        "a[".repeat(200),
        "]".repeat(200)
    );
    support::accept_deep(&hostile);
    // Maximum accepted type nesting reaches the scalar just before the old
    // diagnostic array cutoff. Full text must reach the leaf.
    let source = format!(
        "function deep(values:int{}):string{{return String(values);}}function alias(values:int{}):string{{return values.toString();}}function main(){{}}",
        "[]".repeat(128),
        "[]".repeat(128)
    );
    let nested = (0..126).fold(
        "Array::from([NumericArray::from([7_i64])])".to_owned(),
        |value, _| format!("vec![{value}]"),
    );
    native(
        &source,
        "",
        "",
        &format!(
            "let values={nested};assert_eq!(function0(&values).as_ref(),\"7\");assert_eq!(function1(&values).as_ref(),\"7\");"
        ),
    );
    support::reject(
        &format!(
            "function tooDeep(values:int{}):string{{return String(values);}}function main(){{}}",
            "[]".repeat(129)
        ),
        "unsupported",
    );
}

#[test]
fn array_text_direct_buffer_allocation_and_normal_unwind_disposal_are_observed() {
    native(
        r#"
        function strings(values:string[],separator:string):string{return values.toString();}
        function numbers(values:number[],separator:string):string{return String(values);}
        function integers(values:int[],separator:string):string{return String(values);}
        function fail():string{const bad=1/0;return "unused";}
        function argumentTrap():string{return String(["receiver".repeat(3)],fail());}
        function aggregateTrap():int{const values=[["owned".repeat(1)].toString(),String(["other".repeat(1)])];const bad=1/0;return bad;}
        function indexTrap():string{const values=[["owned".repeat(1)]];return String(values[2]);}
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
            let expected=(0..count).map(|_|"é😀\0").collect::<Vec<_>>().join(",");let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
            let result=function0(std::hint::black_box(&inputs),&String::Borrowed("|"));let allocations=ALLOCS.load(Ordering::SeqCst);assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Owned(_)));assert!(allocations<=16,"no per-element clone or diagnostic allocation: {allocations}");if count==0{assert_eq!(allocations,0);}drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),before);
            let result=function0(&inputs,&String::Borrowed("|"));drop(inputs);assert_eq!(result.as_ref(),expected);drop(result);drop(expected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        let inputs=(0..256).map(|n|n as f64+0.25).collect::<Vec<_>>();let expected=(0..256).map(|n|format!("{}.25",n)).collect::<Vec<_>>().join(",");let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
        let result=function1(std::hint::black_box(&inputs),&String::Borrowed("|"));assert_eq!(result.as_ref(),expected);assert!(ALLOCS.load(Ordering::SeqCst)<=16,"finite Number leaves append without per-element strings");drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),before);drop(inputs);drop(expected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let inputs=NumericArray::Heap((0..256).map(|n|n-128).collect::<Vec<i64>>());let expected=(-128..128).map(|n|n.to_string()).collect::<Vec<_>>().join(",");ALLOCS.store(0,Ordering::SeqCst);
        let result=function2(std::hint::black_box(&inputs),&String::Borrowed("|"));assert_eq!(result.as_ref(),expected);assert!(ALLOCS.load(Ordering::SeqCst)<=16);drop(inputs);drop(result);drop(expected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        // With capacity already available, the actual append path allocates
        // nothing while preserving all values. No method result is discarded.
        let input=[1_i64,-2,3];let mut output=std::string::String::with_capacity(16);ALLOCS.store(0,Ordering::SeqCst);tsr_append_join(&mut output,input.iter(),"|");assert_eq!(output,"1|-2|3");assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for call in [function4 as fn()->String,function6]{let error=std::panic::catch_unwind(call).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function5).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
