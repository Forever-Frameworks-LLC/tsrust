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
    let path = directory.0.join("number-formatting.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("number-formatting-{level}"));
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
fn decimal_precision_ranges_native_rounding_and_layout_are_exact() {
    // No String literal, explicit String type, or other string method may mask
    // runtime selection for an otherwise isolated numeric formatting call.
    for (source, expected) in [
        (
            "function main(){console.log(true.toString());console.log(false.toString());}",
            "true\nfalse\n",
        ),
        ("function main(){console.log(31.toString(16));}", "1f\n"),
        ("function main(){console.log(1.25.toFixed(1));}", "1.2\n"),
        (
            "function main(){console.log(2.5.toExponential(0));}",
            "2e+0\n",
        ),
        (
            "function main(){console.log(9.999.toPrecision(3));}",
            "10.0\n",
        ),
        (
            "function main(){const text=1.25.toFixed(1);console.log(text);}",
            "1.2\n",
        ),
        (
            "function main(){const outputs=[1.25.toFixed(1),2.5.toExponential(0),9.999.toPrecision(3)];for(const output of outputs){console.log(output);}}",
            "1.2\n2e+0\n10.0\n",
        ),
    ] {
        native(source, expected, "", "");
    }
    native(
        r#"
    type Box={text:string;};
    function fixed(value:number,digits:int):string{return value.toFixed(digits);}
    function exponential(value:number,digits:int):string{return value.toExponential(digits);}
    function precision(value:number,digits:int):string{return value.toPrecision(digits);}
    function shortest(value:number):string{return value.toPrecision();}
    function shortestExponential(value:number):string{return value.toExponential();}
    function fixedDefault(value:number):string{return value.toFixed();}
    function main(){
        const whole:number=7;console.log(whole.toFixed());
        console.log(2.5.toFixed());console.log((-2.5).toFixed());console.log(1.25.toFixed(1));
        console.log(3.5.toFixed());console.log(2.5.toExponential(0));console.log(2.5.toPrecision(1));
        console.log((-0.0).toFixed(2));console.log((-0.0).toExponential(2));console.log((-0.0).toPrecision(3));
        console.log((-0.0001).toFixed(2));console.log(0.0.toPrecision());
        console.log(9.999.toPrecision(3));console.log(999.9.toPrecision(3));console.log(0.0000009999.toPrecision(3));
        console.log(1e-6.toPrecision());console.log(1e-7.toPrecision());console.log(1e20.toPrecision());
        console.log(1e21.toPrecision());console.log(1e21.toFixed(100));console.log(1e21.toExponential());
        console.log(Number.NaN.toFixed());console.log(Number.POSITIVE_INFINITY.toExponential(999));
        console.log(Number.NEGATIVE_INFINITY.toPrecision(-1));
        const inferred=1.5.toFixed(1)+2.5.toPrecision(2);console.log(inferred);
        const strings=[1.5.toFixed(1),2.5.toExponential()];console.log(strings[0]);console.log(strings[1]);
        const box:Box={text:1.5.toPrecision(2)};console.log(box.text);
        console.log(1.5.toFixed(1).valueOf().substring(0,1).concat("!").padStart(3,"0"));
    }"#,
        "7\n2\n-2\n1.2\n4\n2e+0\n2\n0.00\n0.00e+0\n0.00\n-0.00\n0\n10.0\n1.00e+3\n0.00000100\n0.000001\n1e-7\n100000000000000000000\n1e+21\n1e+21\n1e+21\nNaN\nInfinity\n-Infinity\n1.52.5\n1.5\n2.5e+0\n1.5\n01!\n",
        "",
        r#"
        // Fixed/scientific digits are supplied by std, including all legal precisions.
        let cells=[0.0,-0.0,2.5,-2.5,1.25,-1.25,1.005,2.675,9.999,1e-7,1e-6,1e20,f64::from_bits(1),-f64::from_bits(1),f64::MIN_POSITIVE];
        for value in cells {
            let zero=if value==0.0{0.0}else{value};
            for digits in 0..=100 {
                assert_eq!(function0(value,digits).as_ref(),format!("{:.*}",digits as usize,zero));
                let raw=format!("{:.*e}",digits as usize,zero);let (mantissa,exponent)=raw.split_once('e').unwrap();let exponent:i32=exponent.parse().unwrap();
                assert_eq!(function1(value,digits).as_ref(),format!("{mantissa}e{exponent:+}"));
            }
            for digits in 1..=100 {
                let result=function2(value,digits);assert!(matches!(result,String::Owned(_)));
                let standard=format!("{:.*e}",digits as usize-1,zero);assert_eq!(result.parse::<f64>().unwrap().to_bits(),standard.parse::<f64>().unwrap().to_bits());
                let significant=result.trim_start_matches('-').split('e').next().unwrap().chars().filter(|c|c.is_ascii_digit()).skip_while(|c|*c=='0').count();
                if value!=0.0 {assert_eq!(significant,digits as usize,"{value} {digits} {result}");}
            }
            assert_eq!(function5(value),function0(value,0));
        }
        for (value,expected) in [(f64::from_bits(1),"5e-324"),(f64::MAX,"1.7976931348623157e+308"),(f64::MIN_POSITIVE,"2.2250738585072014e-308"),(f64::NAN,"NaN"),(f64::INFINITY,"Infinity"),(f64::NEG_INFINITY,"-Infinity")]{assert_eq!(function3(value).as_ref(),expected);}
        assert_eq!(function0(f64::MAX,100).as_ref(),"1.7976931348623157e+308");
        assert_eq!(function1(f64::MAX,2).as_ref(),"1.80e+308");
        assert_eq!(function2(f64::from_bits(1),3).as_ref(),"4.94e-324");
        assert_eq!(function2(f64::MAX,3).as_ref(),"1.80e+308");
        assert_eq!(function2(99.95,3).as_ref(),"100");
        assert_eq!(function2(1e-6,1).as_ref(),"0.000001");assert_eq!(function2(1e-7,1).as_ref(),"1e-7");
        assert_eq!(function2(100.0,3).as_ref(),"100");assert_eq!(function2(100.0,2).as_ref(),"1.0e+2");
        assert_eq!(function4(-0.0).as_ref(),"0e+0");
        "#,
    );
}

#[test]
fn receivers_arguments_contexts_loans_and_selected_temporaries_are_preserved() {
    let analysis = native(
        r#"
    type Box={value:number;};
    function receiver(trace:int[]):number{trace.push(1);return 1.25;}
    function digits(trace:int[]):int{trace.push(2);return 1;}
    function mutate(values:number[],trace:int[]):int{trace.push(3);values[0]=99.0;values.push(17.0);return 1;}
    function display(value:number):string{return value.toFixed(1);}
    function invoke(action:()=>number):string{return action().toPrecision(2);}
    function factory():()=>number{return ():number=>{return 1.25;};}
    function fixed():string{const values=[1.5,2.5];return values[0].toFixed();}
    function main(){
        const trace:int[]=[];console.log(receiver(trace).toFixed(digits(trace)));
        const values=[1.25];console.log(values[0].toFixed(mutate(values,trace)));console.log(values[0]);console.log(values.length);
        for(const n of trace){console.log(n);}
        const captured=1.25;const action=():number=>{return captured;};console.log(invoke(action));console.log(captured);
        const box:Box={value:1.25};const text=():string=>{return box.value.toExponential(1);};console.log(text());console.log(box.value);
        const made=factory();console.log(made().toPrecision(2));console.log(display(1.25));console.log(fixed());
        let value=1.25;console.log(value.toFixed((value=99.0).valueOf()===99.0?1:2));console.log(value);
        for(const flag of [true,false]){
            console.log((flag?1.25:2.5).toFixed(1));
            for(const item of [(flag?1.25:2.5).toPrecision(2)]){console.log(item);}
            console.log("x".concat(flag?1.25.toFixed(1):2.5.toFixed(1)));
        }
    }"#,
        "1.2\n1.2\n99\n2\n1\n2\n3\n1.2\n1.25\n1.2e+0\n1.25\n1.2\n1.2\n2\n1.2\n99\n1.2\n1.2\nx1.2\n2.5\n2.5\nx2.5\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("display.value"), Some(&Mode::Copy));
    assert_eq!(
        analysis.contracts().get("mutate.values"),
        Some(&Mode::Mutate)
    );
    assert!(analysis.emit_rust().contains("[1.5f64, 2.5f64]"));
    let source = "function change(values:number[]):int{values.push(7.0);return 1;}function use(values:number[],text:string):int{return values.length+text.length;}function main(){const values=[1.25];use(values,values[0].toFixed(change(values)));}";
    assert_eq!(check(source).err().unwrap().code, "conflicting-access");
}

#[test]
fn strict_types_arity_range_priority_and_normal_stack_diagnostics_are_bounded() {
    for (source, code) in [
        ("function main(){1.toFixed();}", "type"),
        (
            "function main(){let value:int=1;value.toPrecision();}",
            "type",
        ),
        ("function main(){true.toExponential();}", "type"),
        ("function main(){\"1\".toFixed();}", "type"),
        ("function main(){[1.0].toPrecision();}", "type"),
        ("function main(){1.0.toFixed(2.0);}", "type"),
        ("function main(){1.0.toFixed(true);}", "type"),
        ("function main(){1.0.toExponential(\"2\");}", "type"),
        ("function main(){1.0.toPrecision(1,2);}", "arity"),
        ("function main(){1.0.toFixed(1,2);}", "arity"),
        ("function main(){1.0.toExponential(1,2);}", "arity"),
        ("function main(){1.0.toFixed(,);}", "syntax"),
        ("function main(){1.0.toString(2.5);}", "type"),
    ] {
        let error = support::reject(source, code);
        assert_ne!(error.code, "internal");
        assert!(error.span.end > error.span.start);
    }
    native(
        "function fixed(value:number,d:int):string{return value.toFixed(d);}function exp(value:number,d:int):string{return value.toExponential(d);}function precision(value:number,d:int):string{return value.toPrecision(d);}function main(){}",
        "",
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        for(value,method) in [(1.25,0),(f64::NAN,0),(f64::INFINITY,0),(f64::NEG_INFINITY,0),(1.25,1),(1.25,2)] {
            let invalid: &[i64]=if method==2{&[i64::MIN,-1,0,101,i64::MAX]}else{&[i64::MIN,-1,101,i64::MAX]};
            for &digits in invalid {
                let error=std::panic::catch_unwind(||match method{0=>function0(value,digits),1=>function1(value,digits),_=>function2(value,digits)}).unwrap_err();
                let message=error.downcast_ref::<std::string::String>().map(|s|s.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();
                assert!(message.contains("precision must be between")&&message.contains("source bytes"));
            }
        }
        for (value,expected) in [(f64::NAN,"NaN"),(f64::INFINITY,"Infinity"),(f64::NEG_INFINITY,"-Infinity")]{for digits in [i64::MIN,0,101,i64::MAX]{assert_eq!(function1(value,digits).as_ref(),expected);assert_eq!(function2(value,digits).as_ref(),expected);}}
        std::panic::set_hook(hook);
        "#,
    );
    for count in [48, 200] {
        let source = format!(
            "function main(){{console.log(1.5{}.toFixed());}}",
            ".valueOf()".repeat(count)
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
fn natural_owned_and_static_results_dispose_normally_and_on_unwind() {
    native(
        r#"
        function fixed(value:number,d:int):string{return value.toFixed(d);}
        function exp(value:number,d:int):string{return value.toExponential(d);}
        function precision(value:number,d:int):string{return value.toPrecision(d);}
        function trap():string{return "temporary".repeat(2).concat(1.5.toFixed(-1));}
        function selected(flag:boolean):string{return "x".concat(flag?1.5.toFixed(2):2.5.toPrecision(3));}
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
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for value in [-0.0,1.25,-1.25,f64::from_bits(1),f64::MAX]{for digits in [1,2,17,100]{for method in 0..3{let result=match method{0=>function0(value,digits),1=>function1(value,digits),_=>function2(value,digits)};assert!(matches!(result,String::Owned(_)));assert!(!result.is_empty());drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}}}
        for(value,expected) in [(f64::NAN,"NaN"),(f64::INFINITY,"Infinity"),(f64::NEG_INFINITY,"-Infinity")]{for method in 0..3{ALLOCS.store(0,Ordering::SeqCst);let result=match method{0=>function0(value,2),1=>function1(value,2),_=>function2(value,2)};assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Borrowed(_)));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}}
        let first=function0(1.25,2);let second=function0(1.25,2);assert_ne!(first.as_ptr(),second.as_ptr());drop(first);assert_eq!(second.as_ref(),"1.25");drop(second);
        for flag in [true,false]{let output=function4(flag);assert_eq!(output.as_ref(),if flag{"x1.50"}else{"x2.50"});drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        for method in 0..3{let error=std::panic::catch_unwind(||match method{0=>function0(1.5,-1),1=>function1(1.5,-1),_=>function2(1.5,0)}).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function3).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
