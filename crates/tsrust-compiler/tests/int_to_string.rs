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
    let directory = support::TempDirectory::new();
    let path = directory.0.join("int-to-string.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("int-to-string-{level}"));
        support::success(&support::run(
            support::rustc()
                .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(executable), 15);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn all_radices_default_signed_width_and_inferred_contexts_are_exact() {
    native(
        r#"
    type Box={text:string;};
    function formatted(value:int,radix:int):string{return value.toString(radix);}
    function decimal(value:int):string{return value.toString();}
    function main(){
        console.log(0.toString());console.log(123.toString());console.log((-123).toString());
        console.log((-9223372036854775808).toString());console.log(9223372036854775807.toString());
        console.log((-255).toString(16));console.log(255.toString(2));console.log(255.toString(8));console.log(35.toString(36));
        const inferred=15.toString(16)+31.toString(16);console.log(inferred);
        const strings=[7.toString(),15.toString(16)];console.log(strings[0]);console.log(strings[1]);
        const box:Box={text:15.toString(16)};console.log(box.text);
        console.log(31.toString(16).valueOf().substring(1).concat("!").padStart(3,"0"));
        console.log(true.toString());console.log("text".toString());console.log(1.5.valueOf());
    }"#,
        "0\n123\n-123\n-9223372036854775808\n9223372036854775807\n-ff\n11111111\n377\nz\nf1f\n7\nf\nf\n0f!\ntrue\ntext\n1.5\n",
        "",
        r#"
    let cells=[i64::MIN,i64::MIN+1,-123456789012345,-35,-1,0,1,35,12345,i64::MAX];
    for radix in 2..=36 {
        for value in cells {
            let result=function0(value,radix);assert!(matches!(result,String::Owned(_)));
            assert_eq!(i128::from_str_radix(result.as_ref(),radix as u32).unwrap(),value as i128);
            assert!(result.chars().all(|c|c=='-'||c.is_ascii_digit()||c.is_ascii_lowercase()));
            if value<0 {assert!(result.starts_with('-'));}else{assert!(!result.starts_with('-'));}
            let digits=result.trim_start_matches('-');assert!(!digits.is_empty());assert!(digits=="0"||!digits.starts_with('0'));
            if radix==10 {assert_eq!(result,function1(value));assert_eq!(result.as_ref(),value.to_string());}
        }
    }
    assert_eq!(function0(i64::MIN,2).as_ref(),"-1000000000000000000000000000000000000000000000000000000000000000");
    assert_eq!(function0(i64::MIN,16).as_ref(),"-8000000000000000");assert_eq!(function0(i64::MAX,16).as_ref(),"7fffffffffffffff");
    assert_eq!(function0(12345,3).as_ref(),"121221020");assert_eq!(function0(12345,7).as_ref(),"50664");
    assert_eq!(function0(12345,36).as_ref(),"9ix");
    "#,
    );
}

#[test]
fn receiver_radix_order_mutation_loans_captures_and_temporaries_are_preserved() {
    let analysis = native(
        r#"
    type Box={value:int;};
    function receiver(trace:int[]):int{trace.push(1);return 31;}
    function radix(trace:int[]):int{trace.push(2);return 16;}
    function mutate(values:int[],trace:int[]):int{trace.push(3);values[0]=99;values.push(17);return 16;}
    function display(value:int):string{return value.toString();}
    function invoke(action:()=>int):string{return action().toString(16);}
    function factory():()=>int{return ():int=>{return 31;};}
    function fixed():string{const values=[1,2,15];return values[2].toString(16);}
    function main(){
        const trace:int[]=[];console.log(receiver(trace).toString(radix(trace)));
        const values=[8];console.log(values[0].toString(mutate(values,trace)));console.log(values[0]);console.log(values.length);
        for(const n of trace){console.log(n);}
        const captured=31;const action=():int=>{return captured;};console.log(invoke(action));console.log(captured);
        const box:Box={value:31};const text=():string=>{return box.value.toString(16);};console.log(text());console.log(box.value);
        const made=factory();console.log(made().toString(16));
        console.log(display(7));console.log(fixed());
        for(const flag of [true,false]){
            console.log((flag?15:31).toString(16));
            for(const item of [(flag?15:31).toString(16)]){console.log(item);}
            console.log("x".concat((flag?15:31).toString(16)));
        }
    }"#,
        "1f\n8\n99\n2\n1\n2\n3\n1f\n31\n1f\n31\n1f\n7\nf\nf\nf\nxf\n1f\n1f\nx1f\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("display.value"), Some(&Mode::Copy));
    assert_eq!(
        analysis.contracts().get("mutate.values"),
        Some(&Mode::Mutate)
    );
    assert!(analysis.emit_rust().contains("[1i64, 2i64, 15i64]"));
    let source = "function change(values:int[]):int{values.push(7);return 16;}function use(values:int[],text:string):int{return values.length+text.length;}function main(){const values=[31];use(values,values[0].toString(change(values)));}";
    assert_eq!(check(source).err().unwrap().code, "conflicting-access");
}

#[test]
fn strict_types_arity_invalid_radices_and_normal_stack_diagnostics_are_bounded() {
    for (source, code) in [
        ("function main(){1.toString(2,3);}", "arity"),
        ("function main(){1.toString(2.0);}", "type"),
        ("function main(){1.toString(true);}", "type"),
        ("function main(){1.toString(\"16\");}", "type"),
        ("function main(){1.5.toString(2,3);}", "arity"),
        ("function main(){[1].toString(1);}", "arity"),
        ("function main(){true.toString(16);}", "arity"),
        ("function main(){\"x\".toString(16);}", "arity"),
        ("function main(){1.valueOf(16);}", "arity"),
        ("function main(){1.toString(,);}", "syntax"),
    ] {
        let error = support::reject(source, code);
        assert_ne!(error.code, "internal");
        assert!(error.span.end > error.span.start);
    }
    native(
        "function format(value:int,radix:int):string{return value.toString(radix);}function main(){}",
        "",
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        for radix in [i64::MIN,-1,0,1,37,i64::MAX] {
            let error=std::panic::catch_unwind(||function0(7,radix)).unwrap_err();
            let message=error.downcast_ref::<std::string::String>().map(|s|s.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();
            assert!(message.contains("radix must be between 2 and 36")&&message.contains("source bytes"));
        }
        std::panic::set_hook(hook);
    "#,
    );
    for count in [48, 200] {
        let source = format!(
            "function main(){{console.log(7.toString(){});}}",
            ".toString()".repeat(count)
        );
        let directory = support::TempDirectory::new();
        let output = support::run(
            support::cli().arg("emit").arg(directory.source(&source)),
            15,
        );
        // Deep chains compile through the CLI, which has no nesting limit.
        support::success(&output);
    }
}

#[test]
fn natural_owned_results_have_one_allocation_and_zero_normal_unwind_residue() {
    native(
        r#"
    function format(value:int,radix:int):string{return value.toString(radix);}
    function decimal(value:int):string{return value.toString();}
    function trap():string{return "temporary".repeat(2).concat(7.toString(1));}
    function main(){}
    "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Allocations;
        unsafe impl GlobalAlloc for Allocations{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let pointer=unsafe{System.alloc(layout)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}pointer}
            unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
            unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8{unsafe{System.realloc(pointer,layout,size)}}
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;


        fn measured(value:i64,radix:i64){
            let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);let result=function0(value,radix);
            assert!(matches!(result,String::Owned(_)));assert_eq!(i128::from_str_radix(result.as_ref(),radix as u32).unwrap(),value as i128);
            assert_eq!(ALLOCS.load(Ordering::SeqCst),1);drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),before);
        }
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for value in [i64::MIN,-1,0,1,i64::MAX]{for radix in 2..=36{measured(value,radix);}}
        let first=function0(31,16);let second=function0(31,16);assert_ne!(first.as_ptr(),second.as_ptr());drop(first);assert_eq!(second.as_ref(),"1f");drop(second);
        for value in [0,i64::MIN,i64::MAX]{ALLOCS.store(0,Ordering::SeqCst);let result=function1(value);assert_eq!(ALLOCS.load(Ordering::SeqCst),1);drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        for radix in [i64::MIN,1,37,i64::MAX]{let error=std::panic::catch_unwind(||function0(7,radix)).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function2).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
