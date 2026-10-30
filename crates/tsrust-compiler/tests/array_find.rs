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
    let evidence = std::env::var_os("TSR_GATE88_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-find.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-find-{level}"));
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
fn find_values_cover_dense_families_ranks_empty_and_all_contexts() {
    native(
        r#"
        type Row={n:int;text:string;};type Node={n:int;children:Node[];};
        function scalar(a:int[]):int|undefined{return a.find((v:int):boolean=>{return v>1;});}
        function main(){
            const ints=[1,2,3,4,5];console.log(String(ints.find((v:int):boolean=>{return v>1;})));console.log(String(ints.findLast((v:int):boolean=>{return v>1;})));console.log(String(scalar(ints)));
            const numbers:number[]=[-0.0,2.5,0.0/0.0,1.0/0.0,-1.0/0.0];console.log(String(numbers.find((v:number):boolean=>{return v===0.0;})));console.log(String(numbers.findLast((v:number):boolean=>{return v<0.0;})));
            const booleans=[false,true,false];console.log(String(booleans.find((v:boolean):boolean=>{return v;})));console.log(String(booleans.findLast((v:boolean):boolean=>{return !v;})));
            const words=["é\0".repeat(1),"😀".repeat(1),"last".repeat(1)];{const first=words.find((v:string):boolean=>{return v.length>0;});const last=words.findLast((v:string):boolean=>{return v.length>0;});console.log(String(first));console.log(String(last));if(first!==undefined){console.log(first.concat("!"));}console.log(words.length);}
            const nested=[[1,2],[],[3]];{const first=nested.find((v:int[]):boolean=>{return v.length>0;});const last=nested.findLast((v:int[]):boolean=>{return v.length>0;});if(first!==undefined){console.log(first.join("|"));}if(last!==undefined){console.log(last.join("|"));}}
            const rows:Row[]=[{n:1,text:"a".repeat(1)},{n:2,text:"b".repeat(1)}];{const found=rows.findLast((v:Row):boolean=>{return v.n>0;});if(found!==undefined){console.log(found.n);console.log(found.text.concat("!"));}}
            const nodes:Node[]=[{n:4,children:[{n:5,children:[]}]}];{const found=nodes.find((v:Node):boolean=>{return v.children.length===1;});if(found!==undefined){console.log(found.n);console.log(found.children.length);}}
            const deep=[[[[7]]]];{const found=deep.findLast((v:int[][][]):boolean=>{return v.length>0;});if(found!==undefined){console.log(found.join());}}
            const nils=[null,null];console.log(typeof nils.find(():boolean=>{return true;}));console.log(typeof nils.findLast(():boolean=>{return false;}));
            const missing=[undefined,undefined];console.log(typeof missing.find(():boolean=>{return true;}));console.log(typeof missing.findLast(():boolean=>{return false;}));
            const empty:string[]=[];console.log(String(empty.find(():boolean=>{return true;})));console.log(String(empty.findLast(():boolean=>{return true;})));console.log(String([].find(():boolean=>{return true;})));
            const fresh=["owned".repeat(1)].findLast((v:string):boolean=>{return v.length>0;});if(fresh!==undefined){console.log(fresh.concat("!"));}
            const a=["a"];const b=["bb","c"];{const found=(false?a:b).find((v:string):boolean=>{return v.length===2;});console.log(String(found));}console.log(a.length+b.length);
            console.log(false?"skip":String(ints.find(():boolean=>{return true;})));const values=[ints.find(():boolean=>{return true;})!==undefined,ints.findLast(():boolean=>{return true;})!==undefined];
        }
    "#,
        "2\n5\n2\n0\n-Infinity\ntrue\nfalse\né\0\nlast\né\0!\n3\n1|2\n3\n2\nb!\n4\n1\n7\nobject\nundefined\nundefined\nundefined\nundefined\nundefined\nundefined\nowned!\nbb\n3\n1\n",
        "",
        "",
    );
}

#[test]
fn find_callback_order_short_circuit_views_and_fixed_storage_are_exact() {
    let analysis = native(
        r#"
        function values():int[]{console.log("receiver");return [10,20,30,40,50];}
        function empty():int[]{console.log("empty");return [];}
        function callback():()=>boolean{console.log("callback");return ():boolean=>{console.log("visit");return true;};}
        function choose(values:int[],action:Mutable<Function<boolean, [v:int,i:int,a:ReadOnly<int[]>]>>):int|undefined{return values.findLast(action);}
        function text(value:string|null|undefined):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
        function positive(v:int):boolean{return v>0;}
        function fixed():int{const source=[1,2,3];const unrelated=[21,22];return source.find(positive)!==undefined?unrelated[0]+unrelated[1]:0;}
        function main(){
            console.log(String(values().find(callback())));console.log(String(values().findLast(callback())));console.log(String(empty().find(callback())));console.log(String(empty().findLast(callback())));
            const source=[1,2,3,4,5];let trace=0;console.log(String(source.find((v:int,i:int,a:int[]):boolean=>{trace=trace*10+i+1;return i===2&&a.length===5;})));console.log(trace);trace=0;
            console.log(String(source.findLast((v:int,i:int):boolean=>{trace=trace*10+i+1;return i===3;})));console.log(trace);trace=0;
            console.log(String(source.findLast((v:int,i:int):boolean=>{trace=trace*10+i+1;return false;})));console.log(trace);
            console.log(String(choose(source,(v:int,i:int,a:int[]):boolean=>{return i===1&&a.length===5;})));
            const words=["a".repeat(1),"long".repeat(1)];{const view=words.findLast((v:string):boolean=>{return true;});const alias=true?view:null;console.log(text(alias));console.log(String(alias));const read=():int=>{if(view!==undefined){return view.length;}return -1;};console.log(read());}
            words.push("released");console.log(words.length);const owned=clone(words.find((v:string):boolean=>{return true;}));words.push("again");console.log(String(owned));console.log(words.length);console.log(fixed());
            const skip=[1,2];console.log(String(skip.find((v:int,i:int):boolean=>{if(i>0){const a=[1];a.remove(-1);}return true;})));console.log(String(skip.findLast((v:int,i:int):boolean=>{if(i<1){const a=[1];a.remove(-1);}return true;})));
        }
    "#,
        "receiver\ncallback\nvisit\n10\nreceiver\ncallback\nvisit\n50\nempty\ncallback\nundefined\nempty\ncallback\nundefined\n3\n123\n4\n54\nundefined\n54321\n2\n4\nlong\n4\n3\na\n4\n43\n1\n2\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("choose.values"), Some(&Mode::Read));
    assert_eq!(
        analysis.contracts().get("choose.action"),
        Some(&Mode::Mutate)
    );
    assert!(analysis.emit_rust().contains(": [i64; 2] = [21i64, 22i64]"));
}

#[test]
fn find_diagnostics_preserve_borrowed_views_callback_contracts_and_depth() {
    for method in ["find", "findLast"] {
        let source = format!(
            "function main(){{const a=[1];a.{method}(():boolean=>{{const invalid=[];return true;}});}}"
        );
        assert_eq!(
            check(&source)
                .err()
                .expect("unannotated empty callback local remains unsupported")
                .code,
            "array-type"
        );
        check(&format!(
            "function main(){{[].{method}(():boolean=>{{return true;}});}}"
        ))
        .unwrap()
        .emit_rust();
    }
    if std::env::var_os("TSR_GATE88_DIAGNOSTIC_SUPPLEMENT_ONLY").is_some() {
        return;
    }
    for method in ["find", "findLast"] {
        for (body, code) in [
            (format!("[1].{method}();"), "arity"),
            (
                format!("[1].{method}(():boolean=>{{return true;}},0);"),
                "arity",
            ),
            (format!("[1].{method}((v:int):int=>{{return v;}});"), "type"),
            (
                format!("const a=[1];a.{method}((v:number):boolean=>{{return true;}});"),
                "type",
            ),
            (
                format!("[1].{method}((v:int,i:int,a:int[],extra:int):boolean=>{{return true;}});"),
                "arity",
            ),
            (
                format!("const a=[\"x\"];a.{method}((v:string):boolean=>{{v;return true;}});"),
                "callback-access",
            ),
            (
                format!(
                    "const a=[1];a.{method}((v:int,i:int,all:int[]):boolean=>{{all.push(2);return true;}});"
                ),
                "callback-access",
            ),
            (
                format!(
                    "const a=[\"x\"];const v=a.{method}(():boolean=>{{return true;}});a.push(\"y\");console.log(String(v));"
                ),
                "view-conflict",
            ),
            (
                format!(
                    "const a=[[1]];const v=a.{method}(():boolean=>{{return true;}});if(v!==undefined){{v.push(2);}}"
                ),
                "borrowed-view",
            ),
            (
                format!(
                    "const a=[\"x\"];const v=a.{method}(():boolean=>{{return true;}});const f=():string|undefined=>{{return v;}};f();"
                ),
                "borrowed-view",
            ),
        ] {
            let source = format!("function main(){{{body}}}");
            let e = check(&source).err().expect("invalid find must reject");
            assert_eq!(e.code, code, "{}", e.render("input.tsr", &source));
        }
        let source = format!(
            "function bad(a:string[]):string|undefined{{return a.{method}(():boolean=>{{return true;}});}}function main(){{}}"
        );
        assert_eq!(check(&source).err().unwrap().code, "borrowed-view");
        let source = format!(
            "function once(a:int[],f:Once<Function<boolean>>):int|undefined{{return a.{method}(f);}}function main(){{}}"
        );
        assert_eq!(check(&source).err().unwrap().code, "callback-access");
        let source = format!(
            "function read(v:string|undefined,n:int):int{{return n;}}function grow(a:string[]):int{{a.push(\"x\");return 0;}}function main(){{const a=[\"a\"];read(a.{method}(():boolean=>{{return true;}}),grow(a));}}"
        );
        assert_eq!(check(&source).err().unwrap().code, "conflicting-access");
    }
    for count in [100, 200] {
        let source = format!(
            "function main(){{const a={};console.log(a.length);}}",
            "[".repeat(count) + "1" + &"]".repeat(count)
        );
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            assert_eq!(check(&source).err().unwrap().code, "unsupported");
        }
        let expression = format!("a[{}]", "a[".repeat(count) + "0" + &"]".repeat(count));
        let source = format!("function main(){{const a=[1];{expression};}}");
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            support::accept_deep(&source);
        }
    }
}

#[test]
fn find_exact_bits_owner_pointers_allocation_and_unwind_disposal() {
    native(
        r#"
        type Node={tag:int;text:string;children:Node[];};
        function first(v:number[]):number|undefined{return v.find(():boolean=>{return true;});}
        function last(v:number[]):number|undefined{return v.findLast(():boolean=>{return true;});}
        function peek(v:string[]):int{const selected=v.find((text:string):boolean=>{return text.length>0;});if(selected!==undefined){return selected.length;}return -1;}
        function peekLast(v:string[]):int{const selected=v.findLast((text:string):boolean=>{return text.length>0;});if(selected!==undefined){return selected.length;}return -1;}
        function owner(text:string,matched:boolean):string|undefined{return [text].find(():boolean=>{return matched;});}
        function fail(text:string):string|undefined{return [text].findLast((v:string):boolean=>{const a=[1];return a.remove(-1)>0;});}
        function forest(v:Node[]):int{const selected=v.findLast((n:Node):boolean=>{return n.children.length===1;});if(selected!==undefined){return selected.tag;}return -1;}
        function main(){}
    "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;
        unsafe impl GlobalAlloc for Counts{
            unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
            unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}
            unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
        }
        #[global_allocator]static COUNTS:Counts=Counts;
    "#,
        r#"
        for len in [0,1,4,5,17]{let values=(0..len).map(|i|f64::from_bits([0x8000000000000000,0,0x7ff8000000000042,0x7ff0000000000000,0xfff0000000000000][i%5])).collect::<Vec<_>>();
            for (result,want) in [(function0(&values),values.first()),(function1(&values),values.last())]{match result{Union0::V0(value)=>assert_eq!(value.to_bits(),want.unwrap().to_bits()),Union0::V1(_)=>assert!(want.is_none())}}
        }
        std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for len in [0,1,4,5,17]{let mut values=Array::from([]);for _ in 0..len{values.push(String::Owned("é😀".to_owned()));}let original=values.get(0).map(|v|v.as_ptr());let allocations=ALLOCS.load(Ordering::SeqCst);
            assert_eq!(function2(&values),if len==0{-1}else{6});assert_eq!(function3(&values),if len==0{-1}else{6});assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);assert_eq!(values.get(0).map(|v|v.as_ptr()),original);drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        let text=String::Owned("é😀".to_owned());let pointer=text.as_ptr();let allocations=ALLOCS.load(Ordering::SeqCst);match function4(text,true){Union1::V0(value)=>{assert_eq!(value.as_ptr(),pointer);assert_eq!(value,"é😀");},_=>panic!("present")};assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        drop(function4(String::Owned("absent".to_owned()),false));assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let failure=std::panic::catch_unwind(||function5(String::Owned("unwind".to_owned())));assert!(failure.is_err());drop(failure);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let leaf=Record0{field0:2,field1:String::Owned("leaf".into()),field2:RecordArray0::from([])};let nodes=RecordArray0::from([Record0{field0:1,field1:String::Owned("root".into()),field2:RecordArray0::from([leaf])}]);let allocations=ALLOCS.load(Ordering::SeqCst);assert_eq!(function6(&nodes),1);assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);drop(nodes);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#,
    );
}
