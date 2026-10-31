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
    let evidence = std::env::var_os("TSR_GATE91_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-sort.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-sort-{level}"));
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
fn sorting_defaults_stability_dense_types_and_exact_number_bits() {
    native(
        r#"
        type Row={key:int;label:string;};
        function ints(a:int[]):void{a.sort();}
        function flags(a:boolean[]):void{a.sort();}
        function numbers(a:number[]):void{a.sort();}
        function same(a:number[]):void{a.sort((x:number,y:number):number=>{return 0.0/0.0;});}
        function main(){
            const a=[10,-2,3,1];a.sort();console.log(a.join());console.log(a.toSorted((x:int,y:int):int=>{return y-x;}).join());console.log(a.join());
            const flags=[true,false,true,false];flags.sort();console.log(flags.join());
            const words=["😀","\u{e000}","é","a"];words.sort();console.log(words.join("|"));console.log(words.toSorted().join("|"));
            const rows:Row[]=[{key:2,label:"a"},{key:1,label:"b"},{key:2,label:"c"},{key:1,label:"d"}];rows.sort((x:Row,y:Row):int=>{return x.key-y.key;});console.log(rows[0].label.concat(rows[1].label,rows[2].label,rows[3].label));
            const nested:int[][]=[[3,4],[1],[2,5],[]];nested.sort((x:int[],y:int[]):int=>{return x.length-y.length;});console.log(nested.join("|"));console.log(nested.toSorted((x:int[],y:int[]):number=>{return Number(y.length-x.length);}).join("|"));
            const nils=[null,null];nils.sort();console.log(nils.toSorted().length);const missing=[undefined,undefined];missing.sort();console.log(missing.toSorted().length);
            const ties=[3,1,2];ties.sort(():int=>{return 0;});console.log(ties.join());ties.sort((x:int):int=>{return 0;});console.log(ties.join());
            console.log([].toSorted().length);const empty:string[]=[].toSorted();console.log(empty.length);console.log([].toSorted((x:string,y:string):int=>{return x.length-y.length;}).length);[3,2,1].sort();
        }
    "#,
        "-2,1,3,10\n10,3,1,-2\n-2,1,3,10\nfalse,false,true,true\na|é|\u{e000}|😀\na|é|\u{e000}|😀\nbdac\n|1|3,4|2,5\n3,4|2,5|1|\n2\n2\n3,1,2\n3,1,2\n0\n0\n0\n",
        "",
        r#"
        for len in [0usize,1,4,5,17]{let mut a=NumericArray::from([]);for i in (0..len).rev(){a.push(i as i64);}function0(&mut a);assert_eq!(a.as_slice(),(0..len).map(|i|i as i64).collect::<Vec<_>>());let mut flags=NumericArray::from([]);for i in 0..len{flags.push(i%2==0);}function1(&mut flags);assert!(flags.as_slice().windows(2).all(|w|w[0]<=w[1]));}
        let raw=[0x7ff8000000000042u64,0x8000000000000000,0x7ff0000000000000,0xfff8000000000042,0,0xbff0000000000000,0xfff0000000000000,0x3ff0000000000000,0x7ff8000000000001];let mut a=raw.map(f64::from_bits).to_vec();function2(&mut a);assert_eq!(a.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),[0xfff8000000000042,0xfff0000000000000,0xbff0000000000000,0x8000000000000000,0,0x3ff0000000000000,0x7ff0000000000000,0x7ff8000000000001,0x7ff8000000000042]);let mut a=raw.map(f64::from_bits).to_vec();function3(&mut a);assert_eq!(a.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),raw);
    "#,
    );
}

#[test]
fn sorting_receiver_callback_order_read_contracts_and_scoped_effects() {
    let analysis = native(
        r#"
        type Holder={items:int[];note:string;};
        function values():int[]{console.log("receiver");return [3,1,2];}
        function comparator():(a:int,b:int)=>int{console.log("callback");return (a:int,b:int):int=>{return a-b;};}
        function empty():int[]{console.log("empty");return [];}
        function copy(a:int[],cmp:Mutable<Function<int, [a:int,b:int]>>):int[]{return a.toSorted(cmp);}
        function inplace(a:int[],cmp:Mutable<Function<int, [a:int,b:int]>>):void{a.sort(cmp);}
        function index(trace:int[]):int{trace.push(1);console.log("index");return 0;}
        function main(){
            values().sort(comparator());console.log(values().toSorted(comparator()).join());empty().sort(comparator());console.log(empty().toSorted(comparator()).length);
            const a=[3,1,2];let calls=0;{const cmp=(x:int,y:int):int=>{calls++;return x-y;};inplace(a,cmp);}console.log(a.join());console.log(calls>0);
            const copied=a.toSorted((x:int,y:int):int=>{return a.length*(y-x);});console.log(copied.join());console.log(a.join());
            const rows=[[3,2,1]];const trace:int[]=[];rows[index(trace)].sort();console.log(rows[0].join());console.log(trace.length);
            let box:Holder|null={items:[3,2,1],note:"kept"};if(box!==null){box.items.sort();console.log(box.items.join());console.log(box.note);}
            const owned=[[3,1,2]];const view=owned.at(0);if(view!==undefined){console.log(view.toSorted().join());console.log(view.join());}
            console.log(true?[3,2,1].toSorted().join():"skip");console.log(false?"skip":[4,2].toSorted().join());
        }
    "#,
        "receiver\ncallback\nreceiver\ncallback\n1,2,3\nempty\ncallback\nempty\ncallback\n0\n1,2,3\ntrue\n3,2,1\n1,2,3\nindex\n1,2,3\n1\n1,2,3\nkept\n1,2,3\n3,1,2\n1,2,3\n2,4\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("copy.a"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("inplace.a"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("copy.cmp"), Some(&Mode::Mutate));
    let analysis =
        check("function main(){const sorted=[10,2,1];sorted.sort();console.log(sorted.join());}")
            .unwrap();
    let emitted = analysis.emit_rust();
    assert!(emitted.contains(": [i64; 3] = ["));
    assert!(!emitted.contains("sort_by_cached_key"));
}

#[test]
fn sorting_preserves_owner_pointers_copy_independence_and_unwind_disposal() {
    native(
        r#"
        type Node={id:int;text:string;children:Node[];};
        function words(a:string[]):void{a.sort((x:string,y:string):int=>{return x.length-y.length;});}
        function copied(a:string[]):string[]{return a.toSorted();}
        function nodes(a:Node[]):void{a.sort((x:Node,y:Node):int=>{return x.id-y.id;});}
        function copiedNodes(a:Node[]):Node[]{return a.toSorted((x:Node,y:Node):int=>{return x.id-y.id;});}
        function fail(a:string[],boom:int[]):void{a.sort((x:string,y:string):int=>{return boom[99];});}
        function copyFail(a:string[],boom:int[]):string[]{return a.toSorted((x:string,y:string):int=>{return boom[99];});}
        function main(){const a=[3,1,2];a.sort();}
    "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;
        unsafe impl GlobalAlloc for Counts{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}}
        #[global_allocator]static COUNTS:Counts=Counts;
    "#,
        r#"
        std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for len in [0usize,1,4,5,17]{let mut a=Array::from([]);for i in (0..len).rev(){a.push(String::Owned("x".repeat(i+1)));}let mut pointers=a.iter().map(|v|v.as_ptr() as usize).collect::<Vec<_>>();let allocations=ALLOCS.load(Ordering::SeqCst);function0(&mut a);if len<=4{assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);}assert!(a.iter().map(|v|v.len()).collect::<Vec<_>>().windows(2).all(|w|w[0]<=w[1]));let mut after=a.iter().map(|v|v.as_ptr() as usize).collect::<Vec<_>>();pointers.sort();after.sort();assert_eq!(pointers,after);let copy=function1(&a);assert_eq!(copy.len(),a.len());for value in copy.iter(){assert!(!a.iter().any(|original|original.as_ptr()==value.as_ptr()));}drop(a);drop(copy);drop(pointers);drop(after);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let leaf=Record0{field0:9,field1:String::Owned("leaf".into()),field2:RecordArray0::from([])};let parent=Record0{field0:2,field1:String::Owned("parent".into()),field2:RecordArray0::from([leaf])};let first=Record0{field0:1,field1:String::Owned("first".into()),field2:RecordArray0::from([])};let p=parent.field1.as_ptr();let child=parent.field2[0].field1.as_ptr();let mut a=RecordArray0::from([parent,first]);function2(&mut a);assert_eq!(a[1].field1.as_ptr(),p);assert_eq!(a[1].field2[0].field1.as_ptr(),child);let copy=function3(&a);assert_eq!(copy[1].field2[0].field1.as_ref(),"leaf");assert_ne!(copy[1].field1.as_ptr(),p);assert_ne!(copy[1].field2[0].field1.as_ptr(),child);drop(a);drop(copy);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for copied in [false,true]{let mut a=Array::from([String::Owned("ccc".into()),String::Owned("a".into()),String::Owned("bb".into())]);let pointers=a.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();let state=LIVE.load(Ordering::SeqCst);let boom=NumericArray::from([]);let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||if copied{drop(function5(&a,&boom));}else{function4(&mut a,&boom);}));assert!(result.is_err());drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),state);assert_eq!(a.len(),3);for pointer in &pointers{assert!(a.iter().any(|v|v.as_ptr()==*pointer));}if copied{assert!(a.iter().zip(&pointers).all(|(v,p)|v.as_ptr()==*p));}drop(a);drop(pointers);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        for len in [4usize,17]{let mut a=(0..len).rev().map(|i|String::Owned(i.to_string())).collect::<Vec<_>>();let pointers=a.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();let mut count=0;let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||a.tsr_sort(|x,y|{count+=1;if count==3{panic!("comparator");}x.cmp(y)})));assert!(result.is_err());drop(result);for pointer in &pointers{assert!(a.iter().any(|v|v.as_ptr()==*pointer));}drop(a);drop(pointers);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
    "#,
    );
}

#[test]
fn sorting_strict_types_loans_capture_contracts_and_normal_depth() {
    let mut mismatches = Vec::new();
    for (source, code) in [
        ("function main(){const a=[1];a.sort(1);}", "type"),
        (
            "function main(){const a=[1];a.toSorted(undefined);}",
            "readonly",
        ),
        (
            "function main(){const a=[1];a.sort(():int=>{return 0;},():int=>{return 0;});}",
            "arity",
        ),
        (
            "function main(){const a=[1];a.sort((x:int,y:int,z:int):int=>{return 0;});}",
            "arity",
        ),
        (
            "function main(){const a=[1];a.sort((x:int,y:int):boolean=>{return x<y;});}",
            "type",
        ),
        (
            "function main(){const a=[1];a.sort((x:number,y:number):number=>{return x-y;});}",
            "type",
        ),
        ("function main(){const a=[[1]];a.sort();}", "type"),
        ("function main(){const a=[[1]];a.toSorted();}", "type"),
        (
            "type Row={n:int;};function main(){const a:Row[]=[];a.sort();}",
            "type",
        ),
        (
            "function main(){const a=[1,2];a.sort((x:int,y:int):int=>{a.push(3);return x-y;});}",
            "conflicting-access",
        ),
        (
            "function main(){const a=[1,2];a.sort((x:int,y:int):int=>{return a.length;});}",
            "conflicting-access",
        ),
        (
            "function main(){const a=[1,2];a.toSorted((x:int,y:int):int=>{a.push(3);return x-y;});}",
            "conflicting-access",
        ),
        (
            "function apply(a:string[],cmp:(x:Owned<string>,y:ReadOnly<string>)=>int):void{a.sort(cmp);}function main(){}",
            "callback-access",
        ),
        (
            "function apply(a:int[],cmp:Once<Function<int, [x:int,y:int]>>):void{a.sort(cmp);}function main(){}",
            "callback-access",
        ),
        (
            "function main(){const a=[\"x\"];const view=a.at(0);a.sort();console.log(String(view));}",
            "view-conflict",
        ),
        (
            "function main(){const a=[[1]];const view=a.at(0);if(view!==undefined){view.sort();}}",
            "borrowed-view",
        ),
        (
            "function read(a:int[],n:int):int{return a.length+n;}function change(a:int[]):int{a.sort();return 1;}function main(){const a=[1];read(a,change(a));}",
            "conflicting-access",
        ),
    ] {
        let error = check(source)
            .err()
            .expect("invalid sorting contract must reject");
        if error.code != code {
            mismatches.push(error.render("input.tsr", source));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    for count in [100, 200] {
        let source = format!(
            "function main(){{const values={};values.toSorted((x:int{},y:int{}):int=>{{return 0;}});}}",
            "[".repeat(count) + "1" + &"]".repeat(count),
            "[]".repeat(count - 1),
            "[]".repeat(count - 1)
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
        // Index chains have no nesting limit.
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            support::accept_deep(&source);
        }
    }
    assert_eq!(
        check("function main(){const ordinary=[];}")
            .err()
            .unwrap()
            .code,
        "array-type"
    );
}
