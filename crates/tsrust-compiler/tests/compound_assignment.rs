mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("compound.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("compound.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("compound-{level}"));
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
        let output = support::run(&mut std::process::Command::new(&executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

const ARITHMETIC: [&str; 5] = ["+=", "-=", "*=", "/=", "%="];
const NUMBER: [&str; 12] = [
    "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "<<=", ">>=", ">>>=",
];

#[test]
fn all_supported_typed_operators_preserve_checked_int_and_exact_number_semantics() {
    let mut source = String::new();
    for (id, op) in ARITHMETIC.iter().enumerate() {
        source += &format!(
            "function int{id}(left:int,right:int):int{{let value=left;value {op} right;return value;}}\n"
        );
    }
    for (id, op) in NUMBER.iter().enumerate() {
        source += &format!(
            "function number{id}(left:number,right:number):number{{let value=left;value {op} right;return value;}}\n"
        );
    }
    source += r#"function main(){let integer=1;integer+=2;integer*=4;integer-=1;integer/=2;integer%=3;console.log(integer);
        let number:number=1;for(let count=0;count<4;count+=1){number*=2;}console.log(number);let k=0;for(k+=1;k<3;k+=1){}console.log(k);}"#;
    native(
        &source,
        "2\n16\n3\n",
        r#"
        let integers:[fn(i64,i64)->i64;5]=[function0,function1,function2,function3,function4];
        for (operation,expected) in integers.into_iter().zip([23,17,60,6,2]){assert_eq!(operation(20,3),expected);}
        assert_eq!(function0(i64::MIN,0),i64::MIN);assert_eq!(function1(i64::MAX,0),i64::MAX);assert_eq!(function3(-7,3),-2);assert_eq!(function4(-7,3),-1);
        let numbers:[fn(f64,f64)->f64;12]=[function5,function6,function7,function8,function9,function10,function11,function12,function13,function14,function15,function16];
        let cases:[(f64,f64,f64);12]=[(1.5,2.25,3.75),(-0.0,0.0,-0.0),(-0.0,3.0,-0.0),(1.0,-0.0,f64::NEG_INFINITY),(5.5,2.0,1.5),(-2.0,3.0,-8.0),
            (4294967295.0,2147483648.0,-2147483648.0),(2147483648.0,1.0,-2147483647.0),(4294967295.0,4294967295.0,0.0),
            (1.0,33.0,2.0),(2147483648.0,33.0,-1073741824.0),(-1.0,33.0,2147483647.0)];
        for (operation,(left,right,expected)) in numbers.into_iter().zip(cases){assert_eq!(operation(left,right).to_bits(),expected.to_bits());}
        assert!(function5(f64::NAN,1.0).is_nan());assert!(function8(0.0,0.0).is_nan());assert!(function10(-1.0,0.5).is_nan());
        assert_eq!(function10(2.0,-1.0).to_bits(),0.5f64.to_bits());assert_eq!(function10(-0.0,3.0).to_bits(),(-0.0f64).to_bits());
        assert_eq!(function5(f64::from_bits(1),0.0).to_bits(),1);assert_eq!(function7(f64::MAX,2.0),f64::INFINITY);
        assert_eq!(function11(f64::INFINITY,7.0).to_bits(),0.0f64.to_bits());assert_eq!(function14(1.0,-1.0),-2147483648.0);
        assert_eq!(function16(-1.0,f64::NAN),4294967295.0);assert_eq!(function16(-1.0,64.0),4294967295.0);
    "#,
        " ",
    );
}

#[test]
fn targets_select_once_snapshot_before_rhs_and_preserve_storage_and_capture_rules() {
    let analysis = native(
        r#"
        type Box={n:int;text:string;other:string;};
        function index():int{console.log("index");return 0;}
        function mark(value:int):int{console.log(value);return value;}
        function mutate(values:int[]):int{values[0]=100;console.log("rhs");return 3;}
        function grow(values:int[]):int{values.push(99);console.log("grow");return 3;}
        function mutateBox(box:Box):int{box.n=100;return 3;}
        function update(box:Box,suffix:string){box.text+=suffix;}
        function main(){
            const values=[7];values[index()]+=mutate(values);console.log(values[0]);values[0]+=grow(values);console.log(values[0]);console.log(values[1]);
            const nested=[[7,8]];nested[mark(0)][mark(1)]*=3;console.log(nested[0][1]);
            const box:Box={n:7,text:"a\u0000😀",other:"saved"};box.n+=mutateBox(box);console.log(box.n);const moved=box.other;box.n+=2;console.log(box.n);
            box.text+=box.text;console.log(box.text==="a\u0000😀a\u0000😀");box.other=moved;update(box,"!");console.log(box.text.length);console.log(box.other);
            let text="x\u0000😀";text+=text;console.log(text==="x\u0000😀x\u0000😀");const suffix="é".toUpperCase();text+=suffix;console.log(suffix);console.log(text.length);
            const words=["x\u0000😀", "second"];words[index()]+=words[0];console.log(words[0]==="x\u0000😀x\u0000😀");console.log(words[1]);
            let count=0;{const next=():int=>{count+=1;return count;};console.log(next());console.log(next());}console.log(count);
            {let captured="c".toUpperCase();const append=():string=>{captured+="!";return clone(captured);};console.log(append());console.log(append());}
            let loop=0;while(loop<2){loop+=1;if(loop<2){continue;}}console.log(loop);
        }
    "#,
        "index\nrhs\n10\ngrow\n13\n99\n0\n1\n24\n10\n12\ntrue\n13\nsaved\ntrue\nÉ\n14\nindex\ntrue\nsecond\n1\n2\n2\nC!\nC!!\n2\n",
        "",
        "",
    );
    for (name, mode) in [
        ("mutate.values", Mode::Mutate),
        ("grow.values", Mode::Mutate),
        ("update.box", Mode::Mutate),
        ("update.suffix", Mode::Read),
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&mode));
    }
    let fixed = native(
        "function main(){const values=[1,2,3];values[1]+=4;console.log(values[1]);const words=[\"a\",\"é\"];let text=\"x\";text+=words[1];console.log(text);}",
        "6\nxé\n",
        "",
        "",
    );
    assert!(fixed.emit_rust().contains("LiteralStringArray"));
}

#[test]
fn diagnostics_and_traps_require_available_targets_and_commit_only_after_success() {
    for op in NUMBER {
        for target in ["const value=1.0", "function f(value:number)"] {
            let source = if target.starts_with("const") {
                format!("function main(){{{target};value {op} 1;}}")
            } else {
                format!("{target}{{value {op} 1;}}function main(){{}}")
            };
            assert_eq!(
                check(&source).err().unwrap().code,
                "unsupported",
                "{source}"
            );
        }
    }
    for op in ["**=", "&=", "|=", "^=", "<<=", ">>=", ">>>="] {
        let source = format!("function main(){{let value=1;value {op} 1.0;}}");
        assert_eq!(check(&source).err().unwrap().code, "type");
    }
    for (source, code) in [
        ("function main(){let text=\"x\";text-=\"y\";}", "type"),
        ("function main(){let value=1;value+=1.0;}", "type"),
        (
            "function main(){let value=1.0;const integer=1;value+=integer;}",
            "type",
        ),
        ("function main(){let flag=true;flag+=false;}", "type"),
        ("function main(){let values=[1];values+=[2];}", "type"),
        ("function main(){1+=2;}", "unsupported"),
        (
            "function main(){let text=\"x\";const moved=text;text+=\"!\";}",
            "transferred",
        ),
        (
            "type Box={text:string;};function main(){const box:Box={text:\"x\"};const moved=box.text;box.text+=\"!\";}",
            "partial-transfer",
        ),
        (
            "function take(text:string):string{return text;}function main(){let text=\"x\";text+=take(text);}",
            "conflicting-access",
        ),
        (
            "function grow(values:int[][]):int{values.push([2]);return 3;}function main(){const values=[[7]];values[0][0]+=grow(values);}",
            "conflicting-access",
        ),
        (
            "type Box={text:string;};function mutate(box:Box):string{box.text=\"new\";return \"!\";}function main(){const box:Box={text:\"old\"};box.text+=mutate(box);}",
            "conflicting-access",
        ),
        (
            "function main(){let text=\"x\";const read=():string=>{return clone(text);};text+=\"!\";console.log(read());}",
            "capture-conflict",
        ),
        (
            "function main(){const values=[\"x\"];for(const value of values){value+=\"!\";}}",
            "borrowed-element",
        ),
        (
            "function main(){const values=[1];for(const value of values){values[0]+=1;}}",
            "iteration-conflict",
        ),
        (
            "function main(){let text=\"old\";console.log(text+=\"new\");}",
            "unsupported",
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
    for source in [
        "function main(){const n=1;n++;}",
        "function main(){const n=1;--n;}",
        "function main(){let flag=1;flag&&=false;}",
        "function main(){let flag=1;flag||=false;}",
        "function main(){let flag=true;(true?flag:flag)??=false;}",
    ] {
        assert!(check(source).is_err());
    }
    // Rust observers catch native traps to inspect effects and the unchanged destination.
    native(
        r#"function change(values:int[]):int{values[0]=9;return 1;}function add(values:int[]){values[0]+=change(values);}function main(){}"#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        let mut values=NumericArray::from([i64::MAX]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function1(&mut values))).is_err());assert_eq!(values.get(0),Some(&9));std::panic::set_hook(hook);
    "#,
        "",
    );
    for (target, types, setup) in [
        ("values[-1]", "int[]", "const values=[7];"),
        ("values[1]", "int[]", "const values=[7];"),
        ("values[1].n", "Box[]", "const values:Box[]=[{n:7}];"),
    ] {
        let source = format!(
            "type Box={{n:int;}};function rhs():int{{console.log(\"rhs\");return 3;}}function apply(values:{types}){{{target}+=rhs();}}function main(){{{setup}apply(values);}}"
        );
        // Invalid final selection must fail before the RHS marker at O0 and O3.
        let emitted = check(&source).unwrap().emit_rust();
        let directory = support::TempDirectory::new();
        let path = directory.0.join("bounds.rs");
        std::fs::write(&path, emitted).unwrap();
        for level in ["0", "3"] {
            let executable = directory.0.join(format!("bounds-{level}"));
            support::success(&support::run(
                support::rustc()
                    .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                    .arg(&path)
                    .arg("-o")
                    .arg(&executable),
                60,
            ));
            let output = support::run(&mut std::process::Command::new(&executable), 10);
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
            assert!(
                String::from_utf8(output.stderr)
                    .unwrap()
                    .contains("array index out of bounds at source bytes")
            );
        }
    }
}

#[test]
fn string_updates_allocate_fresh_results_and_dispose_normally_and_on_unwind() {
    let source = "function append(left:string,right:string):string{let target=left;target+=right;return target;}function main(){}";
    let analysis = native(
        source,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for (a,b,expected) in [("","",""),("","é😀","é😀"),("left","","left"),("a\0","e\u{301}","a\0e\u{301}")] {for owned in [false,true] {
            let left=if owned{String::Owned(a.to_owned())}else{String::Borrowed(a)};let right=if owned{String::Owned(b.to_owned())}else{String::Borrowed(b)};
            let before=LIVE.load(Ordering::SeqCst);let count=ALLOCATIONS.load(Ordering::SeqCst);let result=std::hint::black_box(function0(left,std::hint::black_box(&right)));
            assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Owned(_)));assert_eq!(ALLOCATIONS.load(Ordering::SeqCst)-count,usize::from(!expected.is_empty()));
            assert_eq!(LIVE.load(Ordering::SeqCst),before-(if owned{a.len() as isize}else{0})+expected.len() as isize);assert_eq!(right.as_ref(),b);
            drop(result);drop(right);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
            assert!(std::panic::catch_unwind(||{let left=String::Owned(a.to_owned());let right=String::Owned(b.to_owned());let result=function0(left,&right);std::hint::black_box(&result);panic!("dispose operands and result");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
        }}std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
    );
    assert_eq!(
        analysis.contracts().get("append.left"),
        Some(&Mode::Consume)
    );
    assert_eq!(analysis.contracts().get("append.right"), Some(&Mode::Read));
}

// Test-only forwarding allocator; generated code remains safe standard Rust.
const ALLOCATOR: &str = r#"
use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
static LIVE:AtomicIsize=AtomicIsize::new(0);static ALLOCATIONS:AtomicUsize=AtomicUsize::new(0);struct Observer;
unsafe impl GlobalAlloc for Observer{
unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){LIVE.fetch_add(layout.size() as isize,Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}p}
unsafe fn dealloc(&self,p:*mut u8,layout:Layout){LIVE.fetch_sub(layout.size() as isize,Ordering::SeqCst);unsafe{System.dealloc(p,layout)}}
unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8{let q=unsafe{System.realloc(p,layout,size)};if !q.is_null(){LIVE.fetch_add(size as isize-layout.size() as isize,Ordering::SeqCst);ALLOCATIONS.fetch_add(1,Ordering::SeqCst);}q}
}
#[global_allocator]static GLOBAL:Observer=Observer;
"#;
