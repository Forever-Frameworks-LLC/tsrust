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
    let path = directory.0.join("nullable-unions.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("nullable-unions-{level}"));
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
fn nullable_canonical_copy_transport_contextual_literals_and_observers() {
    for leaf in [
        "int_null",
        "boolean_undefined",
        "number_null",
        "string_undefined",
        "array_null",
        "record_undefined",
    ] {
        let source = std::fs::read_to_string(format!(
            "../../benchmarks/runtime/workloads/nullable_{leaf}.tsr"
        ))
        .unwrap();
        check(&source).unwrap().emit_rust();
    }
    native(
        include_str!("../../../benchmarks/runtime/nullable-transport-source-order.tsr"),
        include_str!("../../../benchmarks/runtime/nullable-transport-source-order-expected.txt"),
        "",
        "",
    );
    let analysis = native(
        r#"
    function integer(value:undefined|int|null|int):int|null|undefined{return value;}
    function number(value:number|null):null|number{return value;}
    function logical(value:boolean|undefined):undefined|boolean{return value;}
    function absent(value:null|undefined):undefined|null|null{return value;}
    function callback(f:ReadOnly<Function<int|null, [value:int]>>,n:int):int|null{return f(n);}
    function main(){
        const x:null|int|undefined=integer(7);console.log(String(x));console.log(typeof x);console.log(Boolean(x));console.log(x===null);console.log(undefined===x);console.log(Array.isArray(x));
        console.log(String(integer(null)));console.log(String(integer(undefined)));console.log(String(absent(null)));console.log(String(absent(undefined)));
        let y:int|null=1;console.log(String(y));y=null;console.log(String(y));const z:int|null|undefined=y;console.log(String(z));console.log(String(y));
        console.log(String(number(-0)));console.log(String(number(Number.NaN)));console.log(Boolean(number(-0.0)));console.log(Boolean(number(Number.NaN)));console.log(String(number(Number.POSITIVE_INFINITY)));
        console.log(String(logical(false)));console.log(String(logical(undefined)));console.log(typeof logical(true));
        const inferred=true?7:null;console.log(String(inferred));const both=false?undefined:null;console.log(String(both));console.log(null===both);console.log(both!==undefined);
        const f=(n:int):int|null=>{return n>0?n:null;};console.log(String(callback(f,4)));console.log(String(callback(f,0)));
        const collapsed:int|int=2;console.log(collapsed+1);const grouped:(int)[]=[1,2];console.log(grouped.toString());
        for(const outer of [true,false]){for(const inner of [true,false]){
            const nested=outer?null:(inner?undefined:7);console.log(String(nested));
            const flipped=outer?(inner?7:null):undefined;console.log(String(flipped));
        }}
    }"#,
        "7\nnumber\ntrue\nfalse\nfalse\nfalse\nnull\nundefined\nnull\nundefined\n1\nnull\nnull\nnull\n0\nNaN\nfalse\nfalse\nInfinity\nfalse\nundefined\nboolean\n7\nnull\ntrue\ntrue\n4\nnull\n3\n1,2\nnull\n7\nnull\nnull\nundefined\nundefined\n7\nundefined\n",
        "",
        r#"
    let minus_zero=Union1::V0(f64::from_bits(0x8000000000000000));let transported=function1(minus_zero);let Union1::V0(value)=transported else{panic!("Number payload lost")};assert_eq!(value.to_bits(),0x8000000000000000);
    let nan=Union1::V0(f64::from_bits(0x7ff8000000000042));let Union1::V0(value)=function1(nan) else{panic!("Number payload lost")};assert_eq!(value.to_bits(),0x7ff8000000000042);
    "#,
    );
    assert_eq!(analysis.contracts().get("integer.value"), Some(&Mode::Copy));
    assert_eq!(analysis.contracts().get("number.value"), Some(&Mode::Copy));
}

#[test]
fn nullable_owned_transport_replacement_captures_laziness_and_recursive_storage() {
    native(
        r#"
    type Plain={n:int;};
    type Hidden={s:string|null;};
    type Row={text:string;values:int[];};
    type Box={value:string|null;other:string;};
    type Node={value:int;children:Node[]|null;};
    function text(value:string|null):null|string{return value;}
    function array(value:int[]|null):null|int[]{return value;}
    function record(value:Row|undefined):undefined|Row{return value;}
    function inspect(value:string|null|undefined):string{return typeof value;}
    function same(value:string|null):string{return String(value);}
    function make(trace:int[],flag:boolean):string|null{trace.push(1);return flag?"é😀".repeat(1):null;}
    function fail():string{const bad=1/0;return "unused";}
    function ownedCallback(seed:string|null):(flag:boolean)=>string|null{return (flag:boolean):string|null=>{return flag?clone(seed):null;};}
    function main(){
        const trace:int[]=[];const a:string|null=make(trace,true);console.log(same(a));console.log(same(a));const transported=text(a);console.log(String(transported));
        let value:string|null=null;value="owned".repeat(1);console.log(String(value));value=null;console.log(String(value));value=make(trace,false);console.log(String(value));
        const words:string|null="array".repeat(1);const boxed:Box={value:words,other:"other".repeat(1)};const read=():string=>{return String(boxed.value);};console.log(read());console.log(String(boxed.value));
        const data:int[]|null=array([1,2]);console.log(String(data));console.log(Array.isArray(data));console.log(Array.isArray(array(null)));console.log(Boolean(array([])));
        const row:Row|undefined=record({text:"field".repeat(1),values:[3]});console.log(String(row));console.log(typeof row);console.log(String(record(undefined)));
        const strings:string[]|null=["a","é😀"];console.log(String(strings));
        const numbers:number[]|undefined=[-0.0,Number.NaN];console.log(String(numbers));
        const nested:int[][]|null=[[1,2],[],[3]];console.log(String(nested));
        const recordRows:Row[][]|undefined=[[{text:"ignored",values:[1]}]];console.log(String(recordRows));
        const empty:Node[]|null=null;const node:Node={value:1,children:empty};const nodes:Node[]|null=[node];const copy=clone(nodes);console.log(String(nodes));console.log(String(copy));
        const closure=ownedCallback("captured".repeat(1));console.log(String(closure(true)));console.log(String(closure(false)));
        for(const flag of [true,false]){console.log(String(flag?make(trace,true):make(trace,false)));console.log(typeof (flag?"fresh".repeat(1):null));console.log(Boolean(flag?[]:null));}
        const chosen:string|null=true?"chosen".repeat(1):fail();console.log(String(chosen));console.log(trace.length);
        const narrow:string|null="wide".repeat(1);console.log(inspect(narrow));
        const plain:Plain={n:7};console.log(plain);
    }"#,
        "é😀\né😀\né😀\nowned\nnull\nnull\narray\narray\n1,2\ntrue\nfalse\ntrue\n[object Object]\nobject\nundefined\na,é😀\n0,NaN\n1,2,,3\n[object Object]\n[object Object]\n[object Object]\ncaptured\nnull\né😀\nstring\ntrue\nnull\nobject\nfalse\nchosen\n4\nstring\n{ n: 7 }\n",
        "",
        "",
    );
    native(
        r#"
    function word(trace:int[],mark:int):string{trace.push(mark);return "word".repeat(1);}
    function payload(trace:int[],mark:int):int[]{trace.push(mark);return [mark,8];}
    function main(){
        const trace:int[]=[];
        for(const outer of [true,false]){for(const inner of [true,false]){
            const a=outer?null:(inner?undefined:word(trace,1));console.log(String(a));console.log(typeof a);
            const b=outer?(inner?word(trace,2):null):undefined;console.log(String(b));console.log(typeof b);
            const c=outer?null:(inner?undefined:payload(trace,3));console.log(String(c));console.log(Array.isArray(c));
            const d=outer?(inner?payload(trace,4):null):undefined;console.log(String(d));console.log(Array.isArray(d));
        }}
        console.log(trace.join());
    }"#,
        "null\nobject\nword\nstring\nnull\nfalse\n4,8\ntrue\nnull\nobject\nnull\nobject\nnull\nfalse\nnull\nfalse\nundefined\nundefined\nundefined\nundefined\nundefined\nfalse\nundefined\nfalse\nword\nstring\nundefined\nundefined\n3,8\ntrue\nundefined\nfalse\n2,4,1,3\n",
        "",
        "",
    );
}

#[test]
fn nullable_diagnostics_projections_hidden_consumers_moves_and_depth_are_precise() {
    for (source, code) in [
        ("function main(){const x:int|void=1;}", "unsupported"),
        ("function main(){const x:void|null=null;}", "unsupported"),
        ("function main(){const x:(int|void)[]=[];}", "type"),
        (
            "function main(){const x:string|null=null;console.log(x.length);}",
            "type",
        ),
        (
            "function main(){const x:int[]|null=null;console.log(x[0]);}",
            "type",
        ),
        ("function main(){const x:int[]|null=null;x.join();}", "type"),
        (
            "function main(){const x:int[]|null=null;const y:int[]|null=null;console.log(x===y);}",
            "unsupported",
        ),
        (
            "function main(){const x:int|null=1;console.log(Number(x));}",
            "unsupported",
        ),
        (
            "function main(){const x:string|null=\"x\";console.log(`${x}`);}",
            "type",
        ),
        (
            "function main(){const x:string|null=\"x\";const y=x;String(x);}",
            "transferred",
        ),
        (
            "function read(x:string|null|undefined):string{return String(x);}function main(){const x:string|null=\"x\";read(x);String(x);}",
            "transferred",
        ),
        (
            "function read(x:string|null):string{return String(x);}function main(){const x=\"x\".repeat(1);read(x);console.log(x);}",
            "transferred",
        ),
        (
            "function use(a:int[]|null,n:int):string{return String(a);}function take(a:int[]|null):int{a;return 0;}function main(){const a:int[]|null=[1];use(a,take(a));}",
            "conflicting-access",
        ),
        (
            "function main(){let x:string|null=\"x\";const f=():string=>{return String(x);};x=null;f();}",
            "capture-conflict",
        ),
        (
            "type A={next:A|null;};function main(){}",
            "recursive-layout",
        ),
        (
            "function main(){const x:int|null=1;clone(x);}",
            "unsupported",
        ),
    ] {
        assert!(check(source).is_err(), "expected rejection for: {source}");
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
    }
    // Nullable callable transport is supported since gate 97; these are no longer rejection witnesses.
    for source in [
        "function main(){const f=():int=>{return 1;};const x=true?f:null;}",
        "function bad(f:((n:int)=>int)|null){}function main(){}",
    ] { check(source).unwrap(); }
    let valid = format!(
        "function main(){{const a=[0];console.log({}0{});const x:int|null=1;console.log(String(x));}}",
        "a[".repeat(100),
        "]".repeat(100)
    );
    check(&valid).unwrap().emit_rust();
    let invalid = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
        "a[".repeat(200),
        "]".repeat(200)
    );
    support::reject(&invalid, "unsupported");
}

#[test]
fn nullable_injection_allocations_payload_pointer_and_normal_unwind_disposal() {
    native(
        r#"
    type Node={value:int;children:Node[]|null;};
    function pass(value:string|null):string|null{return value;}
    function absent():string|null{return null;}
    function owned():string|null{return "é😀".repeat(2);}
    function panicOwned():int{const value:string|null=owned();const words:string[]=[String(value)];const bad=1/0;return bad;}
    function panicBranch():string|null{return false?"untaken".repeat(3):bad();}
    function bad():string|null{const bad=1/0;return null;}
    function node(value:Node[]|null):Node[]|null{return value;}
    function main(){}
    "#,
        "",
        ALLOCATION_OBSERVER,
        r#"
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
    let original=String::Owned("pointer-é😀".to_owned());let pointer=original.as_ptr();let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
    // Node field resolution creates Union0; string payloads use Union1.
    let tagged=Union1::V0(original);let output=function0(std::hint::black_box(tagged));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);let Union1::V0(value)=output else{panic!("owned payload absent")};assert_eq!(value.as_ptr(),pointer);assert_eq!(LIVE.load(Ordering::SeqCst),before);drop(value);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    ALLOCS.store(0,Ordering::SeqCst);assert!(matches!(function1(),Union1::V1(_)));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);
    for _ in 0..8{let output=function2();let Union1::V0(value)=output else{panic!("payload absent")};assert_eq!(value.as_ref(),"é😀é😀");drop(value);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
    // Nullable recursive-array fields retain the iterative clone/drop path.
    let mut children=RecordArray0(Vec::new());for n in 0..2000{children=RecordArray0(vec![Record0{field0:n,field1:Union0::V0(children)}]);}
    let value=Union0::V0(children);let cloned=value.clone();let output=function6(value);drop(output);drop(cloned);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    for call in [function3 as fn()->i64]{let error=std::panic::catch_unwind(call).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
    let error=std::panic::catch_unwind(function4).err().expect("expected panic");drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
    "#,
    );
}

const ALLOCATION_OBSERVER: &str = r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Allocations;
        unsafe impl GlobalAlloc for Allocations{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){LIVE.fetch_add(layout.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
            unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe{System.dealloc(p,layout)};LIVE.fetch_sub(layout.size() as isize,Ordering::SeqCst);}
            unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8{let next=unsafe{System.realloc(p,layout,size)};if !next.is_null(){LIVE.fetch_add(size as isize-layout.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}next}
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;

"#;
