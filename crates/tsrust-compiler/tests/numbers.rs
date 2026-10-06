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
    let path = directory.0.join("numbers.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("numbers-{level}"));
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
fn decimal_scientific_and_contextual_integer_literals_use_f64_without_int_range_leakage() {
    let analysis = native(
        r#"
        function echo(value: number): number { return value; }
        function large(): number { return 9223372036854775808; }
        function zero(): number { return -0; }
        function checked(value: int): int { return value + 1; }
        function main() {
            const first: number = 1; const negative: number = -2;
            console.log(first);console.log(negative);console.log(.5);console.log(1.);console.log(1e3);console.log(1.E-2);console.log(2e+2);
            console.log(echo(3));console.log(+echo(4));console.log(-echo(5));
            console.log(1 + 2.5);console.log(1 + echo(2));console.log(1 < echo(2));console.log(2 === 2.0);
            const rounded: number = 9007199254740993;console.log(rounded);
            console.log(1e999); console.log(-1e999);
            const exact: int = 9223372036854775807; console.log(exact); console.log(-9223372036854775808);
        }
    "#,
        "1\n-2\n0.5\n1\n1000\n0.01\n200\n3\n4\n-5\n3.5\n3\ntrue\ntrue\n9007199254740992\nInfinity\n-Infinity\n9223372036854775807\n-9223372036854775808\n",
        r#"
        assert_eq!(function1().to_bits(),9223372036854775808f64.to_bits());
        assert_eq!(function2().to_bits(),(-0.0f64).to_bits());
        assert!(function0(f64::NAN).is_nan());
        assert_eq!(function0(f64::NEG_INFINITY),f64::NEG_INFINITY);
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        assert!(std::panic::catch_unwind(||function3(i64::MAX)).is_err());
        std::panic::set_hook(hook);
    "#,
    );
    assert_eq!(analysis.contracts().get("echo.value"), Some(&Mode::Copy));
}

#[test]
fn number_arithmetic_and_comparisons_preserve_ieee_nan_infinities_and_signed_zero() {
    native(
        r#"
        function add(a: number,b: number): number {return a+b;}
        function subtract(a: number,b: number): number {return a-b;}
        function multiply(a: number,b: number): number {return a*b;}
        function divide(a: number,b: number): number {return a/b;}
        function remainder(a: number,b: number): number {return a%b;}
        function equal(a: number,b: number): boolean {return a===b;}
        function less(a: number,b: number): boolean {return a<b;}
        function main() {
            const nan=0.0/0.0;console.log(nan===nan);console.log(nan!==nan);
            console.log(nan<0.0);console.log(nan<=0.0);console.log(nan>0.0);console.log(nan>=0.0);
            const positive=1.0/0.0;const negative=-1.0/0.0;console.log(positive>1);console.log(negative<0);
            const zero: number=-0;console.log(1.0/zero);console.log(zero===0);
            console.log(5.5%2);console.log(-5.5%2);console.log(1e308*1e308);
        }
    "#,
        "false\ntrue\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\n-Infinity\ntrue\n1.5\n-1.5\nInfinity\n",
        r#"
        let values=[f64::NAN,f64::INFINITY,f64::NEG_INFINITY,-0.0,0.0,1.0,-5.25,f64::MAX,f64::MIN_POSITIVE];
        for a in values {for b in values {
            for (actual,expected) in [(function0(a,b),a+b),(function1(a,b),a-b),(function2(a,b),a*b),(function3(a,b),a/b),(function4(a,b),a%b)] {
                if expected.is_nan(){assert!(actual.is_nan());}else{assert_eq!(actual.to_bits(),expected.to_bits());}
            }
            assert_eq!(function5(a,b),a==b);assert_eq!(function6(a,b),a<b);
        }}
    "#,
    );
}

#[test]
fn number_records_arrays_nested_ranks_and_captures_use_standard_owned_values() {
    let analysis = native(
        r#"
        type Box={value:number;values:number[];rows:number[][];};
        type Point={x:number;};
        function change(box:Box, value:number){box.value=value;box.values.push(value);}
        function sum(values:number[]):number {let total:number=0;for(const value of values){total=total+value;}return total;}
        function identity(values:number[][][]):number[][][]{return values;}
        function make(value:number):()=>number {return ():number=>{return value+0.5;};}
        function main(){
            const values=[1,2.5,3];values.push(4);values.reverse();console.log(sum(values));console.log(values.remove(0));
            const copied=clone(values);copied[0]=9;console.log(values[0]);console.log(copied[0]);
            const rows=[[],[1,2.5]];rows[0].push(3);console.log(rows[0][0]);console.log(rows[1][1]);
            const cubes=identity([[[1],[2.5]],[]]);cubes[1].push([3]);console.log(cubes[1][0][0]);
            const points:Point[]=[{x:1},{x:2.5}];points[0].x=3;console.log(points[0].x);console.log(points[1].x);
            const box:Box={value:1,values:[],rows:[[2],[3.5]]};change(box,4);console.log(box.value);console.log(box.values[0]);console.log(box.rows[1][0]);
            const action=make(1);console.log(action());console.log(action());
            let current:number=1;const advance=():number=>{current=current+0.25;return current;};console.log(advance());console.log(advance());
            const empty:number[]=[];console.log(empty.length);
            const many:number[]=[0,1,2,3,4,5];many.reverse();console.log(many.remove(0));console.log(many.length);
        }
    "#,
        "10.5\n4\n3\n9\n3\n2.5\n3\n3\n2.5\n4\n4\n3.5\n1.5\n1.5\n1.25\n1.5\n0\n5\n5\n",
        r#"
        let values: Vec<f64>=vec![1.0,2.5];assert_eq!(function1(&values),3.5);
        let nested: Vec<Vec<Vec<f64>>>=vec![vec![vec![-0.0,f64::NAN]]];
        let returned: Vec<Vec<Vec<f64>>>=function2(nested);
        assert_eq!(returned[0][0][0].to_bits(),(-0.0f64).to_bits());assert!(returned[0][0][1].is_nan());
    "#,
    );
    assert_eq!(analysis.contracts().get("sum.values"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("change.box"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("make.value"), Some(&Mode::Copy));
}

#[test]
// includes() finds NaN as documented and as Node does; indexOf() stays strict.
fn float_array_search_finds_nan_with_includes_and_preserves_operand_order_and_loans() {
    native(
        r#"
        type Box={values:number[];n:int;};
        function receiver(box:Box):number[]{console.log("receiver");return clone(box.values);}
        function needle():number{console.log("needle");return 2;}
        function position(box:Box):int{console.log("position");box.values.push(8);return 0;}
        function main(){
            const nan=0.0/0.0;const zero:number=-0;const values:number[]=[nan,zero,1,2,2];
            console.log(values.includes(nan));console.log(values.indexOf(nan));console.log(values.includes(0));console.log(values.indexOf(0));
            console.log(values.includes(2,-1));console.log(values.indexOf(2,4));console.log(values.indexOf(2,999));
            const box:Box={values:[2],n:0};console.log(receiver(box).includes(needle(),position(box)));console.log(box.values.length);
        }
    "#,
        "true\n-1\ntrue\n1\ntrue\n4\n-1\nreceiver\nneedle\nposition\ntrue\n2\n",
        "",
    );
    let source = "function grow(values:number[]):number{values.push(2);return 2;}function main(){const values:number[]=[1];console.log(values.includes(grow(values)));}";
    assert_eq!(check(source).err().unwrap().code, "conflicting-access");
}

#[test]
fn number_parameters_callbacks_reject_implicit_int_conversion_and_wrong_coordinates() {
    native(
        r#"
        function invoke(action:Mutable<Function<number, [value:number]>>,value:number):number{return action(value);}
        function factory(action:Function<number, [value:number]>):(value:number)=>number{return action;}
        function main(){
            const twice=factory((value:number):number=>{return value*2;});console.log(invoke(twice,1));console.log(invoke(twice,2.5));
            let value:number=1;value=value+1;console.log(value);console.log(value);
        }
    "#,
        "2\n5\n2\n2\n",
        "",
    );
    for (source, code) in [
        (
            "function f(value:number){}function main(){const integer=1;f(integer);}",
            "type",
        ),
        (
            "function main(){const integer=1;console.log(integer+1.5);}",
            "type",
        ),
        (
            "function main(){const values:number[]=[1];console.log(values[0.0]);}",
            "type",
        ),
        (
            "function main(){const values:number[]=[1];values.remove(0.0);}",
            "type",
        ),
        (
            "function main(){const values:number[]=[1];values.includes(1,0.0);}",
            "type",
        ),
        ("function main(){const values:int[]=[1.5];}", "type"),
        ("function main(){const integer:int=1.5;}", "type"),
        ("function main(){if(1.0){}}", "type"),
        ("function main(){const number:number=true;}", "type"),
        ("type number={value:int;};function main(){}", "duplicate"),
        (
            "function main(){const values:number[]=[1];const moved=values;console.log(values.length);}",
            "transferred",
        ),
        (
            "function f(action:(value:ReadOnly<number>)=>number){}function main(){}",
            "callback-access",
        ),
        ("function main(){const value=1e+;}", "syntax"),
        ("function main(){const value=1.5__0;}", "syntax"),
        (
            "function main(){const value=9223372036854775808;}",
            "integer-range",
        ),
    ] {
        assert_eq!(
            check(source)
                .err()
                .unwrap_or_else(|| panic!("Expected {code}: {source}"))
                .code,
            code,
            "{source}"
        );
    }
}

#[test]
fn explicit_number_conversion_uses_standard_casts_and_trimmed_f64_parsing() {
    let analysis = native(
        r#"
        function parse(text:string):number{return Number(text);}
        function integer(value:int):number{return Number(value);}
        function boolean(value:boolean):number{return Number(value);}
        function identity(value:number):number{return Number(value);}
        function negativeLiteral():number{return Number(-18446744073709551616);}
        function positiveLiteral():number{return Number(+18446744073709551616);}
        function wrappedNegative():number{return Number(-(18446744073709551616));}
        function wrappedPositive():number{return Number(+(18446744073709551616));}
        function main(){
            console.log(Number());console.log(Number(true));console.log(Number(false));
            const n:int=3;console.log(Number(n)/2);console.log(n);
            console.log(Number(""));console.log(Number(" \t\n"));console.log(Number(" -0 "));
            console.log(Number(" 2.5e1 "));console.log(Number("1e309"));
            const bad=Number("invalid");console.log(bad===bad);
            const hex=Number("0x10");console.log(hex===hex);
            const bom=Number("\uFEFF2");console.log(bom===bom);
            const outside=Number(9223372036854775808);console.log(outside>0);
            console.log(Number(+18446744073709551616)>0);console.log(Number(-(18446744073709551616))<0);
            const text=" 2.5 ";const read=():number=>{return Number(text);};console.log(read());console.log(read());console.log(text);
        }
    "#,
        "0\n1\n0\n1.5\n3\n0\n0\n-0\n25\nInfinity\nfalse\nfalse\nfalse\ntrue\ntrue\ntrue\n2.5\n2.5\n 2.5 \n",
        r#"
        for text in [""," ","\u{0085}2\u{0085}","-0","+0","2.5e1","1e309","inf","Infinity","NaN","0x10","invalid","\u{feff}2"] {
            let expected=if text.trim().is_empty(){0.0}else{text.trim().parse::<f64>().unwrap_or(f64::NAN)};
            for value in [String::Borrowed(text),String::Owned(text.to_owned())] {
                let actual=function0(&value);
                if expected.is_nan(){assert!(actual.is_nan());}else{assert_eq!(actual.to_bits(),expected.to_bits());}
                assert_eq!(value.as_ref(),text);
            }
        }
        for value in [i64::MIN,i64::MAX,-1,0,1,9007199254740993] {assert_eq!(function1(value).to_bits(),(value as f64).to_bits());}
        assert_eq!(function2(false),0.0);assert_eq!(function2(true),1.0);
        for value in [-0.0,0.0,f64::INFINITY,f64::NEG_INFINITY] {assert_eq!(function3(value).to_bits(),value.to_bits());}
        assert!(function3(f64::NAN).is_nan());
        let magnitude=18446744073709551616f64;
        assert_eq!(function4().to_bits(),(-magnitude).to_bits());assert_eq!(function5().to_bits(),magnitude.to_bits());
        assert_eq!(function6().to_bits(),(-magnitude).to_bits());assert_eq!(function7().to_bits(),magnitude.to_bits());
    "#,
    );
    assert_eq!(analysis.contracts().get("parse.text"), Some(&Mode::Read));
    for parameter in ["integer.value", "boolean.value", "identity.value"] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Copy));
    }
}

#[test]
fn number_conversion_evaluates_once_borrows_string_and_checks_arity_and_types() {
    native(
        r#"
        type Box={text:string;};
        function next():string{console.log("once");return " 2.5 ";}
        function change(box:Box):string{console.log("change");box.text="3";return "1";}
        function main(){
            console.log(Number(next()));
            const box:Box={text:"2"};console.log(Number(change(box))+Number(box.text));console.log(box.text);
            const text="7";console.log(Number(text));console.log(text);
        }
    "#,
        "once\n2.5\nchange\n4\n3\n7\n7\n",
        "",
    );
    for (source, code) in [
        ("function main(){Number(1,2);}", "arity"),
        ("function main(){Number([1]);}", "type"),
        ("type Box={n:int;};function main(){Number({n:1});}", "type"),
        ("function main(){Number(console.log(1));}", "type"),
        (
            "function main(){const action=():int=>{return 1;};Number(action);}",
            "type",
        ),
        ("function Number(){}function main(){}", "duplicate"),
        ("function main(){const Number=1;}", "duplicate"),
        ("function f(Number:int){}function main(){}", "duplicate"),
        (
            "function take(text:string):string{return text;}function main(){const text=\"2\";const gone=take(text);Number(text);}",
            "transferred",
        ),
        (
            "type Box={text:string;};function pair(text:string,n:int):string{return clone(text);}function change(box:Box):int{box.text=\"3\";return 0;}function main(){const box:Box={text:\"2\"};Number(pair(box.text,change(box)));}",
            "conflicting-access",
        ),
    ] {
        assert_eq!(
            check(source)
                .err()
                .unwrap_or_else(|| panic!("Expected {code}: {source}"))
                .code,
            code,
            "{source}"
        );
    }
}
