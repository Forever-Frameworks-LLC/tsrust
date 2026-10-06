mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    assert!(!rust.contains("unsafe"));
    let rust = rust.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("array-predicates.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-predicates-{level}"));
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
        let output = support::run(&mut std::process::Command::new(executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn predicates_cover_all_leaves_ranks_literal_contexts_and_empty_results() {
    let cases = [
        ("[1,2,3,4,5]", "value:int", "value>0", "true\ntrue\n0\n4\n"),
        ("[1,2]", "value:number", "value>0.0", "true\ntrue\n0\n1\n"),
        (
            "[true,true]",
            "value:boolean",
            "value",
            "true\ntrue\n0\n1\n",
        ),
        (
            r#"["x","é","\u0000"]"#,
            "value:string",
            "value.length>0",
            "true\ntrue\n0\n2\n",
        ),
        (
            "[{n:1},{n:2}]",
            "value:Row",
            "value.n>0",
            "true\ntrue\n0\n1\n",
        ),
        (
            "[[1],[2],[]]",
            "value:int[]",
            "value.length>0",
            "true\nfalse\n0\n1\n",
        ),
        (
            "[[1.5],[]]",
            "value:number[]",
            "value.length>0",
            "true\nfalse\n0\n0\n",
        ),
        (
            "[[true],[]]",
            "value:boolean[]",
            "value.length>0",
            "true\nfalse\n0\n0\n",
        ),
        (
            r#"[["x"],[]]"#,
            "value:string[]",
            "value.length>0",
            "true\nfalse\n0\n0\n",
        ),
        (
            "[[{n:1}],[]]",
            "value:Row[]",
            "value.length>0",
            "true\nfalse\n0\n0\n",
        ),
        (
            "[[[1]],[]]",
            "value:int[][]",
            "value.length>0",
            "true\nfalse\n0\n0\n",
        ),
        (
            r#"[{name:"root",children:[]}]"#,
            "value:Node",
            r#"value.name==="root""#,
            "true\ntrue\n0\n0\n",
        ),
    ];
    let mut body =
        String::from("type Row={n:int;};type Node={name:string;children:Node[];};function main(){");
    let mut expected = String::new();
    for (values, parameter, predicate, output) in cases {
        for method in ["some", "every", "findIndex", "findLastIndex"] {
            body.push_str(&format!(
                "console.log({values}.{method}(({parameter}):boolean=>{{return {predicate};}}));"
            ));
        }
        expected.push_str(output);
    }
    body.push_str(r#"
        const empty:string[]=[];
        console.log(empty.some(():boolean=>{return true;}));console.log(empty.every(():boolean=>{return false;}));
        console.log(empty.findIndex(():boolean=>{return true;}));console.log(empty.findLastIndex(():boolean=>{return true;}));
        console.log([].every((value:Row):boolean=>{return value.n>0;}));
        console.log([].findLastIndex((value:number[]):boolean=>{return value.length>0;}));
        const numbers:number[]=[1.0,Number("NaN"),-0.0];
        console.log(numbers.some((value:number):boolean=>{return Number.isNaN(value);}));
        console.log(numbers.every((value:number):boolean=>{return Number.isFinite(value);}));
        console.log(numbers.findIndex((value:number):boolean=>{return Number.isNaN(value);}));
        console.log(numbers.findLastIndex((value:number):boolean=>{return Number.isFinite(value);}));
    }"#);
    expected.push_str("false\ntrue\n-1\n-1\ntrue\n-1\ntrue\nfalse\n1\n2\n");
    native(&body, &expected, "", "");
}

#[test]
fn short_circuit_original_indices_receiver_and_callback_effects_are_exact() {
    let analysis = native(
        r#"
    type Box={n:int;};type Grid={rows:int[][];};
    function choose(values:int[],action:Mutable<Function<boolean, [value:int,index:int,source:ReadOnly<int[]>]>>):int{return values.findLastIndex(action);}
    function values():int[]{console.log("receiver");return [10,20,30,40,50];}
    function empty():int[]{console.log("empty");return [];}
    function action():()=>boolean{console.log("callback");return ():boolean=>{console.log("visit");return true;};}
    function main(){
        console.log(values().some(action()));console.log(empty().every(action()));
        console.log(empty().some(action()));console.log(empty().findIndex(action()));console.log(empty().findLastIndex(action()));
        const source=[1,2,3,4,5];let trace=0;
        console.log(source.some((value:int,index:int,all:int[]):boolean=>{trace=trace*10+index+1;return index===2&&all.length===5;}));console.log(trace);trace=0;
        console.log(source.every((value:int,index:int):boolean=>{trace=trace*10+index+1;return index<2;}));console.log(trace);trace=0;
        console.log(source.findIndex((value:int,index:int):boolean=>{trace=trace*10+index+1;return index===3;}));console.log(trace);trace=0;
        console.log(source.findLastIndex((value:int,index:int):boolean=>{trace=trace*10+index+1;return index===1;}));console.log(trace);trace=0;
        console.log(source.findLastIndex((value:int,index:int):boolean=>{trace=trace*10+index+1;return false;}));console.log(trace);
        const target=3;const read=(value:int):boolean=>{return value===target;};
        console.log(source.findIndex(read));console.log(source.findLastIndex(read));console.log(read(3));
        const full=(value:int,index:int,all:int[]):boolean=>{return value===all[index];};console.log(choose(source,full));console.log(full(3,2,source));
        const other:Box={n:0};console.log(source.some((value:int):boolean=>{other.n=other.n+1;return value===2;}));console.log(other.n);other.n=9;console.log(other.n);
        const grid:Grid={rows:[[0,1],[2,3]]};console.log(grid.rows[1].findLastIndex((value:int):boolean=>{return value>1;}));
        console.log([false,true].some((value:boolean,index:int):boolean=>{return value&&index===1;}));
        console.log([1,2,3,4].every(():boolean=>{return true;}));
    }"#,
        "receiver\ncallback\nvisit\ntrue\nempty\ncallback\ntrue\nempty\ncallback\nfalse\nempty\ncallback\n-1\nempty\ncallback\n-1\ntrue\n123\nfalse\n123\n3\n1234\n1\n5432\n-1\n54321\n2\n2\ntrue\n4\ntrue\ntrue\n2\n9\n1\ntrue\ntrue\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("choose.values"), Some(&Mode::Read));
    assert_eq!(
        analysis.contracts().get("choose.action"),
        Some(&Mode::Mutate)
    );
}

#[test]
fn borrowed_owned_values_do_not_clone_and_short_circuit_or_unwind_releases_temporaries() {
    native(
        r#"
    type Node={name:string;children:Node[];};
    function some(values:string[]):boolean{return values.some((value:string):boolean=>{return value.length>0;});}
    function every(values:string[]):boolean{return values.every((value:string):boolean=>{return value.length>0;});}
    function first(values:string[]):int{return values.findIndex((value:string):boolean=>{return value.length>0;});}
    function last(values:string[]):int{return values.findLastIndex((value:string):boolean=>{return value.length>0;});}
    function nested(values:int[][]):int{return values.findLastIndex((value:int[]):boolean=>{return value.length>0;});}
    function forest(values:Node[]):boolean{return values.every((value:Node):boolean=>{return value.children.length===1;});}
    function early(values:string[]):int{return values.findIndex((value:string,index:int):boolean=>{if(index>0){const failures=[1];const failure=failures.remove(-1);}return true;});}
    function failure(values:string[]):boolean{return values.some((value:string,index:int):boolean=>{if(index===1){const failures=[1];const failure=failures.remove(-1);}return false;});}
    function main(){}
    "#,
        "",
        r#"
    use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,Ordering};
    static LIVE:AtomicIsize=AtomicIsize::new(0);struct Allocations;
    unsafe impl GlobalAlloc for Allocations{
        unsafe fn alloc(&self,layout:Layout)->*mut u8{let pointer=unsafe{System.alloc(layout)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);}pointer}
        unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
    }
    #[global_allocator]static ALLOCATOR:Allocations=Allocations;
    "#,
        r#"
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
    let input=Array::Heap(vec![String::Owned("left".repeat(16)),String::Owned("right".repeat(16))]);
    let live=LIVE.load(Ordering::SeqCst);let pointer=input.get(0).unwrap().as_ptr();
    assert!(function0(&input));assert!(function1(&input));assert_eq!(function2(&input),0);assert_eq!(function3(&input),1);assert_eq!(function6(&input),0);
    assert_eq!(LIVE.load(Ordering::SeqCst),live);assert_eq!(input.get(0).unwrap().as_ptr(),pointer);
    assert!(std::panic::catch_unwind(||function7(&input)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    assert_eq!(function3(&Array::Heap(vec![String::Owned("temporary".repeat(8))])),0);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let input=Array::Heap(vec![NumericArray::Heap(vec![1,2]),NumericArray::Heap(vec![])]);let live=LIVE.load(Ordering::SeqCst);assert_eq!(function4(&input),0);assert_eq!(LIVE.load(Ordering::SeqCst),live);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let leaf=Record0{field0:String::Owned("leaf".into()),field1:RecordArray0::from([])};
    let input=RecordArray0::from([Record0{field0:String::Owned("root".into()),field1:RecordArray0::from([leaf])}]);let live=LIVE.load(Ordering::SeqCst);assert!(function5(&input));assert_eq!(LIVE.load(Ordering::SeqCst),live);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    std::panic::set_hook(hook);
    "#,
    );
}

#[test]
fn predicates_enforce_boolean_reusable_callbacks_and_existing_receiver_loans() {
    for method in ["some", "every", "findIndex", "findLastIndex"] {
        for (source, code) in [
            (format!("function main(){{[1].{method}();}}"), "arity"),
            (
                format!("function main(){{[1].{method}(():boolean=>{{return true;}},0);}}"),
                "arity",
            ),
            (
                format!("function main(){{(1).{method}(():boolean=>{{return true;}});}}"),
                "type",
            ),
            (format!("function main(){{[1].{method}(1);}}"), "type"),
            (
                format!("function main(){{[1].{method}((v:int):int=>{{return v;}});}}"),
                "type",
            ),
            (
                format!("function main(){{[1].{method}((v:boolean):boolean=>{{return v;}});}}"),
                "type",
            ),
            (
                format!(
                    "function main(){{[1].{method}((v:int,i:number):boolean=>{{return true;}});}}"
                ),
                "type",
            ),
            (
                format!(
                    "function main(){{[1].{method}((v:int,i:int,all:string[]):boolean=>{{return true;}});}}"
                ),
                "type",
            ),
            (
                format!(
                    "function main(){{[1].{method}((v:int,i:int,all:int[],extra:int):boolean=>{{return true;}});}}"
                ),
                "arity",
            ),
            (
                format!("function main(){{[].{method}(():boolean=>{{return true;}});}}"),
                "array-type",
            ),
            (
                format!(
                    "function main(){{[\"x\"].{method}((v:string):boolean=>{{const moved=v;return true;}});}}"
                ),
                "callback-access",
            ),
            (
                format!(
                    "function main(){{[[1]].{method}((v:int[]):boolean=>{{v.push(2);return true;}});}}"
                ),
                "callback-access",
            ),
            (
                format!(
                    "function main(){{[1].{method}((v:int,i:int,all:int[]):boolean=>{{all.push(2);return true;}});}}"
                ),
                "callback-access",
            ),
            (
                format!(
                    "function main(){{const values=[1];values.{method}((v:int):boolean=>{{values.push(2);return true;}});}}"
                ),
                "conflicting-access",
            ),
            (
                format!(
                    "function bad(values:int[],action:Once<Function<boolean, [v:int]>>):boolean{{return values.{method}(action)===true;}}function main(){{}}"
                ),
                "callback-access",
            ),
        ] {
            let error = check(&source)
                .err()
                .unwrap_or_else(|| panic!("Expected {code}: {source}"));
            assert_eq!(error.code, code, "{source}: {}", error.message);
        }
    }
}
