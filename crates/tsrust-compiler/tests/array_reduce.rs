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
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("array-reduce.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-reduce-{level}"));
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
fn reductions_cover_all_data_leaves_ranks_contexts_and_both_directions() {
    let cases = [
        ("[1,2,3]", "int", "v", "", "3\n1\n", 3),
        ("[1.25,2.5]", "number", "v", "", "2.5\n1.25\n", 2),
        ("[true,false]", "boolean", "v", "", "false\ntrue\n", 2),
        (
            r#"["é","𐐀","\u0000"]"#,
            "string",
            "clone(v)",
            ".length",
            "1\n2\n",
            3,
        ),
        ("[{n:1},{n:2}]", "Row", "clone(v)", ".n", "2\n1\n", 2),
        ("[[1,2],[3]]", "int[]", "clone(v)", ".length", "1\n2\n", 2),
        (
            "[[1.25],[]]",
            "number[]",
            "clone(v)",
            ".length",
            "0\n1\n",
            2,
        ),
        (
            "[[true],[]]",
            "boolean[]",
            "clone(v)",
            ".length",
            "0\n1\n",
            2,
        ),
        (
            r#"[["é"],[]]"#,
            "string[]",
            "clone(v)",
            ".length",
            "0\n1\n",
            2,
        ),
        ("[[{n:1}],[]]", "Row[]", "clone(v)", ".length", "0\n1\n", 2),
        ("[[[1]],[]]", "int[][]", "clone(v)", ".length", "0\n1\n", 2),
        (
            "[{n:1,children:[]},{n:2,children:[{n:3,children:[]}]}]",
            "Node",
            "clone(v)",
            ".n",
            "2\n1\n",
            2,
        ),
    ];
    let mut source =
        String::from("type Row={n:int;};type Node={n:int;children:Node[];};function main(){");
    let mut expected = String::new();
    for (case, value, selected, observation, output, len) in cases {
        for method in ["reduce", "reduceRight"] {
            source.push_str(&format!("console.log({case}.{method}((a:{value},v:{value}):{value}=>{{return {selected};}}){observation});"));
        }
        expected.push_str(output);
        for method in ["reduce", "reduceRight"] {
            source.push_str(&format!("console.log({case}.{method}((a:int,v:{value},index:int,source:{value}[]):int=>{{return a+index+source.length;}},0));"));
            expected.push_str(&format!("{}\n", len * len + len * (len - 1) / 2));
        }
    }
    source.push_str("const empty:int[]=[];console.log(empty.reduce(():int=>{return 99;},7));console.log(empty.reduceRight(():int=>{return 99;},8));console.log([1,2].reduce(():int=>{return 9;},0));console.log([1,2].reduceRight((a:int):int=>{return a+1;},0));console.log([1,2].reduce((a:int,v:int,i:int):int=>{return a+v+i;},0));}");
    expected.push_str("7\n8\n9\n2\n4\n");
    native(&source, &expected, "", "");
}

#[test]
fn reduction_callbacks_inputs_captures_and_owned_accumulator_modes_preserve_order() {
    let analysis = native(
        r#"
    function apply(values:int[], action:Mutable<Function<int, [a:int,v:int,i:int,source:ReadOnly<int[]>]>>):int{return values.reduce(action,0);}
    function makeValues():int[]{console.log("receiver");return [1,2,3];}
    function makeAction():(a:int,v:int,i:int)=>int{console.log("callback");return(a:int,v:int,i:int):int=>{console.log(i);return a*10+v;};}
    function initial():int{console.log("initial");return 0;}
    function mutating(values:int[]):int[]{return values.reduce((a:int[],v:int):int[]=>{a.push(v);return clone(a);},[]);}
    function consuming(values:int[]):int[]{return values.reduceRight((a:int[],v:int):int[]=>{a.push(v);return a;},[]);}
    function reading(values:string[]):string{return values.reduce((a:string,v:string):string=>{return v.trim();},"initial");}
    function forwardOwned(values:int[], action:Mutable<Function<int[], [a:Owned<int[]>,v:int]>>):int[]{return values.reduce(action,[]);}
    function forwardRead(values:string[], action:Mutable<Function<string, [a:ReadOnly<string>,v:ReadOnly<string>]>>):string{return values.reduceRight(action,"initial");}
    function forwardMutate(values:int[], action:Mutable<Function<int[], [a:Mutable<int[]>,v:int]>>):int[]{return values.reduce(action,[]);}
    function main(){
        console.log(makeValues().reduce(makeAction(),initial()));
        console.log(makeValues().reduceRight(makeAction(),initial()));
        let trace=0;let visits=0;const values=[1,2,3];
        console.log(values.reduce((a:int,v:int,i:int,source:int[]):int=>{trace=trace*10+i+1;visits=visits+1;return a+v+source.length;},0));console.log(trace);console.log(visits);
        const action=(a:int,v:int,i:int,source:int[]):int=>{return a+v+source.length;};console.log(action(0,1,0,values));
        const output=mutating(values);console.log(output[0]);console.log(output[2]);
        const reversed=consuming(values);console.log(reversed[0]);console.log(reversed[2]);
        const take=(a:int[],v:int):int[]=>{a.push(v);return a;};const moved=forwardOwned(values,take);console.log(moved[2]);
        console.log(reading([" left "," right "]));
        const read=(a:string,v:string):string=>{return v.trim();};console.log(forwardRead([" left "," right "],read));
        const mutate=(a:int[],v:int):int[]=>{a.push(v);return clone(a);};console.log(forwardMutate(values,mutate)[2]);
        console.log(values.map((v:int):int=>{return [v,v].reduce((a:int,b:int):int=>{return a+b;},0);}).reduceRight((a:int,v:int):int=>{return a+v;},0));
        const box:Box={values:[4,5]};console.log(box.values.reduce((a:int,v:int):int=>{return a+v;},0));
        const rows=[[6],[7]];console.log(rows[1].reduce((a:int,v:int):int=>{return a+v;},0));
        const zero:int[]=[];console.log(zero.reduce(makeAction(),initial()));
        console.log([].reduce(makeAction(),0));
        console.log([5].reduce((a:int,v:int):int=>{console.log("never");return a+v;}));
        const one=values.reduce((a:Row,v:int):Row=>{return {n:a.n+v};},{n:0});console.log(one.n);
        const deep=values.reduce((a:int[][],v:int):int[][]=>{a.push([v]);return a;},[]);console.log(deep[2][0]);
    }
    type Box={values:int[];};type Row={n:int;};
    "#,
        "receiver\ncallback\ninitial\n0\n1\n2\n123\nreceiver\ncallback\ninitial\n2\n1\n0\n321\n15\n123\n3\n4\n1\n3\n3\n1\n3\nright\nleft\n3\n12\n9\n7\ncallback\ninitial\n0\ncallback\n0\n5\n6\n3\n",
        "",
        "",
    );
    for name in [
        "apply.values",
        "mutating.values",
        "consuming.values",
        "reading.values",
        "forwardOwned.values",
        "forwardRead.values",
        "forwardMutate.values",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
    for name in [
        "apply.action",
        "forwardOwned.action",
        "forwardRead.action",
        "forwardMutate.action",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Mutate));
    }
}

#[test]
fn reduction_diagnostics_reject_invalid_types_consuming_callbacks_and_aliases() {
    for (body, code) in [
        ("[1].reduce();", "arity"),
        ("[1].reduce((a:int):int=>{return a;},0,1);", "arity"),
        ("[1].reduce(7,0);", "type"),
        ("[1].reduce((a:int):boolean=>{return true;},0);", "type"),
        ("[1].reduce((a:int):int=>{return a;},true);", "type"),
        ("[1].reduce((a:int,v:boolean):int=>{return a;},0);", "type"),
        (
            "[1].reduce((a:int,v:int,i:number):int=>{return a;},0);",
            "type",
        ),
        (
            "[1].reduce((a:int,v:int,i:int,s:int[],extra:int):int=>{return a;},0);",
            "arity",
        ),
        ("[1].reduce(():void=>{},0);", "type"),
        ("[1].reduce((a:string):string=>{return a;});", "type"),
        (
            "const a=[[1]];a.reduce((acc:int[],v:int[]):int[]=>{v.push(2);return acc;},[]);",
            "callback-access",
        ),
        (
            "const a=[[1]];a.reduce((acc:int[],v:int[]):int[]=>{return v;},[]);",
            "callback-access",
        ),
        (
            "const a=[1];a.reduce((acc:int,v:int,i:int,s:int[]):int=>{s.push(2);return acc;},0);",
            "callback-access",
        ),
        (
            "const a=[1];a.reduce((acc:int,v:int):int=>{a.push(2);return acc;},0);",
            "conflicting-access",
        ),
        (
            "const a=[1];a.reduce((acc:int):int=>{return acc;},change(a));",
            "conflicting-access",
        ),
        (
            "const captured=\"owned\";const action=(acc:string,v:int):string=>{return captured;};[1].reduce(action,\"initial\");",
            "callback-access",
        ),
        (
            "const values=[\"x\"];values.reduce((a:string,v:string):string=>{return a;},values[0]);",
            "borrowed-element",
        ),
    ] {
        support::reject(
            &format!(
                "function change(a:int[]):int{{a.push(3);return 0;}}function main(){{{body}}}"
            ),
            code,
        );
    }
    // Existing named callback captures remain live after the reduction.
    support::reject(
        "function main(){let n=0;const f=(a:int,v:int):int=>{n=n+1;return a+v;};[1].reduce(f,0);console.log(n);}",
        "capture-conflict",
    );
}

#[test]
fn no_initial_empty_failure_preserves_receiver_callback_order_and_native_fixed_abi() {
    for method in ["reduce", "reduceRight"] {
        let source = format!(
            r#"
            function values():int[]{{console.log("receiver");return [];}}
            function callback():(a:int,v:int)=>int{{console.log("callback");return(a:int,v:int):int=>{{console.log("unreachable");return a+v;}};}}
            function main(){{values().{method}(callback());}}
        "#
        );
        let rust = check(&source).unwrap().emit_rust();
        let directory = support::TempDirectory::new();
        let path = directory.0.join("empty-reduction.rs");
        std::fs::write(&path, rust).unwrap();
        for level in ["0", "3"] {
            let executable = directory.0.join(format!("empty-{level}"));
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
            assert!(!output.status.success());
            assert_eq!(output.stdout, b"receiver\ncallback\n");
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("empty array reduction without an initial value")
            );
        }
    }
    let rust = native("function fixed(sum:(acc:int,v:int)=>int){const a=[1,2,3];console.log(a.reduce(sum,0));console.log(a.reduceRight(sum));a[0]=4;console.log(a[0]);}function main(){fixed((acc:int,v:int):int=>{return acc+v;});}", "6\n6\n4\n", "", "").emit_rust();
    assert!(rust.contains("[i64; 3]"));
    let source = "function ordinary(withSource:(acc:int,v:int,i:int,source:ReadOnly<int[]>)=>int){const a=[1,2,3];console.log(a.reduce(withSource,0));a[0]=4;console.log(a[0]);}function main(){ordinary((acc:int,v:int,i:int,source:int[]):int=>{return acc+source[i];});}";
    let rust = check(source).unwrap().emit_rust();
    assert!(!rust.contains("[i64; 3]"));
}

#[test]
fn owned_seeds_accumulators_and_temporary_sources_dispose_on_normal_and_unwind_paths() {
    native(
        r#"
    type Node={name:string;children:Node[];};
    function seed(values:string[]):string{return values.reduce((a:string,v:string):string=>{return a;});}
    function rows(values:int[][]):int[]{return values.reduceRight((a:int[],v:int[]):int[]=>{return a;});}
    function forest(values:Node[]):Node{return values.reduce((a:Node,v:Node):Node=>{return a;});}
    function failConsume(values:string[],initial:string):string{return values.reduce((a:string,v:string,i:int):string=>{if(i===1){const bad=[1];bad.remove(-1);}return a;},initial);}
    function failMutate(values:int[],initial:int[]):int[]{return values.reduceRight((a:int[],v:int,i:int):int[]=>{a.push(v);if(i===0){const bad=[1];bad.remove(-1);}return clone(a);},initial);}
    function temporary():string{return [" x ".repeat(8)," y ".repeat(8)].reduce((a:string,v:string):string=>{return clone(v);});}
    function numbers(values:number[]):number{return values.reduce((a:number,v:number):number=>{return a;});}
    function numbersRight(values:number[]):number{return values.reduceRight((a:number,v:number):number=>{return a;});}
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
    let input=Array::Heap(vec![String::Owned("left".repeat(16)),String::Owned("right".repeat(16))]);let live=LIVE.load(Ordering::SeqCst);
    let output=function0(&input);assert_eq!(output.as_ref(),input.get(0).unwrap().as_ref());assert_ne!(output.as_ptr(),input.get(0).unwrap().as_ptr());drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    assert!(std::panic::catch_unwind(||function3(&input,String::Owned("initial".repeat(12)))).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let output=function5();assert_eq!(output.as_ref()," y ".repeat(8));drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let rows=Array::Heap(vec![NumericArray::from([1i64,2]),NumericArray::from([3i64])]);let mut selected=function1(&rows);*selected.get_mut(0).unwrap()=9;assert_eq!(*rows.get(1).unwrap().get(0).unwrap(),3);drop(selected);drop(rows);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let ints=NumericArray::from([1i64,2]);assert!(std::panic::catch_unwind(||function4(&ints,NumericArray::Heap(vec![7i64;8]))).is_err());drop(ints);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let tree=Record0{field0:String::Owned("root".repeat(8)),field1:RecordArray0::from([Record0{field0:String::Owned("leaf".repeat(8)),field1:RecordArray0::from([])}])};let nodes=RecordArray0::from([tree]);let mut selected=function2(&nodes);selected.field0=String::Borrowed("changed");assert_eq!(nodes[0].field0.as_ref(),"root".repeat(8));drop(selected);drop(nodes);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    for value in [-0.0f64,0.0,f64::INFINITY,f64::NEG_INFINITY,f64::from_bits(0x7ff8_0000_0000_0123)]{let values=vec![value,1.25,value];assert_eq!(function6(&values).to_bits(),value.to_bits());assert_eq!(function7(&values).to_bits(),value.to_bits());drop(values);}assert_eq!(LIVE.load(Ordering::SeqCst),live);
    drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
    "#,
    );
}
