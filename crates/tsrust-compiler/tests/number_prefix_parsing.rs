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
    let path = directory.0.join("number-prefix-parsing.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("number-prefix-parsing-{level}"));
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
fn longest_decimal_infinity_prefixes_and_ieee_boundaries_use_standard_parsing() {
    // Each isolated call must independently select all required runtime support.
    native(
        "function main(){console.log(Number.parseFloat(\"1.25tail\"));}",
        "1.25\n",
        "",
        "",
    );
    native(
        r#"
        function parse(text:string):number{return Number.parseFloat(text);}
        function main(){
            for(const text of ["  +12.5suffix","-.5x","1.e2!","1e+","1e-","1e","1.2.3","0x10","0b11","0001.50tail","-0","-0e999","1e999tail","-1e999tail","Infinitysuffix","+Infinity0","-Infinity!","1é","1\u0000x"]){console.log(Number.parseFloat(text));}
            console.log(Number.isNaN(Number.parseFloat("")));console.log(Number.isNaN(Number.parseFloat("+")));
            console.log(Number.isNaN(Number.parseFloat(".")));console.log(Number.isNaN(Number.parseFloat("-.")));
            console.log(Number.isNaN(Number.parseFloat("inf")));console.log(Number.isNaN(Number.parseFloat("NaN")));
            console.log(Number.isNaN(Number.parseFloat("infinity")));console.log(Number.isNaN(Number.parseFloat("１２")));
            console.log(Number.isNaN(Number("1.25tail")));console.log(Number(""));
            console.log(Number.isNaN(Number.parseFloat("\u00851")));console.log(Number.isNaN(Number.parseFloat("\u180e1")));console.log(Number.isNaN(Number.parseFloat("\u200b1")));
        }"#,
        "12.5\n-0.5\n100\n1\n1\n1\n1.2\n0\n0\n1.5\n-0\n-0\nInfinity\n-Infinity\nInfinity\nInfinity\n-Infinity\n1\n1\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n0\ntrue\ntrue\ntrue\n",
        "",
        r#"
        for (input,prefix) in [
            ("9007199254740993suffix","9007199254740993"),
            ("1.7976931348623157e308tail","1.7976931348623157e308"),
            ("1.7976931348623159e308tail","1.7976931348623159e308"),
            ("2.2250738585072014e-308!","2.2250738585072014e-308"),
            ("4.9406564584124654e-324!","4.9406564584124654e-324"),
            ("2.4703282292062327e-324!","2.4703282292062327e-324"),
            ("-1e-999tail","-1e-999"),
            ("1.00000000000000011102230246251565404236316680908203125!","1.00000000000000011102230246251565404236316680908203125"),
            (".125E+2z",".125E+2"),("+.125E-2z","+.125E-2"),("12.e+0z","12.e+0"),
            ("-0e+!","-0"),("7e+-2","7"),("1_000","1"),("1e2.3","1e2"),
        ] {
            let actual=function0(&String::Borrowed(input));let expected=prefix.parse::<f64>().unwrap();
            assert_eq!(actual.to_bits(),expected.to_bits(),"{input}");
        }
        let whitespace=['\u{0009}','\u{000a}','\u{000b}','\u{000c}','\u{000d}','\u{0020}','\u{00a0}','\u{1680}','\u{2000}','\u{2001}','\u{2002}','\u{2003}','\u{2004}','\u{2005}','\u{2006}','\u{2007}','\u{2008}','\u{2009}','\u{200a}','\u{2028}','\u{2029}','\u{202f}','\u{205f}','\u{3000}','\u{feff}'];
        for character in whitespace {
            let input=String::Owned(format!("{character}-1.25suffix"));assert_eq!(function0(&input),-1.25);
        }
        for input in [""," ","+","-",".","+.","-.","e10",".e10","+-1","--1","NaN","inf","infinity","\0","é1","\u{0085}1","\u{180e}1","\u{200b}1"]{assert!(function0(&String::Borrowed(input)).is_nan(),"{input:?}");}
        assert_eq!(function0(&String::Borrowed("-0")).to_bits(),(-0.0f64).to_bits());
        assert_eq!(function0(&String::Borrowed("-0.000e-9999")).to_bits(),(-0.0f64).to_bits());
        let exponent=String::Owned(format!("1e{}trailing","9".repeat(100000)));assert_eq!(function0(&exponent),f64::INFINITY);
        let zero_exponent=String::Owned(format!("1e{}!","0".repeat(100000)));assert_eq!(function0(&zero_exponent),1.0);
        let mantissa=String::Owned(format!("0.{}1tail","0".repeat(100000)));assert_eq!(function0(&mantissa),0.0);
        "#,
    );
}

#[test]
fn borrowed_operands_effects_contexts_fields_and_selected_temporaries_are_preserved() {
    let analysis = native(
        r#"
        type Box={text:string;value:number;};
        function parse(text:string):number{return Number.parseFloat(text);}
        function source(trace:int[]):string{trace.push(1);return "2.5tail".repeat(1);}
        function take(text:string):string{return text;}
        function read(text:string,value:number):number{return value;}
        function index(trace:int[]):int{trace.push(2);return 0;}
        function factory(text:string):()=>number{return ():number=>{return Number.parseFloat(text);};}
        function fixed():number{const texts=["1.25x","2.5y"];return Number.parseFloat(texts[0]);}
        function main(){
            const trace:int[]=[];console.log(Number.parseFloat(source(trace)));
            const text="1.25suffix".repeat(1);console.log(Number.parseFloat(text));console.log(text);
            const texts=["7.5tail","2.5tail"];console.log(Number.parseFloat(texts[index(trace)]));console.log(texts[0]);
            const box:Box={text:"3.5tail",value:0.0};box.value=Number.parseFloat(box.text);console.log(box.value);console.log(box.text);
            const inferred=Number.parseFloat("1.5tail")+2;console.log(inferred);
            const numbers=[Number.parseFloat("4.5tail"),Number.parseFloat("6.5tail")];console.log(numbers[1]);
            console.log(read(text,Number.parseFloat(text)));const action=factory(clone(text));console.log(action());console.log(text);
            console.log(fixed());console.log(parse("9.5tail"));
            for(const flag of [true,false]){
                console.log(Number.parseFloat(flag?"1.25tail".repeat(1):"2.5tail".repeat(1)));
                console.log(Number.parseFloat(flag?text:box.text));
                for(const n of [Number.parseFloat(flag?"1.25tail".repeat(1):"2.5tail".repeat(1))]){console.log(n);}
            }
            for(const n of trace){console.log(n);}
            console.log(Number.parseFloat("4.5tail").toFixed(1));
        }"#,
        "2.5\n1.25\n1.25suffix\n7.5\n7.5tail\n3.5\n3.5tail\n3.5\n6.5\n1.25\n1.25\n1.25suffix\n1.25\n9.5\n1.25\n1.25\n1.25\n2.5\n3.5\n2.5\n1\n2\n4.5\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("parse.text"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("read.text"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("take.text"), Some(&Mode::Consume));
    assert!(analysis.emit_rust().contains("LiteralStringArray"));
    for source in [
        "function take(text:string):string{return text;}function read(text:string,value:number):number{return value;}function main(){const text=\"1\".repeat(1);read(text,Number.parseFloat(take(text)));}",
        "function take(text:string):string{return text;}function read(text:string,value:number):number{return value;}function main(){const a=\"1\".repeat(1);const b=\"2\".repeat(1);read(true?a:b,Number.parseFloat(take(a)));}",
    ] {
        assert_eq!(check(source).err().unwrap().code, "conflicting-access");
    }
}

#[test]
fn strict_namespace_string_arity_and_normal_stack_diagnostics_are_bounded() {
    for (source, code) in [
        ("function main(){Number.parseFloat();}", "arity"),
        ("function main(){Number.parseFloat(\"1\",\"2\");}", "arity"),
        ("function main(){Number.parseFloat(1);}", "type"),
        ("function main(){Number.parseFloat(1.0);}", "type"),
        ("function main(){Number.parseFloat(true);}", "type"),
        ("function main(){Number.parseFloat([\"1\"]);}", "type"),
        (
            "function main(){const value:int=Number.parseFloat(\"1\");}",
            "type",
        ),
        (
            "function main(){const value:boolean=Number.parseFloat(\"1\");}",
            "type",
        ),
        // The global spelling is accepted; an unknown global stays a name error.
        ("function main(){parseDouble(\"1\");}", "name"),
        ("function main(){Number.parseFloat(,);}", "syntax"),
        ("function main(){const Number=\"1\";}", "duplicate"),
    ] {
        let error = support::reject(source, code);
        assert_ne!(error.code, "internal");
        assert!(error.span.end > error.span.start);
    }
    // Arity takes precedence over analysis of an invalid extra operand.
    assert_eq!(
        check("function main(){Number.parseFloat(\"1\",missing);}")
            .err()
            .unwrap()
            .code,
        "arity"
    );
    for depth in [100, 200] {
        for expression in [
            format!("{}Number.parseFloat(\"1.5tail\")", "- ".repeat(depth)),
            format!(
                "{}Number.parseFloat(\"1.5tail\"){}",
                "true?".repeat(depth),
                ":0.0".repeat(depth)
            ),
        ] {
            let source = format!("function main(){{console.log({expression});}}");
            let directory = support::TempDirectory::new();
            let output = support::run(
                support::cli().arg("emit").arg(directory.source(&source)),
                15,
            );
            // Deep chains compile through the CLI, which has no nesting limit.
            support::success(&output);
        }
    }
    for count in [48, 200] {
        let source = format!(
            "function main(){{console.log(Number.parseFloat(\"1.5tail\"{}));}}",
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
fn parsing_allocates_nothing_and_owned_temporaries_dispose_normally_and_on_unwind() {
    native(
        r#"
        function parse(text:string):number{return Number.parseFloat(text);}
        function temporary(flag:boolean):number{return Number.parseFloat(flag?"1.25tail".repeat(1):"2.5tail".repeat(1));}
        function trap():number{const texts=["1.25tail".repeat(1)];return Number.parseFloat(texts[9]);}
        function main(){}
        "#,
        "",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCS:AtomicUsize=AtomicUsize::new(0);struct Allocations;
        unsafe impl GlobalAlloc for Allocations{
            unsafe fn alloc(&self,layout:Layout)->*mut u8{let pointer=unsafe{System.alloc(layout)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);ALLOCS.fetch_add(1,Ordering::SeqCst);}pointer}
            unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout){unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);}
            unsafe fn realloc(&self,pointer:*mut u8,layout:Layout,size:usize)->*mut u8{ALLOCS.fetch_add(1,Ordering::SeqCst);unsafe{System.realloc(pointer,layout,size)}}
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
        for (input,expected) in [("1.25tail",1.25),("\u{feff}-0tail",-0.0),("1e+tail",1.0),("1e999tail",f64::INFINITY),("-1e-999tail",-0.0),("Infinitysuffix",f64::INFINITY),("NaN",f64::NAN),("invalid",f64::NAN)]{
            let borrowed=String::Borrowed(input);ALLOCS.store(0,Ordering::SeqCst);let value=function0(std::hint::black_box(&borrowed));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);if expected.is_nan(){assert!(value.is_nan());}else{assert_eq!(value.to_bits(),expected.to_bits());}assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            let owned=String::Owned(input.to_owned());let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);let value=function0(std::hint::black_box(&owned));assert_eq!(ALLOCS.load(Ordering::SeqCst),0);if expected.is_nan(){assert!(value.is_nan());}else{assert_eq!(value.to_bits(),expected.to_bits());}assert_eq!(LIVE.load(Ordering::SeqCst),before);assert_eq!(owned.as_ref(),input);drop(owned);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        // Construct long owned inputs before measuring the parser itself.
        for (input,expected) in [
            (String::Owned(format!("0.{}1tail","0".repeat(100000))),0.0),
            (String::Owned(format!("1{}tail","0".repeat(100000))),f64::INFINITY),
            (String::Owned(format!("1e{}tail","9".repeat(100000))),f64::INFINITY),
            (String::Owned(format!("1e{}tail","0".repeat(100000))),1.0),
            (String::Owned(format!("1{}e-100000tail","0".repeat(100000))),1.0),
        ] {
            let before=LIVE.load(Ordering::SeqCst);ALLOCS.store(0,Ordering::SeqCst);
            let actual=function0(&input);assert_eq!(ALLOCS.load(Ordering::SeqCst),0);
            assert_eq!(actual.to_bits(),expected.to_bits());assert_eq!(LIVE.load(Ordering::SeqCst),before);
            drop(input);
        }
        assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for flag in [true,false]{assert_eq!(function1(flag),if flag{1.25}else{2.5});assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}
        let error=std::panic::catch_unwind(function2).unwrap_err();drop(error);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
        "#,
    );
}
