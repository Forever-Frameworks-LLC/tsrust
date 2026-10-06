mod support;
use tsrust_compiler::check;

#[test]
fn logical_operators_return_values_with_truthiness_and_lazy_fallbacks() {
    support::execute(r#"function mark(trace:int[],value:string):string{trace.push(trace.length+1);return value;}function main(){
        const users=['Alice','Bob','Charlie'];const userOne=users.shift();const alice=userOne||'Alice';console.log(alice);
        const empty:string[]=[];console.log(empty.shift()||'Guest');console.log(''||'fallback',' '||'fallback');
        console.log(0||7,2||7,0&&7,2&&7);console.log(false||true,true&&false);
        console.log(0.0||7.0,Number.NaN||7.0,Number.NaN&&7.0);
        const trace:int[]=[];console.log(mark(trace,'first')||mark(trace,'second'));console.log(trace.length);
        console.log(mark(trace,'')&&mark(trace,'skipped'));console.log(trace.length);
        console.log(mark(trace,'')||mark(trace,'last'));console.log(trace.length);
        const a:int|null=null;console.log(a||9,a&&9);
        const b:int|undefined=0;console.log(b||9,b&&9);
        const array:int[]=[];const fallback:int[]=[7];const chosen=array||fallback;console.log(chosen.length,fallback.length);
    }"#, "Alice\nGuest\nfallback  \n7 2 0 7\ntrue false\n7 7 NaN\nfirst\n1\n\n2\nlast\n4\n9 null\n9 0\n0 1\n",None);
}

#[test]
fn nullable_results_borrow_owned_sources_and_preserve_existing_boolean_guards() {
    support::execute(r#"type Row={name:string;};function main(){
        const value:string|undefined='Pat';console.log(value||'Guest');console.log(value);
        const absent:string|undefined=undefined;console.log(absent&&'yes');
        const maybe:Row|null={name:'Pat'};const name=maybe&&maybe.name.toUpperCase();console.log(name);console.log(maybe);
        const nullable:string|null='';const selected=nullable&&'yes';console.log(selected);
        const words=['Pat'];console.log(words.at(0)||'Guest',words[0]);
        const n:int|null=7;if(n!==null&&n>0){console.log(n+1);}
    }"#, "Pat\nPat\nundefined\nPAT\n{ name: 'Pat' }\n\nPat Pat\n8\n",None);
}

#[test]
fn logical_results_keep_transfer_and_unsupported_union_diagnostics() {
    let moved="function main(){const source:string|undefined='Pat';const selected=source||'Guest';console.log(source);}";
    assert_eq!(check(moved).err().unwrap().code,"transferred");
    let borrowed="function main(){const words=['Pat'];const selected=words.at(0)||'Guest';}";
    assert!(check(borrowed).is_err());
    let mixed="function main(){const flag=true;const selected=flag||'Guest';}";
    // Different data types now select into one tagged union value.
    assert!(check(mixed).is_ok());
}

#[test]
fn logical_borrows_and_transfers_preserve_payloads_without_implicit_clones() {
    let source="function width(value:string|undefined,fallback:string):int{return (value||fallback).length;}function choose(value:string|undefined,fallback:string):string{return value||fallback;}function main(){}";
    let analysis=check(source).unwrap();
    let rust=analysis.emit_rust().replace("fn main()","fn source_main()")+r#"
use std::alloc::{GlobalAlloc,Layout,System};
use std::sync::atomic::{AtomicUsize,Ordering};
static ALLOCS:AtomicUsize=AtomicUsize::new(0);
struct CountAlloc;
unsafe impl GlobalAlloc for CountAlloc {
 unsafe fn alloc(&self,layout:Layout)->*mut u8 {ALLOCS.fetch_add(1,Ordering::SeqCst);unsafe{System.alloc(layout)}}
 unsafe fn dealloc(&self,ptr:*mut u8,layout:Layout){unsafe{System.dealloc(ptr,layout)}}
 unsafe fn realloc(&self,ptr:*mut u8,layout:Layout,size:usize)->*mut u8{ALLOCS.fetch_add(1,Ordering::SeqCst);unsafe{System.realloc(ptr,layout,size)}}
}
#[global_allocator] static ALLOCATOR:CountAlloc=CountAlloc;
fn main(){
 let input:Union0=Union0::V0(String::Owned(std::string::String::from("Pat")));
 let fallback=String::Owned(std::string::String::from("Guest"));
 let pointer=match &input{Union0::V0(text)=>text.as_ptr(),_=>unreachable!()};
 ALLOCS.store(0,Ordering::SeqCst);
 assert_eq!(function0(input.tsr_view(),&fallback),3);
 assert_eq!(function0(Union0::V1(TsrUndefined).tsr_view(),&fallback),5);
 let empty:Union0=Union0::V0(String::Borrowed(""));
 assert_eq!(function0(empty.tsr_view(),&fallback),5);
 assert_eq!(ALLOCS.load(Ordering::SeqCst),0);
 assert_eq!(match &input{Union0::V0(text)=>text.as_ptr(),_=>unreachable!()},pointer);
 ALLOCS.store(0,Ordering::SeqCst);
 let selected=function1(input,fallback);
 assert_eq!(selected.as_ref(),"Pat");assert_eq!(selected.as_ptr(),pointer);
 assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(selected);
 let fallback=String::Owned(std::string::String::from("Guest"));let pointer=fallback.as_ptr();
 ALLOCS.store(0,Ordering::SeqCst);
 let selected=function1(Union0::V1(TsrUndefined),fallback);
 assert_eq!(selected.as_ref(),"Guest");assert_eq!(selected.as_ptr(),pointer);
 assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(selected);
}
"#;
    let dir=support::TempDirectory::new();let input=dir.0.join("logical-owned.rs");std::fs::write(&input,rust).unwrap();
    for level in ["0","3"] {
        let binary=dir.0.join(format!("logical-owned-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-Awarnings","-C",&format!("opt-level={level}")]).arg(&input).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),15));
    }
}

#[test]
fn logical_selection_keeps_normal_stack_depth_limits() {
    let source=format!("function main(){{const selected={}7{};console.log(selected);}}","0||(".repeat(48),")".repeat(48));
    check(&source).unwrap().emit_rust();
    let excessive=format!("function main(){{const selected={}7{};}}","0||(".repeat(200),")".repeat(200));
    let error=check(&excessive).err().unwrap();assert_eq!(error.code,"unsupported");assert!(error.message.contains("nesting"));
}
