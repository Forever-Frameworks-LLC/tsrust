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
    let evidence = std::env::var_os("TSR_GATE89_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-unshift.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-unshift-{level}"));
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
fn unshift_dense_families_ranks_sizes_and_expression_contexts() {
    native(
        r#"
        type Row={id:int;text:string;};type Node={id:int;children:Node[];};
        function prepend(a:int[]):int{return a.unshift(1,2,3);}
        function main(){
            for(const size of [0,1,4,5,17]){const a:int[]=[];for(let i=0;i<size;i++){a.push(i+10);}console.log(a.unshift());console.log(prepend(a));console.log(a.join("|"));}
            const n:number[]=[-0.0,0.0/0.0];console.log(n.unshift(2.5,-1.5));console.log(n.join("|"));
            const b=[true,false];console.log(b.unshift(false,true));console.log(b.join("|"));
            const s=["tail".repeat(1)];const head="é\0😀".repeat(1);console.log(s.unshift(head,"mid".repeat(1)));console.log(s.join("|"));
            const nested:int[][]=[[3]];console.log(nested.unshift([1,2],[]));console.log(nested.join("|"));
            const rows:Row[]=[{id:2,text:"b".repeat(1)}];console.log(rows.unshift({id:1,text:"a".repeat(1)}));console.log(rows[0].text);console.log(rows[1].text);
            const nodes:Node[]=[];console.log(nodes.unshift({id:1,children:[{id:2,children:[]}]}));console.log(nodes[0].children[0].id);
            const nils:null[]=[];console.log(nils.unshift(null,null));console.log(nils.join("|"));const absent:undefined[]=[];console.log(absent.unshift(undefined));console.log(absent.join());
            console.log([1].unshift(2,3));console.log([].unshift());
            let value:int=0;const a:int[]=[];value=a.unshift(1);const lengths=[a.unshift(2),true?a.unshift(3):a.unshift(9)];console.log(value);console.log(lengths.join());console.log(a.unshift(4)<<1);
            let count=0;while(a.unshift()<5){a.unshift(0);count++;}console.log(count);for(let i=0;i<2;i=a.unshift()){console.log(i);}console.log(a.length);
        }
    "#,
        "0\n3\n1|2|3\n1\n4\n1|2|3|10\n4\n7\n1|2|3|10|11|12|13\n5\n8\n1|2|3|10|11|12|13|14\n17\n20\n1|2|3|10|11|12|13|14|15|16|17|18|19|20|21|22|23|24|25|26\n4\n2.5|-1.5|0|NaN\n4\nfalse|true|true|false\n3\né\0😀|mid|tail\n3\n1,2||3\n2\na\nb\n1\n2\n2\n|\n1\n\n3\n0\n1\n2,3\n8\n1\n0\n5\n",
        "",
        "",
    );
}

#[test]
fn unshift_arguments_selection_effects_and_readonly_zero_are_ordered() {
    let analysis = native(
        r#"
        type Box={items:int[];note:string;};
        function grow(a:int[],n:int):int{console.log(n);a.push(n+100);return n;}
        function index(trace:int[]):int{trace.push(1);console.log("index");return 0;}
        function select(trace:int[]):int[]{trace.push(1);console.log("receiver");return [7];}
        function arg(trace:int[],n:int):int{trace.push(n);console.log(n);return n;}
        function readonly(a:int[]):int{return a.unshift();}
        function fixed():int{const values=[1,2,3];const other=[9,10];return values.unshift()+other[0];}
        function main(){
            const a=[10];console.log(a.unshift(grow(a,1),grow(a,2)));console.log(a.join("|"));console.log(a.unshift(a.remove(0)));console.log(a.join("|"));
            const trace:int[]=[];const rows=[[8]];console.log(rows[index(trace)].unshift(arg(trace,3),arg(trace,4)));console.log(rows[0].join());console.log(trace.join());
            console.log(select(trace).unshift(arg(trace,5),arg(trace,6)));console.log(trace.join());
            const x:int[]=[];console.log(x.unshift(x.unshift(1)));console.log(x.join());
            const box:Box|null={items:[2],note:"n"};if(box!==null){console.log(box.items.unshift(1));console.log(box.items.length);console.log(box.note);}
            const nested=[[1,2]];{const view=nested.at(0);if(view!==undefined){console.log(view.unshift());console.log(view.length);}}
            const read=(v:int[]):int=>{return v.unshift();};console.log(read(a));console.log(readonly(a));console.log(fixed());
        }
    "#,
        "1\n2\n5\n1|2|10|101|102\n5\n1|2|10|101|102\nindex\n3\n4\n3\n3,4,8\n1,3,4\nreceiver\n5\n6\n3\n1,3,4,1,5,6\n2\n1,1\n2\n2\nn\n2\n2\n5\n5\n12\n",
        "",
        "",
    );
    assert!(
        analysis.emit_rust().contains(": [i64; 3] = ["),
        "zero-argument call keeps fixed storage"
    );
    assert_eq!(analysis.contracts()["readonly.a"], Mode::Read);
}

#[test]
fn unshift_strict_types_loans_views_transfers_and_normal_depth_are_preserved() {
    for (source, code) in [
        ("function main(){const a=[1];a.unshift(true);}", "type"),
        ("function main(){const a=[1];a.unshift(1.5);}", "type"),
        (
            "function main(){const a:number[]=[1.0];let n:int=1;a.unshift(n);}",
            "type",
        ),
        (
            "type A={n:int;};type B={n:int;};function main(){const a:A[]=[];const b:B={n:1};a.unshift(b);}",
            "type",
        ),
        ("function main(){const s=\"a\";s.unshift();}", "type"),
        (
            "function main(){const a=[\"a\"];a.unshift(a[0]);}",
            "borrowed-element",
        ),
        (
            "function main(){const a=[\"a\"];const x=\"x\".repeat(1);a.unshift(x);console.log(x);}",
            "transferred",
        ),
        (
            "function main(){const a=[[1]];const v=a.at(0);if(v!==undefined){v.unshift(2);}}",
            "borrowed-view",
        ),
        (
            "function main(){const a=[\"a\"];const v=a.at(0);a.unshift(\"x\");console.log(String(v));}",
            "view-conflict",
        ),
        (
            "function read(a:int[],n:int):int{return a.length+n;}function main(){const a=[1];read(a,a.unshift(2));}",
            "conflicting-access",
        ),
        (
            "function grow(a:int[][]):int{a.push([1]);return 2;}function main(){const a=[[1]];a[0].unshift(grow(a));}",
            "conflicting-access",
        ),
    ] {
        let error = check(source).err().expect("invalid unshift must reject");
        assert_eq!(error.code, code, "{}", error.render("input.tsr", source));
    }
    for count in [100, 200] {
        let source = format!(
            "function main(){{const a={};a.unshift();}}",
            "[".repeat(count) + "1" + &"]".repeat(count)
        );
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            assert_eq!(check(&source).err().unwrap().code, "unsupported");
        }
        let source = format!(
            "function main(){{const a=[1];a[{}];}}",
            "a[".repeat(count) + "0" + &"]".repeat(count)
        );
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            assert_eq!(check(&source).err().unwrap().code, "unsupported");
        }
    }
    native(
        r#"
        function fail(a:int[]):int{const empty:int[]=[];return empty.remove(0);}
        function first(a:int[]):int{a.push(9);return 1;}
        function append(a:int[]):int{return a.unshift(first(a),fail(a));}
        function nested(a:int[][],trace:int[]):int{return a[-1].unshift(first(trace));}
        function main(){}
    "#,
        "",
        "",
        r#"
        std::panic::set_hook(Box::new(|_|{}));let mut a=NumericArray::from([7]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function2(&mut a))).is_err());assert_eq!(a.as_slice(),&[7,9]);
        let mut rows=Array::from([NumericArray::from([7])]);let mut trace=NumericArray::from([4]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut rows,&mut trace))).is_err());assert_eq!(rows.get(0).unwrap().as_slice(),&[7]);assert_eq!(trace.as_slice(),&[4]);
    "#,
    );
}

#[test]
fn unshift_moves_owner_pointers_preserves_bits_and_disposes_normal_unwind() {
    native(
        r#"
        type Node={id:int;text:string;children:Node[];};
        function scalar(a:int[],v:int):int{return a.unshift(v);}
        function numbers(a:number[],v:number):int{return a.unshift(v);}
        function strings(a:string[],v:string):int{return a.unshift(v);}
        function nodes(a:Node[],v:Node):int{return a.unshift(v);}
        function fail():string{const a:int[]=[];a.remove(0);return "never";}
        function unwind(a:string[],v:string):int{return a.unshift(v,fail());}
        function main(){const a:undefined[]=[];a.unshift();}
    "#,
        "",
        r#"        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;
        unsafe impl GlobalAlloc for Counts{
            unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
            unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}
            unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
        }
        #[global_allocator]static COUNTS:Counts=Counts;
"#,
        r#"
        std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for len in [0,1,4,5,17]{
            let mut a=NumericArray::from([]);for i in 0..len{a.push(i as i64);}assert_eq!(function0(&mut a,-7),len as i64+1);assert_eq!(a.get(0),Some(&-7));for i in 0..len{assert_eq!(a.get(i+1),Some(&(i as i64)));}drop(a);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            let mut n=(0..len).map(|i|f64::from_bits([0x8000000000000000,0x7ff8000000000042,0x7ff0000000000000,0xfff0000000000000][i%4])).collect::<Vec<_>>();let old=n.iter().map(|v|v.to_bits()).collect::<Vec<_>>();assert_eq!(function1(&mut n,f64::from_bits(0xfff8000000001234)),len as i64+1);assert_eq!(n[0].to_bits(),0xfff8000000001234);assert_eq!(n[1..].iter().map(|v|v.to_bits()).collect::<Vec<_>>(),old);drop(n);drop(old);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            let mut small=NumericArray::from([]);for i in 0..len{small.push(f64::from_bits([0x8000000000000000,0x7ff8000000000042][i%2]));}let old=small.iter().map(|v|v.to_bits()).collect::<Vec<_>>();small.tsr_unshift([f64::from_bits(0xfff8000000001234)],"length");assert_eq!(small.get(0).unwrap().to_bits(),0xfff8000000001234);assert_eq!(small.iter().skip(1).map(|v|v.to_bits()).collect::<Vec<_>>(),old);drop(small);drop(old);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            let mut strings=Array::from([]);for _ in 0..len{strings.push(String::Owned("oldé😀".into()));}let pointers=strings.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();let added=String::Owned("new\0é".into());let pointer=added.as_ptr();let allocations=ALLOCS.load(Ordering::SeqCst);assert_eq!(function2(&mut strings,added),len as i64+1);assert_eq!(strings.get(0).unwrap().as_ptr(),pointer);assert!(strings.iter().skip(1).zip(&pointers).all(|(v,p)|v.as_ptr()==*p));if len<4{assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);}drop(strings);drop(pointers);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        let mut strings=Array::from([String::Owned("kept".into())]);let pointer=strings.get(0).unwrap().as_ptr();let state=LIVE.load(Ordering::SeqCst);let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function5(&mut strings,String::Owned("discarded".into()))));assert!(result.is_err());drop(result);assert_eq!(strings.len(),1);assert_eq!(strings.get(0).unwrap().as_ptr(),pointer);assert_eq!(LIVE.load(Ordering::SeqCst),state);drop(strings);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let leaf=Record0{field0:2,field1:String::Owned("leaf".into()),field2:RecordArray0::from([])};let node=Record0{field0:1,field1:String::Owned("root".into()),field2:RecordArray0::from([leaf])};let pointer=node.field1.as_ptr();let mut nodes=RecordArray0::from([]);assert_eq!(function3(&mut nodes,node),1);assert_eq!(nodes[0].field1.as_ptr(),pointer);drop(nodes);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        if usize::BITS==64{let mut absent=vec![(); i64::MAX as usize];let len=absent.len();let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||absent.tsr_unshift([()],"limit")));assert!(result.is_err());drop(result);assert_eq!(absent.len(),len);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);drop(absent);}
    "#,
    );
}
