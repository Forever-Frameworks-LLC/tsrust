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
    let path = directory.0.join("array-transform.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-transform-{level}"));
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
fn transforms_cover_all_leaf_types_ranks_contexts_and_callback_arities() {
    native(
        r#"
    type Row={value:number;text:string;values:int[];};
    type Node={name:string;children:Node[];};
    function main(){
        const ints=[2,4,6];const mapped=ints.map((value:int,index:int,source:int[]):int=>{return value+index+source.length;});
        console.log(mapped[0]);console.log(mapped[2]);const selected=ints.filter((value:int):boolean=>{return value>2;});console.log(selected.length);console.log(selected[0]);
        const zeros=ints.map(():int=>{return 7;});console.log(zeros[1]);console.log(ints.filter(():boolean=>{return false;}).length);
        const decimals=[1,2].map((value:number):number=>{return value+0.5;});console.log(decimals[1]);
        const flags=[true,false,true].map((value:boolean,index:int):boolean=>{return value&&index<2;});console.log(flags[0]);console.log(flags[2]);
        const text=[" A ","𐐀","é"].map((value:string):string=>{return value.trim();});console.log(text[0]);
        const kept=text.filter((value:string,index:int,source:string[]):boolean=>{return index>0&&source.length===3;});console.log(kept[0]);
        const rows=[[1,2],[3],[]].map((row:int[],index:int):int[]=>{return row.slice(0,index);});console.log(rows[0].length);console.log(rows[1][0]);
        const nested=[[[true]],[]].filter((row:boolean[][]):boolean=>{return row.length>0;});console.log(nested[0][0][0]);
        const numbers:number[][]=[[1.25],[]];console.log(numbers.map((row:number[]):number=>{return Number(row.length);})[0]);
        const strings:string[][]=[["x"],[]];console.log(strings.filter((row:string[]):boolean=>{return row.length>0;})[0][0]);
        const records=[{value:1,text:"a",values:[1]}].map((row:Row):Row=>{return clone(row);});console.log(records[0].value);
        const typed:Row[][]=[[{value:2,text:"b",values:[]}]];console.log(typed.map((row:Row[]):Row[]=>{return clone(row);})[0][0].text);
        const forest:Node[]=[{name:"root",children:[{name:"leaf",children:[]}]}];const copy=forest.filter((node:Node):boolean=>{return true;});
        copy[0].name="copy";console.log(forest[0].name);console.log(copy[0].children[0].name);
        const empty=[].map((value:string):int=>{return value.length;});console.log(empty.length);
        const emptyRecords=[].filter((row:Row):boolean=>{return true;});console.log(emptyRecords.length);
        const changed=ints.map((value:int):Row=>{return {value:Number(value),text:"row",values:[value]};});console.log(changed[2].values[0]);
        const deep=ints.map((value:int):int[][]=>{return [[value],[]];});console.log(deep[1][0][0]);
    }
    "#,
        "5\n11\n2\n4\n7\n0\n2.5\ntrue\nfalse\nA\n𐐀\n0\n3\ntrue\n1\nx\n1\nb\nroot\nleaf\n0\n0\n6\n4\n",
        "",
        "",
    );
}

#[test]
fn reusable_callbacks_captures_and_receivers_run_once_in_source_order() {
    let analysis = native(
        r#"
    function apply(values:int[], action:Mutable<Function<int, [value:int,index:int,source:ReadOnly<int[]>]>>):int[]{return values.map(action);}
    function choose(values:string[],action:Mutable<Function<boolean, [value:ReadOnly<string>]>>):string[]{return values.filter(action);}
    function makeValues():int[]{console.log("receiver");return [10,20,30];}
    function makeAction():(value:int,index:int)=>int{
        console.log("callback");let count=0;
        return (value:int,index:int):int=>{count=count+1;console.log(index);return value+count;};
    }
    function empty():int[]{console.log("empty");return [];}
    function mutateOther(values:int[], other:int[]):int[]{return values.map((value:int):int=>{other.push(value);return value;});}
    function main(){
        const result=makeValues().map(makeAction());console.log(result[2]);
        console.log(empty().filter((value:int):boolean=>{console.log("unreachable");return true;}).length);
        console.log(empty().map(makeAction()).length);
        let trace=0;const values=[1,2,3];
        const result2=values.map((value:int,index:int):int=>{trace=trace*10+value;return value+index;});console.log(trace);console.log(result2[2]);
        const read=(value:int,index:int,source:int[]):int=>{return value+index+source.length;};
        console.log(apply(values,read)[2]);console.log(read(1,0,values));
        const prefix="x";const match=(value:string):boolean=>{return value===prefix;};console.log(choose(["x","y"],match).length);
        const other:int[]=[];console.log(mutateOther(values,other).length);console.log(other[2]);
        const retained=values.filter((value:int):boolean=>{return values.length===3;});values.push(4);console.log(retained.length);
    }
    "#,
        "receiver\ncallback\n0\n1\n2\n33\nempty\n0\nempty\ncallback\n0\n123\n5\n8\n4\n1\n3\n3\n3\n",
        "",
        "",
    );
    for name in ["apply.values", "choose.values", "mutateOther.values"] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
    for name in ["apply.action", "choose.action", "mutateOther.other"] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Mutate));
    }
}

#[test]
fn selected_owned_values_are_independent_and_partial_results_drop_on_unwind() {
    native(
        r#"
    type Node={name:string;children:Node[];};
    function texts(values:string[]):string[]{return values.filter((value:string):boolean=>{return true;});}
    function nested(values:int[][]):int[][]{return values.filter((value:int[]):boolean=>{return true;});}
    function forests(values:Node[]):Node[]{return values.filter((value:Node):boolean=>{return true;});}
    function failMap(values:string[]):string[]{return values.map((value:string,index:int):string=>{if(index===1){const failures=[1];const failure=failures.remove(-1);}return clone(value);});}
    function failFilter(values:string[]):string[]{return values.filter((value:string,index:int):boolean=>{if(index===1){const failures=[1];const failure=failures.remove(-1);}return true;});}
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
    let live=LIVE.load(Ordering::SeqCst);let mut output=function0(&input);
    assert_eq!(output.get(0).unwrap().as_ref(),input.get(0).unwrap().as_ref());assert_ne!(output.get(0).unwrap().as_ptr(),input.get(0).unwrap().as_ptr());
    *output.get_mut(0).unwrap()=String::Borrowed("changed");assert_eq!(input.get(0).unwrap().len(),64);drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    assert!(std::panic::catch_unwind(||function3(&input)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    assert!(std::panic::catch_unwind(||function4(&input)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let input=Array::Heap(vec![NumericArray::Heap(vec![1,2,3])]);let mut output=function1(&input);output.get_mut(0).unwrap().push(4);assert_eq!(input.get(0).unwrap().len(),3);drop(output);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let mut root=Record0{field0:String::Owned("leaf".into()),field1:RecordArray0::from([])};
    for _ in 0..4096{root=Record0{field0:String::Owned("node".into()),field1:RecordArray0::from([root])};}
    let input=RecordArray0::from([root]);let output=function2(&input);assert_ne!(output[0].field0.as_ptr(),input[0].field0.as_ptr());drop(output);drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    std::panic::set_hook(hook);
    "#,
    );
}

#[test]
fn callback_diagnostics_preserve_source_loans_access_arity_and_type_constraints() {
    for (source, code) in [
        ("function main(){[1].map();}", "arity"),
        (
            "function main(){[1].filter(():boolean=>{return true;},0);}",
            "arity",
        ),
        ("function main(){1.map(():int=>{return 1;});}", "type"),
        ("function main(){[1].map(1);}", "type"),
        (
            "function main(){[1].map((value:int,index:int,source:int[],extra:int):int=>{return value;});}",
            "arity",
        ),
        (
            "function main(){[1].filter((value:int):int=>{return value;});}",
            "type",
        ),
        (
            "function main(){[1].map((value:boolean):int=>{return 1;});}",
            "type",
        ),
        (
            "function main(){[1].map((value:int,index:number):int=>{return value;});}",
            "type",
        ),
        (
            "function main(){[1].map((value:int,index:int,source:string[]):int=>{return value;});}",
            "type",
        ),
        (
            "function main(){[1].map((value:int):void=>{});}",
            "array-element",
        ),
        (
            "function main(){[].map(():int=>{return 1;});}",
            "array-type",
        ),
        (
            "function main(){[\"x\"].map((value:string):string=>{return value;});}",
            "callback-access",
        ),
        (
            "function main(){[[1]].filter((row:int[]):boolean=>{row.push(2);return true;});}",
            "callback-access",
        ),
        (
            "function main(){[1].map((value:int,index:int,source:int[]):int=>{source.push(2);return value;});}",
            "callback-access",
        ),
        (
            "function main(){const values=[1];values.map((value:int):int=>{values.push(2);return value;});}",
            "conflicting-access",
        ),
        (
            "function main(){const text=\"x\";const once=():string=>{return text;};[1].map(once);}",
            "callback-access",
        ),
        (
            "function bad(values:int[],action:Once<Function<int, [value:int]>>):int[]{return values.map(action);}function main(){}",
            "callback-access",
        ),
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("Expected {code}: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
}

#[test]
fn nested_transform_parents_preserve_native_callback_results() {
    let map = format!("{}7{}", "[7].map((value:int):int=>".repeat(16), ")[0]".repeat(16));
    let foreach = format!("{}console.log(7);{}", "[7].forEach((value:int):void=>{".repeat(32), "});".repeat(32));
    let groups = format!("{}7{}", "Map.groupBy([7],(value:int):int=>".repeat(16), ").size".repeat(16));
    let source = support::format_round_trip(&format!("function main(){{console.log({map});{foreach}console.log({groups});}}"));
    native(&source, "7\n7\n1\n", "", "");
}
