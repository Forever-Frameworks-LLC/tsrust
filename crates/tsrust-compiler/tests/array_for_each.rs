mod support;
use tsrust_compiler::check;

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
    let path = directory.0.join("array-for-each.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-for-each-{level}"));
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
fn readonly_slice_traversal_preserves_ranges_and_normal_callback_entry_points() {
    let analysis=native(
        r#"
        type Row={n:int;items:int[];};
        function make():Row[]{return [{n:1,items:[1]},{n:2,items:[]},{n:3,items:[2,3]},{n:4,items:[]},{n:5,items:[4]},{n:6,items:[]}];}
        function walk(from:int,to:int):int{
            const base=make();const selected=base.slice(from,to);let total=0;
            selected.forEach((row:Row,index:int,source:Row[]):void=>{let weight=row.n;for(const value of row.items){weight=weight+value;}total=total+weight+index+source.length;});
            return total;
        }
        function tail(from:int):int{
            const base=make();const selected=base.slice(from);let total=0;
            selected.forEach((row:Row,index:int,source:Row[]):void=>{total=total+row.n+source.length;});return total;
        }
        function forward(values:Row[],action:Mutable<Function<void,[v:ReadOnly<Row>,i:int,s:ReadOnly<Row[]>]>>):void{values.forEach(action);}
        function again():int{
            const base=make();const selected=base.slice(1,2);let total=0;
            {const read=(row:Row,index:int,source:Row[]):void=>{total=total+row.n+index+source.length;};
            selected.forEach(read);read({n:9,items:[]},4,[{n:1,items:[]}]);forward([{n:7,items:[]}],read);}return total;
        }
        function texts(input:string):int{const base=[clone(input),"","x",input.repeat(2),"last"];const selected=base.slice(1,4);let out=0;selected.forEach((text:string,index:int,source:string[]):void=>{out=out+text.length+index+source.length;});return out;}
        function main(){console.log(walk(0,6),walk(1,4),walk(-2,9223372036854775807),walk(-9223372036854775808,1),walk(4,1),walk(20,30),tail(-2),again(),texts("é"));}
        "#,
        "82 26 20 3 0 0 15 25 17\n", "", "",
    );
    assert!(analysis.emit_rust().contains("fn call_range"));
    assert!(analysis.emit_rust().contains(".skip(for_each_start).take(for_each_len)"));
}

#[test]
fn borrowed_ranges_discard_owned_results_and_preserve_regular_callback_returns() {
    let analysis=native(
        r#"
        type Row={text:string;items:string[][];};
        function forward(action:Mutable<Function<string[][],[v:int,i:int,source:ReadOnly<int[]>]>>):int{const result=action(4,2,[1,2]);return result[0][0].length;}
        function walk():int{
            const base=[1,2,3,4,5];let seen=0;
            {const visit=(v:int,index:int,source:int[]):string[][]=>{seen=seen+v+index+source.length;return [["x".repeat(v%3+1)],[]];};
            const selected=base.slice(1,4);selected.forEach(visit);
            const result=visit(9,4,[1]);console.log(result[0][0]);console.log(forward(visit));}
            return seen;
        }
        function records():int{
            const base=[1,2,3,4,5];const selected=base.slice(-3);let seen=0;
            selected.forEach((v:int,index:int,source:int[]):Row=>{seen=seen+v+index+source.length;return {text:"owned".repeat(v),items:[["nested".repeat(v)]]};});return seen;
        }
        function callables():int{
            const base=[1,2,3,4,5];const selected=base.slice(1,3);let seen=0;
            selected.forEach((v:int,index:int,source:int[]):()=>string=>{seen=seen+v+index+source.length;const text="owned".repeat(v);return ():string=>{return text;};});return seen;
        }
        function main(){console.log(walk(),records(),callables());}
        "#,
        "x\n2\n43 24 10\n", "", "",
    );
    assert!(analysis.emit_rust().contains("fn call_range"));
    assert_eq!(analysis.emit_rust().matches(".skip(for_each_start).take(for_each_len)").count(),3);
}

#[test]
fn nested_range_results_keep_empty_variable_and_fallible_return_contracts() {
    native(r#"
        function variable(flag:boolean):int{
            const base=[1,2,3,4,5];const selected=base.slice(0,2);let out=0;let size=0;
            {const action=(v:int,index:int,source:int[]):string[][]=>{out=out+v+source.length;if(flag){return [["a"]];}return [["a"],[]];};
            selected.forEach(action);const result=action(1,0,[1]);size=result.length;}return out+size;
        }
        function empty():int{
            const base=[1,2,3,4,5];const selected=base.slice(0,2);let out=0;let size=0;
            {const action=(v:int,index:int,source:int[]):string[][]=>{out=out+v+source.length;return [];};
            selected.forEach(action);const result=action(1,0,[1]);size=result.length;}return out+size;
        }
        function spread():int{
            const base=[1,2,3,4,5];const selected=base.slice(0,1);let out=0;let size=0;
            {const action=(v:int,index:int,source:int[]):string[][]=>{out=out+v+source.length;return [...[["a"],[]]];};
            selected.forEach(action);const result=action(5,0,[]);size=result.length;}return out+size;
        }
        function main(){console.log(variable(true),variable(false),empty(),spread());}
    "#, "10 11 9 9\n", "", "");
    native(r#"
        function piece(i:int):string{if(i===1){throw "fault";}return "x";}
        function walk():int{const base=[1,2,3,4,5];const selected=base.slice(0,3);let out=0;selected.forEach((v:int,i:int,source:int[]):string[][]=>{out=out+v+source.length;return [[piece(i)],[]];});return out;}
        function main(){try{console.log(walk());}catch(error){if(typeof error==="string"){console.log(error);}else{throw error;}}}
    "#, "fault\n", "", "");
    let items=std::iter::repeat_n("[]",64).collect::<Vec<_>>().join(",");
    let source=format!("function main(){{const base=[1,2,3,4,5];const selected=base.slice(0,1);let out=0;let size=0;{{const action=(v:int,index:int,source:int[]):string[][]=>{{out=out+v+source.length;return [{items}];}};selected.forEach(action);const result=action(5,0,[]);size=result.length;}}console.log(out+size);}}");
    let analysis=native(&source,"71\n","","");
    assert!(!analysis.emit_rust().contains("-> [Array<String>;64]"),"oversized return must retain owned heap storage");
}

#[test]
fn slice_views_fall_back_for_source_moves_mutation_and_whole_array_reads() {
    let analysis=native(
        r#"
        type Row={n:int;items:int[];};
        function make():Row[]{return [{n:1,items:[]},{n:2,items:[3]},{n:4,items:[]},{n:5,items:[6,7]},{n:8,items:[]}];}
        function consume(values:Row[]):void{const owned=values;console.log(owned.length);}
        function moved(flag:boolean):int{const base=make();const selected=base.slice(0,1);let out=0;if(flag){consume(base);selected.forEach((row:Row,index:int,source:Row[]):void=>{out=out+row.n+source.length;});return out;}selected.forEach((row:Row,index:int,source:Row[]):void=>{out=out+row.n+source.length;});return out;}
        function edit(values:Row[]):void{values[0].n=99;}
        function changed():int{const base=make();const selected=base.slice(0,1);edit(base);let out=0;selected.forEach((row:Row,index:int,source:Row[]):void=>{out=out+row.n+source.length;});return out;}
        function indexed():int{const base=make();const selected=base.slice(1,3);let out=0;selected.forEach((row:Row,index:int,source:Row[]):void=>{out=out+source[0].n;});return out;}
        function main(){console.log(moved(true));console.log(moved(false),changed(),indexed());}
        "#,
        "5\n2\n2 2 4\n", "", "",
    );
    assert!(!analysis.emit_rust().contains("fn call_range"));
}

#[test]
fn borrowed_slice_traversal_coexists_with_typed_exception_transport() {
    native(
        r#"
        type Row={n:int;items:int[];};
        function walk():int{const base:Row[]=[{n:1,items:[]},{n:2,items:[3]},{n:4,items:[]},{n:5,items:[6,7]},{n:8,items:[]}];const selected=base.slice(1,3);let out=0;selected.forEach((row:Row,index:int,source:Row[]):void=>{out=out+row.n+source.length;});return out;}
        function caught():int{try{const values=[1];return values[9];}catch{return 7;}}
        function guarded():int{const base:Row[]=[{n:1,items:[]},{n:2,items:[3]},{n:4,items:[]},{n:5,items:[6,7]},{n:8,items:[]}];const selected=base.slice(0,1);let out=0;selected.forEach((row:Row,index:int,source:Row[]):void=>{try{const values=[0];out=out+values[9];}catch{out=out+row.n+source.length;}});return out;}
        function main(){console.log(walk(),caught(),guarded());}
        "#,
        "10 7 2\n", "", "",
    );
}

#[test]
fn slice_range_arguments_run_before_traversal_and_unwind_in_order() {
    let analysis=native(
        r#"
        type Row={n:int;items:int[];};
        function make():Row[]{return [{n:1,items:[]},{n:2,items:[3]},{n:4,items:[]},{n:5,items:[6,7]},{n:8,items:[]}];}
        function start():int{console.log("from");return 1;}
        function end(divisor:int):int{console.log("to");return 3/divisor;}
        function walk(divisor:int):void{const base=make();const selected=base.slice(start(),end(divisor));selected.forEach((row:Row,index:int,source:Row[]):void=>{console.log(row.n,index,source.length);});}
        function main(){walk(1);}
        "#,
        "from\nto\n2 0 2\n4 1 2\nfrom\nto\n", "",
        "std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||function3(0)).is_err());",
    );
    assert!(analysis.emit_rust().contains("fn call_range"));
}

#[test]
fn compact_array_iterator_skips_match_standard_slice_iterators() {
    native(
        "function main(){const values:string[]=[];values.push(\"x\");}","","",
        r#"
        for array in [Array::<i64>::from([]),Array::from([0]),Array::from([0,1,2,3]),Array::Heap((0..10).collect())]{
            let model=array.iter().copied().collect::<Vec<_>>();
            for first in [0,1,3,4,9,10,usize::MAX]{
                let mut actual=array.iter();let mut expected=model.iter();
                assert_eq!(actual.nth(first),expected.nth(first));assert_eq!(actual.len(),expected.len());
                assert_eq!(actual.next_back(),expected.next_back());assert_eq!(actual.len(),expected.len());
                assert_eq!(actual.nth(1),expected.nth(1));assert_eq!(actual.len(),expected.len());
                assert_eq!(actual.nth(usize::MAX),expected.nth(usize::MAX));assert_eq!(actual.len(),expected.len());
            }
        }
        "#,
    );
}

#[test]
fn for_each_visits_all_data_leaves_ranks_and_current_receivers_without_output_storage() {
    let cases = [
        ("int", "[1,2,3,4]", "v", vec!["1", "2", "3", "4"]),
        (
            "number",
            "[1.25,-0.0,2.5,3.75,4.5]",
            "v",
            vec!["1.25", "-0", "2.5", "3.75", "4.5"],
        ),
        (
            "boolean",
            "[true,false,true]",
            "v",
            vec!["true", "false", "true"],
        ),
        (
            "string",
            r#"["é","😀","a\u0000b","e\u0301"]"#,
            "v",
            vec!["é", "😀", "a\0b", "e\u{301}"],
        ),
        ("Row", "[{n:1},{n:2}]", "v.n", vec!["1", "2"]),
        ("int[]", "[[1,2],[],[3]]", "v.length", vec!["2", "0", "1"]),
        (
            "number[]",
            "[[1.25],[],[2.5,3.75]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "boolean[]",
            "[[true],[],[false,true]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "string[]",
            r#"[["é"],[],["😀","x"]]"#,
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "Row[]",
            "[[{n:1}],[],[{n:2},{n:3}]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "int[][]",
            "[[[1]],[],[[2],[3]]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "Node",
            "[{n:1,children:[]},{n:2,children:[{n:3,children:[]}]}]",
            "v.n",
            vec!["1", "2"],
        ),
    ];
    let mut source = String::from(
        "type Row={n:int;};type Node={n:int;children:Node[];};type Box={items:int[];};function main(){",
    );
    let mut expected = String::new();
    for (index, (element, values, observe, items)) in cases.iter().enumerate() {
        source.push_str(&format!("const values{index}:{element}[]={values};values{index}.forEach((v:{element},i:int,s:{element}[]):void=>{{console.log(i);console.log({observe});console.log(s.length);}});console.log(values{index}.length);"));
        for (position, item) in items.iter().enumerate() {
            expected.push_str(&format!("{position}\n{item}\n{}\n", items.len()));
        }
        expected.push_str(&format!("{}\n", items.len()));
    }
    source.push_str("const empty:int[]=[];empty.forEach(():void=>{console.log(99);});[7].forEach(():void=>{console.log(7);});[8].forEach((v:int):int=>{console.log(v);return v;});[9].forEach((v:int,i:int):boolean=>{console.log(v+i);return true;});const box:Box={items:[10]};box.items.forEach((v:int):void=>{console.log(v);});const grid=[[11],[12]];grid[1].forEach((v:int):void=>{console.log(v);});[13].slice().forEach((v:int):void=>{console.log(v);});}");
    expected.push_str("7\n8\n9\n10\n12\n13\n");
    native(&source, &expected, "", "");
    let fixed=native("function apply(action:(v:int)=>void):void{const values=[1,2,3];values.forEach(action);values[0]=4;console.log(values[0]);}function main(){apply((v:int):void=>{console.log(v);});}","1\n2\n3\n4\n","","").emit_rust();
    assert!(
        fixed.contains("[i64; 3]"),
        "readonly forward traversal should preserve the existing eligible fixed scalar storage proof"
    );
}

#[test]
fn for_each_evaluation_captures_aliases_factories_and_valid_return_discard_preserve_contracts() {
    native(
        r#"
    type Box={n:int;};
    function receiver():int[]{console.log("receiver");return [4,5];}
    function factory():(v:int,i:int,s:ReadOnly<int[]>)=>int{console.log("callback");return(v:int,i:int,s:int[]):int=>{console.log(i);console.log(v);console.log(s.length);return 99;};}
    function empty():int[]{console.log("empty");return [];}
    function forward(values:int[],action:Mutable<Function<void, [v:int,i:int,s:ReadOnly<int[]>]>>):void{values.forEach(action);}
    function owning(prefix:string):(v:int)=>string{return(v:int):string=>{return prefix.trim();};}
    function main(){
      receiver().forEach(factory());empty().forEach(factory());
      let trace=0;{const action=(v:int,i:int,s:int[]):void=>{trace=trace*10+i+1;};const alias=action;forward([1,2,3],alias);forward([4],alias);}console.log(trace);
      const owned=owning(" owned ");[1,2].forEach(owned);[3].forEach(owned);
      [1,2].forEach((v:int):number=>{console.log(v);return Number(v)/4.0;});
      [1,2].forEach((v:int):string=>{return "owned".repeat(v);});
      [1,2].forEach((v:int):Box=>{return {n:v};});
      [1,2].forEach((v:int):int[][]=>{return [[v],[]];});
      [1,2].forEach((v:int):()=>int=>{return ():int=>{return v;};});
      [1,2].forEach((v:int):()=>()=>int=>{return ():()=>int=>{return ():int=>{return v;};};});
      let state=0;{[1,2].forEach((v:int):void=>{state=state+v;});}console.log(state);
      const unchanged=[[1],[2]];unchanged.forEach((v:int[]):int[]=>{return clone(v);});console.log(unchanged[0][0]);
    }
    "#,
        "receiver\ncallback\n0\n4\n2\n1\n5\n2\nempty\ncallback\n1231\n1\n2\n3\n1\n",
        "",
        "",
    );
}

#[test]
fn for_each_diagnostics_retain_readonly_sources_reusable_environments_arity_and_types() {
    for (body, code) in [
        ("[1].forEach();", "arity"),
        ("[1].forEach(():void=>{},0);", "arity"),
        ("[1].forEach(1);", "type"),
        ("(1).forEach(():void=>{});", "type"),
        ("true.forEach(():void=>{});", "type"),
        ("\"text\".forEach(():void=>{});", "type"),
        (
            "[1].forEach((a:int,b:int,c:int[],d:int):void=>{});",
            "arity",
        ),
        ("[1].forEach((v:string):void=>{});", "type"),
        ("[1].forEach((v:int,i:number):void=>{});", "type"),
        ("const result=[1].forEach(():int=>{return 2;});", "type"),
        (
            "const values=[[1]];values.forEach((v:int[]):void=>{v.push(2);});",
            "callback-access",
        ),
        (
            "const values=[[1]];values.forEach((v:int[]):int[]=>{return v;});",
            "callback-access",
        ),
        (
            "const values=[1];values.forEach((v:int,i:int,s:int[]):void=>{s.push(2);});",
            "callback-access",
        ),
        (
            "const values=[1];values.forEach((v:int):void=>{values.push(2);});",
            "conflicting-access",
        ),
        (
            "const captured=\"owned\";const action=(v:int):string=>{return captured;};[1].forEach(action);",
            "callback-access",
        ),
        (
            "const values=[1];values.forEach(changing(values));",
            "conflicting-access",
        ),
    ] {
        support::reject(
            &format!(
                "function changing(values:int[]):(v:int)=>void{{values.push(2);return(v:int):void=>{{}};}}function main(){{{body}}}"
            ),
            code,
        );
    }
    support::reject(
        "function main(){let n=0;const action=(v:int):void=>{n=n+v;};[1].forEach(action);console.log(n);}",
        "capture-conflict",
    );
    support::reject(
        "function main(){const rows=[[1]];rows.forEach((v:int[]):()=>int[]=>{return ():int[]=>{return v;};});}",
        "callback-access",
    );
}

#[test]
fn discarded_owned_and_callable_results_drop_per_invocation_and_unwind_without_leaks() {
    native(
        r#"
    type Box={text:string;rows:string[];};type Node={text:string;children:Node[];};
    function strings(values:int[]):void{values.forEach((v:int):string=>{return "allocated".repeat(16);});}
    function records(values:int[]):void{values.forEach((v:int):Box=>{return {text:"allocated".repeat(16),rows:["row".repeat(16)]};});}
    function arrays(values:int[]):void{values.forEach((v:int):string[][]=>{return [["allocated".repeat(16)],[]];});}
    function callables(values:int[]):void{values.forEach((v:int):()=>string=>{const text="allocated".repeat(16);return ():string=>{return text;};});}
    function recursive(values:int[]):void{values.forEach((v:int):Node=>{return {text:"root".repeat(16),children:[{text:"leaf".repeat(16),children:[]}]};});}
    function failing(values:int[]):void{values.forEach((v:int,i:int):string=>{const text="allocated".repeat(16);if(i===1){const bad=[1];bad.remove(-1);}return text;});}
    function empty(values:int[]):void{values.forEach((v:int):string=>{return "never".repeat(16);});}
    function fixed(values:int[]):void{const base=clone(values);const selected=base.slice(0);selected.forEach((v:int,i:int,source:int[]):string[][]=>{return [["allocated".repeat(v%3+1)],[]];});}
    function partial(values:int[]):void{const base=clone(values);const selected=base.slice(0);selected.forEach((v:int,i:int,source:int[]):string[][]=>{return [["first".repeat(v%3+1)],[piece(i)]];});}
    function piece(i:int):string{const text="second".repeat(i%3+1);if(i===1){const bad=[1];bad.remove(-1);}return text;}
    function main(){}
    "#,
        "",
        r#"
    use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,Ordering};
    static ALLOCATIONS:AtomicIsize=AtomicIsize::new(0);static LIVE:AtomicIsize=AtomicIsize::new(0);static PEAK:AtomicIsize=AtomicIsize::new(0);struct Observer;
    unsafe impl GlobalAlloc for Observer{
      unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){ALLOCATIONS.fetch_add(1,Ordering::SeqCst);let live=LIVE.fetch_add(1,Ordering::SeqCst)+1;PEAK.fetch_max(live,Ordering::SeqCst);}p}
      unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe{System.dealloc(p,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
    }
    #[global_allocator]static OBSERVER:Observer=Observer;
    "#,
        r#"
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
    let values=NumericArray::Heap(vec![1i64;256]);let live=LIVE.load(Ordering::SeqCst);
    for action in [function0 as fn(&NumericArray<i64>),function1,function2,function3,function4]{PEAK.store(live,Ordering::SeqCst);action(&values);assert_eq!(LIVE.load(Ordering::SeqCst),live);assert!(PEAK.load(Ordering::SeqCst)<live+16,"returned owners retained across invocations");}
    assert!(std::panic::catch_unwind(||function5(&values)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let allocations=ALLOCATIONS.load(Ordering::SeqCst);function7(&values);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    assert!(ALLOCATIONS.load(Ordering::SeqCst)-allocations<=257,"discarded outer arrays must not allocate per invocation");
    assert!(std::panic::catch_unwind(||function8(&values)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let empty=NumericArray::<i64>::from([]);PEAK.store(live,Ordering::SeqCst);function6(&empty);assert_eq!(LIVE.load(Ordering::SeqCst),live);assert_eq!(PEAK.load(Ordering::SeqCst),live);
    drop(empty);drop(values);std::panic::set_hook(hook);
    "#,
    );
}
