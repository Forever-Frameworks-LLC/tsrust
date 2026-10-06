mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, assertions: &str) -> tsrust_compiler::Analysis {
    native_with_observer(source, expected, "", assertions)
}
fn native_with_observer(
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
    let path = directory.0.join("math-approx.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("math-approx-{level}"));
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
        let output = support::run(&mut std::process::Command::new(executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn all_approximate_math_methods_have_native_number_abi_and_mathematical_smoke_cases() {
    let analysis = native(
        "function op0(value:number):number{return Math.acos(value);}function op1(value:number):number{return Math.acosh(value);}function op2(value:number):number{return Math.asin(value);}function op3(value:number):number{return Math.asinh(value);}function op4(value:number):number{return Math.atan(value);}function op5(value:number):number{return Math.atanh(value);}function op6(value:number):number{return Math.cbrt(value);}function op7(value:number):number{return Math.cos(value);}function op8(value:number):number{return Math.cosh(value);}function op9(value:number):number{return Math.exp(value);}function op10(value:number):number{return Math.expm1(value);}function op11(value:number):number{return Math.log(value);}function op12(value:number):number{return Math.log10(value);}function op13(value:number):number{return Math.log1p(value);}function op14(value:number):number{return Math.log2(value);}function op15(value:number):number{return Math.sin(value);}function op16(value:number):number{return Math.sinh(value);}function op17(value:number):number{return Math.tan(value);}function op18(value:number):number{return Math.tanh(value);}function pair(y:number,x:number):number{return Math.atan2(y,x);}function power(base:number,exponent:number):number{return Math.pow(base,exponent);}function three(a:number,b:number,c:number):number{return Math.hypot(a,b,c);}function empty():number{return Math.hypot();}function main(){}",
        "",
        r#"
fn close(actual:f64,expected:f64){assert!(actual.is_finite());assert!((actual-expected).abs()<=2e-14f64*expected.abs().max(1.0),"actual {actual:?}, expected {expected:?}");}
assert_eq!(function0(1.0).to_bits(),0.0f64.to_bits());
assert_eq!(function1(1.0).to_bits(),0.0f64.to_bits());
assert_eq!(function2(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function3(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function4(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function5(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function6(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function7(0.0).to_bits(),1.0f64.to_bits());
assert_eq!(function8(0.0).to_bits(),1.0f64.to_bits());
assert_eq!(function9(0.0).to_bits(),1.0f64.to_bits());
assert_eq!(function10(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function11(1.0).to_bits(),0.0f64.to_bits());
assert_eq!(function12(1.0).to_bits(),0.0f64.to_bits());
assert_eq!(function13(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function14(1.0).to_bits(),0.0f64.to_bits());
assert_eq!(function15(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function16(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function17(0.0).to_bits(),0.0f64.to_bits());
assert_eq!(function18(0.0).to_bits(),0.0f64.to_bits());
close(function0(0.0),1.5707963267948966);close(function2(0.5),0.5235987755982989);close(function3(1.0),0.881373587019543);close(function4(1.0),0.7853981633974483);close(function5(0.5),0.5493061443340548);close(function6(-8.0),-2.0);close(function11(2.0),0.6931471805599453);close(function12(10.0),1.0);close(function14(8.0),3.0);close(function15(0.5),0.479425538604203);close(function18(1.0),0.7615941559557649);close(function19(1.0,1.0),0.7853981633974483);close(function20(2.0,3.0),8.0);close(function20(-2.0,3.0),-8.0);close(function21(3.0,4.0,12.0),13.0);assert_eq!(function22().to_bits(),0.0f64.to_bits());
"#,
    );
    for index in 0..19 {
        assert_eq!(
            analysis.contracts().get(&format!("op{index}.value")),
            Some(&Mode::Copy)
        );
    }
}

#[test]
fn ieee_domains_signed_zero_pow_guards_and_hypot_classification_are_explicit() {
    native(
        "function op0(value:number):number{return Math.acos(value);}function op1(value:number):number{return Math.acosh(value);}function op2(value:number):number{return Math.asin(value);}function op3(value:number):number{return Math.asinh(value);}function op4(value:number):number{return Math.atan(value);}function op5(value:number):number{return Math.atanh(value);}function op6(value:number):number{return Math.cbrt(value);}function op7(value:number):number{return Math.cos(value);}function op8(value:number):number{return Math.cosh(value);}function op9(value:number):number{return Math.exp(value);}function op10(value:number):number{return Math.expm1(value);}function op11(value:number):number{return Math.log(value);}function op12(value:number):number{return Math.log10(value);}function op13(value:number):number{return Math.log1p(value);}function op14(value:number):number{return Math.log2(value);}function op15(value:number):number{return Math.sin(value);}function op16(value:number):number{return Math.sinh(value);}function op17(value:number):number{return Math.tan(value);}function op18(value:number):number{return Math.tanh(value);}function pair(y:number,x:number):number{return Math.atan2(y,x);}function power(base:number,exponent:number):number{return Math.pow(base,exponent);}function three(a:number,b:number,c:number):number{return Math.hypot(a,b,c);}function empty():number{return Math.hypot();}function main(){}",
        "",
        r#"
for function in [function2,function3,function4,function5,function6,function10,function13,function15,function16,function17,function18]{for value in [0.0f64,-0.0]{assert_eq!(function(value).to_bits(),value.to_bits());}assert!(function(f64::NAN).is_nan());}
for function in [function0,function1,function7,function8,function9,function11,function12,function14]{assert!(function(f64::NAN).is_nan());}
for function in [function0,function2]{assert!(function(2.0).is_nan());assert!(function(-2.0).is_nan());assert!(function(f64::INFINITY).is_nan());}
assert!(function1(0.5).is_nan());assert_eq!(function1(f64::INFINITY),f64::INFINITY);assert!(function1(f64::NEG_INFINITY).is_nan());
assert_eq!(function3(f64::INFINITY),f64::INFINITY);assert_eq!(function3(f64::NEG_INFINITY),f64::NEG_INFINITY);
assert!(function5(2.0).is_nan());assert_eq!(function5(1.0),f64::INFINITY);assert_eq!(function5(-1.0),f64::NEG_INFINITY);
for function in [function11,function12,function14]{assert_eq!(function(0.0),f64::NEG_INFINITY);assert_eq!(function(-0.0),f64::NEG_INFINITY);assert!(function(-1.0).is_nan());assert_eq!(function(f64::INFINITY),f64::INFINITY);}
assert_eq!(function13(-1.0),f64::NEG_INFINITY);assert!(function13(-2.0).is_nan());
for function in [function7,function15,function17]{assert!(function(f64::INFINITY).is_nan());assert!(function(f64::NEG_INFINITY).is_nan());}
assert_eq!(function9(f64::NEG_INFINITY).to_bits(),0.0f64.to_bits());assert_eq!(function9(f64::INFINITY),f64::INFINITY);assert_eq!(function10(f64::NEG_INFINITY),-1.0);assert_eq!(function18(f64::INFINITY),1.0);assert_eq!(function18(f64::NEG_INFINITY),-1.0);
for y in [0.0f64,-0.0]{assert_eq!(function19(y,1.0).to_bits(),y.to_bits());}
for base in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY,-0.0,0.0,-1.0,1.0]{for exponent in [-0.0,0.0]{assert_eq!(function20(base,exponent),1.0);}}
for base in [1.0,-1.0,f64::NAN,0.0]{assert!(function20(base,f64::NAN).is_nan());}
for base in [-1.0,1.0]{for exponent in [f64::INFINITY,f64::NEG_INFINITY]{assert!(function20(base,exponent).is_nan());}}
assert_eq!(function20(-0.0,3.0).to_bits(),(-0.0f64).to_bits());assert_eq!(function20(-0.0,2.0).to_bits(),0.0f64.to_bits());assert_eq!(function20(-0.0,-3.0),f64::NEG_INFINITY);assert_eq!(function20(-0.0,-2.0),f64::INFINITY);assert!(function20(-2.0,0.5).is_nan());
assert_eq!(function21(-0.0,0.0,-0.0).to_bits(),0.0f64.to_bits());assert_eq!(function22().to_bits(),0.0f64.to_bits());
for values in [(f64::NAN,f64::INFINITY,1.0),(f64::INFINITY,f64::NAN,1.0),(f64::NAN,1.0,f64::NEG_INFINITY)]{assert_eq!(function21(values.0,values.1,values.2),f64::INFINITY);}assert!(function21(f64::NAN,1.0,2.0).is_nan());
let large=function21(1e308,1e308,0.0);assert!(large.is_finite()&&large>1e308);let tiny=function21(1e-300,1e-300,0.0);assert!(tiny>1e-300);assert!(function21(f64::MAX,f64::MAX,0.0).is_infinite());
"#,
    );
}

#[test]
fn arguments_evaluate_once_left_to_right_before_edge_checks_and_reuse_captures_fields_arrays() {
    let analysis = native(
        r#"
type Box={trace:int;};type Data={value:number;values:number[];};
function next(box:Box,code:int,value:number):number{box.trace=box.trace*10+code;return value;}
function hold(values:number[],result:number):number{return result+Number(values.length);}
function main(){
    const box:Box={trace:0};
    console.log(Math.pow(next(box,1,Number.NaN),next(box,2,0.0)));console.log(box.trace);box.trace=0;
    console.log(Number.isNaN(Math.pow(next(box,1,1.0),next(box,2,Number.NaN))));console.log(box.trace);box.trace=0;
    console.log(Math.hypot(next(box,1,Number.NaN),next(box,2,Number.POSITIVE_INFINITY),next(box,3,4.0)));console.log(box.trace);box.trace=0;
    console.log(Number.isFinite(Math.hypot(next(box,1,3.0),next(box,2,4.0))));console.log(box.trace);box.trace=0;
    console.log(Math.atan2(next(box,1,0.0),next(box,2,1.0)));console.log(box.trace);
    const data:Data={value:8.0,values:[1.0,8.0]};const base=2.0;
    const action=(value:number):number=>{return Math.pow(base,value)+Math.cbrt(data.value);};
    console.log(action(3.0)>9.0);console.log(action(2.0)>5.0);console.log(base);console.log(data.values[1]);
    const output=data.values.map((value:number):number=>{return Math.cbrt(value);});console.log(output[0]>0.5);console.log(output[1]>1.5);console.log(data.values.length);
    console.log(hold(data.values,Math.cbrt(data.values[1]))>3.5);
    const fixed=[2.0,3.0];const output2=Math.pow(fixed[0],fixed[1]);fixed[0]=4.0;console.log(output2>7.5);console.log(fixed[0]);
}"#,
        "1\n12\ntrue\n12\nInfinity\n123\ntrue\n12\n0\n12\ntrue\ntrue\n2\n8\ntrue\ntrue\n2\ntrue\ntrue\n4\n",
        " ",
    );
    assert_eq!(analysis.contracts().get("next.box"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("hold.values"), Some(&Mode::Read));
}

#[test]
fn approximate_math_arity_context_and_existing_loans_remain_static_diagnostics() {
    for name in [
        "acos", "acosh", "asin", "asinh", "atan", "atanh", "cbrt", "cos", "cosh", "exp", "expm1",
        "log", "log10", "log1p", "log2", "sin", "sinh", "tan", "tanh",
    ] {
        for (source, code) in [
            (format!("function main(){{Math.{name}();}}"), "arity"),
            (format!("function main(){{Math.{name}(1,2);}}"), "arity"),
            (format!("function main(){{Math.{name}(true);}}"), "type"),
            (
                format!("function main(){{const value=1;Math.{name}(value);}}"),
                "type",
            ),
        ] {
            let error = check(&source).err().unwrap();
            assert_eq!(error.code, code, "{source}: {}", error.message);
        }
    }
    for name in ["atan2", "pow"] {
        for (source, code) in [
            (format!("function main(){{Math.{name}(1);}}"), "arity"),
            (format!("function main(){{Math.{name}(1,2,3);}}"), "arity"),
            (format!(r#"function main(){{Math.{name}(1,"x");}}"#), "type"),
        ] {
            let error = check(&source).err().unwrap();
            assert_eq!(error.code, code, "{source}: {}", error.message);
        }
    }
    for (source, code) in [
        (r#"function main(){Math.hypot(1,"x");}"#, "type"),
        ("function main(){Math.hypot([1.0]);}", "type"),
        ("function main(){const value=1;Math.pow(value,2);}", "type"),
        ("function main(){Math.sumPrecise([1.0]);}", "unsupported"),
        (
            "function clear(values:number[]):number{values.push(4.0);return 2.0;}function hold(values:number[],value:number):number{return value;}function main(){const values=[1.0];hold(values,Math.pow(values[0],clear(values)));}",
            "conflicting-access",
        ),
        (
            r#"function take(text:string):number{const owned=text;return Number(owned);}function main(){const text=clone("2");Math.pow(take(text),Number(text));}"#,
            "transferred",
        ),
    ] {
        let error = check(source).err().unwrap();
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
    assert!(check("function main(){const value=Math.pow(18446744073709551616,-2);const other=Math.hypot();}").is_ok());
}

#[test]
fn primitive_variadic_arguments_use_no_heap_and_owned_temporaries_dispose_on_success_or_unwind() {
    native_with_observer(
        r#"
        function norm(a:number,b:number,c:number,d:number):number{return Math.hypot(a,b,c,d);}
        function power(a:number,b:number):number{return Math.pow(a,b);}
        function parse(text:string):number{return Number(text);}
        function once():number{return Math.pow(parse(clone("2")),3);}
        function fail():number{const values=[1.0];return Math.hypot(parse(clone("2")),values[8],parse(clone("3")));}
        function main(){}
    "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        struct Observer;static LIVE:AtomicIsize=AtomicIsize::new(0);static CALLS:AtomicUsize=AtomicUsize::new(0);
        unsafe impl GlobalAlloc for Observer{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{CALLS.fetch_add(1,Ordering::SeqCst);let p=unsafe{System.alloc(layout)};if !p.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);}p}
            unsafe fn dealloc(&self,p:*mut u8,layout:Layout){unsafe{System.dealloc(p,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
        }
        #[global_allocator]static OBSERVER:Observer=Observer;
    "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);let calls=CALLS.load(Ordering::SeqCst);
        assert!((function0(3.0,4.0,12.0,0.0)-13.0).abs()<1e-12);assert!((function1(2.0,3.0)-8.0).abs()<1e-12);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);assert_eq!(CALLS.load(Ordering::SeqCst),calls);
        assert!((function3()-8.0).abs()<1e-12);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        assert!(std::panic::catch_unwind(function4).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
    "#,
    );
}
