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
    let assertions = if assertions.contains("__NUMBER_FUNCTION__") {
        let functions = emitted
            .lines()
            .filter(|line| {
                line.starts_with("fn function") && line.contains("(value0: f64) -> Vec<f64>")
            })
            .collect::<Vec<_>>();
        assert_eq!(functions.len(), 1, "unique Number constructor witness");
        let name = functions[0]
            .split_once('(')
            .unwrap()
            .0
            .strip_prefix("fn ")
            .unwrap();
        assertions.replace("__NUMBER_FUNCTION__", name)
    } else {
        assertions.to_owned()
    };
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let evidence = std::env::var_os("TSR_GATE93_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-of.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-of-{level}"));
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
fn constructor_types_contexts_generics_and_scalar_bits_are_exact() {
    let analysis = native(
        r#"
    type Row={id:int;text:string;};
    type Box<T>={value:T;};
    function numbers(value:number):number[]{return Array.of<number>(value,-0,Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY);}
    function build<T>(value:T):T[]{return Array.of<T>(value);}
    function empty<T>():T[]{return Array.of<T>();}
    function inferred():number[]{return Array.of(1,2);}
    function main(){
      console.log(Array.of(7).join(),Array.of(1,2,3,4,5).join("|"));
      console.log(Array.of(true,false).join(),Array.of("é","😀").join("|"));
      console.log(Array.of(null,null).length,Array.of(undefined,undefined).length);
      const nested:int[][]=Array.of(Array.of(),Array.of(1,2));console.log(nested.join("|"));
      const rows=Array.of<Row>({id:4,text:"row"});console.log(rows[0].id,rows[0].text);
      const box=Array.of<Box<int>>({value:3});console.log(box[0].value);
      console.log(build<int>(8).join(),build<string>("owned".repeat(1)).join());
      console.log(empty<int>().length,empty<Box<string>>().length);
      let contextual:number[]=Array.of();contextual=Array.of(1,-0);console.log(contextual.length,1/contextual[1]);
      console.log(inferred().join());
    }
    "#,
        "7 1|2|3|4|5\ntrue,false é|😀\n2 2\n|1,2\n4 row\n3\n8 owned\n0 0\n2 -Infinity\n1,2\n",
        "",
        r#"
      for bits in [0u64,0x8000000000000000,0x7ff8000000000042,0x7ff0000000000000,0xfff0000000000000] {
        let values=__NUMBER_FUNCTION__(f64::from_bits(bits));assert_eq!(values[0].to_bits(),bits);
        assert_eq!(values[1].to_bits(),0x8000000000000000);assert!(values[2].is_nan());assert_eq!(values[3],f64::INFINITY);assert_eq!(values[4],f64::NEG_INFINITY);
      }
    "#,
    );
    let facts = analysis.checked_type_facts();
    assert_eq!(facts, analysis.checked_type_facts());
    assert!(facts.method_results.iter().any(|x| x.ty == "number[]"));
    assert!(facts.method_results.iter().any(|x| x.ty == "Row[]"));
    assert!(facts.method_results.iter().any(|x| x.ty == "int[][]"));
}

#[test]
fn constructor_items_evaluate_once_in_order_across_supported_contexts() {
    native(
        r#"
    function make(value:int):int[]{return Array.of(value);}
    function item(trace:int[],n:int):int{trace[0]=trace[0]*10+n;return n;}
    function use(values:int[]):int{return values.length;}
    function main(){
      const trace=[0];
      const values=Array.of(item(trace,1),item(trace,2),item(trace,3));console.log(trace[0],values.join());
      let index=0;console.log(Array.of(index++,index++,index++).join(),index);
      console.log((false?Array.of(item(trace,8)):Array.of(item(trace,4))).join(),trace[0]);
      const nested=Array.of(make(1),make(2));console.log(nested.join("|"));
      console.log(use(Array.of()),make(9).join());
      let sum=0;for(const value of Array.of(2,3)){sum+=value;}console.log(sum);
      const create=():string[]=>{return Array.of<string>("closure");};console.log(create().join());
      const carry=Array.of<number>(1);console.log(carry[0]+0.5);
    }
    "#,
        "123 1,2,3\n0,1,2 3\n4 1234\n1|2\n0 9\n5\nclosure\n1.5\n",
        "",
        "",
    );
    let source = "function main(){const values=Array.of(1,2,3);console.log(values.join());}";
    let emitted = check(source).unwrap().emit_rust();
    assert!(emitted.contains(": [i64; 3] = ["), "{emitted}");
    for source in [
        "function read(value:string,n:int):int{return value.length+n;}function take(value:string):int{value;return 0;}function main(){const value=\"x\".repeat(1);Array.of(read(value,take(value)));}",
        "function main(){const rows=[\"x\".repeat(1)];const selected=rows.at(0);if(selected!==undefined){Array.of(selected);}}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
}

#[test]
fn constructor_owned_items_move_and_partial_construction_unwinds_without_clones() {
    native(
        r#"
    type Node={id:int;text:string;children:Node[];};
    function texts(a:string,b:string):string[]{return Array.of(a,b);}
    function integers(a:int,b:int,c:int,d:int):int[]{return Array.of(a,b,c,d);}
    function nodes(node:Node):Node[]{return Array.of<Node>(node);}
    function fail(values:int[]):string[]{return Array.of("temporary".repeat(1),String(values[4]));}
    function copy(value:string):string[]{return Array.of(clone(value));}
    function main(){const owned="live".repeat(1);const values=Array.of(owned);console.log(values.join());}
    "#,
        "live\n",
        r#"
      use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
      static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Counts;
      unsafe impl GlobalAlloc for Counts{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let p=unsafe{System.realloc(p,l,n)};if !p.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}}
      #[global_allocator]static COUNTS:Counts=Counts;
    "#,
        r#"
      std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
      let a=String::Owned("first".into());let b=String::Owned("second".into());let pa=a.as_ptr();let pb=b.as_ptr();ALLOCS.store(0,Ordering::SeqCst);
      let values=function0(a,b);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);assert_eq!(values.get(0).unwrap().as_ptr(),pa);assert_eq!(values.get(1).unwrap().as_ptr(),pb);drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
      ALLOCS.store(0,Ordering::SeqCst);let values=function1(1,2,3,4);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);assert_eq!(values.as_slice(),[1,2,3,4]);drop(values);
      let child=Record0{field0:2,field1:String::Owned("child".into()),field2:RecordArray0(vec![])};let node=Record0{field0:1,field1:String::Owned("root".into()),field2:RecordArray0(vec![child])};let root=node.field1.as_ptr();let child=node.field2[0].field1.as_ptr();let values=function2(node);assert_eq!(values[0].field1.as_ptr(),root);assert_eq!(values[0].field2[0].field1.as_ptr(),child);drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
      let values=NumericArray::from([1]);let result=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&values)));assert!(result.is_err());drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
      let original=String::Owned("copy".into());let result=function4(&original);assert_ne!(result.get(0).unwrap().as_ptr(),original.as_ptr());assert_eq!(original.as_ref(),"copy");drop(result);drop(original);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#,
    );
}

#[test]
fn constructor_type_arity_transfer_and_normal_depth_guards_are_precise() {
    for (source, code) in [
        ("function main(){const a=Array.of();}", "array-type"),
        ("function main(){Array.of().flat();}", "array-type"),
        ("function main(){Array.isArray(Array.of());}", "array-type"),
        ("function main(){Array.of<int>(1.5);}", "type"),
        ("function main(){Array.of<int,string>(1);}", "arity"),
        (
            "function main(){const a=[1];a.slice<int>();}",
            "unsupported",
        ),
        ("function main(){let Array=1;Array.of(1);}", "duplicate"),
        (
            "function main(){const text=\"x\".repeat(1);const a=Array.of(text);console.log(text);}",
            "transferred",
        ),
        (
            "function main(){const a=[\"x\".repeat(1)];a.map((text:string):string[]=>{return Array.of(text);});}",
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
    let source = format!(
        "function make():int{}{{return Array.of<int{}>();}}function main(){{}}",
        "[]".repeat(128),
        "[]".repeat(127)
    );
    native(&source, "", "", "assert_eq!(function0().len(),0);");
    let source = format!("function main(){{Array.of<int{}>();}}", "[]".repeat(128));
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
}
