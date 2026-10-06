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
    function main(){}
    "#,
        "",
        r#"
    use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,Ordering};
    static LIVE:AtomicIsize=AtomicIsize::new(0);static PEAK:AtomicIsize=AtomicIsize::new(0);struct Observer;
    unsafe impl GlobalAlloc for Observer{
      unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){let live=LIVE.fetch_add(1,Ordering::SeqCst)+1;PEAK.fetch_max(live,Ordering::SeqCst);}p}
      unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe{System.dealloc(p,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
    }
    #[global_allocator]static OBSERVER:Observer=Observer;
    "#,
        r#"
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
    let values=NumericArray::Heap(vec![1i64;256]);let live=LIVE.load(Ordering::SeqCst);
    for action in [function0 as fn(&NumericArray<i64>),function1,function2,function3,function4]{PEAK.store(live,Ordering::SeqCst);action(&values);assert_eq!(LIVE.load(Ordering::SeqCst),live);assert!(PEAK.load(Ordering::SeqCst)<live+16,"returned owners retained across invocations");}
    assert!(std::panic::catch_unwind(||function5(&values)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let empty=NumericArray::<i64>::from([]);PEAK.store(live,Ordering::SeqCst);function6(&empty);assert_eq!(LIVE.load(Ordering::SeqCst),live);assert_eq!(PEAK.load(Ordering::SeqCst),live);
    drop(empty);drop(values);std::panic::set_hook(hook);
    "#,
    );
}
