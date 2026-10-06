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
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let evidence = std::env::var_os("TSR_GATE86_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-pop-shift.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-pop-shift-{level}"));
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
fn pop_shift_all_dense_leaves_nested_records_and_empty_results() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for method in ["pop", "shift"] {
        for leaf in [
            "int",
            "number",
            "boolean",
            "string",
            "nested",
            "record",
            "null",
            "undefined",
        ] {
            let source = std::fs::read_to_string(root.join(format!(
                "benchmarks/runtime/workloads/array_{method}_{leaf}.tsr"
            )))
            .unwrap();
            check(&source)
                .unwrap_or_else(|error| panic!("{}", error.render("workload.tsr", &source)))
                .emit_rust();
        }
    }
    native(
        &std::fs::read_to_string(root.join("benchmarks/runtime/array-removal-source-order.tsr"))
            .unwrap(),
        &std::fs::read_to_string(
            root.join("benchmarks/runtime/array-removal-source-order-expected.txt"),
        )
        .unwrap(),
        "",
        "",
    );

    for (method, first, last, number, boolean, text, nested, record, node) in [
        ("pop", "-9", "7", "0", "false", "😀", "3", "two", "2"),
        ("shift", "7", "-9", "NaN", "true", "é\0", "1,2", "one", "1"),
    ] {
        let source = format!(
            r#"
            type Row={{text:string;values:int[];}};type Node={{tag:int;children:Node[];}};
            function main(){{
                const ints=[7,-9];console.log(String(ints.{method}()));console.log(ints.join());console.log(String(ints.{method}()));console.log(String(ints.{method}()));console.log(ints.length);
                const numbers:number[]=[Number.NaN,-0.0];console.log(String(numbers.{method}()));
                const booleans=[true,false];console.log(String(booleans.{method}()));
                const words=["é\0".repeat(1),"😀".repeat(1)];const text=words.{method}();if(text!==undefined){{console.log(text.concat("!"));}}console.log(words.length);
                const rows=[[1,2],[3]];const row=rows.{method}();if(row!==undefined){{console.log(row.join());}}console.log(rows.length);
                const records:Row[]=[{{text:"one".repeat(1),values:[1]}},{{text:"two".repeat(1),values:[2]}}];const record=records.{method}();if(record!==undefined){{console.log(record.text.concat("!"));console.log(record.values.length);}}console.log(records.length);
                const nodes:Node[]=[{{tag:1,children:[]}},{{tag:2,children:[{{tag:3,children:[]}}]}}];const tree=nodes.{method}();if(tree!==undefined){{console.log(tree.tag);}}console.log(nodes.length);
                const nil:null[]=[null];console.log(String(nil.{method}()));console.log(String(nil.{method}()));
                const absent:undefined[]=[undefined];console.log(String(absent.{method}()));console.log(String(absent.{method}()));
                const empty:string[]=[];console.log(String(empty.{method}()));console.log(String([].{method}()));
                const deep=[[[[4,5]]]];const removed=deep.{method}();if(removed!==undefined){{console.log(removed.join("|"));}}console.log(deep.length);
            }}
        "#
        );
        let expected = format!(
            "{first}\n{last}\n{last}\nundefined\n0\n{number}\n{boolean}\n{text}!\n1\n{nested}\n1\n{record}!\n1\n1\n{node}\n1\nnull\nundefined\nundefined\nundefined\nundefined\nundefined\n4,5\n0\n"
        );
        native(&source, &expected, "", "");
    }
}

#[test]
fn pop_shift_receiver_order_captures_nullable_fields_and_storage_contracts() {
    let analysis = native(
        r#"
        type Count={n:int;};type Holder={values:int[]|null;other:string|null;};
        function take(values:int[]):int|undefined{return values.pop();}
        function first(values:int[]):int|undefined{return values.shift();}
        function mark(c:Count):int{console.log("index");c.n+=1;return 0;}
        function fresh(c:Count):string[]{console.log("receiver");c.n+=1;return ["fresh".repeat(1)];}
        function storage():int{const shortened=[1,2];shortened.pop();const fixed=[21,22];return fixed[0]+fixed[1]+shortened.length;}
        function main(){
            const counter:Count={n:0};const rows=[[1,2],[3]];console.log(String(rows[mark(counter)].pop()));console.log(counter.n);console.log(rows[0].join());
            console.log(String(fresh(counter).shift()));console.log(counter.n);
            const values=[4,5];{const next=():int|undefined=>{return values.pop();};console.log(String(next()));console.log(String(next()));}console.log(values.length);
            const a=[6,7];const b=[8,9];const flag=true;console.log(String((flag?a:b).shift()));console.log(a.join());console.log(b.join());
            const holder:Holder={values:[10,11],other:"kept".repeat(1)};if(holder.values!==null&&holder.other!==null){console.log(String(holder.values.pop()));console.log(holder.values.length);console.log(holder.other.concat("!"));}
            const guarded:Holder|null={values:[12],other:"side"};if(guarded!==null){if(guarded.values!==null){console.log(String(guarded.values.shift()));console.log(guarded.values.length);}console.log(guarded.other===null);}
            const partial:Holder={values:[13],other:"moved"};const moved=partial.other;if(partial.values!==null){console.log(String(partial.values.pop()));}console.log(String(moved));
            const untouched=[21,22];console.log(untouched[0]+untouched[1]);
            console.log(String(false?[99].pop():[14].shift()));console.log(storage());
        }
    "#,
        "index\n2\n1\n1\nreceiver\nfresh\n2\n5\n4\n0\n6\n7\n8,9\n11\n1\nkept!\n12\n0\nfalse\n13\nmoved\n43\n14\n44\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("take.values"), Some(&Mode::Mutate));
    assert_eq!(
        analysis.contracts().get("first.values"),
        Some(&Mode::Mutate)
    );
    let emitted = analysis.emit_rust();
    assert!(emitted.contains(": [i64; 2] = [21i64, 22i64]"));
    assert!(emitted.contains("NumericArray::from([4i64, 5i64])"));
}

#[test]
fn pop_shift_strict_types_live_loans_iteration_and_normal_depth_diagnostics() {
    for (source, code) in [
        ("function main(){const a=[1];a.pop(1);}", "arity"),
        ("function main(){const a=[1];a.shift(\"x\");}", "arity"),
        ("function main(){\"x\".pop();}", "type"),
        (
            "function main(){const a:int[]|null=null;a.shift();}",
            "type",
        ),
        (
            "function main(){const a=[\"x\"];console.log(a.pop().length);}",
            "type",
        ),
        (
            "function read(v:int[],n:int|undefined):int{return v.length;}function main(){const a=[1];read(a,a.pop());}",
            "conflicting-access",
        ),
        ("function main(){const a=[1];a;a.shift();}", "transferred"),
        (
            "function main(){const rows=[[1],[2]];for(const row of rows){row.pop();}}",
            "borrowed-element",
        ),
        (
            "function main(){const a=[1];for(const n of a){a.shift();}}",
            "iteration-conflict",
        ),
        (
            "function main(){const a=[1];const read=():int=>{return a.length;};a.pop();read();}",
            "capture-conflict",
        ),
        (
            "function main(){const a:(int|void)[]=[];a.pop();}",
            "type",
        ),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
    }
    // Mixed literals infer a union element type; shift is accepted there.
    assert!(check("function main(){const a=[1,\"x\"];a.shift();}").is_ok());
    let valid = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
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
fn pop_shift_exact_bits_pointer_moves_no_allocation_and_recursive_disposal() {
    native(
        r#"
        type Node={tag:int;text:string;children:Node[];};
        function last(v:number[]):number|undefined{return v.pop();}
        function first(v:number[]):number|undefined{return v.shift();}
        function lastText(v:string[]):string|undefined{return v.pop();}
        function firstText(v:string[]):string|undefined{return v.shift();}
        function firstNode(v:Node[]):Node|undefined{return v.shift();}
        function fail():int{const words=["one".repeat(1),"two".repeat(1)];const removed=words.pop();if(removed!==undefined){removed.concat("!");const bad=1/0;return bad;}return 0;}
        function integers(v:int[]):int|undefined{return v.pop();}
        function main(){}
    "#,
        "",
        ALLOCATION_OBSERVER,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for bits in [0u64,1u64<<63,1,0x8000000000000001,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000042,0x4004000000000000]{
            let value=f64::from_bits(bits);let mut values=vec![std::hint::black_box(value),value];ALLOCS.store(0,Ordering::SeqCst);
            let last=function0(&mut values);assert_eq!(match last{Union0::V0(value)=>value.to_bits(),_=>panic!("present")},bits);
            let first=function1(&mut values);assert_eq!(match first{Union0::V0(value)=>value.to_bits(),_=>panic!("present")},bits);
            assert!(matches!(function0(&mut values),Union0::V1(TsrUndefined)));assert!(matches!(function1(&mut values),Union0::V1(TsrUndefined)));
            assert_eq!(ALLOCS.load(Ordering::SeqCst),0);assert!(values.is_empty());drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        for size in [0usize,1,4,5,17]{for shift in [false,true]{
            let mut values=Array::from([]);for n in 0..size{values.push(String::Owned(format!("value{n}é😀")));}
            let expected=if size==0{None}else{Some(values.get(if shift{0}else{size-1}).unwrap().as_ptr())};ALLOCS.store(0,Ordering::SeqCst);
            let removed=if shift{function3(&mut values)}else{function2(&mut values)};assert_eq!(ALLOCS.load(Ordering::SeqCst),0);
            match (&removed,expected){(Union1::V0(text),Some(pointer))=>assert_eq!(text.as_ptr(),pointer),(Union1::V1(TsrUndefined),None)=>(),_=>panic!("result")};
            assert_eq!(values.len(),size.saturating_sub(1));drop(values);if let Union1::V0(text)=&removed{assert!(text.contains("é😀"));}drop(removed);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }}
        for size in [0usize,1,4,5,17]{for shift in [false,true]{let mut values=NumericArray::from([]);for n in 0..size{values.push(n as i64);}ALLOCS.store(0,Ordering::SeqCst);let result=if shift{if values.len()==0{None}else{Some(values.remove(0))}}else{values.pop()};assert_eq!(result,if size==0{None}else{Some(if shift{0}else{size as i64-1})});assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(values);}}

        for size in [0usize,1,4,5,17]{for shift in [false,true]{
            let bits=[0u64,1u64<<63,1,0x8000000000000001,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000042];
            let mut values=NumericArray::from([]);for n in 0..size{values.push(f64::from_bits(bits[n%bits.len()]));}
            ALLOCS.store(0,Ordering::SeqCst);
            for removed in 0..size{let expected=bits[if shift{removed%bits.len()}else{(size-1-removed)%bits.len()}];let result=if shift{Some(values.remove(0))}else{values.pop()};assert_eq!(result.unwrap().to_bits(),expected);}
            assert!(values.pop().is_none());assert_eq!(values.len(),0);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }}
        let mut node=Record0{field0:0,field1:String::Owned("root".to_owned()),field2:RecordArray0::from([])};
        for depth in 0..2000{node=Record0{field0:depth,field1:String::Owned("node".to_owned()),field2:RecordArray0(vec![node])};}
        let pointer=node.field1.as_ptr();let mut nodes=RecordArray0(vec![node]);ALLOCS.store(0,Ordering::SeqCst);let removed=function4(&mut nodes);
        assert_eq!(ALLOCS.load(Ordering::SeqCst),0);assert_eq!(match &removed{Union2::V0(node)=>node.field1.as_ptr(),_=>panic!("present")},pointer);assert_eq!(nodes.len(),0);drop(nodes);drop(removed);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let panic=std::panic::catch_unwind(function5).err().expect("panic");drop(panic);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
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
