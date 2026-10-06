mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("logical.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("logical.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("logical-{level}"));
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

#[test]
fn truth_tables_values_precedence_and_all_expression_contexts_are_exact() {
    native(
        r#"
        function conjunction(left:boolean,right:boolean):boolean{let target=left;return target&&=right;}
        function disjunction(left:boolean,right:boolean):boolean{let target=left;return target||=right;}
        type Box={flag:boolean;};
        function pair(left:boolean,right:boolean){console.log(left);console.log(right);}
        function main(){
            let a=false;let b=true;let c=false;
            console.log(a ||= b &&= c);console.log(a);console.log(b);console.log(c);
            a=false;b=true;console.log(a ||= b || false);console.log(a &&= b && false);
            a=true;const values=[a&&=false,a||=true];console.log(values[0]);console.log(values[1]);
            const box:Box={flag:a&&=false};console.log(box.flag);
            pair(a||=true,a&&=false);console.log(a);
            const input=[1,2];const result=input.map((n:int):boolean=>{return a||=n>1;});console.log(result[0]);console.log(result[1]);
            {const select=():boolean=>{return a&&=false;};console.log(select());}console.log(a);
            let again=false;let count=0;for(again||=true;again;again&&=false){count++;continue;}console.log(count);console.log(again);
            let runs=false;while(runs||=false){console.log("unreachable");}console.log(runs);
            do{runs||=true;}while(runs&&=false);console.log(runs);
            const guards=[true,false];let index=0;console.log(guards[index++]&&=false);console.log(index);console.log(guards[0]);
        }
    "#,
        "false\nfalse\nfalse\nfalse\ntrue\nfalse\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\nfalse\ntrue\nfalse\nfalse\n1\nfalse\nfalse\nfalse\nfalse\n1\nfalse\n",
        r#"
        for left in [false,true] {for right in [false,true] {assert_eq!(function0(left,right),left&&right);assert_eq!(function1(left,right),left||right);}}
    "#,
        "",
    );
}

#[test]
fn selected_storage_effects_captures_loans_and_partial_fields_are_preserved() {
    let analysis = native(
        r#"
        type Box={flag:boolean;text:string;};
        function change(values:boolean[],answer:boolean):boolean{values[0]=false;values.push(true);console.log("rhs");return answer;}
        function mark():int{console.log("index");return 0;}
        function update(box:Box,right:boolean):boolean{return box.flag&&=right;}
        function counter():(right:boolean)=>boolean{let flag=false;return (right:boolean):boolean=>{return flag||=right;};}
        function quiet(values:boolean[],answer:boolean):boolean{values[0]=false;values.push(true);return answer;}
        function growth(values:boolean[],answer:boolean):boolean{return values[0]&&=quiet(values,answer);}
        function reversed(values:boolean[]):boolean{values.reverse();return true;}
        function reverseApply(values:boolean[]):boolean{return values[0]&&=reversed(values);}
        function removed(values:boolean[]):boolean{values.remove(0);return false;}
        function removeApply(values:boolean[]):boolean{return values[0]&&=removed(values);}
        function lostApply(values:boolean[]):boolean{return values[1]&&=removed(values);}
        function orGrowth(values:boolean[],answer:boolean):boolean{return values[0]||=quiet(values,answer);}
        function main(){
            const values=[true];console.log(values[mark()]&&=change(values,true));console.log(values[0]);console.log(values[1]);
            console.log(values[mark()]||=change(values,false));console.log(values[0]);console.log(values.length);
            const rows=[[true]];let i=0;let j=0;console.log(rows[i++][j++]&&=false);console.log(i);console.log(j);
            const box:Box={flag:true,text:"owner😀"};console.log(update(box,false));const text=box.text;console.log(box.flag||=true);console.log(box.flag&&=false);box.text=text;console.log(box.text);
            let captured=false;{const flip=(right:boolean):boolean=>{return captured||=right;};console.log(flip(false));console.log(flip(true));}console.log(captured);
            const retained=counter();console.log(retained(false));console.log(retained(true));console.log(retained(false));
            let snapshot=false;const read=():boolean=>{return snapshot;};snapshot||=true;console.log(read());console.log(snapshot);
            let lazy=false;console.log(false && (lazy||=true));console.log(true || (lazy||=true));console.log(lazy);
            const flags=[true,false];for(let flag of flags){console.log(flag&&=false);}console.log(flags[0]);
        }
    "#,
        "index\nrhs\ntrue\ntrue\ntrue\nindex\ntrue\ntrue\n2\nfalse\n1\n1\nfalse\ntrue\nfalse\nowner😀\nfalse\ntrue\ntrue\nfalse\ntrue\ntrue\nfalse\ntrue\nfalse\ntrue\nfalse\nfalse\nfalse\ntrue\n",
        r#"
        for answer in [false,true]{let mut values=NumericArray::from([true,false,false,false]);assert_eq!(function5(&mut values,answer),answer);assert_eq!(values.len(),5);assert_eq!(values.get(0),Some(&answer));assert_eq!(values.get(4),Some(&true));}
        for answer in [false,true]{let mut values=NumericArray::from([false,false,false,false]);assert_eq!(function11(&mut values,answer),answer);assert_eq!(values.len(),5);assert_eq!(values.get(0),Some(&answer));}
        let mut values=NumericArray::from([true,false]);assert!(function7(&mut values));assert_eq!(values.as_slice(),&[true,true]);
        let mut values=NumericArray::from([true,true]);assert!(!function9(&mut values));assert_eq!(values.as_slice(),&[false]);
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let mut values=NumericArray::from([false,true]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function10(&mut values))).is_err());assert_eq!(values.as_slice(),&[true]);std::panic::set_hook(hook);
    "#,
        "",
    );
    assert_eq!(analysis.contracts().get("update.box"), Some(&Mode::Mutate));
    assert_eq!(
        analysis.contracts().get("change.values"),
        Some(&Mode::Mutate)
    );
    let fixed = native(
        "function main(){const flags=[true,false,true];console.log(flags[0]&&=false);console.log(flags[1]||=true);}",
        "false\ntrue\n",
        "",
        "",
    );
    assert!(fixed.emit_rust().contains(": [bool; 3] = ["));
}

#[test]
fn conditional_ownership_diagnostics_bounds_and_traps_preserve_contracts() {
    for (source, code) in [
        ("function main(){let n=1;n&&=true;}", "type"),
        ("function main(){let n=1.0;n||=true;}", "type"),
        ("function main(){let s=\"text\";s||=true;}", "type"),
        ("function main(){let flag=true;flag&&=1;}", "type"),
        (
            "function main(){const flag=false;flag&&=false;}",
            "unsupported",
        ),
        (
            "function f(flag:boolean){flag||=true;}function main(){}",
            "unsupported",
        ),
        ("function main(){console.log(true&&=false);}", "unsupported"),
        (
            "function f():boolean{return true;}function main(){f()||=false;}",
            "unsupported",
        ),
        (
            "function main(){const a=[true];a.length||=true;}",
            "unsupported",
        ),
        (
            "function f():int{let flag=true;return flag&&=false;}function main(){}",
            "type",
        ),
        (
            "function f():number{let flag=true;return flag||=false;}function main(){}",
            "type",
        ),
        (
            "type Box={flag:boolean;};function read(box:Box,value:boolean):boolean{return box.flag;}function main(){const box:Box={flag:true};read(box,box.flag&&=false);}",
            "conflicting-access",
        ),
        (
            "type Box={flag:boolean;values:boolean[];};function main(){const box:Box={flag:true,values:[true]};box.values[0]&&=(box.flag||=false);}",
            "conflicting-access",
        ),
        (
            "function main(){const rows=[[true]];rows[0][0]&&=(rows[0][0]||=false);}",
            "conflicting-access",
        ),
        (
            "function main(){const flags=[true];for(const flag of flags){flags[0]||=false;}}",
            "iteration-conflict",
        ),
        (
            "type Box={flag:boolean;};function main(){const box:Box={flag:false};const read=():boolean=>{return box.flag;};box.flag||=true;console.log(read());}",
            "capture-conflict",
        ),
        (
            "type Box={flag:boolean;};function main(){const box:Box={flag:true};const moved=box;box.flag&&=false;}",
            "transferred",
        ),
        (
            "type Box={part:Part;};type Part={flag:boolean;};function main(){const box:Box={part:{flag:true}};const moved=box.part;box.part.flag||=true;}",
            "partial-transfer",
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
    for op in ["&&=", "||="] {
        for initial in ["true", "false"] {
            let source = format!(
                "type Box={{flag:boolean;}};function take(box:Box):boolean{{box;return true;}}function main(){{const box:Box={{flag:true}};let flag={initial};flag{op}take(box);console.log(box.flag);}}"
            );
            assert_eq!(check(&source).err().unwrap().code, "transferred");
        }
        let source = format!(
            "type Box={{flag:boolean;text:string;}};function take(text:string):boolean{{text;return true;}}function main(){{const box:Box={{flag:true,text:\"owned\"}};box.flag{op}take(box.text);console.log(box.text);}}"
        );
        assert_eq!(check(&source).err().unwrap().code, "partial-transfer");
        let restored = format!(
            "type Box={{flag:boolean;text:string;}};function take(text:string):boolean{{text;return true;}}function main(){{const box:Box={{flag:true,text:\"owned\"}};const replacement=clone(box.text);box.flag{op}take(box.text);box.text=replacement;console.log(box.text);}}"
        );
        assert!(
            check(&restored).is_ok(),
            "unconditional sibling restoration {op}"
        );
    }
    for source in [
        "function main(){let flag=true;(true?flag:flag)??=false;}",
        "function main(){let text=\"old\";text=\"new\";console.log(text=\"owned\");}",
    ] {
        assert!(check(source).is_err());
    }
    let deep = format!(
        "function main(){{const values={}1{};}}",
        "[".repeat(200),
        "]".repeat(200)
    );
    assert_eq!(check(&deep).err().unwrap().code, "unsupported");
    let chain = format!(
        "function main(){{let flag=false;console.log({}true);}}",
        "flag||=".repeat(200)
    );
    assert_eq!(check(&chain).err().unwrap().code, "unsupported");
    native(
        r#"
        type Box={flag:boolean;};
        function rhs(values:boolean[]):boolean{values[0]=false;values.push(true);const n=1/0;return n===0;}
        function apply(values:boolean[]):boolean{return values[0]&&=rhs(values);}
        function bounds(rows:boolean[][],counts:int[]):boolean{return rows[counts[0]++][counts[1]++]||=false;}
        function main(){let flag=false;console.log(flag&&=(1/0===0));flag=true;console.log(flag||=(9223372036854775807+1===0));}
    "#,
        "false\ntrue\n",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        let mut values=NumericArray::from([true]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function1(&mut values))).is_err());assert_eq!(values.as_slice(),&[false,true]);
        for index in [-1,1] {let mut rows=Array::from([NumericArray::from([false])]);let mut counts=NumericArray::from([index,0]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function2(&mut rows,&mut counts))).is_err());assert_eq!(counts.as_slice(),&[index+1,0]);}
        let mut rows=Array::from([NumericArray::from([false])]);let mut counts=NumericArray::from([0,1]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function2(&mut rows,&mut counts))).is_err());assert_eq!(counts.as_slice(),&[1,2]);
        std::panic::set_hook(hook);
    "#,
        "",
    );
}

#[test]
fn scalar_updates_allocate_nothing_and_owned_storage_disposes_normally_and_on_unwind() {
    let analysis = native(
        r#"
        type Box={flag:boolean;text:string;};
        function conjunction(box:Box,right:boolean):boolean{return box.flag&&=right;}
        function disjunction(box:Box,right:boolean):boolean{return box.flag||=right;}
        function fail(box:Box):boolean{return box.flag&&=(1/0===0);}
        function main(){}
    "#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for text in ["","owned\0😀e\u{301}"] {
            let mut owner=Record0{field0:false,field1:String::Owned(text.to_owned())};
            for left in [false,true] {for right in [false,true] {for (operation,and) in [(function0 as fn(&mut Record0,bool)->bool,true),(function1,false)] {
                owner.field0=std::hint::black_box(left);let before=LIVE.load(Ordering::SeqCst);let count=ALLOCATIONS.load(Ordering::SeqCst);
                let result=std::hint::black_box(operation)(std::hint::black_box(&mut owner),std::hint::black_box(right));
                let expected=if and{left&&right}else{left||right};assert_eq!(result,expected);assert_eq!(owner.field0,expected);
                assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),before);
            }}}
            assert_eq!(owner.field1.as_ref(),text);drop(owner);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:true,field1:String::Owned(text.to_owned())};function2(&mut owner);}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:false,field1:String::Owned(text.to_owned())};let result=function1(&mut owner,true);std::hint::black_box((result,owner.field0));panic!("dispose result and owner");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
    );
    assert_eq!(
        analysis.contracts().get("conjunction.box"),
        Some(&Mode::Mutate)
    );
    assert_eq!(
        analysis.contracts().get("disjunction.box"),
        Some(&Mode::Mutate)
    );
}

// This observer forwards System; the generated implementation contains no unsafe.
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
