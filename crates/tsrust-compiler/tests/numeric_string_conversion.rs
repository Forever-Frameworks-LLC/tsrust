mod support;
#[test]
fn complete_numeric_text_converts_with_shared_scalar_union_and_enum_rules() {
    let directory = support::TempDirectory::new();
    let path = directory.source(include_str!("fixtures/numeric-string-conversion/values.tsr"));
    let output = support::run(support::cli().arg("run").arg(path), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn repeated_string_identity_hint_is_bounded() {
    let directory=support::TempDirectory::new();
    let source=format!("function main(){{const text=\"1\"{};console.log(Number(text));}}", ".valueOf()".repeat(48));
    let output=support::run(support::cli().arg("emit").arg(directory.source(&source)),15);
    support::success(&output);
}


#[test]
fn integer_string_conversion_preserves_unions_order_and_independent_mutation() {
    let source=r#"
import {strictEqual} from 'std:testing';
type Box={text:string;};
function direct(value:int):string{return String(value);}
function optional(value:int|undefined):string{return String(value);}
function mixed(value:int|string|boolean|null):string{return String(value);}
function once(trace:int[]):int{trace.push(1);return 9;}
function main(){
 for(const value of [-10,-1,0,1,2,3,4,5,6,7,8,9,10,12345]){
  strictEqual(direct(value),value.toString());
  strictEqual(optional(value),value.toString());
  strictEqual(mixed(value),value.toString());
 }
 strictEqual(direct(-9223372036854775808),'-9223372036854775808');
 strictEqual(direct(9223372036854775807),'9223372036854775807');
 strictEqual(optional(undefined),'undefined');strictEqual(mixed(null),'null');
 strictEqual(mixed(true),'true');strictEqual(mixed('text'),'text');
 let first=direct(3);const second=direct(3);first+='x';
 strictEqual(first,'3x');strictEqual(second,'3');
 const box:Box={text:direct(6)};const saved=clone(box);box.text+='x';
 strictEqual(box.text,'6x');strictEqual(saved.text,'6');
 const values=[direct(4),direct(4)];values[0]+='x';
 strictEqual(values[0],'4x');strictEqual(values[1],'4');
 const captured=direct(5);const read=():string=>{return captured;};strictEqual(read(),'5');
 const trace:int[]=[];strictEqual(String(once(trace)),'9');strictEqual(trace.length,1);
}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);assert!(output.stdout.is_empty());
}


#[test]
fn discarded_integer_string_conversion_selects_runtime_support() {
    let source=support::format_round_trip("function main(){String(7);}");
    let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();
    assert!(emitted.contains("pub fn tsr_int_text("));
    assert!(!emitted.contains("unsafe"));
    let directory=support::TempDirectory::new();
    let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);
    support::success(&output);assert!(output.stdout.is_empty());
}


#[test]
fn number_string_conversion_preserves_specials_enums_and_independent_mutation() {
    let source=r#"
import {strictEqual} from 'std:testing';
enum Phase{Ready=0,Busy=4,Last=9,Fraction=0.5,Far=17}
type Box={text:string;};
function direct(value:number):string{return String(value);}
function optional(value:number|undefined):string{return String(value);}
function mixed(value:number|string|boolean|null):string{return String(value);}
function phase(value:Phase):string{return String(value);}
function maybePhase(value:Phase|null):string{return String(value);}
function once(trace:int[]):number{trace.push(1);return 7.0;}
function specialOnce(trace:int[]):number{trace.push(2);return Number.NaN;}
function main(){
 for(const value of [-10.0,-1.0,-0.0,0.0,1.0,2.0,3.0,4.0,5.0,6.0,7.0,8.0,9.0,10.0,0.5,-2.5,17.0,1e-7,1e21,Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY]){
  strictEqual(direct(value),value.toString());strictEqual(optional(value),value.toString());strictEqual(mixed(value),value.toString());
 }
 const negativeZero=-0.0;strictEqual(direct(negativeZero),'0');strictEqual(Object.is(negativeZero,-0.0),true);
 strictEqual(phase(Phase.Ready),'0');strictEqual(phase(Phase.Busy),'4');strictEqual(phase(Phase.Last),'9');
 strictEqual(phase(Phase.Fraction),'0.5');strictEqual(phase(Phase.Far),'17');
 strictEqual(maybePhase(Phase.Busy),'4');strictEqual(maybePhase(null),'null');
 strictEqual(optional(undefined),'undefined');strictEqual(mixed(null),'null');strictEqual(mixed(true),'true');strictEqual(mixed('text'),'text');
 let first=direct(4.0);const second=direct(4.0);first+='x';strictEqual(first,'4x');strictEqual(second,'4');
 const box:Box={text:direct(6.0)};const saved=clone(box);box.text+='x';strictEqual(box.text,'6x');strictEqual(saved.text,'6');
 const values=[direct(8.0),direct(8.0)];values[0]+='x';strictEqual(values[0],'8x');strictEqual(values[1],'8');
 for(const value of [Number.NaN,Number.POSITIVE_INFINITY,Number.NEGATIVE_INFINITY]){
  let first=direct(value);const second=mixed(value);first+='x';strictEqual(first,value.toString()+'x');strictEqual(second,value.toString());
  const box:Box={text:optional(value)};const saved=clone(box);box.text+='x';strictEqual(saved.text,value.toString());
 }
 const trace:int[]=[];strictEqual(String(once(trace)),'7');strictEqual(trace.length,1);
 strictEqual(String(specialOnce(trace)),'NaN');strictEqual(trace.length,2);
}
"#;
    let directory=support::TempDirectory::new();
    let path=directory.source(&support::format_round_trip(source));
    let output=support::run(support::cli().arg("run").arg(path),60);
    support::success(&output);assert!(output.stdout.is_empty());
}

#[test]
fn discarded_number_string_conversion_selects_its_existing_support() {
    for input in ["function main(){String(Number(7));}","function main(){String(1.25);}","function main(){String(Number.NaN);String(Number.POSITIVE_INFINITY);String(Number.NEGATIVE_INFINITY);}"] {
        let source=support::format_round_trip(input);
        let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();
        assert!(!emitted.contains("unsafe"));
        let directory=support::TempDirectory::new();
        let output=support::run(support::cli().arg("run").arg(directory.source(&source)),60);
        support::success(&output);assert!(output.stdout.is_empty());
    }
}


#[test]
fn decimal_parser_preserves_complete_grammar_rounding_and_special_spellings() {
    let source=support::format_round_trip("function parse(text:string):number{return Number(text);}function main(){}");
    let emitted=tsrust_compiler::check(&source).unwrap().emit_rust();
    assert!(!emitted.contains("unsafe"));
    let assertions=r#"
fn main(){
    for text in ["+", "-", ".", "+.", "-.", "1e", "1e+", "1e-", "1.2.3", "1e2e3", "1_000", "1tail", "1\0", "1 2", "0x", "0b2", "-0x10", "+0x10", "--1", "+-1", "inf", "Inf", "INF", "infinity", "INFINITY", "+inf", "-inf", "+infinity", "-infinity", "Infinitytail", "NaN", "+NaN", "-NaN", "nan", "１", "١", "\u{0085}2", "2\u{200b}"] {
        assert!(function0(&String::Borrowed(text)).is_nan(),"{text:?}");
    }
    for text in ["0","-0","+0",".5","-.5","+.5","1.","1.e2","1.E-2","01","008","2.5e1","1e309","-1e309","1e-999","-1e-999","9007199254740993","5e-324","1.7976931348623157e308"] {
        let expected=text.parse::<f64>().unwrap();
        for value in [String::Borrowed(text),String::Owned(text.to_owned())] {
            assert_eq!(function0(&value).to_bits(),expected.to_bits(),"{text}");
            assert_eq!(value.as_ref(),text);
        }
    }
    for (text,expected) in [("",0.0),(" \t\r\n",0.0),("\u{feff}-.5\u{a0}",-0.5),("Infinity",f64::INFINITY),("+Infinity",f64::INFINITY),("-Infinity",f64::NEG_INFINITY),("0x10",16.0),("0b11",3.0),("0o17",15.0)] {
        assert_eq!(function0(&String::Borrowed(text)).to_bits(),expected.to_bits(),"{text:?}");
    }
}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("decimal.rs");
    std::fs::write(&path,emitted.replace("fn main()","fn source_main()")+assertions).unwrap();
    for level in ["0","3"] {
        let binary=directory.0.join(format!("decimal-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}"),"-C","overflow-checks=on"]).arg(&path).arg("-o").arg(&binary),60));
        support::success(&support::run(&mut std::process::Command::new(binary),30));
    }
}
