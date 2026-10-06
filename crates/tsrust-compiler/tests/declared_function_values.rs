mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("declared-functions.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("declared-functions-{level}"));
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
                .arg(&binary),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(binary), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

fn reject(source: &str, code: &str) {
    let error = check(source).err().expect("source must be rejected");
    assert_eq!(error.code, code, "{source}\n{}", error.message);
}

#[test]
fn declared_callbacks_cover_data_families_arrays_empty_context_and_source_arity() {
    let cases = [
        ("int", "[1,2]", "return v;", vec![1, 2]),
        (
            "number",
            "[1.25,2.5]",
            "if(v<2.0){return 1;}return 2;",
            vec![1, 2],
        ),
        (
            "boolean",
            "[true,false]",
            "if(v){return 1;}return 0;",
            vec![1, 0],
        ),
        (
            "string",
            r#"["é","😀","a\u0000b","e\u0301"]"#,
            "return v.length;",
            vec![2, 4, 3, 3],
        ),
        ("Row", "[{n:1},{n:2}]", "return v.n;", vec![1, 2]),
        ("int[]", "[[1],[],[2,3]]", "return v.length;", vec![1, 0, 2]),
        (
            "number[]",
            "[[1.25],[],[2.5,3.75]]",
            "return v.length;",
            vec![1, 0, 2],
        ),
        (
            "boolean[]",
            "[[true],[],[false,true]]",
            "return v.length;",
            vec![1, 0, 2],
        ),
        (
            "string[]",
            r#"[["é"],[],["😀","x"]]"#,
            "return v.length;",
            vec![1, 0, 2],
        ),
        (
            "Row[]",
            "[[{n:1}],[],[{n:2},{n:3}]]",
            "return v.length;",
            vec![1, 0, 2],
        ),
        (
            "int[][]",
            "[[[1]],[],[[2],[3]]]",
            "return v.length;",
            vec![1, 0, 2],
        ),
        (
            "Node",
            "[{n:1,children:[{n:2,children:[]}]},{n:3,children:[]}]",
            "return v.n+v.children.length;",
            vec![2, 3],
        ),
    ];
    for (leaf, values, body, metrics) in cases {
        let source = format!(
            r#"
            type Row={{n:int;}};type Node={{n:int;children:Node[];}};
            function measure(v:{leaf}):int{{{body}}}
            function accumulate(a:int,v:{leaf}):int{{return a+measure(v);}}
            function positive(v:{leaf}):boolean{{return measure(v)>0;}}
            function show(v:{leaf}){{console.log(measure(v));}}
            function main(){{
                const values:{leaf}[]={values};const output=values.map(measure);
                for(const n of output){{console.log(n);}}
                console.log(values.reduce(accumulate,0));console.log(values.reduceRight(accumulate,0));
                console.log(values.filter(positive).length);console.log(values.some(positive));console.log(values.every(positive));
                console.log(values.findIndex(positive));console.log(values.findLastIndex(positive));values.forEach(show);
            }}
        "#
        );
        let mut expected = metrics.iter().map(|n| format!("{n}\n")).collect::<String>();
        let sum: i64 = metrics.iter().sum();
        let positives = metrics.iter().filter(|&&n| n > 0).count();
        let first = metrics.iter().position(|&n| n > 0).map_or(-1, |n| n as i64);
        let last = metrics
            .iter()
            .rposition(|&n| n > 0)
            .map_or(-1, |n| n as i64);
        expected += &format!(
            "{sum}\n{sum}\n{positives}\n{}\n{}\n{first}\n{last}\n",
            positives > 0,
            positives == metrics.len()
        );
        expected += &metrics.iter().map(|n| format!("{n}\n")).collect::<String>();
        native(&source, &expected, "", "");
    }
    let analysis = native(
        r#"
        function plus(a:number,v:number):number{return a+v;}
        function keep(v:number):number{return v;}
        function source(v:int,i:int,all:int[]):int{return v+i+all.length;}
        function fold(a:int,v:int,i:int,all:int[]):int{return a*10+v+i+all.length;}
        function zero():int{return 9;}
        function visit(){console.log("visit");}
        function main(){console.log([].reduce(plus,0.0));console.log([].map(keep).length);
            const a=[1,2,3];const mapped=a.map(source);console.log(mapped[0]);console.log(mapped[2]);
            console.log(a.reduce(fold,0));console.log(a.map(zero)[1]);const empty:int[]=[];empty.forEach(visit);[1].forEach(visit);}
    "#,
        "0\n0\n4\n8\n468\n9\nvisit\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts()["source.all"], Mode::Read);
    assert_eq!(analysis.contracts()["fold.all"], Mode::Read);
}

#[test]
fn bindings_transfers_replacements_returns_factories_and_captures_preserve_origins() {
    let source = r#"
        function add(a:int,b:int):int{return a+b;}
        function make():(a:int,b:int)=>int{return add;}
        function factory():()=>(a:int,b:int)=>int{return make;}
        function captured():(a:int,b:int)=>int{const local=add;return(a:int,b:int):int=>{return local(a,b);};}
        function disposable(n:int):()=>int{console.log(n);return():int=>{return n;};}
        function main(){
            const direct=add;console.log(direct(1,2));console.log(direct(3,4));
            const moved=direct;console.log(moved(5,6));let replace=add;replace=add;console.log(replace(7,8));
            const maker=make;const one=maker();const two=maker();console.log(one(9,10));console.log(two(11,12));
            const outer=factory;const nested=outer();const selected=nested();console.log(selected(13,14));
            const global=():int=>{const value=add;return value(15,16);};console.log(global());
            const bound=add;{const reader=():int=>{return bound(17,18);};console.log(reader());}console.log(bound(19,20));
            const saved=captured();console.log(saved(21,22));console.log(saved(23,24));[1,2].forEach(disposable);
        }
    "#;
    let analysis = native(
        source,
        "3\n7\n11\n15\n19\n23\n27\n31\n35\n39\n43\n47\n1\n2\n",
        "",
        "",
    );
    for name in ["add", "make", "factory"] {
        let wrappers = analysis
            .closures()
            .into_iter()
            .filter(|c| c.function == name && c.depth == 0)
            .collect::<Vec<_>>();
        assert_eq!(wrappers.len(), 1);
        assert!(wrappers[0].captures.is_empty());
        assert_eq!(wrappers[0].invocation, Mode::Read);
    }
    native(
        "function add(a:int):int{return a+1;}function main(){const add=(a:int):int=>{return a+2;};console.log(add(1));}",
        "3\n",
        "",
        "",
    );
    for (source, code) in [
        (
            "function f():int{return 1;}function main(){const cb=f;const next=cb;cb();}",
            "closure-consumed",
        ),
        (
            "function f():int{return 1;}function g():int{return 2;}function main(){let cb=f;cb=g;}",
            "callable-origin",
        ),
        (
            "function f():int{return 1;}function g():int{return 2;}function make(flag:boolean):()=>int{if(flag){return f;}return g;}function main(){}",
            "unsupported",
        ),
        ("function main(){const entry=main;}", "unsupported"),
        ("function main(){const builtin=clone;}", "name"),
        (
            "function use(f:()=>int):int{return f();}function main(){const consumers=[use];}",
            "array-element",
        ),
        (
            "function f():int{return 1;}function main(){const array=[f];}",
            "array-element",
        ),
    ] {
        reject(source, code);
    }
}

#[test]
fn exact_argument_modes_callback_contracts_loans_and_operand_effects_remain_enforced() {
    let analysis = native(
        r#"
        type Row={n:int;};
        function read(row:Row):int{return row.n;}
        function mutate(row:Row){row.n=row.n+1;}
        function consume(row:Row):Row{return row;}
        function reading(cb:(row:ReadOnly<Row>)=>int,row:Row):int{return cb(row);}
        function writing(cb:(row:Mutable<Row>)=>void,row:Row){cb(row);}
        function taking(cb:Function<Row, [row:Owned<Row>]>,row:Row):Row{return cb(row);}
        function single(cb:Once<Function<int, [row:ReadOnly<Row>]>>,row:Row):int{return cb(row);}
        function both(first:()=>int,second:()=>int):int{return first()+second();}
        function one():int{return 1;}
        function effect(n:int):int{console.log(n);return n;}
        function combine(a:int,b:int):int{return a*10+b;}
        function main(){
            const row:Row={n:3};console.log(reading(read,row));writing(mutate,row);console.log(row.n);console.log(single(read,row));
            const transferred=taking(consume,{n:5});console.log(transferred.n);
            const action=consume;const left=action({n:6});const right=action({n:7});console.log(left.n+right.n);
            console.log(both(one,one));const call=combine;console.log(call(effect(8),effect(9)));
        }
    "#,
        "3\n4\n4\n5\n13\n2\n8\n9\n89\n",
        "",
        "",
    );
    for (name, mode) in [
        ("read.row", Mode::Read),
        ("mutate.row", Mode::Mutate),
        ("consume.row", Mode::Consume),
    ] {
        assert_eq!(analysis.contracts()[name], mode);
    }
    let consume = analysis
        .closures()
        .into_iter()
        .find(|c| c.function == "consume" && c.depth == 0)
        .unwrap();
    assert_eq!(consume.invocation, Mode::Read);
    assert_eq!(consume.parameters[0].access, Mode::Consume);
    for (source, code) in [
        (
            "type R={n:int;};function write(r:R){r.n=2;}function use(cb:(r:ReadOnly<R>)=>void){}function main(){use(write);}",
            "callback-access",
        ),
        (
            "function take(s:string):string{return s;}function main(){[\"x\"].map(take);}",
            "callback-access",
        ),
        (
            "function f(n:number):number{return n;}function main(){const cb=f;const n=1;cb(n);}",
            "type",
        ),
        (
            "function f(a:int,b:int,c:int,d:int):int{return a+b+c+d;}function main(){[1].map(f);}",
            "arity",
        ),
        (
            "function f():int{return 1;}function both(a:Mutable<Function<int>>,b:Mutable<Function<int>>):int{return a()+b();}function main(){const cb=f;both(cb,cb);}",
            "conflicting-access",
        ),
        (
            "function f():int{return 1;}function use(a:Mutable<Function<int>>,n:int):int{return a()+n;}function main(){const cb=f;use(cb,cb());}",
            "conflicting-access",
        ),
        (
            "function f():int{return 1;}function main(){let cb=f;const reader=():int=>{return cb();};cb=f;reader();}",
            "capture-conflict",
        ),
        (
            "function f():int{return 1;}function factory(cb:Mutable<Function<int>>):()=>int{return():int=>{return cb();};}function main(){}",
            "callback-escape",
        ),
    ] {
        reject(source, code);
    }
    native(
        r#"
        function values():int[]{console.log("receiver");return [1,2,3];}
        function initial():int{console.log("initial");return 0;}
        function fold(a:int,v:int,i:int):int{console.log(i);return a*10+v;}
        function main(){console.log(values().reduce(fold,initial()));console.log(values().reduceRight(fold,initial()));}
    "#,
        "receiver\ninitial\n0\n1\n2\n123\nreceiver\ninitial\n2\n1\n0\n321\n",
        "",
        "",
    );
}

#[test]
fn recursive_data_functions_and_resolved_callable_factory_origins_survive_discovery() {
    native(
        r#"
        function even(n:int):boolean{if(n===0){return true;}return odd(n-1);}
        function odd(n:int):boolean{if(n===0){return false;}return even(n-1);}
        function sum(n:int):int{if(n===0){return 0;}const recursive=sum;return n+recursive(n-1);}
        function base():int{return 7;}
        function left(n:int):()=>int{if(n>0){return right(n-1);}return base;}
        function right(n:int):()=>int{return left(n);}
        function relay(n:int):()=>int{const base=cycle(n);return base;}
        function cycle(n:int):()=>int{if(n>0){return relay(n-1);}return():int=>{return 11;};}
        function factory(seed:int,n:int):(x:int)=>int{if(n>0){return factory(seed,n-1);}return(x:int):int=>{return seed+x;};}
        function main(){const parity=even;console.log(parity(6));console.log(parity(5));const total=sum;console.log(total(4));
            const make=left;const resolved=make(3);console.log(resolved());
            const proxy=relay;const localShadow=proxy(2);console.log(localShadow());
            const maker=factory;const first=maker(8,2);const second=maker(9,1);console.log(first(1));console.log(first(2));console.log(second(1));}
    "#,
        "true\nfalse\n10\n7\n11\n9\n10\n10\n",
        "",
        "",
    );
    for source in [
        "function f():()=>int{return f();}function main(){const factory=f;}",
        "function f(n:int):()=>int{const previous=f(n-1);return():int=>{return previous();};}function main(){const factory=f;}",
    ] {
        reject(source, "unsupported");
    }
    for depth in [100, 200] {
        let expression = format!("{}1{}", "-(".repeat(depth), ")".repeat(depth));
        let source = format!(
            "function value():int{{return {expression};}}function main(){{const cb=value;console.log(cb());}}"
        );
        if depth == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            reject(&source, "unsupported");
        }
    }
    let analysis = check("function unused():int{return 1;}function main(){}").unwrap();
    assert!(analysis.closures().is_empty());
}

#[test]
fn zero_capture_values_owned_outputs_disposal_unwind_threads_and_fixed_fallback() {
    // Native observers are outside source operations. The emitted data ABI is
    // validated by Rust itself; no representation policy is relaxed here.
    let source = r#"
        function copy(text:string):string{return clone(text);}
        function take(text:string):string{return text;}
        function fail(text:string,values:int[]):string{values.remove(-1);return text;}
        function mapped(text:string):string{return text.toUpperCase();}
        function source(value:int,index:int,all:int[]):int{return value+index+all.length;}
        function scalar(value:int):int{return value+1;}
        function main(){const copyValue=copy;const input="é😀a\u0000b";console.log(copyValue(input).length);
            const numbers=[1,2,3];console.log(numbers.map(scalar)[2]);console.log(numbers.map(source)[2]);
            const upper=["é","ß"].map(mapped);console.log(upper[0]);console.log(upper[1]);}
    "#;
    let analysis = check(source).unwrap();
    let closures = analysis.closures();
    let copy_id = closures
        .iter()
        .find(|c| c.function == "copy" && c.depth == 0)
        .unwrap()
        .id;
    // Reference fail/take as values in source so their wrappers participate in
    // real analysis rather than adding an unused runtime-only constructor.
    let source = source.replace(
        "const copyValue=copy;",
        "const copyValue=copy;const takeValue=take;const failure=fail;",
    );
    let analysis = check(&source).unwrap();
    let closures = analysis.closures();
    let copy_id_actual = closures
        .iter()
        .find(|c| c.function == "copy" && c.depth == 0)
        .unwrap()
        .id;
    assert_eq!(copy_id_actual, copy_id);
    let take_id = closures
        .iter()
        .find(|c| c.function == "take" && c.depth == 0)
        .unwrap()
        .id;
    let fail_id = closures
        .iter()
        .find(|c| c.function == "fail" && c.depth == 0)
        .unwrap()
        .id;
    let assertions = format!(
        r#"
        fn send_sync<T:Send+Sync>(){{}}send_sync::<Closure{copy_id}>();
        assert_eq!(std::mem::size_of::<Closure{copy_id}>(),0);
        std::thread::spawn(||{{}}).join().unwrap();
        let before=LIVE.load(std::sync::atomic::Ordering::SeqCst);
        {{let cb=Closure{copy_id}{{}};let start=ALLOCS.load(std::sync::atomic::Ordering::SeqCst);let another=Closure{copy_id}{{}};
            assert_eq!(ALLOCS.load(std::sync::atomic::Ordering::SeqCst),start);
            let input:String=std::borrow::Cow::Owned("é😀\0".repeat(32));let saved=input.as_ptr();let output=cb.call(&input);
            assert_eq!(output.as_ref(),input.as_ref());assert_ne!(output.as_ptr(),saved);drop(output);
            let result=another.call(&input);drop(result);drop(input);
            let owned:String=std::borrow::Cow::Owned("owned".repeat(32));let pointer=owned.as_ptr();
            let moved=Closure{take_id}{{}}.call(owned);assert_eq!(moved.as_ptr(),pointer);drop(moved);
            let mut values=NumericArray::from([1i64,2,3]);
            let old_hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{{}}));
            let panic=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{{
                Closure{fail_id}{{}}.call(std::borrow::Cow::Owned("failure".repeat(32)),&mut values)
            }}));std::panic::set_hook(old_hook);assert!(panic.is_err());drop(panic);drop(values);
            std::thread::spawn(move||{{let text:String=std::borrow::Cow::Owned("thread".into());another.call(&text)}}).join().unwrap();
        }}assert_eq!(LIVE.load(std::sync::atomic::Ordering::SeqCst),before);
    "#
    );
    native(&source, "9\n4\n8\nÉ\nSS\n", ALLOCATOR, &assertions);
    let owners = r#"
        type Row={text:string;values:int[];};type Node={text:string;children:Node[];};
        function copyRow(row:Row):Row{return clone(row);}
        function copyNested(values:int[][]):int[][]{return clone(values);}
        function copyTree(root:Node):Node{return clone(root);}
        function numbers(values:number[]):number[]{return values;}
        function main(){const row=copyRow;const nested=copyNested;const tree=copyTree;const bits=numbers;}
    "#;
    let owner_analysis = check(owners).unwrap();
    let metadata = owner_analysis.closures();
    let id = |name: &str| {
        metadata
            .iter()
            .find(|c| c.function == name && c.depth == 0)
            .unwrap()
            .id
    };
    let row = id("copyRow");
    let nested = id("copyNested");
    let tree = id("copyTree");
    let bits = id("numbers");
    let assertions = format!(
        r#"
        let baseline=LIVE.load(Ordering::SeqCst);
        {{
            let input=Record0{{field0:String::Owned("row".repeat(32)),field1:NumericArray::Heap(vec![1i64,2,3,4,5])}};
            let mut output=Closure{row}{{}}.call(&input);
            assert_ne!(output.field0.as_ptr(),input.field0.as_ptr());
            *output.field1.get_mut(0).unwrap()=99;assert_eq!(*input.field1.get(0).unwrap(),1);
            assert_eq!(output.field0.as_ref(),input.field0.as_ref());drop(output);drop(input);
            let input=Array::Heap(vec![NumericArray::Heap(vec![1i64,2,3,4,5]),NumericArray::from([])]);
            let mut output=Closure{nested}{{}}.call(&input);
            *output.get_mut(0).unwrap().get_mut(0).unwrap()=88;
            assert_eq!(*input.get(0).unwrap().get(0).unwrap(),1);assert_eq!(output.get(1).unwrap().len(),0);drop(output);drop(input);
            let input=Record1{{field0:String::Owned("root".repeat(32)),field1:RecordArray1::from([
                Record1{{field0:String::Owned("leaf".repeat(32)),field1:RecordArray1::from([])}}])}};
            let mut output=Closure{tree}{{}}.call(&input);
            assert_ne!(output.field0.as_ptr(),input.field0.as_ptr());
            output.field1.get_mut(0).unwrap().field0=String::Borrowed("changed");
            assert_eq!(input.field1.get(0).unwrap().field0.as_ref(),"leaf".repeat(32));drop(output);drop(input);
            let input=vec![-0.0f64,0.0,f64::INFINITY,f64::NEG_INFINITY,f64::from_bits(0x7ff8_0000_0000_0123)];
            let expected=input.iter().map(|v|v.to_bits()).collect::<Vec<_>>();let pointer=input.as_ptr();
            let output=Closure{bits}{{}}.call(input);assert_eq!(output.as_ptr(),pointer);
            assert_eq!(output.iter().map(|v|v.to_bits()).collect::<Vec<_>>(),expected);drop(output);drop(expected);
        }}assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#
    );
    native(owners, "", ALLOCATOR, &assertions);
    let rust = analysis.emit_rust();
    // A closure-free, source-argument-free declared callback does not disable
    // existing fixed scalar proof. A full source callback keeps ordinary ABI.
    let fixed=check("function next(v:int):int{return v+1;}function main(){const values=[1,2,3];console.log(values.map(next)[0]);}").unwrap().emit_rust();
    assert!(fixed.contains("[i64; 3]"));
    assert!(!rust.contains("unsafe"));
}

const ALLOCATOR: &str = r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);
struct Observer;
unsafe impl GlobalAlloc for Observer{
    unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size() as isize,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}
    unsafe fn dealloc(&self,p:*mut u8,l:Layout){LIVE.fetch_sub(l.size() as isize,Ordering::SeqCst);unsafe{System.dealloc(p,l)}}
    unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let q=unsafe{System.realloc(p,l,n)};if !q.is_null(){LIVE.fetch_add(n as isize-l.size() as isize,Ordering::SeqCst);}q}
}
#[global_allocator]static GLOBAL:Observer=Observer;
"#;
