mod support;
use tsrust_compiler::check;

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let assertions = if assertions.contains("__NUMBER_FUNCTION__") {
        let functions = emitted
            .lines()
            .filter(|line| {
                line.starts_with("fn function") && line.contains("(value0: f64) -> Vec<f64>")
            })
            .collect::<Vec<_>>();
        assert_eq!(functions.len(), 1, "unique Number constructor witness");
        let name = functions[0]
            .split_once('(')
            .unwrap()
            .0
            .strip_prefix("fn ")
            .unwrap();
        assertions.replace("__NUMBER_FUNCTION__", name)
    } else {
        assertions.to_owned()
    };
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let evidence = std::env::var_os("TSR_GATE96_EVIDENCE_DIR").map(|root| {
        std::path::PathBuf::from(root).join(format!(
            "case-{}",
            CASE_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        ))
    });
    if let Some(path) = &evidence {
        std::fs::create_dir_all(path).unwrap();
        std::fs::write(path.join("input.tsr"), source).unwrap();
        std::fs::write(path.join("expected.txt"), expected).unwrap();
        std::fs::write(path.join("generated.rs"), &rust).unwrap();
    }
    let directory = support::TempDirectory::new();
    let path = directory.0.join("optional-chaining.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("optional-chaining-{level}"));
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
        if let Some(path) = &evidence {
            std::fs::write(path.join(format!("stdout-O{level}.txt")), &output.stdout).unwrap();
            std::fs::write(path.join(format!("stderr-O{level}.txt")), &output.stderr).unwrap();
        }
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn complete_chains_grouping_types_and_bits() {
    native(
        "function keep(x:number):number|undefined{const row:number[]=[x];return row?.[0];}function main(){}",
        "",
        "",
        r#"for bits in [0u64,0x8000000000000000,0x7ff8000000000042,0xfff8000000000071,0x7ff0000000000000,0xfff0000000000000,1,0x3ff0000000000000] {let Union0::V0(value)=function0(f64::from_bits(bits)) else{panic!("missing")};assert_eq!(value.to_bits(),bits);} "#,
    );
    native(
        r#"function main(){const ns:null[]|null=[null];const us:undefined[]|null=[undefined];const bs:boolean[]|null=[false];const ss:string[]|null=["é"];const rs:int[][]|null=[[7]];console.log(String(ns?.[0]),String(us?.[0]),String(bs?.[0]),String(ss?.[0]),String(rs?.[0].join()));}"#,
        "null undefined false é 7\n",
        "",
        "",
    );
    native(
        r#"type Row={n:int;num:number;flag:boolean;text:string;rows:int[];};type Box={item:Row|null;};
function inspect(row:Row|null):void{console.log(String(row?.n),String(row?.num),String(row?.flag),String(row?.text.length),String(row?.rows[0]));console.log(String(row?.text.toUpperCase()),String(row?.rows.join("|")));}
function chain(box:Box|null):void{console.log(String(box?.item?.text.concat("!")));}
function main(){inspect(null);inspect({n:0,num:-0,flag:false,text:"é😀",rows:[7]});chain(null);chain({item:null});chain({item:{n:1,num:Number.NaN,flag:true,text:"x",rows:[]}});console.log(String?.(0),Math?.abs(-2),Array?.isArray([1]),"é".toUpperCase?.());const xs=Array?.of<int>(1);const ys=Array.of?.<int>(2);console.log(xs.join(),ys.join());}
"#,
        "undefined undefined undefined undefined undefined\nundefined undefined\n0 0 false 6 7\nÉ😀 7\nundefined\nundefined\nx!\n0 2 true É\n1 2\n",
        "",
        "",
    );
    native(
        "function identity<T>(x:T):T{return x;}function main(){console.log(identity?.<int>(3));}",
        "3\n",
        "",
        "",
    );
}

#[test]
fn lazy_indices_arguments_and_mutation_contracts() {
    native(
        r#"type Item={text:string;};function get(trace:int[],flag:boolean):Item|null{trace.push(1);return flag?{text:"kept"}:null;}function main(){const trace:int[]=[];console.log(String(get(trace,false)?.text.repeat(trace.length)),trace.join());console.log(String(get(trace,true)?.text.repeat(trace.length)),trace.join());}"#,
        "undefined 1\nkeptkept 1,1\n",
        "",
        "",
    );
    native(
        r#"type Trace={calls:int;};function next(t:Trace):int{t.calls+=1;return 0;}function words(t:Trace):string{t.calls+=1;return "!";}function read(xs:int[]|null,t:Trace):void{console.log(String(xs?.[next(t)]),t.calls);console.log(String(xs?.join(words(t))),t.calls);}function mutate(xs:int[]|null):void{console.log(String(xs?.push(8)),String(xs?.pop()),String(xs?.length));}function main(){const t:Trace={calls:0};read(null,t);read([4],t);let absent:int[]|null=null;mutate(absent);let present:int[]|null=[1];mutate(present);const a=[3,1,2];a?.sort();console.log(a.join());a?.reverse();console.log(a.join());}
"#,
        "undefined 0\nundefined 0\n4 1\n4 2\nundefined undefined undefined\nundefined 8 1\n1,2,3\n3,2,1\n",
        "",
        "",
    );
}

#[test]
fn owned_views_fresh_outputs_and_lifetimes() {
    native(
        r#"type Item={text:string;rows:string[];};function fresh(flag:boolean):Item|null{return flag?{text:"é".repeat(1),rows:["a".repeat(1),"b".repeat(1)]}:null;}function main(){let source:Item|null={text:"alive".repeat(1),rows:["x".repeat(1)]};{const selected=source?.text;console.log(String(selected),String(source));const own=clone(selected);console.log(String(own));}source=null;console.log(String(source));const moved=fresh(true)?.text;console.log(String(moved));const absent=fresh(false)?.rows;console.log(String(absent));const output=fresh(true)?.rows.join("|");console.log(String(output));}
"#,
        "alive [object Object]\nalive\nnull\né\nundefined\na|b\n",
        "",
        "",
    );
}

#[test]
fn precise_errors_and_normal_depth() {
    for source in [
        "type Row={text:string;};function f(row:Row|null):int{return (row?.text).length;}",
        "function f(xs:int[]|null):int{return (xs?.[0])+1;}",
        "function main(){let x:int|null=1;x?.length=2;}",
        "function main(){let x:string[]|null=[];const v=x?.[0];x=null;console.log(String(v));}",
        "function f(x:string[]|null):string|undefined{const v=x?.[0];return v;}",
        "function main(){let x:int[]|null=[];x?.join(1);}",
        "function main(){const rows:int[][]=[[1]];rows.at(0)?.push(2);}",
        "function main(){const rows:int[][]=[[1]];rows.find((row:int[]):boolean=>{return true;})?.push(2);}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
    for depth in [32usize, 100, 200] {
        let source = format!(
            "type Node={{n:int;children:Node[];}};function inspect(node:Node|null):int|undefined{{return node{}?.n;}}function main(){{}}",
            "?.children[0]".repeat(depth)
        );
        let result = check(&source);
        if depth < 64 {
            result.unwrap().emit_rust();
        } else {
            assert!(result.is_err());
        }
    }
    let good = format!(
        "function f(xs:int{}|null):int|undefined{{return xs?.length;}}function main(){{}}",
        "[]".repeat(127)
    );
    assert!(check(&good).is_ok());
    let too_deep = format!(
        "function f(xs:int{}|null):int|undefined{{return xs?.length;}}function main(){{}}",
        "[]".repeat(128)
    );
    assert!(check(&too_deep).is_err());
}

#[test]
fn selection_pointer_allocations_and_unwind_boundaries() {
    native(
        r#"type Item={text:string;rows:int[];};function read(item:Item|null):int{return item?.text.length??0;}function make(text:string):Item|null{return {text:text,rows:[7]};}function take(text:string):string|undefined{return make(text)?.text;}function fail(item:Item|null,indices:int[]):int|undefined{return item?.rows[indices[8]];}function main(){}"#,
        "",
        r#"use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;unsafe impl GlobalAlloc for Counts{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}}#[global_allocator]static COUNTS:Counts=Counts;"#,
        r#"std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);let owner=String::Owned("kept".into());let pointer=owner.as_ptr();let item=Record0{field0:owner,field1:NumericArray::from([7])};ALLOCS.store(0,Ordering::SeqCst);assert_eq!(function0(ViewUnion0::V0(&item)),4);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);assert_eq!(item.field0.as_ptr(),pointer);assert_eq!(function0(ViewUnion0::V1(&TsrNull)),0);let indices=NumericArray::from([0]);assert!(matches!(function3(ViewUnion0::V1(&TsrNull),&indices),Union2::V1(_)));let failed=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(ViewUnion0::V0(&item),&indices)));assert!(failed.is_err());assert_eq!(item.field0.as_ptr(),pointer);drop(failed);drop(indices);drop(item);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);let input=String::Owned("moved".into());let pointer=input.as_ptr();ALLOCS.store(0,Ordering::SeqCst);let result=function2(input);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);let Union1::V0(value)=result else{panic!("lost owner")};assert_eq!(value.as_ptr(),pointer);drop(value);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);"#,
    );
    native(
        r#"type Item={text:string;};type Box={item:Item|null;};function main(){let holder:Box|null={item:{text:"nested".repeat(1)}};{const selected=(holder?.item)?.text;console.log(String(selected));}holder=null;console.log(String(holder));}"#,
        "nested\nnull\n",
        "",
        "",
    );
}

#[test]
fn current_storage_call_forms_and_nullable_boundaries() {
    native(
        r#"type H={text:string|null;};function read(h:H|undefined):void{if(h!==undefined){const value=h?.text;console.log(String(value));}}function main(){read({text:null});read({text:"s"});const literal=["a","é"];{const selected=literal?.[1];console.log(String(selected));}const nested:string[][]=[["x"]];console.log(String(nested?.[0][0]));}"#,
        "null\ns\né\nx\n",
        "",
        "",
    );
    native(
        r#"type Item={rows:int[];};function fresh():Item|null{return {rows:[3,1,2]};}function text():string|null{return "owned".repeat(1);}function factory():(value:int)=>int{return (value:int):int=>{return value+1;};}function main(){console.log(String(text()?.toUpperCase()),String(fresh()?.rows.pop()));console.log(factory()?.(2));const a=[3,1,2];a?.sort();a?.reverse();console.log(a.join());const items:Item[]=[{rows:[1,2]}];console.log(String(items?.[0].rows.join()));}"#,
        "OWNED 2\n3\n3,2,1\n1,2\n",
        "",
        "",
    );
    let rank = format!(
        "function main(){{const x=true?Array.of<int{}>():null;console.log(String(x?.length));}}",
        "[]".repeat(127)
    );
    native(&rank, "0\n", "", "");
    for source in [
        "function f(x:string[]|null):string|undefined{const v=x?.[0];return v;}function main(){}",
        "function main(){const rows:int[][]=[[1]];rows.at(0)?.push(2);}",
        "function main(){const rows:int[][]=[[1]];rows.find((row:int[]):boolean=>{return true;})?.push(2);}",
    ] {
        let error = check(source).err().expect(source);
        assert!(
            matches!(error.code, "borrowed-view" | "borrowed-place"),
            "{source}:{} {}",
            error.code,
            error.message
        );
        assert!(error.span.end > error.span.start);
    }
}

#[test]
fn diagnostic_codes_spans_and_existing_inferred_rank_boundary() {
    for (source, code, token, message) in [
        (
            "type Row={text:string;};function f(row:Row|null):int{return (row?.text).length;}function main(){}",
            "type",
            "row?.text",
            "Length requires",
        ),
        (
            "function f(xs:int[]|null):int{return (xs?.[0])+1;}function main(){}",
            "type",
            "xs?.[0]",
            "Expected int",
        ),
        (
            "function main(){let x:int|null=1;x?.length=2;}",
            "unsupported",
            "x?.length",
            "replaced",
        ),
        (
            "function main(){let x:string[]|null=[];const v=x?.[0];x=null;console.log(String(v));}",
            "view-conflict",
            "x",
            "scoped readonly",
        ),
        (
            "function f(x:string[]|null):string|undefined{const v=x?.[0];return v;}function main(){}",
            "borrowed-view",
            "v",
            "readonly consumer",
        ),
        (
            "function main(){let x:int[]|null=[];x?.join(1);}",
            "type",
            "1",
            "Expected string",
        ),
        (
            "function main(){const rows:int[][]=[[1]];rows.at(0)?.push(2);}",
            "borrowed-view",
            "rows.at(0)",
            "readonly use",
        ),
        (
            "function main(){const rows:int[][]=[[1]];rows.find((row:int[]):boolean=>{return true;})?.push(2);}",
            "borrowed-view",
            "rows.find((row:int[]):boolean=>{return true;})",
            "readonly use",
        ),
    ] {
        let error = check(source).err().expect(source);
        assert_eq!(error.code, code, "{source}: {}", error.message);
        assert_eq!(&source[error.span.start..error.span.end], token, "{source}");
        assert!(
            error.message.contains(message),
            "{source}: {}",
            error.message
        );
        if code == "view-conflict" {
            assert_eq!(error.span.start, source.find("x=null").unwrap());
        }
        if code == "borrowed-view" && token == "v" {
            assert_eq!(error.span.start, source.find("return v").unwrap() + 7);
        }
    }
    // Existing constructor inference can form rank128 without a rank129 annotation
    // wrapper. Optional selection does not change this preexisting storage limit.
    let ordinary = format!(
        "function main(){{const x=true?Array.of<int{}>():null;if(x!==null){{console.log(x.length);}}}}",
        "[]".repeat(127)
    );
    let ordinary_rust = check(&ordinary).unwrap().emit_rust();
    assert!(ordinary_rust.contains("fn main()"));
    let annotation = format!(
        "function f(x:int{}|null):int|undefined{{return x?.length;}}function main(){{}}",
        "[]".repeat(128)
    );
    let error = check(&annotation)
        .err()
        .expect("annotation wrapper exceeds existing depth");
    assert_eq!(error.code, "unsupported");
    assert!(error.message.contains("128"), "{}", error.message);
    assert!(error.span.end > error.span.start);
}

#[test]
fn optional_mutator_receiver_classification_and_existing_argument_effects() {
    native(
        r#"function grow(xs:int[]):int{xs.push(7);return 8;}function alter(xs:int[]|null):void{console.log(String(xs?.unshift(2,3)),String(xs?.splice(-1,1,4)),String(xs?.fill(9,0,1)),String(xs?.copyWithin(1,0,1)),String(xs?.shift()),String(xs?.pop()));}function fresh():int[]|null{return [1,2];}function main(){let none:int[]|null=null;alter(none);let some:int[]|null=[1];alter(some);const plain=[1];console.log(plain?.unshift(grow(plain)),plain.join());console.log(plain?.splice(-1,1,grow(plain)).join(),plain.join());console.log(String(fresh()?.unshift(3)),String(fresh()?.splice(0,1,5)));}"#,
        "undefined undefined undefined undefined undefined undefined\n3 1 undefined undefined 9 4\n3 8,1,7\n7 8,1,7,8\n3 1\n",
        "",
        "",
    );
    let source = "function grow(xs:int[]):int{xs.push(7);return 8;}function main(){let xs:int[]|null=[1];xs?.unshift(grow(xs));}";
    let error = check(source)
        .err()
        .expect("nullable selected storage retains ordinary projection loan");
    assert_eq!(error.code, "conflicting-access");
    assert_eq!(&source[error.span.start..error.span.end], "xs");
}
