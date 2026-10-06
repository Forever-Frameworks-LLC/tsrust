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
    let path = directory.0.join("math-exact.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("math-exact-{level}"));
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
fn exact_math_round_sign_fround_and_integer_conversion_preserve_ieee_edges() {
    let analysis = native(
        "function rounded(value:number):number{return Math.round(value);}function sign(value:number):number{return Math.sign(value);}function single(value:number):number{return Math.fround(value);}function count(value:number):number{return Math.clz32(value);}function multiply(left:number,right:number):number{return Math.imul(left,right);}function main(){}",
        "",
        r#"
        let cases:[(f64,f64);15]=[(-0.0,-0.0),(0.0,0.0),(-0.25,-0.0),(0.25,0.0),(-0.5,-0.0),(0.5,1.0),(-1.5,-1.0),(1.5,2.0),(-2.5,-2.0),(2.5,3.0),(-1.75,-2.0),(1.75,2.0),(4503599627370495.5,4503599627370496.0),(-4503599627370495.5,-4503599627370495.0),(9007199254740991.0,9007199254740991.0)];
        for (value,expected) in cases{assert_eq!(function0(value).to_bits(),expected.to_bits(),"round {value:?}");}
        for (value,expected) in [(f64::from_bits(0x3fdfffffffffffff),0.0),(f64::from_bits(0x3fe0000000000001),1.0),(f64::from_bits(0xbfdfffffffffffff),-0.0),(f64::from_bits(0xbfe0000000000001),-1.0),(f64::from_bits(1),0.0),(-f64::from_bits(1),-0.0),(f64::MAX,f64::MAX),(-f64::MAX,-f64::MAX),(f64::INFINITY,f64::INFINITY),(f64::NEG_INFINITY,f64::NEG_INFINITY)]{assert_eq!(function0(value).to_bits(),expected.to_bits());}
        for value in [f64::NAN,-f64::NAN,f64::from_bits(0x7ff0000000000001)]{assert!(function0(value).is_nan());assert!(function1(value).is_nan());assert!(function2(value).is_nan());}
        for (value,expected) in [(-0.0,-0.0f64),(0.0,0.0),(-0.25,-1.0),(0.25,1.0),(f64::NEG_INFINITY,-1.0),(f64::INFINITY,1.0),(-f64::from_bits(1),-1.0),(f64::from_bits(1),1.0)]{assert_eq!(function1(value).to_bits(),expected.to_bits());}
        // Predeclared binary32 nearest-even fixtures, not f32 casts as an oracle.
        for (value,expected) in [(-0.0,-0.0),(0.0,0.0),(16777217.0,16777216.0),(16777219.0,16777220.0),(1.0000000596046448,1.0),(f64::from_bits(0x3ff0000010000001),1.0000001192092896),(f64::from_bits(0x3690000000000000),0.0),(-f64::from_bits(0x3690000000000000),-0.0),(f64::from_bits(0x36a0000000000000),f64::from_bits(0x36a0000000000000)),(f64::MAX,f64::INFINITY),(-f64::MAX,f64::NEG_INFINITY),(f64::INFINITY,f64::INFINITY),(f64::NEG_INFINITY,f64::NEG_INFINITY)]{assert_eq!(function2(value).to_bits(),expected.to_bits(),"fround {value:?}");}
        // Decode exact binary64 integers and reduce their low bits independently
        // of the runtime's floating-point remainder and u32 cast.
        fn low_bits(value:f64)->u32{let bits=value.to_bits();let exponent=((bits>>52)&2047) as i32;if exponent==2047{return 0;}let power=exponent-1023;if power<0{return 0;}let mantissa=(bits&((1u64<<52)-1))|(1u64<<52);let shift=power-52;let unsigned=if shift>=32{0}else if shift>=0{(mantissa as u32)<<shift}else{(mantissa>>(-shift)) as u32};if bits>>63!=0{0u32.wrapping_sub(unsigned)}else{unsigned}}
        let mut values=vec![-0.0,0.0,1.0,-1.0,1.9,-1.9,4294967295.0,4294967296.0,4294967297.0,-4294967297.0,9007199254740991.0,f64::MAX,-f64::MAX,f64::NAN,f64::INFINITY,f64::NEG_INFINITY];
        for exponent in [0u64,1,1000,1022,1023,1054,1075,1100,2046,2047]{for fraction in [0u64,1,(1<<51)+3,(1<<52)-1]{for sign in [0u64,1<<63]{values.push(f64::from_bits(sign|(exponent<<52)|fraction));}}}
        for &left in &values{let word=low_bits(left);let mut x=word;let mut leading=32;while x>0{x/=2;leading-=1;}assert_eq!(function3(left),leading as f64);for &right in &[0.0,1.0,-1.0,2.5,2147483648.0,4294967295.0,f64::INFINITY,f64::NAN]{let product=(u64::from(word)*u64::from(low_bits(right)))&0xffffffff;let signed=if product>=2147483648{product as i64-4294967296}else{product as i64};assert_eq!(function4(left,right),signed as f64);}}
        "#,
    );
    for parameter in [
        "rounded.value",
        "sign.value",
        "single.value",
        "count.value",
        "multiply.left",
        "multiply.right",
    ] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Copy));
    }
}

#[test]
fn exact_math_reductions_bind_all_arguments_once_and_preserve_zero_and_nan() {
    let analysis = native(
        r#"
        type Box={calls:int;};
        function next(box:Box,value:number):number{box.calls=box.calls+1;console.log(box.calls);return value;}
        function minimum(left:number,right:number):number{return Math.min(left,right);}
        function maximum(left:number,right:number):number{return Math.max(left,right);}
        function emptyMinimum():number{return Math.min();}
        function emptyMaximum():number{return Math.max();}
        function main(){
            const box:Box={calls:0};console.log(Math.min(next(box,Number.NaN),next(box,-0.0),next(box,3)));console.log(box.calls);
            console.log(Math.max(next(box,Number.POSITIVE_INFINITY),next(box,Number.NaN),next(box,2)));console.log(box.calls);
            console.log(Math.imul(next(box,-1),next(box,2)));console.log(box.calls);
            let current:number=0;{const advance=():number=>{current=current+0.5;return Math.round(current);};console.log(advance());console.log(advance());}console.log(current);
        }
        "#,
        "1\n2\n3\nNaN\n3\n4\n5\n6\nNaN\n6\n7\n8\n-2\n8\n1\n1\n1\n",
        r#"
        assert_eq!(function3(),f64::INFINITY);assert_eq!(function4(),f64::NEG_INFINITY);
        for (left,right,min,max) in [(-0.0,0.0,-0.0,0.0),(0.0,-0.0,-0.0,0.0),(-0.0,-0.0,-0.0,-0.0),(0.0,0.0,0.0,0.0),(1.0,-2.0,-2.0,1.0),(f64::INFINITY,f64::NEG_INFINITY,f64::NEG_INFINITY,f64::INFINITY)]{assert_eq!(function1(left,right).to_bits(),min.to_bits());assert_eq!(function2(left,right).to_bits(),max.to_bits());}
        for value in [f64::NAN,-f64::NAN]{for other in [-0.0,0.0,1.0,f64::INFINITY,f64::NEG_INFINITY]{assert!(function1(value,other).is_nan());assert!(function1(other,value).is_nan());assert!(function2(value,other).is_nan());assert!(function2(other,value).is_nan());}}
        "#,
    );
    assert_eq!(analysis.contracts().get("next.box"), Some(&Mode::Mutate));
    let emitted = analysis.emit_rust();
    assert!(emitted.contains("tsr_math_extreme(&[math_arg0, math_arg1, math_arg2]"));
    assert!(!emitted.contains("let math_args = vec!"));
    let stable = native(
        "function main(){const values=[1.5,-2.5,3.5];console.log(Math.min(values[0],values[1],values[2]));values[1]=4.5;console.log(Math.max(values[0],values[1],values[2]));}",
        "-2.5\n4.5\n",
        "",
    );
    assert!(stable.emit_rust().contains(": [f64; 3] = ["));
}

#[test]
fn numeric_namespace_constants_have_exact_bits_context_and_ordinary_ownership() {
    native(
        "function constant(index:int):number{return [Math.E,Math.LN10,Math.LN2,Math.LOG10E,Math.LOG2E,Math.PI,Math.SQRT1_2,Math.SQRT2,Number.EPSILON,Number.MAX_VALUE,Number.MIN_VALUE,Number.MAX_SAFE_INTEGER,Number.MIN_SAFE_INTEGER,Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY][index];}function main(){console.log(Number(Math.PI)===Math.PI);console.log(Math.min(Number.MIN_VALUE,1)===Number.MIN_VALUE);console.log(Number.isSafeInteger(Number.MAX_SAFE_INTEGER));}",
        "true\ntrue\ntrue\n",
        r#"
        let expected=[0x4005bf0a8b145769u64,0x40026bb1bbb55516,0x3fe62e42fefa39ef,0x3fdbcb7b1526e50e,0x3ff71547652b82fe,0x400921fb54442d18,0x3fe6a09e667f3bcd,0x3ff6a09e667f3bcd,0x3cb0000000000000,0x7fefffffffffffff,1,0x433fffffffffffff,0xc33fffffffffffff,0x7ff8000000000000,0x7ff0000000000000,0xfff0000000000000];
        for (index,bits) in expected.into_iter().enumerate(){let value=function0(index as i64);if index==13{assert!(value.is_nan());}else{assert_eq!(value.to_bits(),bits,"constant {index}");}}
        assert!(function0(10)<f64::MIN_POSITIVE);
        "#,
    );
}

#[test]
fn exact_math_static_typing_arity_capture_and_constant_write_diagnostics_are_preserved() {
    for method in ["round", "sign", "fround", "clz32"] {
        for args in ["", "1,2"] {
            let source = format!("function main(){{Math.{method}({args});}}");
            assert_eq!(check(&source).err().unwrap().code, "arity");
        }
    }
    for args in ["", "1", "1,2,3"] {
        assert_eq!(
            check(&format!("function main(){{Math.imul({args});}}"))
                .err()
                .unwrap()
                .code,
            "arity"
        );
    }
    for method in ["round", "sign", "fround", "min", "max", "clz32", "imul"] {
        for value in ["true", "\"1\"", "[1]"] {
            let args = if method == "imul" {
                format!("{value},1")
            } else {
                value.to_owned()
            };
            assert_eq!(
                check(&format!("function main(){{Math.{method}({args});}}"))
                    .err()
                    .unwrap()
                    .code,
                "type"
            );
        }
        let args = if method == "imul" { "value,1" } else { "value" };
        let source = format!("function main(){{const value=1;Math.{method}({args});}}");
        let error = check(&source).err().unwrap();
        assert_eq!(error.code, "type");
        assert!(error.message.contains("Number(...)"));
        let args = if method == "imul" {
            "Number(value),1"
        } else {
            "Number(value)"
        };
        check(&format!(
            "function main(){{const value=1;const result:number=Math.{method}({args});}}"
        ))
        .unwrap();
        let args = if method == "imul" { "1,2" } else { "1" };
        assert_eq!(
            check(&format!(
                "function main(){{const result:int=Math.{method}({args});}}"
            ))
            .err()
            .unwrap()
            .code,
            "type"
        );
    }
    for name in [
        "Math.E",
        "Math.LN10",
        "Math.LN2",
        "Math.LOG10E",
        "Math.LOG2E",
        "Math.PI",
        "Math.SQRT1_2",
        "Math.SQRT2",
        "Number.EPSILON",
        "Number.MAX_VALUE",
        "Number.MIN_VALUE",
        "Number.MAX_SAFE_INTEGER",
        "Number.MIN_SAFE_INTEGER",
        "Number.NaN",
        "Number.POSITIVE_INFINITY",
        "Number.NEGATIVE_INFINITY",
    ] {
        check(&format!(
            "function main(){{const result:number={name};console.log(result);}}"
        ))
        .unwrap();
        let error = check(&format!("function main(){{{name}=1;}}"))
            .err()
            .unwrap();
        assert_eq!(error.code, "type");
        assert!(error.message.contains("read-only"));
    }
    for source in [
        "function main(){const f=Math.round;}",
        "function main(){Math.PI();}",
        "function main(){const Math=1;}",
        "function main(){const Number=1;}",
        "function main(){const value:number=1;value.round();}",
        "function main(){Math.random(1,2);}",
        "function main(){const value:int=Math.PI;}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
    for (source, code) in [
        (
            "type Box={value:number;};function mutate(box:Box):number{box.value=2;return box.value;}function hold(box:Box,value:number){}function main(){const box:Box={value:1};hold(box,Math.max(mutate(box),1));}",
            "conflicting-access",
        ),
        (
            "function main(){let value:number=1;const change=():number=>{value=value+1;return value;};console.log(Math.sign(value));}",
            "capture-conflict",
        ),
        (
            "function main(){const values:number[]=[1];const moved=values;Math.min(values[0]);}",
            "transferred",
        ),
    ] {
        assert_eq!(check(source).err().unwrap().code, code);
    }
    for depth in [100, 200] {
        let source = format!("function main(){{Math.round({}1.5);}}", "- ".repeat(depth));
        let result = check(&source);
        if depth == 100 {
            result.unwrap();
        } else {
            assert_eq!(result.err().unwrap().code, "unsupported");
        }
    }
}

#[test]
fn exact_math_variadic_arguments_use_stack_storage_and_dispose_operand_owners() {
    native_with_observer(
        "function maximum(value:number):number{return Math.max(value,-0.0,1,Number.NaN,Number.POSITIVE_INFINITY);}function minimum(value:number):number{return Math.min(value,0,1);}function parsed(input:string):number{const values=[clone(input),\"3\"];return Math.min(Number(values[0]),4);}function main(){}",
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicUsize,Ordering};
        struct Alloc;static ALLOCS:AtomicUsize=AtomicUsize::new(0);static LIVE:AtomicUsize=AtomicUsize::new(0);
        unsafe impl GlobalAlloc for Alloc{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){ALLOCS.fetch_add(1,Ordering::SeqCst);LIVE.fetch_add(1,Ordering::SeqCst);}p}
            unsafe fn dealloc(&self,p:*mut u8,layout:Layout){LIVE.fetch_sub(1,Ordering::SeqCst);unsafe{System.dealloc(p,layout)}}
            unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8{let q=unsafe{System.realloc(p,layout,size)};if !q.is_null(){ALLOCS.fetch_add(1,Ordering::SeqCst);}q}
        }
        #[global_allocator]static ALLOCATOR:Alloc=Alloc;
        "#,
        r#"
        let before=ALLOCS.load(Ordering::SeqCst);for value in [-0.0,0.0,1.5,f64::NAN,f64::INFINITY]{assert!(std::hint::black_box(function0(value)).is_nan());let _=std::hint::black_box(function1(value));}assert_eq!(ALLOCS.load(Ordering::SeqCst),before);
        let baseline=LIVE.load(Ordering::SeqCst);{let input=String::Owned("2".to_owned());let held=LIVE.load(Ordering::SeqCst);assert_eq!(function2(&input),2.0);assert_eq!(LIVE.load(Ordering::SeqCst),held);}assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        "#,
    );
}
