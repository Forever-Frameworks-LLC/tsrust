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
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let evidence = std::env::var_os("TSR_GATE92_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-flat.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-flat-{level}"));
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
fn flattening_dense_types_ranks_contexts_and_exact_number_bits() {
    let analysis = native(
        r#"
    type Row={id:int;label:string;};
    function numbers(values:number[][]):number[]{return values.flat();}
    function mappedNumbers(values:number[]):number[]{return values.flatMap((item:number):number[]=>{return [item];});}
    function main(){
      const deep:int[][][]=[[[1,2],[]],[[3]],[]];console.log(deep.flat().join("|"));console.log(deep.flat(2).join());console.log(deep.flat(99).join());console.log(deep.flat(0).length);console.log(deep.flat(-9223372036854775808).length);
      const ints=[1,2,3];console.log(ints.flat(9).join());console.log(ints.flatMap(():int=>{return 7;}).join());console.log(ints.flatMap((item:int):int=>{return item*2;}).join());console.log(ints.flatMap((item:int,position:int):int[]=>{return [item,position];}).join());console.log(ints.flatMap((item:int,position:int,rows:int[]):int[]=>{return [rows.length];}).join());
      console.log([[true,false],[],[true]].flat().join());console.log([true,false].flatMap((flag:boolean):boolean=>{return !flag;}).join());
      console.log([["é".repeat(1)],[],["😀"]].flat().join("|"));console.log(["é","x"].flatMap((text:string):string=>{return text.concat("!");}).join("|"));
      const rows:Row[][]=[[{id:1,label:"a"}],[],[{id:2,label:"b"}]];const flat=rows.flat();console.log(flat[0].label.concat(flat[1].label));console.log(rows.length);
      const mapped:Row[]=flat.flatMap((row:Row):Row=>{return {id:row.id,label:row.label.concat("!")};});console.log(mapped[0].label.concat(mapped[1].label));
      const nested:int[][]=ints.flatMap((item:int):int[][]=>{return [[item],[]];});console.log(nested.join("|"));
      console.log([[null],[],[null]].flat().join("|"));console.log([null,null].flatMap(():null=>{return null;}).length);console.log([[undefined],[undefined]].flat().length);console.log([undefined].flatMap(():undefined[]=>{return [undefined,undefined];}).length);
      const empty:string[]=[].flat();const nestedEmpty:string[]=[[]].flat();const mappedEmpty:string[]=[].flatMap((item:string):string[]=>{return [];});console.log(empty.length,nestedEmpty.length,mappedEmpty.length);
    }
    "#,
        "1,2||3\n1,2,3\n1,2,3\n3\n3\n1,2,3\n7,7,7\n2,4,6\n1,0,2,1,3,2\n3,3,3\ntrue,false,true\nfalse,true\né|😀\né!|x!\nab\n3\na!b!\n1||2||3|\n|\n2\n2\n2\n0 0 0\n",
        "",
        r#"
    let bits=[0x8000000000000000u64,0,0x7ff8000000000042,0xfff0000000000000,0x7ff0000000000000];let a=vec![bits[..2].iter().map(|n|f64::from_bits(*n)).collect::<Vec<_>>(),Vec::new(),bits[2..].iter().map(|n|f64::from_bits(*n)).collect()];let result=function0(&a);assert_eq!(result.iter().map(|n|n.to_bits()).collect::<Vec<_>>(),bits);let original=bits.map(f64::from_bits).to_vec();assert_eq!(function1(&original).iter().map(|n|n.to_bits()).collect::<Vec<_>>(),bits);assert_eq!(original.iter().map(|n|n.to_bits()).collect::<Vec<_>>(),bits);
    "#,
    );
    let facts = analysis.checked_type_facts();
    assert!(facts.method_results.iter().any(|f| f.ty == "int[]"));
    assert!(facts.method_results.iter().any(|f| f.ty == "Row[]"));
    assert_eq!(
        facts.method_results,
        analysis.checked_type_facts().method_results
    );
}

#[test]
fn flattening_evaluates_once_in_order_and_preserves_read_loans() {
    native(
        r#"
    function values():int[]{console.log("receiver");return [2,3];}
    function depth():int{console.log("depth");return -1;}
    function callback():()=>int[]{console.log("factory");return ():int[]=>{console.log("call");return [7];};}
    function main(){console.log(values().flat(depth()).join());console.log(values().flatMap(callback()).join());let visits=0;const a=[3,4];const b=a.flatMap((item:int,position:int,source:int[]):int[]=>{visits++;return [item,position,source.length];});console.log(visits,b.join(),a.join());const empty:int[]=[];console.log(empty.flatMap(callback()).length);let depthCalls=0;const same=a.flat(depthCalls++);console.log(depthCalls,same.join());const lazy=false?values().flatMap(callback()):[9];console.log(lazy.join());}
    "#,
        "receiver\ndepth\n2,3\nreceiver\nfactory\ncall\ncall\n7,7\n2 3,0,2,4,1,2 3,4\nfactory\n0\n1 3,4\n9\n",
        "",
        "",
    );
    for source in [
        "function main(){const a=[1,2];const out=a.flatMap((item:int):int=>{a.push(3);return item;});}",
        "function take(v:string):string{v;return \"x\";}function main(){const a=[\"x\".repeat(1)];a.flatMap((item:string):string=>{return take(item);});}",
        "function main(){const a=[1];a.flatMap((item:int,index:int,source:int[]):int=>{source.push(3);return item;});}",
        "function alter(v:int[]):int{v.push(3);return 0;}function main(){const a=[1];a.flat(alter(a));}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
    let fixed=check("function same(item:int):int{return item;}function main(){const a=[1,2,3];console.log(a.flat().join());console.log(a.flatMap(same).join());}").unwrap().emit_rust();
    assert!(fixed.contains(": [i64; 3] = ["));
}

#[test]
fn flattening_moves_temporary_owners_copies_named_owners_and_disposes_on_unwind() {
    native(
        r#"
    type Node={id:int;text:string;children:Node[];};
    function copied(values:string[][]):string[]{return values.flat();}
    function moved(values:string[][]):string[]{return supply(values).flat();}
    function supply(values:string[][]):string[][]{return values;}
    function mapped(values:int[]):string[]{return values.flatMap((item:int):string[]=>{return ["x".repeat(item)];});}
    function fail(values:string[],boom:int[]):string[]{return values.flatMap((item:string,position:int):string[]=>{return [item.concat("!"),boom[position].toString()];});}
    function copyNodes(values:Node[][]):Node[]{return values.flat();}
    function moveNodes(values:Node[][]):Node[]{return supplyNodes(values).flat();}
    function supplyNodes(values:Node[][]):Node[][]{return values;}
    function integers(values:int[]):int[]{return values.flat(0);}
    function main(){}
    "#,
        "",
        r#"
    use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;
    unsafe impl GlobalAlloc for Counts{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}}
    #[global_allocator]static COUNTS:Counts=Counts;
    "#,
        r#"
    std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
    for len in [0usize,1,4,5,17]{let values=(0..len).map(|i|Array::from([String::Owned(i.to_string())])).collect::<Vec<_>>();let pointers=values.iter().map(|row|row.get(0).unwrap().as_ptr()).collect::<Vec<_>>();let copied=function0(&values);assert_eq!(copied.len(),len);for (item,pointer)in copied.iter().zip(&pointers){assert_ne!(item.as_ptr(),*pointer);}drop(copied);let moved=function1(values);assert_eq!(moved.len(),len);for(item,pointer)in moved.iter().zip(&pointers){assert_eq!(item.as_ptr(),*pointer);}drop(moved);drop(pointers);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
    let inputs=NumericArray::from([1,2,3]);ALLOCS.store(0,Ordering::SeqCst);let out=function3(&inputs);assert_eq!(ALLOCS.load(Ordering::SeqCst),4);assert_eq!(out.iter().map(|v|v.as_ref()).collect::<Vec<_>>(),["x","xx","xxx"]);drop(out);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);ALLOCS.store(0,Ordering::SeqCst);let out=function8(&inputs);assert_eq!(ALLOCS.load(Ordering::SeqCst),1);assert_eq!(out.as_slice(),[1,2,3]);drop(out);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let values=Array::from([String::Owned("a".into()),String::Owned("b".into())]);let boom=NumericArray::from([7]);let state=LIVE.load(Ordering::SeqCst);let pointers=values.iter().map(|v|v.as_ptr()).collect::<Vec<_>>();let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function4(&values,&boom)));assert!(result.is_err());drop(result);drop(pointers);assert_eq!(LIVE.load(Ordering::SeqCst),state);assert_eq!(values.get(0).unwrap().as_ref(),"a");drop(values);
    let child=Record0{field0:2,field1:String::Owned("child".into()),field2:RecordArray0(vec![])};let node=Record0{field0:1,field1:String::Owned("root".into()),field2:RecordArray0(vec![child])};let root_ptr=node.field1.as_ptr();let child_ptr=node.field2[0].field1.as_ptr();let forest=vec![RecordArray0(vec![node])];let cloned=function5(&forest);assert_ne!(cloned[0].field1.as_ptr(),root_ptr);assert_ne!(cloned[0].field2[0].field1.as_ptr(),child_ptr);drop(cloned);let moved=function6(forest);assert_eq!(moved[0].field1.as_ptr(),root_ptr);assert_eq!(moved[0].field2[0].field1.as_ptr(),child_ptr);drop(moved);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#,
    );
}

#[test]
fn flattening_diagnostics_literal_depth_and_normal_stack_limits_are_precise() {
    for (source, code) in [
        ("function main(){[1].flat(1,2);}", "arity"),
        ("function main(){[1].flat(1.0);}", "type"),
        (
            "function main(){let depth:int=1;[[1]].flat(depth);}",
            "unsupported",
        ),
        ("function main(){[[1]].flat(1+1);}", "unsupported"),
        ("function main(){[1].flatMap();}", "arity"),
        (
            "function main(){[1].flatMap((x:int):int=>{return x;},3);}",
            "arity",
        ),
        (
            "function main(){[1].flatMap((x:string):int=>{return 1;});}",
            "type",
        ),
        (
            "function main(){[1].flatMap((x:int,a:int,b:int[],c:int):int=>{return x;});}",
            "arity",
        ),
        (
            "function main(){[1].flatMap((x:int):void=>{});}",
            "array-element",
        ),
        (
            "function main(){const a=[\"a\".repeat(1)];a.flatMap((x:string):string=>{return x;});}",
            "callback-access",
        ),
    ] {
        let error = check(source).err().expect(source);
        assert_eq!(error.code, code, "{source}:{}", error.message);
        assert!(error.span.end > error.span.start);
    }
    let source = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
        "a[".repeat(100),
        "]".repeat(100)
    );
    check(&source).unwrap().emit_rust();
    let source = format!(
        "function main(){{const a=[0];console.log({}0{});}}",
        "a[".repeat(200),
        "]".repeat(200)
    );
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
    maximum_rank_native();
    let source = format!(
        "function bad(values:int{}){{}}function main(){{}}",
        "[]".repeat(129)
    );
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
}

fn maximum_rank_native() {
    let source = format!(
        "function deep(values:int{}):int[]{{return values.flat(127);}}function result(values:int[]):int{}{{return values.flatMap((item:int):int{}=>{{return [];}});}}function main(){{}}",
        "[]".repeat(128),
        "[]".repeat(128),
        "[]".repeat(128)
    );
    let nested = (0..126).fold(
        "Array::from([NumericArray::from([7_i64])])".to_owned(),
        |value, _| format!("vec![{value}]"),
    );
    native(
        &source,
        "",
        "",
        &format!(
            "let values={nested};assert_eq!(function0(&values).as_slice(),[7]);let empty=NumericArray::from([1]);assert_eq!(function1(&empty).len(),0);"
        ),
    );
}
