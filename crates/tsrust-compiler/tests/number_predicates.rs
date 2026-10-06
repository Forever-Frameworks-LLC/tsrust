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
    let path = directory.0.join("number-predicates.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("number-predicates-{level}"));
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
fn number_predicates_match_ieee_categories_and_integer_boundaries() {
    let analysis = native(
        r#"
        function finite(value:number):boolean{return Number.isFinite(value);}
        function nan(value:number):boolean{return Number.isNaN(value);}
        function integer(value:number):boolean{return Number.isInteger(value);}
        function safe(value:number):boolean{return Number.isSafeInteger(value);}
        function main(){
            console.log(Number.isFinite(1));console.log(Number.isFinite(-0.0));
            console.log(Number.isFinite(1.0/0.0));console.log(Number.isNaN(0.0/0.0));
            console.log(Number.isNaN(1));console.log(Number.isInteger(3.5));
            console.log(Number.isInteger(-0));console.log(Number.isInteger(1e308));
            console.log(Number.isSafeInteger(9007199254740991));
            console.log(Number.isSafeInteger(9007199254740992));
            console.log(Number.isSafeInteger(-9007199254740991));
            console.log(Number.isSafeInteger(-9007199254740992));
            console.log(Number.isSafeInteger(9007199254740993));
        }
        "#,
        "true\ntrue\nfalse\ntrue\nfalse\nfalse\ntrue\ntrue\ntrue\nfalse\ntrue\nfalse\nfalse\n",
        r#"
        let limit=9007199254740991.0f64;
        let values=[
            -0.0,0.0,-1.0,1.0,-0.25,0.25,-1.5,1.5,
            f64::MAX,-f64::MAX,f64::MIN_POSITIVE,-f64::MIN_POSITIVE,
            f64::from_bits(1),-f64::from_bits(1),f64::NAN,-f64::NAN,
            f64::from_bits(0x7ff0000000000001),f64::INFINITY,f64::NEG_INFINITY,
            limit,-limit,9007199254740992.0,-9007199254740992.0,
            f64::from_bits(limit.to_bits()-1),f64::from_bits(limit.to_bits()+1),
            f64::from_bits(limit.to_bits()+2),
        ];
        for value in values {
            let finite=!matches!(value.classify(),std::num::FpCategory::Infinite|std::num::FpCategory::Nan);
            let integer=finite && value.trunc()==value;
            assert_eq!(function0(value),finite,"finite {value:?}");
            assert_eq!(function1(value),value!=value,"nan {value:?}");
            assert_eq!(function2(value),integer,"integer {value:?}");
            assert_eq!(function3(value),integer && (-limit..=limit).contains(&value),"safe {value:?}");
        }
        "#,
    );
    for parameter in ["finite.value", "nan.value", "integer.value", "safe.value"] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Copy));
    }
}

#[test]
fn predicates_evaluate_once_and_preserve_copy_capture_loans_and_fixed_arrays() {
    let analysis = native(
        r#"
        type Box={value:number;calls:int;};
        function next(box:Box):number{
            console.log("next");box.calls=box.calls+1;return box.value;
        }
        function observe(flag:boolean,value:number):boolean{return flag;}
        function make(value:number):()=>boolean{
            return ():boolean=>{return Number.isInteger(value);};
        }
        function main(){
            const box:Box={value:1.5,calls:0};
            console.log(Number.isFinite(next(box)));console.log(Number.isNaN(next(box)));
            console.log(Number.isInteger(next(box)));console.log(Number.isSafeInteger(next(box)));
            console.log(box.calls);
            console.log(observe(Number.isInteger(box.value),next(box)));console.log(box.calls);
            const action=make(2);console.log(action());console.log(action());
            const values=[1.5,2];console.log(Number.isInteger(values[0]));
            values[0]=4;console.log(Number.isInteger(values[0]));
            const ints=[1,2,3];console.log(Number.isSafeInteger(Number(ints[1])));
            ints[0]=9;console.log(ints[0]);
            let current:number=1;
            {
                const advance=():boolean=>{
                    current=current+0.5;return Number.isInteger(current);
                };
                console.log(advance());console.log(advance());
            }
            console.log(current);
        }
        "#,
        "next\ntrue\nnext\nfalse\nnext\nfalse\nnext\nfalse\n4\nnext\nfalse\n5\ntrue\ntrue\nfalse\ntrue\ntrue\n9\nfalse\ntrue\n2\n",
        "",
    );
    assert_eq!(analysis.contracts().get("next.box"), Some(&Mode::Mutate));
    for parameter in ["make.value", "observe.flag", "observe.value"] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Copy));
    }
    // Closures conservatively retain ordinary aggregate storage. Check the
    // predicate's fixed-array use separately in a closure-free function.
    let fixed = native(
        r#"
        function main(){
            const ints=[1,2,3];
            console.log(Number.isSafeInteger(Number(ints[1])));
            ints[0]=9;console.log(ints[0]);
        }
        "#,
        "true\n9\n",
        "",
    );
    assert!(fixed.emit_rust().contains(": [i64; 3] = ["));
}

#[test]
fn predicates_require_static_namespace_number_argument_arity_and_existing_loans() {
    for method in ["isFinite", "isNaN", "isInteger", "isSafeInteger"] {
        for arguments in ["", "1,2"] {
            let source = format!("function main(){{Number.{method}({arguments});}}");
            let error = check(&source).err().expect("arity must reject");
            assert_eq!(error.code, "arity", "{source}");
            assert!(error.message.contains("Expected 1 arguments"));
        }
        for argument in ["true", "\"1\"", "[1]"] {
            let source = format!("function main(){{Number.{method}({argument});}}");
            let error = check(&source).err().expect("non-number must reject");
            assert_eq!(error.code, "type", "{source}");
        }
        let source = format!("function main(){{const value=1;Number.{method}(value);}}");
        let error = check(&source)
            .err()
            .expect("int binding requires explicit conversion");
        assert_eq!(error.code, "type");
        assert!(error.message.contains("requires a number argument"));
        check(&format!(
            "function main(){{const value=1;Number.{method}(Number(value));}}"
        ))
        .unwrap();
    }
    for (source, code) in [
        (
            "type Box={n:int;};function main(){Number.isFinite({n:1});}",
            "type",
        ),
        (
            "function main(){const result:number=Number.isFinite(1);}",
            "type",
        ),
        (
            "function main(){const value:number=1;value.isFinite(1);}",
            "unsupported",
        ),
        ("function main(){Number.isUnknown(1);}", "unsupported"),
        (
            "type Box={value:number;};function mutate(box:Box):number{box.value=2;return box.value;}function hold(box:Box,flag:boolean){}function main(){const box:Box={value:1};hold(box,Number.isFinite(mutate(box)));}",
            "conflicting-access",
        ),
        (
            "function main(){let value:number=1;const change=():number=>{value=value+1;return value;};console.log(Number.isInteger(value));}",
            "capture-conflict",
        ),
        (
            "function main(){const values:number[]=[1];const moved=values;Number.isFinite(values[0]);}",
            "transferred",
        ),
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("Expected {code}: {source}"));
        assert_eq!(error.code, code, "{source}");
    }
}
