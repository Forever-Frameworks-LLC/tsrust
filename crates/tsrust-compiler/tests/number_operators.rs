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
    let path = directory.0.join("number-operators.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("number-operators-{level}"));
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
fn native_number_bitwise_domains_and_power_ieee_edges() {
    let analysis = native(
        "function band(a:number,b:number):number{return a&b;}function bor(a:number,b:number):number{return a|b;}function bxor(a:number,b:number):number{return a^b;}function bnot(a:number):number{return ~a;}function left(a:number,b:number):number{return a<<b;}function right(a:number,b:number):number{return a>>b;}function unsigned(a:number,b:number):number{return a>>>b;}function power(a:number,b:number):number{return a**b;}function main(){}",
        "",
        r#"
assert_eq!(function0(7.9,3.1),3.0);assert_eq!(function1(4.0,3.0),7.0);assert_eq!(function2(7.0,3.0),4.0);
assert_eq!(function3(0.0),-1.0);assert_eq!(function3(-0.0),-1.0);assert_eq!(function3(4294967295.0),0.0);
for special in [f64::NAN,f64::INFINITY,f64::NEG_INFINITY,0.0,-0.0]{assert_eq!(function0(special,4294967295.0).to_bits(),0.0f64.to_bits());assert_eq!(function1(special,1.0),1.0);assert_eq!(function3(special),-1.0);}
assert_eq!(function1(2147483648.0,0.0),-2147483648.0);assert_eq!(function1(4294967295.0,0.0),-1.0);assert_eq!(function1(4294967296.0,0.0),0.0);assert_eq!(function1(4294967297.0,0.0),1.0);assert_eq!(function1(-4294967297.0,0.0),-1.0);assert_eq!(function1(-1.9,0.0),-1.0);
assert_eq!(function4(1.0,31.0),-2147483648.0);assert_eq!(function4(1.0,32.0),1.0);assert_eq!(function4(1.0,33.0),2.0);assert_eq!(function4(1.0,-1.0),-2147483648.0);assert_eq!(function4(1.0,f64::NAN),1.0);assert_eq!(function4(3.0,1.9),6.0);
assert_eq!(function5(-2147483648.0,31.0),-1.0);assert_eq!(function5(-1.0,1.0),-1.0);assert_eq!(function6(-1.0,0.0),4294967295.0);assert_eq!(function6(-1.0,1.0),2147483647.0);assert_eq!(function6(-1.0,31.0),1.0);assert_eq!(function6(-1.0,32.0),4294967295.0);
for base in [f64::NAN,f64::INFINITY,-0.0,1.0,-1.0]{assert_eq!(function7(base,0.0),1.0);assert_eq!(function7(base,-0.0),1.0);}
assert!(function7(1.0,f64::NAN).is_nan());assert!(function7(-1.0,f64::INFINITY).is_nan());assert!(function7(1.0,f64::NEG_INFINITY).is_nan());assert!(function7(-2.0,0.5).is_nan());
assert_eq!(function7(-0.0,3.0).to_bits(),(-0.0f64).to_bits());assert_eq!(function7(-0.0,2.0).to_bits(),0.0f64.to_bits());assert_eq!(function7(-0.0,-3.0),f64::NEG_INFINITY);
let actual=function7(2.0,0.5);assert!((actual-1.4142135623730951).abs()<2e-14);
"#,
    );
    for name in ["band", "bor", "bxor", "left", "right", "unsigned", "power"] {
        for parameter in ["a", "b"] {
            assert_eq!(
                analysis.contracts().get(&format!("{name}.{parameter}")),
                Some(&Mode::Copy)
            );
        }
    }
}

#[test]
fn operator_precedence_grouping_right_association_and_int_extension_are_preserved() {
    native(
        r#"function main(){
console.log(1 | 2 ^ 3 & 4);console.log(1 << 2 + 1);console.log(16 >> 1 >> 1);
console.log(2 ** 3 ** 2);console.log((2 ** 3) ** 2);console.log((-2) ** 3);console.log(2 ** -2);
console.log(~1);console.log((~1) ** 2);console.log(2 * 3 ** 2);console.log(2 ** 3 * 2);
console.log((1 | 2) === 3.0 && (8 >>> 1) === 4.0);console.log((2 < 3) === true);
const integer=-9223372036854775808;console.log(integer);console.log(7/3);console.log(1+2*3);
}"#,
        "3\n8\n4\n512\n64\n-8\n0.25\n-2\n4\n18\n16\ntrue\ntrue\n-9223372036854775808\n2\n7\n",
        "",
    );
}

#[test]
fn operands_run_once_left_to_right_and_reuse_captures_fields_and_arrays() {
    let analysis = native(
        r#"type Trace={value:int;};type Data={n:number;values:number[];};
function next(trace:Trace,code:int,value:number):number{trace.value=trace.value*10+code;return value;}
function mutate(data:Data):number{data.n=3.0;return data.n;}
function main(){const trace:Trace={value:0};
console.log(next(trace,1,Number.NaN)**next(trace,2,0));console.log(trace.value);trace.value=0;
console.log(next(trace,1,Number.NaN)|next(trace,2,1));console.log(trace.value);trace.value=0;
console.log(next(trace,1,8)>>>next(trace,2,1));console.log(trace.value);
const data:Data={n:2.0,values:[3.0,4.0]};console.log(data.n ** mutate(data));console.log(data.n);
const mask=1.0;const action=(n:number):number=>{return (data.values[0]^mask)|n;};
console.log(action(4));console.log(action(1));console.log(data.values[0]);console.log(mask);
const result=data.values.map((n:number):number=>{return n<<1;});console.log(result[0]);console.log(result[1]);console.log(data.values.length);
const scalar: number = 1 << 3;console.log(scalar);console.log(Number(2) | 1);
}"#,
        "1\n12\n1\n12\n4\n12\n8\n3\n6\n3\n3\n1\n6\n8\n2\n8\n3\n",
        "",
    );
    assert_eq!(analysis.contracts().get("next.trace"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("mutate.data"), Some(&Mode::Mutate));
}

#[test]
fn numeric_operator_types_grammar_depth_and_enclosing_loans_remain_checked() {
    for op in ["&", "|", "^", "<<", ">>", ">>>", "**"] {
        for expression in [
            format!("true {op} 1"),
            format!("1 {op} false"),
            format!("\"x\" {op} 1"),
            format!("integer {op} 1.0"),
            format!("1.0 {op} integer"),
        ] {
            let source = format!("function main(){{const integer=1;console.log({expression});}}");
            let error = check(&source).err().unwrap();
            assert_eq!(error.code, "type", "{source}: {}", error.message);
        }
    }
    for expression in ["~true", "~\"x\"", "~(integer + 1.0)"] {
        let source = format!("function main(){{const integer=1;console.log({expression});}}");
        assert_eq!(check(&source).err().unwrap().code, "type");
    }
    for expression in [
        "-2 ** 2",
        "+2 ** 2",
        "~2 ** 2",
        "!true ** 2",
        "2 ** -2 ** 2",
    ] {
        let source = format!("function main(){{console.log({expression});}}");
        assert_eq!(check(&source).err().unwrap().code, "syntax", "{source}");
    }
    for op in ["&=", "|=", "^=", "<<=", ">>=", ">>>=", "**="] {
        let source = format!("function main(){{const value=1.0;value {op} 1;}}");
        let error = check(&source).err().unwrap();
        assert_eq!(error.code, "unsupported");
        assert_eq!(&source[error.span.start..error.span.end], "value");
    }
    for op in ["&", "**"] {
        let source = format!(
            "type Data={{n:number;}};function mutate(data:Data):number{{data.n=2.0;return data.n;}}function hold(data:Data,n:number){{console.log(data.n);}}function main(){{const data:Data={{n:1.0}};hold(data,data.n {op} mutate(data));}}"
        );
        assert_eq!(check(&source).err().unwrap().code, "conflicting-access");
    }
    let source = r#"function take(text:string):number{const owned=text;return Number(owned);}function main(){const text=clone("2");console.log(take(text) ** Number(text));}"#;
    assert_eq!(check(source).err().unwrap().code, "transferred");
    for count in [100, 200] {
        for expression in [
            format!("{}1", "~".repeat(count)),
            format!("{}1", "1 ** ".repeat(count)),
        ] {
            let source = format!("function main(){{console.log({expression});}}");
            if count == 100 {
                check(&source).unwrap().emit_rust();
            } else {
                assert_eq!(check(&source).err().unwrap().code, "unsupported");
            }
        }
    }
}
