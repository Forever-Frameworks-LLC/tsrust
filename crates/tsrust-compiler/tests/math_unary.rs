mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, assertions: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("math-unary.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("math-unary-{level}"));
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
        let output = support::run(&mut std::process::Command::new(&executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn math_unary_operations_preserve_native_ieee_values_and_signed_zero() {
    let analysis = native(
        r#"
        function absolute(value:number):number{return Math.abs(value);}
        function floor(value:number):number{return Math.floor(value);}
        function ceil(value:number):number{return Math.ceil(value);}
        function trunc(value:number):number{return Math.trunc(value);}
        function squareRoot(value:number):number{return Math.sqrt(value);}
        function main(){
            console.log(Math.abs(-2.5));console.log(Math.floor(-2.5));
            console.log(Math.ceil(-2.5));console.log(Math.trunc(-2.5));
            console.log(Math.sqrt(9));console.log(Math.sqrt(2.25));
            console.log(1+Math.sqrt(4));console.log(Math.sqrt(4)===2);
            console.log(1.0/Math.abs(-0.0));console.log(1.0/Math.sqrt(-0.0));
            console.log(1.0/Math.ceil(-0.25));console.log(1.0/Math.trunc(-0.25));
        }
        "#,
        "2.5\n-3\n-2\n-2\n3\n1.5\n3\ntrue\nInfinity\n-Infinity\n-Infinity\n-Infinity\n",
        r#"
        let nan=f64::NAN;
        let cases=[
            (-0.0,0.0,-0.0,-0.0,-0.0,-0.0),
            (0.0,0.0,0.0,0.0,0.0,0.0),
            (-0.25,0.25,-1.0,-0.0,-0.0,nan),
            (0.25,0.25,0.0,1.0,0.0,0.5),
            (-1.5,1.5,-2.0,-1.0,-1.0,nan),
            (2.25,2.25,2.0,3.0,2.0,1.5),
            (9.0,9.0,9.0,9.0,9.0,3.0),
            (f64::INFINITY,f64::INFINITY,f64::INFINITY,f64::INFINITY,f64::INFINITY,f64::INFINITY),
            (f64::NEG_INFINITY,f64::INFINITY,f64::NEG_INFINITY,f64::NEG_INFINITY,f64::NEG_INFINITY,nan),
            (nan,nan,nan,nan,nan,nan),
            (-nan,nan,nan,nan,nan,nan),
            (f64::from_bits(0x7ff0000000000001),nan,nan,nan,nan,nan),
            (f64::MIN_POSITIVE,f64::MIN_POSITIVE,0.0,1.0,0.0,f64::from_bits(0x2000000000000000)),
            (f64::from_bits(1),f64::from_bits(1),0.0,1.0,0.0,f64::from_bits(0x1e60000000000000)),
            (-f64::from_bits(1),f64::from_bits(1),-1.0,-0.0,-0.0,nan),
            (-f64::MAX,f64::MAX,-f64::MAX,-f64::MAX,-f64::MAX,nan),
        ];
        for (value,abs,floor,ceil,trunc,sqrt) in cases {
            for (actual,expected) in [(function0(value),abs),(function1(value),floor),(function2(value),ceil),(function3(value),trunc),(function4(value),sqrt)] {
                if expected.is_nan(){assert!(actual.is_nan(),"{value:?}");}
                else{assert_eq!(actual.to_bits(),expected.to_bits(),"{value:?}");}
            }
        }
        assert_eq!(function0(f64::MAX),f64::MAX);
        assert_eq!(function1(f64::MAX),f64::MAX);
        assert_eq!(function2(f64::MAX),f64::MAX);
        assert_eq!(function3(f64::MAX),f64::MAX);
        let root=function4(f64::MAX);
        assert!(root.is_finite() && root>0.0);
        assert!(((root / f64::MAX.sqrt())-1.0).abs() <= f64::EPSILON);
        "#,
    );
    for parameter in [
        "absolute.value",
        "floor.value",
        "ceil.value",
        "trunc.value",
        "squareRoot.value",
    ] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Copy));
    }
}

#[test]
fn math_arguments_evaluate_once_with_copied_values_captures_and_fixed_array_reads() {
    let analysis = native(
        r#"
        type Box={value:number;calls:int;};
        function next(box:Box):number{
            console.log("next");box.calls=box.calls+1;return box.value;
        }
        function observe(first:number,second:number):number{return first;}
        function make(value:number):()=>number{
            return ():number=>{return Math.ceil(value);};
        }
        function main(){
            const box:Box={value:4,calls:0};
            console.log(Math.abs(next(box)));console.log(Math.floor(next(box)));
            console.log(Math.ceil(next(box)));console.log(Math.trunc(next(box)));
            console.log(Math.sqrt(next(box)));console.log(box.calls);
            console.log(observe(Math.floor(box.value),next(box)));console.log(box.calls);
            const action=make(1.5);console.log(action());console.log(action());
            const values=[1.5,-2.25];console.log(Math.abs(values[1]));
            console.log(Math.floor(values[0]));values[0]=3.25;console.log(Math.ceil(values[0]));
            let current:number=1;
            {
                const advance=():number=>{
                    current=current+0.25;return Math.floor(current);
                };
                console.log(advance());console.log(advance());
            }
            console.log(current);
        }
        "#,
        "next\n4\nnext\n4\nnext\n4\nnext\n4\nnext\n2\n5\nnext\n4\n6\n2\n2\n2.25\n1\n4\n1\n1\n1.5\n",
        "",
    );
    assert_eq!(analysis.contracts().get("next.box"), Some(&Mode::Mutate));
    for parameter in ["observe.first", "observe.second", "make.value"] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Copy));
    }
    // Closures retain aggregate storage conservatively. Prove the indexed
    // scalar use in a separate function without closures.
    let fixed = native(
        r#"
        function main(){
            const ints=[1,2,3];console.log(Math.abs(Number(ints[0])));
            ints[0]=-9;console.log(Math.abs(Number(ints[0])));
        }
        "#,
        "1\n9\n",
        "",
    );
    assert!(fixed.emit_rust().contains(": [i64; 3] = ["));
}

#[test]
fn math_requires_static_namespace_number_argument_exact_arity_and_existing_loans() {
    for method in ["abs", "floor", "ceil", "trunc", "sqrt"] {
        for arguments in ["", "1,2"] {
            let source = format!("function main(){{Math.{method}({arguments});}}");
            let error = check(&source).err().expect("arity must reject");
            assert_eq!(error.code, "arity", "{source}");
            assert!(error.message.contains("Expected 1 arguments"));
        }
        for argument in ["true", "\"1\"", "[1]"] {
            let source = format!("function main(){{Math.{method}({argument});}}");
            assert_eq!(
                check(&source).err().expect("non-number must reject").code,
                "type",
                "{source}"
            );
        }
        let source = format!("function main(){{const value=1;Math.{method}(value);}}");
        let error = check(&source)
            .err()
            .expect("int binding requires explicit conversion");
        assert_eq!(error.code, "type");
        assert!(error.message.contains("requires a number argument"));
        check(&format!(
            "function main(){{const value=1;Math.{method}(Number(value));}}"
        ))
        .unwrap();
    }
    for (source, code) in [
        (
            "type Box={n:int;};function main(){Math.abs({n:1});}",
            "type",
        ),
        ("function main(){const result:int=Math.floor(1.5);}", "type"),
        (
            "function main(){const result:boolean=Math.sqrt(4);}",
            "type",
        ),
        (
            "function main(){const value:number=1;value.abs(1);}",
            "unsupported",
        ),
        ("function main(){Math.sumPrecise([1.0]);}", "unsupported"),
        ("function main(){const Math=1;}", "duplicate"),
        ("function Math(){}function main(){}", "duplicate"),
        (
            "function read(Math:number):number{return Math;}function main(){}",
            "duplicate",
        ),
        (
            "function main(){const action=(Math:number):number=>{return Math;};}",
            "duplicate",
        ),
        (
            "type Box={value:number;};function mutate(box:Box):number{box.value=2;return box.value;}function hold(box:Box,value:number){}function main(){const box:Box={value:1};hold(box,Math.abs(mutate(box)));}",
            "conflicting-access",
        ),
        (
            "function main(){let value:number=1;const change=():number=>{value=value+1;return value;};console.log(Math.abs(value));}",
            "capture-conflict",
        ),
        (
            "function main(){const values:number[]=[1];const moved=values;Math.abs(values[0]);}",
            "transferred",
        ),
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("Expected {code}: {source}"));
        assert_eq!(error.code, code, "{source}");
    }
}
