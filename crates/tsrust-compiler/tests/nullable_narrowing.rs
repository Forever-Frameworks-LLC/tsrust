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
    let evidence = std::env::var_os("TSR_GATE85_EVIDENCE_DIR").map(|root| {
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
    let path = directory.0.join("nullable-narrowing.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("nullable-narrowing-{level}"));
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
fn narrowing_guards_residual_sets_contexts_scalar_bits_and_loop_paths() {
    let source = r###"type Snapshot = {value: int | null;};
function n(value: number | null | undefined): number {
    if (value === null) { return 0.0; }
    if (undefined === value) { return 1.0; }
    return value;
}
function clear(holder: Snapshot): int { holder.value = null; return 3; }
function score(value: undefined | int | null | int): int {
    if (null === value) { return 10; }
    if (value === undefined) { return 20; }
    return value + 1;
}
function residual(value: int | null | undefined): int {
    if (value !== null) { if (undefined !== value) { return value + 2; } return 30; }
    return 40;
}
function selected(value: int | null | undefined): int { return value !== null && value !== undefined ? value + 3 : 99; }
function positive(value: int | null | undefined): boolean { return !(value === null || undefined === value) && value >= 0; }
function main() {
    console.log(score(7)); console.log(score(null)); console.log(score(undefined));
    console.log(residual(7)); console.log(residual(null)); console.log(residual(undefined));
    console.log(selected(7)); console.log(selected(null)); console.log(selected(undefined));
    console.log(positive(7)); console.log(positive(null)); console.log(positive(undefined));
    console.log(n(2.5)); console.log(n(null)); console.log(n(undefined));
    let integer: int | null | undefined = 4;
    if (integer !== null && integer !== undefined) { console.log(integer += 3); }
    if (integer !== null && integer !== undefined) { console.log(integer++); }
    if (integer !== null && integer !== undefined) { console.log(++integer); }
    if (integer !== null && integer !== undefined) { console.log(integer); }
    let number: number | null | undefined = 5.5;
    if (number !== null && number !== undefined) { console.log(number *= 2.0); }
    if (number !== null && number !== undefined) { console.log(number--); }
    if (number !== null && number !== undefined) { console.log(++number); }
    if (number !== null && number !== undefined) { console.log(number); }
    const holder: Snapshot = {value: 7};
    if (holder.value !== null) { const result = holder.value + clear(holder); console.log(result); console.log(holder.value === null); }
}
"###;
    let analysis = native(
        source,
        r###"8
10
20
9
40
30
10
99
99
true
false
false
2.5
0
1
7
7
9
9
11
11
11
11
10
true
"###,
        "",
        r#"
        for bits in [0u64,1u64<<63,1,0x8000000000000001,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000042,0x4004000000000000] {
            let number=f64::from_bits(bits);
            assert_eq!(function0(Union1::V0(std::hint::black_box(number))).to_bits(),bits);
        }
        assert_eq!(function0(Union1::V1(TsrNull)),0.0);
        assert_eq!(function0(Union1::V2(TsrUndefined)),1.0);
    "#,
    );
    let views = analysis.projections();
    assert_eq!(views, analysis.projections());
    assert!(views.iter().any(|view| view.ty == "number"));
    assert!(views.iter().any(|view| view.ty == "int"));
    assert!(
        views
            .iter()
            .all(|view| source.get(view.span.start..view.span.end).is_some())
    );
    assert!(views.windows(2).all(
        |pair| (pair[0].span.start, pair[0].span.end) <= (pair[1].span.start, pair[1].span.end)
    ));
    native(
        r###"function main() {
    let value: int | null = 0; let steps = 0;
    while (value !== null) {
        console.log(value + 10); steps = steps + 1;
        if (steps === 2) { value = null; continue; }
        value = value + 1;
    }
    console.log(steps);
    let item: int | null = 2;
    for (let index = 0; index < 3; index++) {
        if (item === null) { console.log(90); break; }
        console.log(item + 20); item = null;
    }
    let current: int | null = 3; let count = 0;
    do {
        if (current === null) { console.log(80); break; }
        console.log(current + 30); current = null; count = count + 1; continue;
    } while (count < 2);
    console.log(count);
    if (current !== null) { console.log(current + 40); } else { console.log(70); }
}
"###,
        r###"10
11
2
22
90
33
80
1
70
"###,
        "",
        "",
    );
    native(
        r#"function main(){let x:int|null=4;if(x!==null){x+=1;x++;console.log(x+1);}let n:number|null=2.5;if(n!==null){n*=2.0;n--;console.log(n+0.5);}let b:boolean|null=false;if(b!==null){b||=true;b&&=false;console.log(b.valueOf());}}"#,
        "7\n4.5\nfalse\n",
        "",
        "",
    );
    native(
        r#"type Holder={value:int|null;other:string|null;};function main(){const h:Holder={value:4,other:"kept"};if(h.value!==null&&h.other!==null){h.value=h.value+1;console.log(String(h.value));console.log(h.other.length);}if(h.value!==null){console.log(h.value+2);h.value=null;console.log(h.value===null);}}"#,
        "5\n4\n7\ntrue\n",
        "",
        "",
    );
    native(
        r#"function missing(value:string|null):null{if(value===null){return value;}return null;}function main(){const x:boolean|null=true;if(x!==null){console.log(x.valueOf());}const y:number|null=2.5;if(y!==null){console.log(y.valueOf());}const owner:string|null="owned".repeat(1);missing(owner);console.log(String(owner));const absent:string|null=null;missing(absent);console.log(String(absent));}"#,
        "true\n2.5\nowned\nnull\n",
        "",
        r#"let _:TsrNull=function0(Union0::V1(TsrNull).tsr_view());"#,
    );
}

#[test]
fn narrowing_owned_payloads_fields_mutation_clones_and_transport_keep_tags_sound() {
    let source = r###"type Inner = { rows: int[][]; label: string | null; };
type Holder = { payload: Inner | null; sibling: string | null; };
function main() {
    let original: string | null | undefined = "echo".repeat(1);
    if (original !== null) {
        const copied: string | undefined = clone(original);
        console.log(String(copied)); console.log(String(original));
        if (copied !== undefined) { console.log(copied.concat("!")); }
        if (original !== undefined) { console.log(original.concat("?")); }
    }
    const missing: string | null | undefined = undefined;
    if (missing !== null) {
        const copiedMissing: string | undefined = clone(missing);
        console.log(typeof copiedMissing); console.log(typeof missing);
    }
    const holder: Holder = {payload: {rows: [[1, 2], [3]], label: "kept".repeat(1)}, sibling: "side".repeat(1)};
    if (holder.payload !== null && holder.sibling !== null) {
        holder.payload.rows[0].push(7);
        holder.payload.rows[1][0] += 4;
        console.log(holder.payload.rows[0].join("|"));
        console.log(holder.payload.rows[1].join("|"));
        console.log(holder.sibling.concat("!"));
        if (holder.payload.label !== null) { console.log(holder.payload.label.concat("!")); }
    }
}
"###;
    let analysis = native(
        source,
        r###"echo
echo
echo!
echo?
undefined
undefined
1|2|7
7
side!
kept!
"###,
        "",
        "",
    );
    let views = analysis.projections();
    assert_eq!(views, analysis.projections());
    assert!(views.iter().any(|view| view.ty == "string | undefined"));
    assert!(views.iter().any(|view| view.ty == "Inner"));
    assert!(
        views
            .iter()
            .any(
                |view| source.get(view.span.start..view.span.end) == Some("holder.sibling")
                    && view.ty == "string"
            )
    );
    native(
        r#"
        type Row={text:string;items:int[];};
        function make(value:string|null|undefined):string|undefined{
            if(value===null){return undefined;}return value;
        }
        function main(){
            const a:string|null|undefined="a".repeat(1);
            if(a!==null){console.log(typeof a);console.log(String(a));console.log(Boolean(a));console.log(a===undefined);}
            const b:string|null|undefined=undefined;
            if(b!==null){console.log(typeof b);console.log(String(b));console.log(Boolean(b));console.log(b===undefined);}
            const moved:string|undefined=make("m".repeat(1));console.log(String(moved));
            const row:Row|null={text:"row".repeat(1),items:[1]};
            if(row!==null){const own:Row=row;console.log(own.text);}
            const c:string|null="guarded".repeat(1);const read=():int=>{if(c===null){return 0;}return c.length;};console.log(read());
        }
    "#,
        "string\na\ntrue\nfalse\nundefined\nundefined\nfalse\ntrue\nm\nrow\n7\n",
        "",
        "",
    );
    native(
        r#"function size(x:string):int{return x.length;}function main(){const x:string|null="abc".repeat(1);if(x!==null){console.log(String(size(x)));console.log(typeof x.length);console.log(Array.isArray(x.concat("!")));}}"#,
        "3\nnumber\nfalse\n",
        "",
        "",
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for id in ["int", "number", "boolean", "string", "array", "record"] {
        let path = root.join(format!("benchmarks/runtime/workloads/narrowing_{id}.tsr"));
        let source = std::fs::read_to_string(path).unwrap();
        check(&source)
            .unwrap_or_else(|e| panic!("{}", e.render("workload.tsr", &source)))
            .emit_rust();
    }
    let source =
        std::fs::read_to_string(root.join("benchmarks/runtime/narrowing-source-order.tsr"))
            .unwrap();
    let expected = std::fs::read_to_string(
        root.join("benchmarks/runtime/narrowing-source-order-expected.txt"),
    )
    .unwrap();
    native(&source, &expected, "", "");
}

#[test]
fn narrowing_invalidations_loans_partial_moves_closures_and_depth_are_sound() {
    support::execute("type Row={text:string;};function main(){const value:Row|null={text:\"x\"};if(value!==null){const text=value.text;console.log(text);}}","x\n",None);
    for (source, code) in [
        (
            "function main(){const x:string|null=null;console.log(x.length);}",
            "type",
        ),
        (
            "function main(){let x:string|null=\"x\";if(x!==null){x=null;console.log(x.length);}}",
            "type",
        ),
        (
            "type Holder={value:string|null;other:int;};function clear(h:Holder):boolean{h.value=null;return true;}function main(){const h:Holder={value:\"x\",other:0};if(h.value!==null&&clear(h)){console.log(h.value.length);}}",
            "type",
        ),
        (
            "type Holder={value:string|null;};function clear(h:Holder):boolean{h.value=null;return true;}function main(){const h:Holder={value:\"x\"};if(!(h.value===null||clear(h))){console.log(h.value.length);}}",
            "type",
        ),
        (
            "type Holder={value:string|null;};function clear(h:Holder):boolean{h.value=null;return true;}function read(s:string,n:boolean):int{return s.length;}function main(){const h:Holder={value:\"x\"};if(h.value!==null){read(h.value,clear(h));}}",
            "conflicting-access",
        ),
        (
            "function value():string|null{return \"x\";}function main(){if(value()!==null){console.log(value().length);}}",
            "type",
        ),
        (
            "type Holder={value:string|null;};function main(){const a:Holder[]=[{value:\"x\"}];if(a[0].value!==null){console.log(a[0].value.length);}}",
            "type",
        ),
        // Moving a narrowed field moves the whole local; later use is rejected.
        (
            "type Row={text:string;};function main(){const value:Row|null={text:\"x\"};if(value!==null){const text=value.text;console.log(text,value);}}",
            "transferred",
        ),
        (
            "function main(){const value:string|null=\"x\";if(value!==null){const read=():int=>{return value.length;};read();}}",
            "type",
        ),
        (
            "function main(){let value:string|null=\"x\";let n=0;if(value!==null){while(n<2){console.log(value.length);value=null;n++;}}}",
            "loop-narrowing",
        ),
        (
            "function main(){let value:string|null=\"x\";if(value!==null){for(let n=0;n<2;n++){console.log(value.length);value=null;}}}",
            "loop-narrowing",
        ),
        (
            "function main(){const value:string|null|undefined=\"x\";if(value!==null){console.log(value.length);}}",
            "type",
        ),
    ] {
        let error = support::reject(source, code);
        assert!(error.span.end > error.span.start);
    }
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
fn narrowing_borrow_copy_projections_do_not_allocate_and_owners_dispose_on_unwind() {
    native(
        r#"
    function scalar(value:int|null):int{if(value===null){return 0;}return value+1;}
    function inspect(value:string|null):int{if(value===null){return 0;}return value.length;}
    function owned():string|null{return "é😀".repeat(2);}
    function panicOwned():int{
        const value:string|null=owned();
        if(value!==null){const words:string[]=[value];const bad=1/0;return bad;}
        return 0;
    }
    function panicBranch():string|null{
        const value:string|null=owned();
        if(value!==null){value.concat("unused");const bad=1/0;return value;}
        return null;
    }
    function main(){}
    "#,
        "",
        ALLOCATION_OBSERVER,
        r#"
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
    let baseline=LIVE.load(Ordering::SeqCst);
    ALLOCS.store(0,Ordering::SeqCst);
    assert_eq!(function0(Union0::V0(std::hint::black_box(7))),8);
    assert_eq!(function0(Union0::V1(TsrNull)),0);
    assert_eq!(ALLOCS.load(Ordering::SeqCst),0);
    let input:Union1=Union1::V0(std::borrow::Cow::Owned(std::string::String::from("é😀")));
    let pointer=match &input{Union1::V0(text)=>text.as_ptr(),_=>unreachable!()};
    ALLOCS.store(0,Ordering::SeqCst);
    assert_eq!(function1(input.tsr_view()),6);assert_eq!(function1(Union1::V1(TsrNull).tsr_view()),0);
    assert_eq!(ALLOCS.load(Ordering::SeqCst),0);
    assert_eq!(match &input{Union1::V0(text)=>text.as_ptr(),_=>unreachable!()},pointer);
    drop(input);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let output=function2();assert_eq!(match &output{Union1::V0(text)=>text.as_ref(),_=>unreachable!()},"é😀é😀");
    drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let error=std::panic::catch_unwind(function3).err().expect("expected panic");drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    let error=std::panic::catch_unwind(function4).err().expect("expected panic");drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    std::panic::set_hook(hook);
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
