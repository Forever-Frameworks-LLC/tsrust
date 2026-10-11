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
    let evidence = std::env::var_os("TSR_GATE87_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("array-at.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-at-{level}"));
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
fn at_complete_dense_families_bounds_observers_and_contexts() {
    native(
        r#"
        type Row={text:string;values:int[];};type Node={tag:int;children:Node[];};
        function text(value:string|undefined):int {if(value===undefined){return -1;}return value.length;}
        function nested(value:int[]|undefined):int {if(value===undefined){return -1;}return value.length;}
        function row(value:Row|undefined):int {if(value===undefined){return -1;}return value.values.length+value.text.length;}
        function node(value:Node|undefined):int {if(value===undefined){return -1;}return value.tag+value.children.length;}
        function main(){
            const ints=[10,20];for(let i=-3;i<3;i++){console.log(String(ints.at(i)));}console.log(ints.length);
            console.log(String(ints.at(-9223372036854775808)));console.log(String(ints.at(9223372036854775807)));
            const nil:null[]=[null];const absent:undefined[]=[undefined];console.log(String(nil.at(-1)));console.log(String(nil.at(1)));console.log(String(absent.at(0)));console.log(String([].at(0)));
            const numbers:number[]=[-0.0,Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY];console.log(String(numbers.at(-4)));console.log(String(numbers.at(1)));console.log(String(numbers.at(-2)));console.log(String(numbers.at(-1)));
            const bools=[true,false];console.log(String(bools.at(-1)));console.log(String(bools.at(2)));
            const words=["é\0".repeat(1),"😀".repeat(1)];{const selected=words.at(-1);const alias=selected;console.log(typeof alias);console.log(Boolean(alias));console.log(Array.isArray(alias));console.log(String(alias));console.log(text(alias));if(alias!==undefined){console.log(alias.concat("!"));}console.log(words.length);}words.push("x");console.log(words.length);
            const arrays=[[1,2],[3]];{const selected=arrays.at(0);console.log(Array.isArray(selected));console.log(String(selected));console.log(nested(selected));if(selected!==undefined){console.log(selected.join("|"));}console.log(arrays.length);}
            const rows:Row[]=[{text:"row".repeat(1),values:[7,8]}];{const selected=rows.at(0);console.log(String(selected));console.log(row(selected));if(selected!==undefined){console.log(selected.text.concat("!"));console.log(selected.values.join());}}
            const nodes:Node[]=[{tag:4,children:[{tag:5,children:[]}]}];{const selected=nodes.at(-1);console.log(node(selected));console.log(typeof selected);}
            const deep=[[[[7]]]];{const selected=deep.at(-1);if(selected!==undefined){console.log(selected.join());}}
            const literal=["a","é"];{const selected=literal.at(1);if(selected!==undefined){console.log(selected.valueOf());}}console.log(literal.length);
            const nestedLiteral=[["é","😀"]];{const selected=nestedLiteral.at(0);if(selected!==undefined){console.log(selected.join("|"));}}
            const fresh=["owned".repeat(1)].at(0);if(fresh!==undefined){console.log(fresh.concat("!"));}
            const copied=clone(words.at(0));words.push("after");if(copied!==undefined){console.log(copied.concat("!"));}console.log(words.length);
            const empty:string[]=[];{const selected=empty.at(0);console.log(text(selected));console.log(typeof selected);console.log(Boolean(selected));}
        }
    "#,
        "undefined\n10\n20\n10\n20\nundefined\n2\nundefined\nundefined\nnull\nundefined\nundefined\nundefined\n0\nNaN\nInfinity\n-Infinity\nfalse\nundefined\nstring\ntrue\nfalse\n😀\n4\n😀!\n2\n3\ntrue\n1,2\n2\n1|2\n2\n[object Object]\n5\nrow!\n7,8\n5\nobject\n7\né\n2\né|😀\nowned!\né\0!\n4\n-1\nundefined\nfalse\n",
        "",
        "",
    );
}

#[test]
fn at_borrowed_arguments_alias_scopes_index_order_and_fixed_storage() {
    native(
        r#"
        function wider(value:string|null|undefined):int{if(value!==null&&value!==undefined){return value.length;}return -1;}
        function mark():int{return "index".repeat(1).length-5;}
        function main(){
            const words=["é😀".repeat(1)];
            {const view=words.at(mark());const alias=true?view:undefined;console.log(wider(alias));console.log(String(alias));console.log(wider(false?undefined:view));console.log(wider(false?view:undefined));
             const residual=true?view:null;console.log(wider(residual));console.log(String(residual));const owned=clone(residual);if(owned!==null&&owned!==undefined){console.log(owned.concat("!"));}}
            words.push("after");console.log(words.length);
        }
    "#,
        "6\né😀\n6\n-1\n6\né😀\né😀!\n2\n",
        "",
        "",
    );
    for source in [
        "function main(){const a=[\"x\"];const v=true?a.at(0):[\"fresh\".repeat(1)].at(0);console.log(String(v));}",
        "function main(){const a=[\"x\"];const v=true?a.at(0):\"fresh\".repeat(1);console.log(String(v));}",
    ] {
        assert_eq!(
            check(source)
                .err()
                .expect("mixed borrowed owning must reject")
                .code,
            "borrowed-view"
        );
    }
    for source in [
        "function wider(v:string|null|undefined):int{return 1;} function main(){const a=[\"x\"];const v=true?a.at(0):null; a.push(\"y\");console.log(wider(v));}",
        "function main(){const a=[\"x\"];const v=a.at(0);const f=():string|undefined=>{return v;}; f();}",
        "type R={v:string|undefined;}; function main(){const a=[\"x\"];const r:R={v:a.at(0)};}",
        "function f():string|undefined{const a=[\"x\"];return true?a.at(0):undefined;}function main(){}",
    ] {
        let e = check(source).err().expect("view must reject");
        assert!(
            matches!(
                e.code,
                "borrowed-view"
                    | "view-conflict"
                    | "borrowed-escape"
                    | "borrowed-move"
                    | "unsupported"
            ),
            "{}: {source}",
            e.render("input.tsr", source)
        );
    }
    native(
        r#"function read(v:string|undefined):int{if(v!==undefined){return v.length;}return -1;}
        function main(){const a=["a".repeat(1)];const b=["bb".repeat(1),"c".repeat(1)];
        {const selected=(true?a:b).at(-1);console.log(read(selected));console.log(a.length+b.length);}
        {const selected=(false?a:b).at(-1);console.log(read(selected));}
        a.push("released");console.log(a.length);b.push("released");console.log(b.length);
        const n=[1,2];const m=[3];console.log(String((false?n:m).at(0)));console.log(n.length+m.length);}
    "#,
        "1\n3\n1\n2\n3\n3\n3\n",
        "",
        "",
    );
    for source in [
        r#"function take(a:string[]):int{a;return 0;}function read(v:string|null|undefined,n:int):int{return n;}function main(){const a=["a"];console.log(read(true?a.at(0):null,take(a)));}"#,
        r#"function grow(a:string[]):int{a.push("x");return 0;}function main(){const a=["a"];const b=["b"];console.log(String((true?a:b).at(grow(b))));}"#,
    ] {
        let e = check(source)
            .err()
            .expect("inherited view loan must reject");
        assert!(
            matches!(e.code, "conflicting-access" | "view-conflict"),
            "{}",
            e.render("input.tsr", source)
        );
    }
    if std::env::var_os("TSR_GATE87_SUPPLEMENT_ONLY").is_some() {
        return;
    }

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
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
        let source = std::fs::read_to_string(
            root.join(format!("benchmarks/runtime/workloads/array_at_{leaf}.tsr")),
        )
        .unwrap();
        check(&source)
            .unwrap_or_else(|e| panic!("{}", e.render("workload.tsr", &source)))
            .emit_rust();
    }
    native(
        &std::fs::read_to_string(root.join("benchmarks/runtime/array-at-source-order.tsr"))
            .unwrap(),
        &std::fs::read_to_string(
            root.join("benchmarks/runtime/array-at-source-order-expected.txt"),
        )
        .unwrap(),
        "",
        "",
    );

    let analysis = native(
        r#"
        type Count={n:int;};type Holder={items:string[]|null;other:string|null;};
        function mark(c:Count):int{console.log("index");c.n+=1;return -1;}
        function fresh(c:Count):string[]{console.log("receiver");c.n+=1;return ["fresh".repeat(1)];}
        function grow(values:string[]):int{values.push("new".repeat(1));return -1;}
        function read(value:string|undefined):int{if(value===undefined){return -1;}const f=():int=>{if(value!==undefined){return value.length;}return -1;};return f();}
        function fixed():int{const values=[1,2,3];const independent=[21,22];return values.at(-1)!==undefined?independent[0]+independent[1]:0;}
        function main(){
            const count:Count={n:0};const words=["one".repeat(1)];console.log(read(words.at(mark(count))));console.log(count.n);console.log(words.length);
            console.log(String(fresh(count).at(mark(count))));console.log(count.n);
            {const selected=words.at(grow(words));console.log(String(selected));console.log(words.length);}
            {const first=words.at(0);const alias=first;const readAlias=():int=>{if(alias!==undefined){return alias.length;}return -1;};console.log(readAlias());}
            words.push("released");console.log(words.length);
            const holder:Holder={items:["held".repeat(1)],other:"other".repeat(1)};if(holder.items!==null&&holder.other!==null){{const selected=holder.items.at(0);console.log(read(selected));console.log(holder.other.concat("!"));}holder.items.push("later");console.log(holder.items.length);}
            const a=["a".repeat(1)];const b=["bb".repeat(1)];{const selected=true?a.at(0):b.at(0);console.log(read(selected));}console.log(a.length+b.length);
            const owner:string|undefined="owned".repeat(1);console.log(read(owner));console.log(String(owner));
            const callback=(value:string|undefined):int=>{if(value!==undefined){return value.length;}return -1;};console.log(callback(words.at(0)));
            console.log(fixed());
        }
    "#,
        "index\n3\n1\n1\nreceiver\nindex\nfresh\n3\nnew\n2\n3\n3\n4\nother!\n2\n1\n2\n5\nowned\n3\n43\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("read.value"), Some(&Mode::Read));
    assert!(analysis.emit_rust().contains(": [i64; 2] = [21i64, 22i64]"));
    assert!(analysis.projections().iter().any(|p| p.ty == "string"));
}

#[test]
fn at_strict_types_borrowed_view_escapes_conflicts_and_depth() {
    for (source, code) in [
        ("function main(){const a=[1];a.at();}", "arity"),
        ("function main(){const a=[1];a.at(0,1);}", "arity"),
        ("function main(){const a=[1];a.at(0.0);}", "type"),
        ("function main(){const a=[1];a.at(\"0\");}", "type"),
        ("function main(){const n=1.5;n.at(0);}", "type"),
        (
            "function main(){const a=[\"x\".repeat(1)];const v=a.at(0);a.push(\"y\");console.log(String(v));}",
            "view-conflict",
        ),
        (
            "function main(){const a=[\"x\"];const v=a.at(0);const moved=[v];}",
            "borrowed-view",
        ),
        (
            "function bad(a:string[]):string|undefined{return a.at(0);}function main(){}",
            "borrowed-view",
        ),
        (
            "function take(v:string|undefined):string|undefined{return v;}function main(){const a=[\"x\"];take(a.at(0));}",
            "borrowed-view",
        ),
        (
            "function main(){const a=[\"x\"];const v=a.at(0);if(v!==undefined){v.concat(\"y\");v;}}",
            "borrowed-view",
        ),
        (
            "function main(){const a=[[1]];const v=a.at(0);if(v!==undefined){v.push(2);}}",
            "borrowed-view",
        ),
        (
            "function main(){const a=[\"x\"];const v=a.at(0);const f=():string|undefined=>{return v;};f();}",
            "borrowed-view",
        ),
        (
            "function read(v:string|undefined,n:int):int{return n;}function grow(a:string[]):int{a.push(\"x\");return 0;}function main(){const a=[\"a\"];read(a.at(0),grow(a));}",
            "conflicting-access",
        ),
        ("function main(){const a:int[]|null=null;a.at(0);}", "type"),
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("must reject: {source}"));
        assert_eq!(error.code, code, "{}", error.render("negative.tsr", source));
    }
    for depth in [100, 200] {
        let expr = format!("{}1{}", "[".repeat(depth), "]".repeat(depth));
        let source = format!("function main(){{console.log(String({expr}));}}");
        if depth == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            assert_eq!(check(&source).err().unwrap().code, "unsupported");
        }
        let source = format!(
            "function main(){{const a=[0];console.log({}0{});}}",
            "a[".repeat(depth),
            "]".repeat(depth)
        );
        if depth == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            assert_eq!(check(&source).err().unwrap().code, "unsupported");
        }
    }
}

#[test]
fn at_copy_bits_owned_pointers_explicit_clone_and_disposal() {
    native(
        r#"
        function number(values:number[],index:int):number|undefined{return values.at(index);}
        function borrowed(values:string[],index:int):int{const selected=values.at(index);if(selected!==undefined){return selected.length;}return -1;}
        function owned(index:int):string|undefined{return ["é😀".repeat(1),"kept".repeat(1)].at(index);}
        function copied(values:string[],index:int):string|undefined{return clone(values.at(index));}
        function ints(values:int[],index:int):int|undefined{return values.at(index);}
        function main(){}
    "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicUsize,Ordering};
        struct Track;static LIVE:AtomicUsize=AtomicUsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);
        unsafe impl GlobalAlloc for Track{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(l.size(),Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){LIVE.fetch_sub(l.size(),Ordering::SeqCst);unsafe{System.dealloc(p,l)}}unsafe fn realloc(&self,p:*mut u8,l:Layout,n:usize)->*mut u8{let q=unsafe{System.realloc(p,l,n)};if !q.is_null(){LIVE.fetch_sub(l.size(),Ordering::SeqCst);LIVE.fetch_add(n,Ordering::SeqCst);}q}}
        #[global_allocator]static A:Track=Track;
    "#,
        r#"
        for len in [0,1,4,5,17] {
            let bits=[0u64,1u64<<63,0x7ff8000000000042,0x7ff0000000000000,0xfff0000000000000];
            let values=(0..len).map(|i|f64::from_bits(bits[i%bits.len()])).collect::<Vec<_>>();
            for index in [i64::MIN,-18,-5,-1,0,1,4,17,i64::MAX] {
                let want=if index<0 {len.checked_sub(usize::try_from(index.unsigned_abs()).unwrap_or(usize::MAX))}else{usize::try_from(index).ok().filter(|&i|i<len)};
                match function0(&values,index){Union0::V0(value)=>assert_eq!(value.to_bits(),values[want.unwrap()].to_bits()),Union0::V1(_)=>assert!(want.is_none())}
            }
        }
        std::panic::set_hook(Box::new(|_|{}));
        let baseline=LIVE.load(Ordering::SeqCst);
        for len in [0,1,4,5,17] {
            let mut values=Array::from([]);for _ in 0..len{values.push(String::Owned("é😀".to_owned()));}
            let allocations=ALLOCS.load(Ordering::SeqCst);for index in [-1,0,17]{std::hint::black_box(function1(&values,index));}assert_eq!(ALLOCS.load(Ordering::SeqCst),allocations);
            if len>0 {let original=values.get(0).unwrap().as_ptr();let copy=function3(&values,0);match copy{Union1::V0(value)=>{assert_ne!(value.as_ptr(),original);assert_eq!(value,"é😀");},_=>panic!("present")}}
            drop(values);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        for index in [-1,0,2,i64::MIN] {drop(function2(index));assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let result=std::panic::catch_unwind(||{let values=Array::from([String::Owned("owned".to_owned())]);std::hint::black_box(function1(&values,0));panic!("intentional")});drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#,
    );
}

#[test]
fn nullable_elements_preserve_null_and_out_of_bounds_undefined() {
    let source=support::format_round_trip(r#"import {strictEqual} from 'std:testing';function main(){const values:(int|null)[]=[1,null];strictEqual(values.at(0),1);strictEqual(values.at(1),null);strictEqual(values.at(2),undefined);const words:(string|null)[]=['hello',null];const word=words.at(0);if(word!==null && word!==undefined){strictEqual(word.length,5);}strictEqual(words.at(1),null);strictEqual(words.at(-3),undefined);}"#);
    let directory=support::TempDirectory::new();let path=directory.source(&source);
    support::success(&support::run(support::cli().arg("run").arg(path),120));
}
