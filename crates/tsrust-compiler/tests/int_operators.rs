mod support;
use tsrust_compiler::{Mode, check};

const OPERATORS: [&str; 7] = ["&", "|", "^", "<<", ">>", ">>>", "**"];

fn source() -> String {
    let mut source = "function positive(value:int):int{return +value;}function complement(value:int):int{return ~value;}".to_owned();
    for (id, op) in OPERATORS.iter().enumerate() {
        source +=
            &format!("function binary{id}(left:int,right:int):int{{return left {op} right;}}");
    }
    for (id, op) in OPERATORS.iter().enumerate() {
        source += &format!(
            "function compound{id}(left:int,right:int):int{{let value=left;value {op}= right;return value;}}"
        );
    }
    source
}

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis = check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("int-operators.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("int-operators.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("int-operators-{level}"));
        support::success(&support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=no",
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
fn all_sixteen_i64_forms_preserve_number_width_and_literal_defaults() {
    let mut program = source();
    program += "function number(left:number,right:number):number{return left>>>right;}function numberPower(left:number,right:number):number{return left**right;}function main(){console.log(1<<32);console.log(2**-2);console.log(~4294967295);console.log((1+2)<<32);console.log(+1);}";
    let analysis = native(
        &program,
        "1\n0.25\n0\n3\n1\n",
        r#"
        let unary:[fn(i64)->i64;2]=[function0,function1];
        assert_eq!(unary[0](i64::MIN),i64::MIN);assert_eq!(unary[1](0),-1);assert_eq!(unary[1](-1),0);
        let binary:[fn(i64,i64)->i64;7]=[function2,function3,function4,function5,function6,function7,function8];
        let compound:[fn(i64,i64)->i64;7]=[function9,function10,function11,function12,function13,function14,function15];
        let cases:[(i64,i64,i64);7]=[(7,3,3),(4,3,7),(7,3,4),(3,4,48),(-16,2,-4),(-1,1,i64::MAX),(-2,3,-8)];
        for ((left,right,expected),(direct,assigned)) in cases.into_iter().zip(binary.into_iter().zip(compound)) {
            assert_eq!(direct(left,right),expected);assert_eq!(assigned(left,right),expected);
        }
        assert_eq!(function16(-1.0,0.0),4294967295.0);assert_eq!(function16(-1.0,32.0),4294967295.0);
        assert_eq!(function16(1.0,-1.0),0.0);assert_eq!(function16(f64::NAN,1.0).to_bits(),0.0f64.to_bits());
        assert_eq!(function17(2.0,-2.0),0.25);assert!(function17(-2.0,0.5).is_nan());
    "#,
        "",
    );
    for id in 0..7 {
        for name in [format!("binary{id}"), format!("compound{id}")] {
            for parameter in ["left", "right"] {
                assert_eq!(
                    analysis.contracts().get(&format!("{name}.{parameter}")),
                    Some(&Mode::Copy)
                );
            }
        }
    }
}

#[test]
fn int_contexts_anchors_precedence_and_nested_number_rejections_are_explicit() {
    native(
        r#"
        type Box={n:int;};
        function get():int{return 4294967296;}
        function literal():int{return (1<<32)|(2**3);}
        function accept(value:int):int{return value;}
        function unaryReturn():int{return -(1<<2);}
        function main(){
            const integer=4294967296;console.log(integer|1);console.log(1|get());console.log(get()|2);
            const explicit:int=((1<<32)+1)*2;console.log(explicit);
            const negated:int=-(1<<32);console.log(negated);
            const values:int[]=[1<<32,(1<<32)+1,~1];console.log(values[0]);console.log(values[1]);console.log(values[2]);
            const box:Box={n:1<<32};console.log(box.n|1);
            const inferred=[box.n|2,get()|3];console.log(inferred[0]);console.log(inferred[1]);
            const nested=[[integer|4]];console.log(nested[0][0]);console.log(values[0]|5);
            const captured=():int=>{return integer|6;};console.log(captured());
            const literalClosure=():int=>{return (1<<32)+7;};console.log(literalClosure());
            let count=4294967296;console.log(count++|8);console.log(count);
            console.log(literal());console.log(accept(1<<32));
            const numbers=[1<<32,2**-2,~1];console.log(numbers[0]);console.log(numbers[1]);console.log(numbers[2]);
            const precedence:int=1|2^3&4;console.log(precedence);
            const associated:int=2**3**2;console.log(associated);
            const grouped:int=(2**3)**2;console.log(grouped);
            let positive:int=+(2**3);console.log(positive);
            let complement:int=~(-(1<<2));console.log(complement);console.log(unaryReturn());
        }
    "#,
        "4294967297\n4294967297\n4294967298\n8589934594\n-4294967296\n4294967296\n4294967297\n-2\n4294967297\n4294967298\n4294967299\n4294967300\n4294967301\n4294967302\n4294967303\n4294967304\n4294967297\n4294967304\n4294967296\n1\n0.25\n-2\n3\n512\n64\n8\n3\n-4\n",
        "",
        "",
    );
    for op in OPERATORS {
        for expr in [
            format!("integer {op} number"),
            format!("number {op} integer"),
            format!("(integer {op} 1) + number"),
            format!("integer {op} (number + 1)"),
        ] {
            let source =
                format!("function main(){{const integer=1;const number=1.0;console.log({expr});}}");
            assert_eq!(check(&source).err().unwrap().code, "type", "{source}");
        }
        let source = format!("function f():int{{return 1 {op} 1.0;}}function main(){{}}");
        assert_eq!(check(&source).err().unwrap().code, "type", "{source}");
    }
    for expression in ["-2**2", "+2**2", "~2**2", "2**-2**2"] {
        let source = format!("function main(){{console.log({expression});}}");
        assert_eq!(check(&source).err().unwrap().code, "syntax", "{source}");
    }
    for count in [100, 200] {
        let source = format!(
            "function main(){{const integer=1;console.log({}integer);}}",
            "~".repeat(count)
        );
        if count == 100 {
            check(&source).unwrap().emit_rust();
        } else {
            assert_eq!(check(&source).err().unwrap().code, "unsupported");
        }
    }
}

#[test]
fn strict_targets_loans_and_single_operand_selection_survive_failed_commits() {
    let analysis = native(
        r#"
        type Trace={n:int;};
        function index(trace:Trace):int{trace.n=trace.n*10+1;return 0;}
        function rhs(values:int[],trace:Trace,count:int):int{trace.n=trace.n*10+2;values[0]=16;return count;}
        function shift(values:int[],trace:Trace,count:int){values[index(trace)]<<=rhs(values,trace,count);}
        function power(values:int[],trace:Trace,count:int){values[index(trace)]**=rhs(values,trace,count);}
        function next(trace:Trace,code:int,value:int):int{trace.n=trace.n*10+code;return value;}
        function binary(trace:Trace):int{return next(trace,1,7)&next(trace,2,3);}
        function badIndex(trace:Trace):int{trace.n=trace.n*10+1;return 1;}
        function invalidSelection(values:int[],trace:Trace){values[badIndex(trace)]|=rhs(values,trace,1);}
        function failingRhs(values:int[],trace:Trace):int{trace.n=trace.n*10+2;values[0]=16;return 1/0;}
        function rhsTrap(values:int[],trace:Trace){values[index(trace)]^=failingRhs(values,trace);}
        function main(){}
    "#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        let mut trace=Record0{field0:0};let mut values=NumericArray::from([8]);
        function2(&mut values,&mut trace,3);assert_eq!(trace.field0,12);assert_eq!(values.get(0),Some(&64));
        trace.field0=0;*values.get_mut(0).unwrap()=8;assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function2(&mut values,&mut trace,64))).is_err());assert_eq!(trace.field0,12);assert_eq!(values.get(0),Some(&16));
        trace.field0=0;*values.get_mut(0).unwrap()=8;assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut values,&mut trace,63))).is_err());assert_eq!(trace.field0,12);assert_eq!(values.get(0),Some(&16));
        trace.field0=0;*values.get_mut(0).unwrap()=8;assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut values,&mut trace,-1))).is_err());assert_eq!(trace.field0,12);assert_eq!(values.get(0),Some(&16));
        trace.field0=0;assert_eq!(function5(&mut trace),3);assert_eq!(trace.field0,12);
        trace.field0=0;*values.get_mut(0).unwrap()=8;assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function7(&mut values,&mut trace))).is_err());assert_eq!(trace.field0,1);assert_eq!(values.get(0),Some(&8));
        trace.field0=0;*values.get_mut(0).unwrap()=8;assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function9(&mut values,&mut trace))).is_err());assert_eq!(trace.field0,12);assert_eq!(values.get(0),Some(&16));
        std::panic::set_hook(hook);
    "#,
        "",
    );
    assert_eq!(
        analysis.contracts().get("shift.values"),
        Some(&Mode::Mutate)
    );
    assert_eq!(
        analysis.contracts().get("binary.trace"),
        Some(&Mode::Mutate)
    );
    for op in OPERATORS {
        for target in ["const value=1", "function f(value:int)"] {
            let source = if target.starts_with("const") {
                format!("function main(){{{target};value {op}= 1;}}")
            } else {
                format!("{target}{{value {op}= 1;}}function main(){{}}")
            };
            assert_eq!(
                check(&source).err().unwrap().code,
                "unsupported",
                "{source}"
            );
        }
        for declaration in ["let value=true", "let value=\"x\""] {
            let source = format!("function main(){{{declaration};value {op}= 1;}}");
            assert_eq!(check(&source).err().unwrap().code, "type", "{source}");
        }
    }
    for expression in ["+true", "+\"x\"", "~true", "~\"x\""] {
        let source = format!("function main(){{console.log({expression});}}");
        assert_eq!(check(&source).err().unwrap().code, "type", "{source}");
    }
    for op in ["&", "**"] {
        let source = format!(
            "type Box={{n:int;}};function mutate(box:Box):int{{box.n=2;return box.n;}}function hold(box:Box,n:int){{console.log(box.n);}}function main(){{const box:Box={{n:1}};hold(box,box.n {op} mutate(box));}}"
        );
        assert_eq!(
            check(&source).err().unwrap().code,
            "conflicting-access",
            "{source}"
        );
    }
}

#[test]
fn wide_edges_checked_traps_zero_allocation_and_owned_disposal_are_native() {
    let mut program = source();
    program += "type Box={n:int;text:string;};function owned(box:Box,power:int){box.n**=power;}function main(){}";
    let analysis = native(
        &program,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());
        let baseline=LIVE.load(Ordering::SeqCst);
        let binary:[fn(i64,i64)->i64;7]=[function2,function3,function4,function5,function6,function7,function8];
        let compound:[fn(i64,i64)->i64;7]=[function9,function10,function11,function12,function13,function14,function15];
        for pair in [binary,compound] {
            assert_eq!(pair[0](i64::MIN,i64::MAX),0);assert_eq!(pair[1](i64::MIN,i64::MAX),-1);assert_eq!(pair[2](i64::MIN,-1),i64::MAX);
            assert_eq!(pair[3](1,63),i64::MIN);assert_eq!(pair[3](16,63),0);assert_eq!(pair[3](i64::MAX,1),-2);
            assert_eq!(pair[4](i64::MIN,63),-1);assert_eq!(pair[4](-3,1),-2);
            assert_eq!(pair[5](-1,0),-1);assert_eq!(pair[5](-1,1),i64::MAX);assert_eq!(pair[5](i64::MIN,63),1);
            assert_eq!(pair[6](0,0),1);assert_eq!(pair[6](i64::MIN,1),i64::MIN);assert_eq!(pair[6](-1,4294967295),-1);
            for operation in &pair[3..6] {for count in [-1,64,4294967296,i64::MAX] {trapped(||operation(1,count),"shift count");}}
            for exponent in [-1,4294967296,i64::MAX] {trapped(||pair[6](1,exponent),"exponent must");}
            trapped(||pair[6](2,63),"exponentiation overflow");trapped(||pair[6](i64::MIN,2),"exponentiation overflow");
            for operation in pair {
                let count=ALLOCATIONS.load(Ordering::SeqCst);let live=LIVE.load(Ordering::SeqCst);
                std::hint::black_box(std::hint::black_box(operation)(std::hint::black_box(7),std::hint::black_box(3)));
                assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),live);
            }
        }
        for operation in [function0 as fn(i64)->i64,function1] {
            let count=ALLOCATIONS.load(Ordering::SeqCst);let live=LIVE.load(Ordering::SeqCst);
            std::hint::black_box(std::hint::black_box(operation)(std::hint::black_box(i64::MIN)));
            assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        }
        for text in ["","owned\0😀e\u{301}"] {
            let mut owner=Record0{field0:3,field1:String::Owned(text.to_owned())};function16(&mut owner,4);assert_eq!(owner.field0,81);assert_eq!(owner.field1.as_ref(),text);drop(owner);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:2,field1:String::Owned(text.to_owned())};function16(&mut owner,63);}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
    );
    assert_eq!(analysis.contracts().get("owned.box"), Some(&Mode::Mutate));
}

// The observer forwards to System; generated code contains no unsafe operations.
const ALLOCATOR: &str = r#"
use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCATIONS:AtomicUsize=AtomicUsize::new(0);struct Observer;
unsafe impl GlobalAlloc for Observer{
unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){LIVE.fetch_add(layout.size() as isize,Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}p}
unsafe fn dealloc(&self,p:*mut u8,layout:Layout){LIVE.fetch_sub(layout.size() as isize,Ordering::SeqCst);unsafe{System.dealloc(p,layout)}}
unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8{let q=unsafe{System.realloc(p,layout,size)};if !q.is_null(){LIVE.fetch_add(size as isize-layout.size() as isize,Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}q}
}
#[global_allocator]static GLOBAL:Observer=Observer;
fn trapped<F:FnOnce()->i64+std::panic::UnwindSafe>(operation:F,reason:&str){let error=std::panic::catch_unwind(operation).unwrap_err();let message=error.downcast_ref::<std::string::String>().map(|s|s.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();assert!(message.contains(reason));assert!(message.contains("source bytes"));drop(error);}
"#;
