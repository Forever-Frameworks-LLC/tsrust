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
    let evidence = std::env::var_os("TSR_GATE95_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("nullish-assignment.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("nullish-assignment-{level}"));
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
fn nullish_assignment_results_contexts_and_ieee_values_preserve_storage() {
    native(
        r#"type Row={text:string;};type Box<T>={value:T;};type Node={text:string;children:Node[];};function read<T>(value:T):int{return String(value).length;}function main(){let i:int|null|undefined=null;console.log(i??=7,i+1);let b:boolean|undefined=undefined;console.log(b??=false,b??=true);let n:number|null=null;console.log(n??=-0,1/n);let text:string|null=null;{const a=text??="é".repeat(1);const other=a;console.log(a,other,read<string>(other),text.length);}const copy=clone(text??="unused");console.log(copy,text.length);let values:int[]|null=null;{const selected=values??=[1,2];console.log(selected.join("|"),values.length);}let row:Row|undefined=undefined;{const selected=row??={text:"row".repeat(1)};console.log(selected.text,row.text);}let node:Node|null=null;console.log((node??={text:"root",children:[]}).children.length);let boxed:Box<int>|null=null;console.log((boxed??={value:9}).value);let residual:int|null|undefined=undefined;const snapshot=residual??=null;console.log(String(snapshot),String(residual));}"#,
        "7 8\nfalse false\n-0 -Infinity\né é 2 2\né 2\n1|2 2\nrow row\n0\n9\nnull null\n",
        "",
        "",
    );
    native(
        "function f(value:number):number{let x:number|null=null;return x??=value;}function main(){}",
        "",
        "",
        r#"for bits in [0,1u64<<63,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000001234]{assert_eq!(function0(f64::from_bits(bits)).to_bits(),bits);}"#,
    );
    for (ty, values) in [
        ("number[]", "[1,-0]"),
        ("boolean[]", "[true,false]"),
        ("string[]", "[\"é\",\"😀\"]"),
        ("null[]", "[null]"),
        ("undefined[]", "[undefined]"),
        ("int[][]", "[[1],[]]"),
    ] {
        native(
            &format!(
                "function main(){{let x:{ty}|null=null;console.log((x??={values}).length,x.length);}}"
            ),
            if values == "[null]" || values == "[undefined]" {
                "1 1\n"
            } else {
                "2 2\n"
            },
            "",
            "",
        );
    }
}

#[test]
fn nullish_assignment_selection_lazy_effects_reacquisition_and_facts_are_ordered() {
    native(
        r#"type Holder={item:string|null;other:string;};function takeNull(value:null):string{return "fallback";}function replace(holder:Holder):string{holder.item="interim";return "final".repeat(1);}function restore(holder:Holder):string{holder.other="restored".repeat(1);return "assigned".repeat(1);}function main(){let x:string|null=null;console.log(x??=takeNull(x),x.length);const holder:Holder={item:null,other:"moved".repeat(1)};console.log(holder.other);if(holder.item===null){console.log(holder.item??=restore(holder));console.log(holder.other);}const h:Holder={item:null,other:"other"};console.log(h.item??=replace(h),String(h.item));const rows:Holder[]=[{item:null,other:"row"}];let i=0;console.log(rows[i++].item??="selected",i,String(rows[0].item));let calls=0;let present:int|null=0;console.log(present??=(calls+=1),calls);let absent:int|null=null;console.log(absent??=(calls+=1),calls);let certain:null=null;console.log(String(certain??=null));}"#,
        "fallback 8\nmoved\nassigned\nrestored\nfinal final\nselected 1 selected\n0 0\n1 1\nnull\n",
        "",
        "",
    );
    native(
        "function main(){let value:number|null=null;value??=true?1:2;console.log(value);let array:number[]|null=null;console.log((array??=(false?[1]:[2])).join());let lhs:int|null=null;let rhs:int|null=null;console.log(lhs??=rhs??=5,String(lhs),String(rhs));}",
        "1\n2\n5 5 5\n",
        "",
        "",
    );
}

#[test]
fn nullish_assignment_borrowed_aliases_clones_and_unwind_dispose_normally() {
    native(
        r#"type Box={value:string|null;};function choose(box:Box):int{return(box.value??="fresh".repeat(1)).length;}function fail(box:Box,values:int[]):int{return(box.value??=String(values[9])).length;}function rhs(trace:int[]):string{trace.push(1);return "side";}function bad(rows:Box[],trace:int[]):int{return(rows[9].value??=rhs(trace)).length;}function main(){}"#,
        "",
        r#"use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;unsafe impl GlobalAlloc for Counts{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}}#[global_allocator]static COUNTS:Counts=Counts;"#,
        r#"std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);let owner=String::Owned("kept".into());let pointer=owner.as_ptr();let mut box_=Record0{field0:Union0::V0(owner)};ALLOCS.store(0,Ordering::SeqCst);assert_eq!(function0(&mut box_),4);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);let Union0::V0(owner)=&box_.field0 else{panic!("lost present owner")};assert_eq!(owner.as_ptr(),pointer);drop(box_);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);let mut empty=Record0{field0:Union0::V1(TsrNull)};ALLOCS.store(0,Ordering::SeqCst);assert_eq!(function0(&mut empty),5);assert_eq!(ALLOCS.load(Ordering::SeqCst),1);drop(empty);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);let mut absent=Record0{field0:Union0::V1(TsrNull)};let input=NumericArray::from([1]);let failed=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function1(&mut absent,&input)));assert!(failed.is_err());assert!(matches!(absent.field0,Union0::V1(_)));drop(failed);drop(absent);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);let mut rows=std::iter::once(Record0{field0:Union0::V1(TsrNull)}).collect();let mut trace=NumericArray::from([]);let failed=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut rows,&mut trace)));assert!(failed.is_err());assert_eq!(trace.len(),0);drop(failed);drop(rows);drop(trace);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);"#,
    );
    let rank = format!(
        "function main(){{let value=true?Array.of<int{}>():null;console.log((value??=Array.of<int{}>()).length,value.length);}}",
        "[]".repeat(127),
        "[]".repeat(127)
    );
    native(&rank, "0 0\n", "", "");
    if std::env::var_os("TSR_GATE95_RESOURCE_ONLY").is_some() {
        return;
    }

    native(
        r#"function fill():string{let x:string|null=null;return clone(x??="owner".repeat(1));}function view(value:string):int{return value.length;}function main(){let x:string|null=null;{const a=x??="live".repeat(1);const b=a;console.log(view(b),x.length);}x=null;console.log(String(x),fill());let rows:int[]|null=null;rows??=[1];rows.push(2);console.log(rows.join());}"#,
        "4 4\nnull owner\n1,2\n",
        "",
        "",
    );
    for source in [
        "function bad():string{let x:string|null=null;return x??=\"owner\".repeat(1);}function main(){}",
        "function main(){let x:int[]|null=null;(x??=[1]).push(2);}",
        "function main(){let x:string|null=null;const alias=x??=\"a\";x=null;console.log(alias);}",
        "type Row={text:string;};function main(){let x:string|null=null;const r:Row={text:x??=\"a\"};}",
        "function bad():()=>int{let x:string|null=null;const alias=x??=\"a\";return ():int=>{return alias.length;};}function main(){}",
    ] {
        let error = check(source).err().expect(source);
        assert!(
            matches!(
                error.code,
                "borrowed-place" | "view-conflict" | "borrowed-view" | "unsupported"
            ),
            "{source}: {} {}",
            error.code,
            error.message
        );
    }
}

#[test]
fn nullish_assignment_types_loans_spans_and_normal_depth_are_precise() {
    for (source, code) in [
        (
            "function main(){const x:int|null=null;x??=1;}",
            "unsupported",
        ),
        ("function main(){let x:string=\"s\";x??=7;}", "type"),
        ("function main(){let x:int|null=null;x??=true;}", "type"),
        (
            "function main(){let x:int|null=null;x??=undefined;}",
            "type",
        ),
        ("function main(){let x:int|null=null;x??=missing;}", "name"),
        ("function main(){[null][0]??=null;}", "unsupported"),
    ] {
        let e = check(source).err().expect(source);
        assert_eq!(e.code, code, "{source}:{}", e.message);
        assert!(e.span.end > e.span.start);
    }
    for depth in [100, 200] {
        let source = format!(
            "function main(){{let x:int|null=null;{}1;}}",
            "x??=".repeat(depth)
        );
        let result = check(&source);
        if depth == 100 {
            result.unwrap().emit_rust();
        } else {
            support::accept_deep(&source);
        }
    }
    let source = "function main(){let x:string|null=null;const alias=x??=\"a\";console.log(alias.length,x.length);}";
    let analysis = check(source).unwrap();
    assert_eq!(analysis.checked_type_facts(), analysis.checked_type_facts());
    assert!(
        analysis
            .checked_type_facts()
            .method_results
            .iter()
            .any(|f| f.ty == "string")
    );
}

#[test]
fn assignment_alias_conditional_places_retain_fresh_owners_and_all_roots() {
    native(
        r#"function main(){for(let i=0;i<2;i++){let x:string|null=null;let y:string|null=null;{{const alias=i===0?(x??="a"):(y??="b");const second=alias;console.log(second);}const mixed=i===0?(x??="a"):"fresh".repeat(1);const copied=mixed;console.log(copied);let residual:string|null|undefined=null;const optional=i===0?(residual??=undefined):"owned".repeat(1);console.log(String(optional),typeof optional);}x="c";y="d";console.log(String(x),String(y));}let values=[1,2];const selected=values??=[3];console.log(selected.join());}"#,
        "a\na\nundefined undefined\nc d\nb\nfresh\nowned string\nc d\n1,2\n",
        "",
        "",
    );
    native(
        r#"type Item={n:int;};function main(){let item:Item|null=null;let n=(item??={n:4}).n;item={n:9};n++;let values:int[]|null=null;let first=(values??=[7])[0];values=[8];first++;console.log(n,first,String(item),String(values));}"#,
        "5 8 [object Object] 8\n",
        "",
        "",
    );
    let source = r#"function main(){let x:string|null=null;let y:string|null=null;const alias=true?(x??="a"):(y??="b");y="c";console.log(alias);}"#;
    let error = check(source)
        .err()
        .expect("both conditional roots remain borrowed");
    assert_eq!(error.code, "view-conflict");
}
