mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis = check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("string-addition.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("string-addition.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("string-addition-{level}"));
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

const ADD: &str = "function add(left:string,right:string):string{return left+right;}";

#[test]
fn exact_utf8_contents_contextual_chains_and_numeric_addition_remain_distinct() {
    let source = ADD.to_owned()
        + r#"
        type Box={text:string;};
        function decimal(left:number,right:number):number{return left+right;}
        function invokeJoin(f:(left:ReadOnly<string>,right:ReadOnly<string>)=>string,left:string,right:string):string{return f(left,right);}
        function main(){
            console.log(("a"+"b"+"c")==="abc");console.log(("a"+("b"+"c"))==="abc");
            const annotated:string="é"+"😀";console.log(annotated==="é😀");
            const box:Box={text:"e"+"\u0301"};console.log(box.text==="e\u0301");
            const entries:string[]=[""+"x","y"+"",("a"+"b").toUpperCase()];console.log(entries[2]==="AB");
            const callback=(left:string,right:string):string=>{return left+right;};
            console.log(invokeJoin(callback,"a","b")==="ab");console.log(("a"+"\u0000b").length===3);
            console.log(("\u0085"+"\uFEFF").length===5);
            console.log((" A ".trim().toLowerCase()+" B ".trimStart().trimEnd().toUpperCase())==="aB");
            console.log((" A ".trim().toLowerCase()+" B ".trim().toLowerCase())==="ab");
            console.log(" A ".trim().toLowerCase()==="a");console.log(add(" A ".trim().toLowerCase()," B ".trim().toLowerCase())==="ab");
            console.log(2+3===5);console.log(decimal(1.5,2.25)===3.75);
            const numbers:number[]=[1+2,0.5+1];console.log(numbers[0]===3.0);console.log(numbers[1]===1.5);
        }
    "#;
    let analysis = native(
        &source,
        &"true\n".repeat(16),
        r#"
        let cases=[("","",""),("","right","right"),("left","","left"),("a","b","ab"),
            ("é","😀","é😀"),("e","\u{301}","e\u{301}"),("a\0","\0b","a\0\0b"),
            (" \u{85}","\u{feff} "," \u{85}\u{feff} ")];
        for (left,right,expected) in cases {for owned_left in [false,true] {for owned_right in [false,true] {
            let left=if owned_left{String::Owned(left.to_owned())}else{String::Borrowed(left)};
            let right=if owned_right{String::Owned(right.to_owned())}else{String::Borrowed(right)};
            let result=function0(&left,&right);assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Owned(_)));
            if !result.is_empty(){assert_ne!(result.as_ptr(),left.as_ptr());assert_ne!(result.as_ptr(),right.as_ptr());}
            assert_eq!(left.len()+right.len(),result.len());drop(left);drop(right);assert_eq!(result.as_ref(),expected);
        }}}
    "#,
        "",
    );
    assert_eq!(analysis.contracts().get("add.left"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("add.right"), Some(&Mode::Read));
    let rust = analysis.emit_rust();
    assert!(rust.contains("checked_add"));
    assert!(rust.contains("tsr_string_add(string_left, string_right"));
}

#[test]
fn operands_evaluate_once_in_order_and_reuse_temporaries_fields_indices_and_captures() {
    let analysis = native(
        r#"
        type Box={text:string;other:string;};
        function add(left:string,right:string):string{return left+right;}
        function forward(left:string,right:string):string{return add(left,right);}
        function operand(label:string):string{console.log(label);return label.toLowerCase();}
        function rewrite(box:Box):string{box.text="new";console.log("rewrite");return "new";}
        function index():int{console.log("index");return 0;}
        function makeBox():Box{return {text:"temp".toUpperCase(),other:"saved"};}
        function makeArray():string[]{return ["array".toUpperCase(),"unused"];}
        function invoke(f:(value:ReadOnly<string>)=>string,value:string):string{return f(value);}
        function factory(value:string):()=>string{return ():string=>{return value+"!";};}
        function main(){
            console.log(operand("LEFT")+operand("RIGHT"));
            console.log(operand("A")+operand("B")+operand("C"));
            const owner="owned".toUpperCase();const result=forward(owner,"!");console.log(result);console.log(owner);
            const box:Box={text:"OLD".toLowerCase(),other:"saved"};console.log(clone(box.text)+rewrite(box));console.log(box.text);
            const moved=box.other;console.log(box.text+"!");box.other=moved;console.log(box.other);
            const words=["field".toUpperCase(),"second"];const retained=words[index()]+box.text;words[0]="changed";console.log(retained);console.log(words[1]);
            const fixed=["a","b"];console.log(fixed[0]+fixed[1]);
            const growing:string[]=[];growing.push("heap".toUpperCase());growing.push("!");console.log(growing[0]+growing[1]);
            console.log(makeBox().text+"!");console.log(makeArray()[index()]+"!");
            const read=():string=>{return owner+"?";};console.log(read());console.log(read());
            const saved=factory("captured".toUpperCase());console.log(saved());console.log(saved());
            console.log(invoke((input:string):string=>{return input+"*";},owner));console.log(owner);
            for(const value of words){console.log(value+".");}
        }
    "#,
        "LEFT\nRIGHT\nleftright\nA\nB\nC\nabc\nOWNED!\nOWNED\nrewrite\noldnew\nnew\nnew!\nsaved\nindex\nFIELDnew\nsecond\nab\nHEAP!\nTEMP!\nindex\nARRAY!\nOWNED?\nOWNED?\nCAPTURED!\nCAPTURED!\nOWNED*\nOWNED\nchanged.\nsecond.\n",
        "",
        "",
    );
    for parameter in [
        "add.left",
        "add.right",
        "forward.left",
        "forward.right",
        "invoke.value",
    ] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Read));
    }
    let rust = analysis.emit_rust();
    assert!(rust.contains("let string_left: &String"));
    let fixed = native(
        "function main(){const values=[\"a\",\"é\",\"😀\"];console.log(values[0]+values[1]);console.log(values[2]+values[0]);console.log(values[1]);}",
        "aé\n😀a\né\n",
        "",
        "",
    );
    assert!(fixed.emit_rust().contains("LiteralStringArray"));
}

#[test]
fn strict_types_transfers_and_right_operand_conflicts_preserve_existing_contracts() {
    for expression in [
        "[\"x\"]+\"x\"",
        "\"x\"+[\"x\"]",
        "\"x\"-\"y\"",
        "\"x\"*\"y\"",
    ] {
        let source = format!("function main(){{console.log({expression});}}");
        assert_eq!(
            check(&source).err().expect("mixed operands must fail").code,
            "type",
            "{source}"
        );
    }
    for (source, code) in [
        ("function main(){const result:int=\"x\"+\"y\";}", "type"),
        ("function main(){const result:number=\"x\"+\"y\";}", "type"),
        ("function main(){const result:string=1+2;}", "type"),
        ("function main(){let value=1;value+=\"x\";}", "type"),
        (
            "function main(){const value=\"x\";const moved=value;console.log(value+\"y\");}",
            "transferred",
        ),
        (
            "type Box={text:string;};function main(){const box:Box={text:\"x\"};const moved=box.text;console.log(box.text+\"y\");}",
            "partial-transfer",
        ),
        (
            "function take(value:string):string{return value;}function main(){const value=\"x\".toUpperCase();console.log(value+take(value));}",
            "conflicting-access",
        ),
        (
            "type Box={text:string;};function rewrite(box:Box):string{box.text=\"y\";return \"y\";}function main(){const box:Box={text:\"x\"};console.log(box.text+rewrite(box));}",
            "conflicting-access",
        ),
        (
            "function rewrite(values:string[]):string{values[0]=\"y\";return \"y\";}function main(){const values=[\"x\"];console.log(values[0]+rewrite(values));}",
            "conflicting-access",
        ),
        (
            "function take(value:string):string{return value;}function main(){const value=\"x\".toUpperCase();const read=():string=>{return value+\"!\";};const moved=take(value);console.log(read());}",
            "capture-conflict",
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
    native(
        "function take(value:string):string{return value;}function main(){const source=\"x\".toUpperCase();const result=source+\"!\";const moved=take(source);console.log(result);console.log(moved);}",
        "X!\nX\n",
        "",
        "",
    );
}

#[test]
fn exact_capacity_allocates_only_the_result_and_disposes_normally_and_on_unwind() {
    native(
        &(ADD.to_owned()
            + "function chain(a:string,b:string,c:string):string{return a+b+c;}function main(){}"),
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for (left,right,expected) in [("","",""),("","é😀","é😀"),("left","","left"),("a\0","e\u{301}","a\0e\u{301}")] {
            for owned in [false,true] {
                let left=if owned{String::Owned(left.to_owned())}else{String::Borrowed(left)};
                let right=if owned{String::Owned(right.to_owned())}else{String::Borrowed(right)};
                let input=LIVE.load(Ordering::SeqCst);let allocations=ALLOCATIONS.load(Ordering::SeqCst);
                let result=std::hint::black_box(function0(std::hint::black_box(&left),std::hint::black_box(&right)));
                assert_eq!(result.as_ref(),expected);assert!(matches!(result,String::Owned(_)));
                assert_eq!(ALLOCATIONS.load(Ordering::SeqCst)-allocations,usize::from(!expected.is_empty()));
                assert_eq!(LIVE.load(Ordering::SeqCst)-input,expected.len() as isize);
                drop(result);assert_eq!(LIVE.load(Ordering::SeqCst),input);
                assert!(std::panic::catch_unwind(||{let a=left.clone();let b=right.clone();let result=function0(&a,&b);std::hint::black_box(&result);panic!("drop inputs and result");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),input);
                drop(left);drop(right);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
            }
        }
        let a=String::Owned("é".repeat(128));let b=String::Owned("😀".repeat(128));let empty=String::Borrowed("");let input=LIVE.load(Ordering::SeqCst);let count=ALLOCATIONS.load(Ordering::SeqCst);
        let result=std::hint::black_box(function1(&a,&b,&empty));assert_eq!(ALLOCATIONS.load(Ordering::SeqCst)-count,1,"a chain reserves its total once");assert_eq!(result.len(),768);assert_eq!(LIVE.load(Ordering::SeqCst)-input,768);
        drop(a);drop(b);assert_eq!(&result[..256],"é".repeat(128));assert_eq!(&result[256..],"😀".repeat(128));drop(result);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
        std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
    );
}

// Test-only forwarding allocator; generated implementation stays safe Rust.
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

#[test]
fn self_assignment_reuses_local_buffers_and_compacts_owned_results() {
    let source=r#"
        function build(input:string,n:int):string{
            let text="";let i=0;
            while(i<n){text=text+input+"|";i+=1;}
            return text;
        }
        function main(){let text="";text=text+"a"+"é";console.log(text);}
    "#;
    let analysis=native(source,"aé\n",r#"
        let input=String::Owned("é🙂".to_owned());let expected="é🙂|".repeat(63);
        let baseline=LIVE.load(Ordering::SeqCst);
        ALLOCATIONS.store(0,Ordering::SeqCst);
        let output=function0(&input,std::hint::black_box(63));
        let allocations=ALLOCATIONS.load(Ordering::SeqCst);
        assert!(allocations<=12,"amortized appends, not63 new destinations: {allocations}");
        assert_eq!(output.as_ref(),expected);
        match &output{String::Owned(text)=>assert_eq!(text.capacity(),text.len()),_=>panic!("owned output")}
        drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let output=function0(&input,2);drop(input);assert_eq!(output.as_ref(),"é🙂|é🙂|");
    "#,ALLOCATOR);
    let rust=analysis.emit_rust();assert!(rust.contains("let sink=tsr_string_reserve("));
    assert!(rust.contains("tsr_string_finish(value"));
}

#[test]
fn self_assignment_preserves_operand_effects_snapshots_comma_and_capture_rules() {
    let source=r#"
        type Box={text:string;};
        function tag(log:string[],text:string):string{log.push(clone(text));return text;}
        function build(log:string[],input:string):string{
            let text="start".repeat(1);const saved=clone(text);
            text=text+tag(log,"a")+tag(log,"b")+input.slice(2);
            console.log(saved);return text;
        }
        function comma():string{
            let text="x";const number=(text=text+"y"+"z",9);
            text=text+(number===9?"!":"bad");return text;
        }
        function captured(input:string):string{
            let text=String(input);
            const action:Once<Function<string>>=():string=>{text=text+"z";return text;};
            return action();
        }
        function selfReference(input:string):string{let text=String(input);text=text+text+"!";return text;}
        function parameter(text:string):string{let copy=String(text);copy=copy+"!";return copy;}
        function checked():string{
            let text="base";
            try{text=text+"a"+fail();}catch(error){console.log(text);}
            return text;
        }
        function fail():string{throw "boom";}
        function cleanup():string{
            let text="before";
            try{return text;}finally{text="after";text=text+"!";console.log(text);}
        }
        function main(){
            const log:string[]=[];console.log(build(log,"é🙂"),log.join(","));
            console.log(comma());console.log(captured("é"));
            console.log(selfReference("é"),parameter("é"));
            const box:Box={text:"f"};box.text=box.text+"g";console.log(box.text);
            console.log(checked());
            console.log(cleanup());
        }
    "#;
    native(source,"start\nstartab🙂 a,b\nxyz!\néz\néé! é!\nfg\nbase\nbase\nafter!\nbefore\n","","");
    let error=check(r#"function main(){let text="a";const observe=():int=>text.length;text=text+"z";console.log(observe());}"#).err().expect("live capture must prevent replacement");
    assert_eq!(error.code,"capture-conflict");
    let error=check(r#"function parameter(text:string):string{text=text+"!";return text;}function main(){}"#).err().expect("parameters remain immutable");
    assert_eq!(error.code,"unsupported");
}

#[test]
fn self_assignment_keeps_fused_numeric_and_mixed_builders_on_the_original_path() {
    native(r#"
        function build(n:int):string{
            let text="";text=text+"seed";let i=0;
            while(i<n){text=text+String(i)+":";i+=1;}
            return text;
        }
        function main(){}
    "#,"",r#"
        let baseline=LIVE.load(Ordering::SeqCst);ALLOCATIONS.store(0,Ordering::SeqCst);
        let output=function0(std::hint::black_box(3));
        assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),4);
        assert_eq!(output.as_ref(),"seed0:1:2:");
        match &output{String::Owned(text)=>assert_eq!(text.capacity(),33),_=>panic!("owned numeric result")}
        drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
    "#,ALLOCATOR);
}

#[test]
fn self_assignment_proof_budget_falls_back_without_rejecting_source() {
    let source=format!("function main(){{let text=\"\";{}console.log(text.length);}}","text=text+\"x\";".repeat(11_000));
    let analysis=check(&source).unwrap();let rust=analysis.emit_rust();
    assert!(!rust.contains("let sink=tsr_string_reserve("));
    assert!(!rust.contains("tsr_string_finish(value"));
}

#[test]
fn appended_single_digits_reuse_utf8_destinations_and_other_integers_keep_exact_spelling() {
    native(
        r#"function label(value:int):string{return "é"+String(value)+"🙂";}function main(){}"#,
        "",
        r#"
        for value in 0..=9i64 {
            let expected=format!("é{value}🙂");
            let mut text=std::string::String::with_capacity(64);text.push_str("é");
            let pointer=text.as_ptr();let capacity=text.capacity();
            ALLOCATIONS.store(0,Ordering::SeqCst);
            tsr_push_int(std::hint::black_box(&mut text),std::hint::black_box(value));
            text.push_str("🙂");
            assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),0);
            assert_eq!(text.as_ptr(),pointer);assert_eq!(text.capacity(),capacity);
            assert_eq!(text,expected);assert_eq!(function0(value).as_ref(),expected);
        }
        let mut random=0x715f_32c9_9b8a_408du64;
        for value in (-19..=19i64).chain([i64::MIN,i64::MAX,99,100,999,1000])
            .chain((0..1024).map(|_|{random=random.wrapping_mul(6364136223846793005).wrapping_add(1);random as i64})) {
            assert_eq!(function0(value).as_ref(),format!("é{value}🙂"));
        }
        for value in [0.0,-0.0,1.0,9.0,10.0,-1.0] {
            let mut text=std::string::String::from("é");
            assert!(tsr_push_whole_number(&mut text,std::hint::black_box(value)));
            assert_eq!(text,format!("é{}",value as i64));
        }
        "#,
        ALLOCATOR,
    );
}

#[test]
fn appended_numbers_write_digits_into_the_result_in_operand_order() {
    let source = r#"enum Phase { Ready, Busy = 4, Half = 2.5 }
function next(log: string[], value: int): int { log.push(`n${value}`); return value; }
function tag(log: string[]): string { log.push("left"); return "a"; }
function main() {
  const log: string[] = [];
  const first = tag(log) + String(next(log, -12));
  const values: number[] = [-0, 2.5, 1e21, 9007199254740992, 0 / 0, -1 / 0];
  const parts: string[] = [];
  for (const value of values) { parts.push("v" + String(value)); parts.push(`t${value}`); }
  const phase: Phase = Phase.Half;
  console.log(first, log.join(","), parts.join(" "), "p" + String(phase), "i" + String(-9223372036854775807 - 1), `m${9223372036854775807}`);
}"#;
    let analysis = native(
        source,
        "a-12 left,n-12 v0 t0 v2.5 t2.5 v1e+21 t1e+21 v9007199254740992 t9007199254740992 vNaN tNaN v-Infinity t-Infinity p2.5 i-9223372036854775808 m9223372036854775807\n",
        "",
        "",
    );
    let rust = analysis.emit_rust();
    assert!(rust.contains("tsr_push_int(&mut output, number)"), "String(int) appends without a temporary");
    assert!(rust.contains("tsr_push_whole_number(&mut output, number)"), "String(number) appends without a temporary");
    assert!(rust.contains("tsr_push_int(&mut template_output, template_value)"), "int interpolation appends digits");
}

#[test]
fn addition_chains_build_one_buffer_and_evaluate_operands_in_order() {
    let source = r#"enum Phase { Ready, Half = 2.5 }
function tag(log: string[], text: string): string { log.push(clone(text)); return text; }
function num(log: string[], value: int): int { log.push(String(value)); return value; }
function main() {
  const log: string[] = [];
  const joined = tag(log, "a") + String(num(log, -3)) + tag(log, "b") + (tag(log, "c") + tag(log, "d")) + String(2.5) + String(Phase.Half) + "é";
  const empty = "" + "" + "";
  console.log(joined, log.join(","), empty.length, "[" + empty + "]" + String(0.5));
}"#;
    let analysis = native(source, "a-3bcd2.52.5é a,-3,b,c,d 0 []0.5\n", "", "");
    let rust = analysis.emit_rust();
    assert!(rust.contains("try_fold(0usize, usize::checked_add)"), "chains reserve their total size once");
    assert!(!rust.contains("tsr_string_add(string_left"), "no intermediate copies in a chain");
}

#[test]
fn compound_append_to_a_local_reuses_its_buffer() {
    let source = r#"type Box = { text: string; };
function main() {
  let text = "";
  for (let i = 0; i < 5; i++) { text += String(i); text += ","; }
  let twice = "ab";
  twice += twice;
  let empty = "x";
  empty += "";
  const box: Box = { text: "f" };
  box.text += "g";
  let joined = "";
  for (const word of ["é", "z"]) { joined += word; }
  console.log(text, twice, empty, box.text, joined);
}"#;
    let analysis = native(source, "0,1,2,3,4, abab x fg éz\n", "", "");
    let rust = analysis.emit_rust();
    // Locals append in place; self-reference and fields keep the copying form.
    assert_eq!(rust.matches("tsr_string_append(&mut").count(), 4, "{rust}");
    assert_eq!(rust.matches("tsr_string_add(compound_left").count(), 2, "{rust}");
}

#[test]
fn compound_append_of_a_sum_appends_each_operand_in_order() {
    let source = r#"function tag(log: string[], text: string): string { log.push(clone(text)); return text; }
function main() {
  const log: string[] = [];
  let text = "";
  for (let i = 0; i < 3; i++) { text += tag(log, "a") + String(i); text += "<" + tag(log, "b") + ">" + String(2.5) + "é"; }
  let other = "start";
  other += other + "!";
  let empty = "";
  empty += "" + "";
  console.log(text, log.join(","), other, empty.length);
}"#;
    let analysis = native(source, "a0<b>2.5éa1<b>2.5éa2<b>2.5é a,b,a,b,a,b startstart! 0\n", "", "");
    let rust = analysis.emit_rust();
    // Sums on the right side append their operands; a self-reference keeps the copy.
    assert_eq!(rust.matches("tsr_string_reserve(&mut").count(), 3, "{rust}");
    assert_eq!(rust.matches("tsr_string_add(compound_left").count(), 1, "{rust}");
}

#[test]
fn appended_slices_of_locals_borrow_their_range() {
    let source = r#"function main() {
  const first = "héllo".repeat(1);
  let text = "";
  for (let k = 0; k < 2; k++) { text += first.slice(); }
  text += first.slice(1, 3) + "|" + first.slice(-2);
  const joined = "<" + first.slice(3) + ">" + first.slice(9) + "!";
  console.log(text, joined);
}"#;
    let analysis = native(source, "héllohélloé|lo <llo>!\n", "", "");
    let rust = analysis.emit_rust();
    assert_eq!(rust.matches("tsr_slice_str(string_receiver").count(), 5, "{rust}");
    assert!(!rust.contains("tsr_slice(string_receiver"), "no owned slice copies are made for appends");
}

#[test]
fn fractional_appends_reuse_output_without_changing_text_or_scientific_allocations() {
    let source = support::format_round_trip(r#"
enum Phase { Whole = 4, Half = 2.5 }
enum Small { Minus = -7, Zero = 0, Exact = 9007199254740991 }
enum Wide { Above = 9007199254740992, Huge = 1e21 }
function binary(value:number):string{return "é😀e123:"+String(value);}
function chain(value:number):string{return "é😀e123:"+String(value)+":end";}
function append(value:number):string{let output="é😀e123:";output+=String(value)+":end";return output;}
function enumBinary():string{return "é😀e123:"+String(Phase.Half);}
function enumChain():string{return "é😀e123:"+String(Phase.Half)+":end";}
function reference(value:number):string{return String(value);}
function wholeBinary():string{return "v"+String(Number(Small.Exact));}
function wholeChain():string{return "v"+String(Small.Minus)+"|"+String(Small.Zero);}
function wholeAppend():string{let output="v";output+=String(Number(Small.Exact))+"|"+String(Small.Minus);return output;}
function wide():string{return "v"+String(Wide.Huge)+"|"+String(Wide.Above);}
function main(){}
"#);
    native(&source, "", r#"
        let baseline=LIVE.load(Ordering::SeqCst);
        let functions=[(function0 as fn(f64)->String,""),(function1 as fn(f64)->String,":end"),(function2 as fn(f64)->String,":end")];
        for value in [0.5,1.25,2.5,12.75]{
            for &(function,suffix) in &functions{
                let expected=format!("é😀e123:{value}{suffix}");
                let count=ALLOCATIONS.load(Ordering::SeqCst);let output=function(std::hint::black_box(value));
                assert_eq!(ALLOCATIONS.load(Ordering::SeqCst)-count,1,"one owned destination, no temporary number text");
                assert_eq!(output.as_ref(),expected);drop(output);drop(expected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            }
        }
        for (function,expected) in [(function3 as fn()->String,"é😀e123:2.5"),(function4 as fn()->String,"é😀e123:2.5:end")]{
            let count=ALLOCATIONS.load(Ordering::SeqCst);let output=function();
            assert_eq!(ALLOCATIONS.load(Ordering::SeqCst)-count,1);assert_eq!(output.as_ref(),expected);
            drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        assert_eq!(function6().as_ref(),"v9007199254740991");
        assert_eq!(function7().as_ref(),"v-7|0");
        assert_eq!(function8().as_ref(),"v9007199254740991|-7");
        assert_eq!(function9().as_ref(),"v1e+21|9007199254740992");
        assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let verify=|value:f64|{
            for &(function,suffix) in &functions{
                let count=ALLOCATIONS.load(Ordering::SeqCst);
                let mut old=std::string::String::with_capacity("é😀e123:".len()+24+suffix.len());old.push_str("é😀e123:");
                if !tsr_push_whole_number(&mut old,value){old.push_str(function5(value).as_ref());}old.push_str(suffix);
                let old_allocations=ALLOCATIONS.load(Ordering::SeqCst)-count;
                let count=ALLOCATIONS.load(Ordering::SeqCst);let output=function(std::hint::black_box(value));
                let new_allocations=ALLOCATIONS.load(Ordering::SeqCst)-count;
                assert_eq!(output.as_ref(),old);assert!(matches!(output,String::Owned(_)));
                assert!(new_allocations<=old_allocations,"allocations {value:e}: {old_allocations} -> {new_allocations}");
                drop(output);drop(old);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            }
        };
        for value in [0.0,-0.0,f64::NAN,f64::INFINITY,f64::NEG_INFINITY,f64::MAX,f64::MIN_POSITIVE,f64::from_bits(1),1e-7,1e-6,1e20,1e21,-1e21,9007199254740991.0,9007199254740992.0,5.960464477539062e-7,123456789012345680000.0]{verify(value);}
        let mut bits=0x123456789abcdef0u64;
        for _ in 0..2048{bits=bits.wrapping_mul(6364136223846793005).wrapping_add(1);verify(f64::from_bits(bits));}
    "#, ALLOCATOR);
    let isolated = support::format_round_trip("function text(value:number):string{return \"v\"+String(value);}function main(){}");
    native(&isolated, "", r#"
        assert_eq!(function0(0.5).as_ref(),"v0.5");
        assert_eq!(function0(f64::INFINITY).as_ref(),"vInfinity");
        assert_eq!(function0(1e21).as_ref(),"v1e+21");
    "#, "");
}
