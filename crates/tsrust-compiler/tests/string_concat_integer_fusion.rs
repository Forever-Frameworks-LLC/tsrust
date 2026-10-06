mod support;
use std::sync::atomic::{AtomicUsize, Ordering};
use tsrust_compiler::check;
static CASE: AtomicUsize = AtomicUsize::new(0);
fn native(source: &str, expected: &str, observer: &str, assertions: &str) -> String {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    // An evidence run names its artifact directory; an ordinary test run uses
    // a scratch directory under the system temporary directory.
    let root = std::env::var_os("TSR_FUSION_NATIVE_DIR").map_or_else(
        || std::env::temp_dir().join(format!("tsrust-fusion-native-{}", std::process::id())),
        std::path::PathBuf::from,
    );
    let dir = root.join(format!("case-{}", CASE.fetch_add(1, Ordering::SeqCst)));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("input.tsr"), source).unwrap();
    std::fs::write(dir.join("expected.txt"), expected).unwrap();
    std::fs::write(dir.join("generated.rs"), &rust).unwrap();
    for level in ["0", "3"] {
        let exe = dir.join(format!("native-{level}"));
        let build = support::run(
            support::rustc()
                .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                .arg(dir.join("generated.rs"))
                .arg("-o")
                .arg(&exe),
            60,
        );
        std::fs::write(dir.join(format!("build-{level}.stderr")), &build.stderr).unwrap();
        support::success(&build);
        let out = support::run(&mut std::process::Command::new(&exe), 15);
        std::fs::write(dir.join(format!("stdout-{level}.txt")), &out.stdout).unwrap();
        std::fs::write(dir.join(format!("stderr-{level}.txt")), &out.stderr).unwrap();
        support::success(&out);
        assert_eq!(out.stdout, expected.as_bytes());
        assert!(out.stderr.is_empty());
    }
    emitted
}
const ALLOC: &str = r#"
struct Tracker;
static ALLOCS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static LIVE:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
unsafe impl std::alloc::GlobalAlloc for Tracker{
 unsafe fn alloc(&self,l:std::alloc::Layout)->*mut u8{let p=unsafe{std::alloc::System.alloc(l)};if !p.is_null(){ALLOCS.fetch_add(1,std::sync::atomic::Ordering::Relaxed);LIVE.fetch_add(l.size(),std::sync::atomic::Ordering::Relaxed);}p}
 unsafe fn dealloc(&self,p:*mut u8,l:std::alloc::Layout){LIVE.fetch_sub(l.size(),std::sync::atomic::Ordering::Relaxed);unsafe{std::alloc::System.dealloc(p,l)}}
 unsafe fn realloc(&self,p:*mut u8,l:std::alloc::Layout,n:usize)->*mut u8{let q=unsafe{std::alloc::System.realloc(p,l,n)};if !q.is_null(){LIVE.fetch_sub(l.size(),std::sync::atomic::Ordering::Relaxed);LIVE.fetch_add(n,std::sync::atomic::Ordering::Relaxed);}q}
}
#[global_allocator]static TRACKER:Tracker=Tracker;
"#;
#[test]
fn exact_integer_extremes_unicode_and_other_conversion_paths() {
    let e = native(
        r#"function render(n:int):string{return String(n).concat(":",String(n),"é😀\u0000");}function main(){console.log(render(0),render(-9223372036854775808),render(9223372036854775807));console.log("".concat(String(-7)),String(9).concat(),"x".concat(String(true),String(null),String(undefined),String(-0.0)));const kept=String(4);console.log(kept.concat("!"),kept);}"#,
        "0:0é😀\0 -9223372036854775808:-9223372036854775808é😀\0 9223372036854775807:9223372036854775807é😀\0\n-7 9 xtruenullundefined0\n4! 4\n",
        "",
        "",
    );
    assert!(e.contains("concat_operand0: i64"));
    assert!(e.contains("checked_ilog10"));
    assert!(e.contains("tsr_string_concat(string_receiver"));
}
#[test]
fn receiver_and_arguments_snapshot_once_before_later_effects() {
    native(
        r#"type Box={n:int;};function change(box:Box):string{box.n=9;return "!";}function changeArray(values:int[]):string{values[0]=9;return "!";}function receiver(trace:int[]):string{trace.push(0);return "R".repeat(1);}function first(trace:int[]):int{trace.push(1);return 7;}function extra(trace:int[]):int{trace.push(2);return 8;}function last(trace:int[]):string{trace.push(3);return "L".repeat(1);}function main(){const trace:int[]=[];console.log(receiver(trace).concat(String(first(trace),extra(trace)),last(trace)),trace.join());let i=0;console.log("x".concat(String(i++,i++),":",String(i++),String(i)),i);const box:Box={n:3};console.log(String(box.n).concat(change(box)),box.n);box.n=4;console.log("x".concat(String(box.n),change(box)),box.n);const values=[5];console.log(String(values[0]).concat(changeArray(values)),values[0]);let nullable:int|null=6;if(nullable!==null){console.log(String(nullable).concat(String(nullable+=1)),nullable);}let empty=0;console.log(String(empty++,empty++).concat(),empty);}"#,
        "R7L 0,1,2,3\nx0:23 3\n3! 9\nx4! 9\n5! 9\n67 7\n0 2\n",
        "",
        "",
    );
}
#[test]
fn one_output_allocation_preserves_read_owners_and_unwind_disposal() {
    let e = native(
        r#"function fused(input:string,n:int):string{return input.concat(":",String(n),":",String(n));}function ordinary(input:string,n:int):string{const a=String(n);const b=String(n);return input.concat(":",a,":",b);}function receiver(n:int):string{return String(n).concat();}function fail(values:int[]):string{return "R".repeat(1).concat(String(values[0]),"side".repeat(1),String(values[9]));}function main(){}"#,
        "",
        ALLOC,
        r#"
 use std::sync::atomic::Ordering::Relaxed;
 let input=String::Borrowed("é-source");let base=LIVE.load(Relaxed);
 ALLOCS.store(0,Relaxed);let fused=function0(&input,i64::MIN);assert_eq!(fused.as_ref(),"é-source:-9223372036854775808:-9223372036854775808");assert_eq!(ALLOCS.load(Relaxed),1);assert_ne!(fused.as_ptr(),input.as_ptr());drop(fused);assert_eq!(LIVE.load(Relaxed),base);
 ALLOCS.store(0,Relaxed);let ordinary=function1(&input,i64::MIN);assert_eq!(ordinary.as_ref(),"é-source:-9223372036854775808:-9223372036854775808");assert_eq!(ALLOCS.load(Relaxed),3);drop(ordinary);assert_eq!(LIVE.load(Relaxed),base);
 ALLOCS.store(0,Relaxed);let value=function2(i64::MAX);assert_eq!(value.as_ref(),"9223372036854775807");assert_eq!(ALLOCS.load(Relaxed),1);drop(value);assert_eq!(LIVE.load(Relaxed),base);
 let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Relaxed);let values=NumericArray::from([7]);let failed=std::panic::catch_unwind(||function3(&values));assert!(failed.is_err());drop(failed);assert_eq!(LIVE.load(Relaxed),baseline);std::panic::set_hook(hook);
 "#,
    );
    assert!(e.find("concat_operand3:").unwrap() < e.find("let mut concat_capacity").unwrap());
}
#[test]
fn checked_target_types_loans_shadowing_and_normal_depth_stay_unchanged() {
    for source in [
        "function main(){\"x\".concat(7);}",
        "function main(){String().concat(1);}",
        "type Box={text:string;};function change(b:Box):int{b.text=\"changed\";return 2;}function main(){const b:Box={text:\"owner\".repeat(1)};b.text.concat(String(change(b)));}",
    ] {
        let e = check(source).err().expect(source);
        assert!(matches!(e.code, "type" | "conflicting-access"));
    }
    native(
        r#"function stringify(n:int):string{return "user";}function main(){console.log("x".concat(stringify(7)),"x".concat(String(3.5)));}"#,
        "xuser x3.5\n",
        "",
        "",
    );
    let shadow = "function main(){const String=(n:int):string=>{return \"shadow\";};String(7).concat(\"!\");}";
    let error = check(shadow)
        .err()
        .expect("reserved namespace remains rejected");
    assert_eq!(error.code, "duplicate");
    assert_eq!(&shadow[error.span.start..error.span.end], "String");
    // Exact existing String and concat catalog examples, with isolated bindings.
    native(
        r####"function main(){{console.log("Hello".concat(" ", "world"));}{console.log(String(null),String(undefined),String([null,null]),String(-0.0));}{const values:int[][]=[[1,2],[],[3]];console.log(String(values));console.log(values[0][0]);}{const values:number[]=[-0.0,0.0/0.0,1.0/0.0,-1.0/0.0];console.log(String(values));}{const values:int[]=[];console.log(String(values));}{let value:string|null|undefined="text";console.log(String(value));value=null;console.log(String(value));value=undefined;console.log(String(value));}}"####,
        r####"Hello world
null undefined , 0
1,2,,3
1
0,NaN,Infinity,-Infinity

text
null
undefined
"####,
        "",
        "",
    );
    let source = format!(
        "function main(){{const x={}String(7){};}}",
        "\"x\".concat(".repeat(100),
        ")".repeat(100)
    );
    check(&source).unwrap().emit_rust();
    let source = format!(
        "function main(){{const x={}String(7){};}}",
        "\"x\".concat(".repeat(200),
        ")".repeat(200)
    );
    assert_eq!(check(&source).err().unwrap().code, "unsupported");
}

#[test]
fn selected_copy_views_are_snapshots_before_later_parent_mutation() {
    native(
        r#"type Row={n:int;};type Holder={n:int|null;};function change(row:Row):string{row.n=9;return "!";}function changeHolder(holder:Holder):string{holder.n=9;return "!";}function grow(values:int[]):string{values.push(9);values[0]=8;return "!";}function main(){const row:Row={n:4};console.log(String(row?.n).concat(change(row)),row.n);const holder:Holder={n:6};if(holder.n!==null){console.log("p".concat(String(holder.n),changeHolder(holder)),String(holder.n));}const values=[5];console.log(String(values.at(0)??0).concat(grow(values)),values.join());let n=1;const extra:Row={n:7};console.log(String(n++,extra.n).concat(change(extra)),n,extra.n);}"#,
        "4! 9\np6! 9\n5! 8,9\n1! 2 9\n",
        "",
        "",
    );
}
