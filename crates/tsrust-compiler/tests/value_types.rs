mod support;
use tsrust_compiler::{Mode, check};
fn node_native(source: &str) {
    assert_eq!(
        std::process::Command::new("node")
            .arg("--version")
            .output()
            .unwrap()
            .stdout,
        b"v24.11.1\n"
    );
    let typescript = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/runtime/node_modules/typescript");
    let reference=support::run(std::process::Command::new("node").args(["-e","const ts=require(process.argv[1]);eval(ts.transpileModule(process.argv[2],{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText+'\\nmain();');"]).arg(typescript).arg(source),15);
    support::success(&reference);
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("values.tsr", source)));
    let dir = support::TempDirectory::new();
    let input = dir.0.join("values.rs");
    let code = analysis.emit_rust();
    assert!(!code.contains("unsafe"));
    std::fs::write(&input, code).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("values-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-Awarnings",
                    "-C",
                    &format!("opt-level={level}"),
                ])
                .arg(&input)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(binary), 15);
        support::success(&output);
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            String::from_utf8(reference.stdout.clone()).unwrap()
        );
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            String::from_utf8(reference.stderr.clone()).unwrap()
        );
    }
}
#[test]
fn literals_annotations_fields_and_homogeneous_arrays_match_node() {
    node_native(
        r#"type Values={nothing:null;missing:undefined;};function n(value:null):null{return value;}function u(value:undefined):undefined{return value;}function main(){let nothing:null=null;let missing:undefined=undefined;nothing=null;missing=undefined;console.log(n(nothing),u(missing));const values:Values={nothing:null,missing:undefined};console.log(values);console.log([null,null],[undefined,undefined]);console.log([[null],[null]],[[undefined]]);const ns:null[]=[];ns.push(null);console.log(ns.includes(null),ns.indexOf(null),ns.lastIndexOf(null));const us:undefined[]=[undefined];console.log(us.includes(undefined),us.slice(0),us[0]);console.log(values.nothing,values.missing);const local=(undefined:int):int=>{return undefined;};console.log(local(7));}"#,
    );
}
#[test]
fn equality_typeof_void_conversion_and_templates_match_node() {
    node_native(
        r#"function n():null{console.log("n");return null;}function u():undefined{console.log("u");return undefined;}function effect():void{console.log("effect");}function main(){console.log(null===null,undefined===undefined,null===undefined,null!==undefined);console.log(null===0,0===null,undefined===false,"x"!==undefined);console.log(n()===u());const values=[1];console.log(values===null,null!==values,values.length);console.log(typeof null,typeof undefined,typeof 1,typeof 1.5,typeof true,typeof "x",typeof values);console.log(typeof absent,typeof console,typeof Math,typeof Number,typeof n);console.log(typeof effect());console.log(void effect());let count=0;const missing:undefined=void count++;console.log(count,missing);console.log(!null,!undefined,Number(null),Number(undefined));console.log(`null=${null}; undefined=${undefined}`);}"#,
    );
}
#[test]
fn console_defaults_json_and_inspection_match_node() {
    node_native(
        r#"function main(){console.log(null,undefined);console.warn(null,undefined);console.log("%s %d %i %f %j %o %O",null,null,null,null,null,null,null);console.log("%s %d %i %f %j %o %O",undefined,undefined,undefined,undefined,undefined,undefined,undefined);console.log("%j",{a:null,b:undefined});console.log("%j %j",[null],[undefined]);console.table(null);console.table(undefined);console.table([null,null]);console.table([undefined,undefined]);console.table({a:null,b:undefined});console.table([1,2],undefined);console.assert(null);console.assert(undefined);console.assert(false,undefined);console.dir(null);console.dir(undefined);console.dir({a:null,b:undefined},{colors:true});console.dir({a:{b:1}},{depth:null});console.dir(1,undefined);console.dir(1,null);console.count(undefined);console.countReset(undefined);console.count(null);console.countReset("null");}"#,
    );
}
#[test]
fn nullish_callback_returns_captures_and_transformations_match_node() {
    node_native(
        r#"function main(){const value:null=null;const callback=():null=>{return value;};console.log(callback(),value);const ns=[null,null];console.log(ns.map((n:null):undefined=>{return undefined;}));console.log(ns.filter((n:null):boolean=>{return n===null;}));console.log(ns.every((n:null):boolean=>{return n===null;}));console.log(ns.reduce((a:null,b:null):null=>{return a;},null));let us:undefined[]=[undefined];const copy=us.slice();us.push(undefined);console.log(us,copy);}"#,
    );
    let analysis=check("function copy(n:null,u:undefined):void{console.log(n,u); } function main(){copy(null,undefined);}").unwrap();
    assert_eq!(analysis.contracts()["copy.n"], Mode::Copy);
    assert_eq!(analysis.contracts()["copy.u"], Mode::Copy);
}
#[test]
fn nullish_values_do_not_erase_static_errors_or_move_ownership() {
    for source in [
        "function main(){let x:null=undefined;}",
        "function main(){let x:undefined=null;}",
        "function main(){const n=null;n.foo;}",
        "function main(){const n=undefined;n.foo();}",
        "function main(){console.log(null+1);}",
        "function effect():void{} function main(){console.log(effect()===null);}",
        "function main(){let null=1;}",
        "function main(){undefined=1;}",
        "type null={x:int;};function main(){}",
        "function main(){String(():int=>{return 1;});}",
        "function main(){const x=typeof missing.foo;}",
        "function main(){console.log(typeof x);const x=1;}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
    // Mixed absent literals infer a union element type.
    assert!(check("function main(){const n=[null,undefined];}").is_ok());
    let valid = "function main(){let values=[1];console.log(typeof values);values.push(2);console.log(values.length);}";
    assert!(check(valid).is_ok());
}
#[test]
fn array_namespace_and_explicit_conversions_match_node() {
    node_native(
        r#"function effect(value:int):int{console.log(value);return value;}function empty():void{console.log("empty");}function main(){console.log(Array.isArray(),Array.isArray(null),Array.isArray(undefined),Array.isArray([]),Array.isArray([null]),Array.isArray([[1]]),Array.isArray({name:"Pat"}),Array.isArray(effect(3)));const callback=():int=>{return 1;};console.log(Array.isArray(callback),Boolean(callback),typeof callback);console.log(Boolean(),Boolean(null),Boolean(undefined),Boolean(0),Boolean(-0.0),Boolean(Number.NaN),Boolean(""),Boolean("false"),Boolean([]),Boolean({a:1}));console.log(String(),String(null),String(undefined),String(true),String(7),String(1.5),String(-0.0),String(Number.NaN),String(Number.POSITIVE_INFINITY));console.log(String([null,null]),String([undefined]),String([[1,2],[3]]),String([{name:"Pat"}]),String({name:"Ada"}));let text="é".repeat(2);const converted=String(text);text="other";console.log(converted,text);console.log(String(effect(4),effect(5)),Boolean(effect(6),effect(7)),Array.isArray([1],effect(8)));console.log(String(empty()),Boolean(empty()),Array.isArray(empty()));console.log(typeof String,typeof Boolean,typeof Array);}"#,
    );
}
#[test]
fn singleton_payloads_and_arrays_allocate_no_heap() {
    let source = r#"type Empty={nil:null;missing:undefined;};function main(){let ns:null[]=[];let us:undefined[]=[];let records:Empty[]=[];let i=0;while(i<12){ns.push(null);us.push(undefined);records.push({nil:null,missing:undefined});i++;}const copy=clone(ns);const value:null=null;const f=():null=>{return value;};const result=f();const kind=typeof ns;const yes=Array.isArray(us);const no=Boolean(undefined);}"#;
    let mut rust = check(source)
        .unwrap()
        .emit_rust()
        .replace("fn main()", "fn values_main()");
    rust.push_str(r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering};
static VALUE_ALLOCS:AtomicUsize=AtomicUsize::new(0);
struct ValueAllocator;
unsafe impl GlobalAlloc for ValueAllocator {
 unsafe fn alloc(&self,layout:Layout)->*mut u8{VALUE_ALLOCS.fetch_add(1,Ordering::SeqCst);unsafe{System.alloc(layout)}}
 unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)}}
 unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8{VALUE_ALLOCS.fetch_add(1,Ordering::SeqCst);unsafe{System.realloc(pointer,layout,size)}}
}
#[global_allocator]static VALUE_ALLOCATOR:ValueAllocator=ValueAllocator;
fn main(){assert_eq!(std::mem::size_of::<TsrNull>(),0);assert_eq!(std::mem::size_of::<TsrUndefined>(),0);VALUE_ALLOCS.store(0,Ordering::SeqCst);values_main();assert_eq!(VALUE_ALLOCS.load(Ordering::SeqCst),0);}
"#);
    let dir = support::TempDirectory::new();
    let input = dir.0.join("value-allocations.rs");
    std::fs::write(&input, rust).unwrap();
    for level in ["0", "3"] {
        let binary = dir.0.join(format!("values-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-Awarnings",
                    "-C",
                    &format!("opt-level={level}"),
                ])
                .arg(&input)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        support::success(&support::run(&mut std::process::Command::new(binary), 15));
    }
}
#[test]
fn pending_conversion_workloads_have_equivalent_complete_outputs() {
    for (id, arguments) in [
        ("boolean_string_truthiness", "9,7,\"é😀\""),
        ("string_explicit_int", "9,7"),
        ("string_array_conversion", "9,7,\"é😀\""),
    ] {
        let source = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(format!("../../benchmarks/runtime/workloads/{id}.tsr")),
        )
        .unwrap();
        let source = source[..source.find("function main()").unwrap()].to_string()
            + &format!(
                "function main(){{const values=workload({arguments});for(const value of values){{console.log(value);}}}}"
            );
        node_native(&source);
    }
}
