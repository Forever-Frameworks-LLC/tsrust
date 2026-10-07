mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("conditional.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("conditional.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("conditional-{level}"));
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
fn precedence_lazy_effects_scalar_bits_and_expression_contexts_are_exact() {
    native(
        r#"
        function integer(flag:boolean,left:int,right:int):int{return flag?left:right;}
        function number(flag:boolean,left:number,right:number):number{return flag?left:right;}
        function contextual(flag:boolean):number{return flag?-0:1;}
        function inferred(flag:boolean):number{return flag?1:2.0;}
        function anchor(flag:boolean,value:int):int{return (flag?value:0)<<1;}
        function pair(left:int,right:int):int{return left*100+right;}
        function predicate(counts:int[]):boolean{counts[0]++;return counts[0]===1;}
        function yes(counts:int[]):int{counts[1]++;return 7;}
        function no(counts:int[]):int{counts[2]++;return 9;}
        function marked(counts:int[]):int{return predicate(counts)?yes(counts):no(counts);}
        function lazy(flag:boolean):int{return flag?3:1/0;}
        function contexts():int[]{
            let n=0;const a=true?n=3:n=9;const b=n=false?7:11;
            const c=false?1:true?2:3;const d=true?false?4:5:6;
            const e=false||true?8:9;const f=true? n+=2:n*=4;
            const values=[true?n++:++n,false?0:n];const row:Cell={n:false?2:n};
            const called=pair(true?n+=1:0,false?0:n*=2);
            const input=[1,2];const mapped=input.map((value:int):int=>{return value===1?value+10:value+20;});
            {const change=(flag:boolean):int=>{return flag?n=2:n=4;};change(false);}
            let count=0;while(count<2?true:false){count++;}let sum=0;for(let i=0;i<3;i<2?i++:i+=1){sum+=i;continue;}
            const selected=[7,9];const indexed=selected[false?1:0];
            return[a,b,c,d,e,f,values[0],values[1],row.n,called,mapped[0],mapped[1],n,count,sum,indexed];
        }
        type Cell={n:int;};
        function main(){console.log(false?false:true?true:false);console.log(true?"left":"right");console.log(true?3:1/0);}
    "#,
        "true\nleft\n3\n",
        r#"
        for flag in [false,true]{for (left,right) in [(i64::MIN,i64::MAX),(-7,9),(0,1)]{assert_eq!(function0(flag,left,right),if flag{left}else{right});}}
        let bits=[0_u64,0x8000000000000000,1,0x0010000000000000,0x7fefffffffffffff,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000042,0xfff8000000000001];for flag in [false,true]{for &left in &bits{for &right in &bits{assert_eq!(function1(flag,f64::from_bits(left),f64::from_bits(right)).to_bits(),if flag{left}else{right});}}}
        assert_eq!(function2(true).to_bits(),(-0.0_f64).to_bits());assert_eq!(function2(false),1.0);assert_eq!(function3(true),1.0);assert_eq!(function3(false),2.0);assert_eq!(function4(true,1_i64<<40),1_i64<<41);
        let mut counts=NumericArray::from([0,0,0]);assert_eq!(function9(&mut counts),7);assert_eq!(counts.as_slice(),&[1,1,0]);assert_eq!(function9(&mut counts),9);assert_eq!(counts.as_slice(),&[2,1,1]);assert_eq!(function10(true),3);
        assert_eq!(function11().as_slice(),&[3,11,2,5,8,13,13,14,14,1530,11,22,4,2,3,7]);
    "#,
        "",
    );
    // Literal-only bitwise keeps the Number default; a concrete branch anchors int.
    let literal = check("function main(){console.log((true?1:2)<<1);}")
        .unwrap()
        .emit_rust();
    assert!(literal.contains("tsr_math_uint32"));
}

#[test]
fn owned_values_loans_callable_origins_and_fixed_storage_follow_branch_joins() {
    let analysis = native(
        r#"
        type Row={n:int;text:string;values:int[];};
        function record(flag:boolean):Row{return flag?{n:1,text:"left",values:[7,8]}:{n:2,text:"right",values:[]};}
        function string(flag:boolean,left:string,right:string):string{return flag?left:right;}
        function integers(flag:boolean):int[]{return flag?[1,2]:[3,4,5];}
        function numbers(flag:boolean):number[]{return flag?[-0,1]:[Number.NaN,Number.POSITIVE_INFINITY];}
        function booleans(flag:boolean):boolean[]{return flag?[true]:[false,true];}
        function strings(flag:boolean):string[]{return flag?["é","😀"]:["e\u0301","\u0000"] ;}
        function nested(flag:boolean):int[][]{return flag?[[1],[2,3]]:[[],[4]];}
        function read(values:int[],value:int):int{return values.length+value;}
        function grow(values:int[]):int{values.push(5);return values.length;}
        function mixed(values:int[],flag:boolean):int{const fixed=[1];return read(flag?fixed:values,0);}
        function same(value:int):int{return value+1;}
        function make(flag:boolean):()=>int{const text="owned";const f=():int=>{return text.length;};const g=flag?f:f;return g;}
        function direct(flag:boolean,value:int):int{return (flag?same:same)(value);}
        function apply(callback:(value:int)=>int,flag:boolean,value:int):int{return (flag?callback:callback)(value);}
        function main(){
            const a=[1];const b=[2,3];console.log((true?a:b).length);console.log(read(false?a:b,0));
            const c=[1,2,3,4];const d=[9];console.log(grow(true?c:d));console.log(c.length);console.log(d.length);
            const e=[1,2];const f=[3,4,5];const chosen=false?e:f;console.log(chosen.length);
            const first:Row={n:1,text:"one",values:[1]};const second:Row={n:2,text:"two",values:[2]};console.log((true?first:second).text);
            const owner:Row={n:1,text:"kept",values:[1]};const moved=owner.text;console.log(true?(owner.n=2):(owner.n=3));owner.text=moved;console.log(owner.text);
            const x=true?"a":"b";console.log(x.trim());console.log((false?"unselected":"é🙂").length);
            const returned=make(true);console.log(returned());console.log(direct(false,4));console.log(apply(same,true,6));
            let target=0;{const change=(value:int):int=>{return target+=value;};console.log((true?change:change)(3));console.log((false?change:change)(2));}console.log(target);
            const fixed=[1,2,3];console.log(fixed[1]);
            const borrowed="alias";console.log((true?borrowed:" new ".repeat(2)).trim());console.log(read(false?c:[1,2],0));console.log((true?" left ".repeat(1):" right ".repeat(1)).trim());
        }
    "#,
        "1\n2\n5\n5\n1\n3\none\n2\nkept\na\n6\n5\n5\n7\n3\n5\n5\n2\nalias\n2\nleft\n",
        r#"
        for flag in [false,true]{let row=function0(flag);assert_eq!(row.field0,if flag{1}else{2});assert_eq!(row.field1.as_ref(),if flag{"left"}else{"right"});assert_eq!(row.field2.as_slice(),if flag{&[7,8][..]}else{&[][..]});
        let text=function1(flag,String::Owned("left\0😀".to_owned()),String::Owned("righté".to_owned()));assert_eq!(text.as_ref(),if flag{"left\0😀"}else{"righté"});assert_eq!(function2(flag).as_slice(),if flag{&[1,2][..]}else{&[3,4,5][..]});
        let numbers=function3(flag);assert_eq!(numbers.len(),2);if flag{assert_eq!(numbers[0].to_bits(),(-0.0_f64).to_bits());assert_eq!(numbers[1],1.0);}else{assert!(numbers[0].is_nan());assert_eq!(numbers[1],f64::INFINITY);}let bools=function4(flag);assert_eq!(bools.as_slice(),if flag{&[true][..]}else{&[false,true][..]});let strings=function5(flag);assert_eq!(strings.get(0).unwrap().as_ref(),if flag{"é"}else{"e\u{301}"});assert_eq!(strings.get(1).unwrap().as_ref(),if flag{"😀"}else{"\0"});let nested=function6(flag);assert_eq!(nested.len(),2);assert_eq!(nested.get(0).unwrap().as_slice(),if flag{&[1][..]}else{&[][..]});}
        let values=NumericArray::from([8,9,10]);assert_eq!(function9(&values,true),1);assert_eq!(function9(&values,false),3);
    "#,
        "",
    );
    assert_eq!(
        analysis.contracts().get("string.left"),
        Some(&Mode::Consume)
    );
    assert_eq!(
        analysis.contracts().get("string.right"),
        Some(&Mode::Consume)
    );
    // Existing closures conservatively disable fixed storage for their containing function.
    // Prove unrelated fixed locals survive conditional ABI fallback in a closure-free body.
    let fixed = native(
        "function fixed(flag:boolean):int{const a=[1,2];const b=[3,4,5];const chosen=flag?a:b;const unrelated=[1,2,3];return chosen.length+unrelated[1];}function main(){}",
        "",
        "for flag in [false,true]{assert_eq!(std::hint::black_box(function0)(std::hint::black_box(flag)),if flag{4}else{5});}",
        "",
    );
    assert!(fixed.emit_rust().contains(": [i64; 3] = ["));
    // Direct selected calls reuse ordinary callback-origin substitution and recursive factories.
    native(
        r#"
        function one():int{return 7;}
        function make(seed:int,n:int):(x:int)=>()=>int{if(n>0){return make(seed,n-1);}return (x:int):()=>int=>{return ():int=>{return seed+x;};};}
        function main(){const forward=(c:Function<int>):()=>int=>{return c;};const result=(true?forward:forward)(one);console.log(result());const factory=make(7,3);const action=(false?factory:factory)(1);console.log(action());}
        "#,
        "7\n8\n",
        "",
        "",
    );
    let predicate_effects = native(
        r#"
        function one():int{return 7;}
        function make():(c:Function<int>)=>()=>int{return (c:Function<int>):()=>int=>{return c;};}
        function f(value:int):int{return value+7;}
        function grow(values:int[],value:int):boolean{values.push(value);return true;}
        function main(){const forward=make();const values=[1,2,3,4];const unrelated=[1,2];const result=(grow(values,5)?forward:forward)(one);console.log(values.length);console.log(result());console.log(unrelated[1]);console.log((grow(values,9)?f:f)(0));console.log(values.length);}
        "#,
        "5\n7\n2\n7\n6\n",
        "",
        "",
    );
    assert!(predicate_effects.emit_rust().contains(": [i64; 2] = ["));
    // Fresh borrowed branches live through each consumer and complete for-of loop.
    native(
        r#"
        function read(values:int[]):int{return values.length;}
        function main(){let total=0;for(const n of (true?[1,2]:[3])){total+=n;}console.log(total);let count=0;while((count++<2)&&read(true?[1,2]:[3])>0){console.log((false?" wrong ".repeat(1):" right ".repeat(1)).trim());}let text="x";text+=(true?"y".repeat(2):"z".repeat(1));console.log(text);}
        "#,
        "3\nright\nright\nxyy\n",
        "",
        "",
    );
    native(
        r#"
        function conditions(flag:boolean,probe:Mutable<Function<boolean>>):int{let i=0;while(probe()&&(flag?"abcdefgh".repeat(1):"abcdefgh".repeat(2)).length>0&&i<3){i++;}return i;}
        function iteration(flag:boolean,probe:Mutable<Function<boolean>>):int{let n=0;for(const text of (flag?["abcd".repeat(2),"efgh".repeat(2)]:["abcdefgh".repeat(1)])){n+=text.length;}probe();return n;}
        function initializer(flag:boolean,probe:Mutable<Function<boolean>>):int{const n=(flag?"abcdefgh".repeat(1):"abcdefgh".repeat(2)).length;probe();return n;}
        function trap(flag:boolean):int{return (flag?"abcdefgh".repeat(1):"abcdefgh".repeat(2)).length+1/0;}
        function main(){}
        "#,
        "",
        r#"
        struct Probe{baseline:isize,calls:usize}impl Callback<(),bool> for Probe{fn invoke(&mut self,_:())->bool{assert_eq!(LIVE.load(Ordering::SeqCst),self.baseline);self.calls+=1;true}}
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for flag in [false,true]{let mut probe=Probe{baseline,calls:0};assert_eq!(function0(flag,&mut probe),3);assert_eq!(probe.calls,4);assert_eq!(function1(flag,&mut probe),if flag{16}else{8});assert_eq!(function2(flag,&mut probe),if flag{8}else{16});assert_eq!(LIVE.load(Ordering::SeqCst),baseline);assert!(std::panic::catch_unwind(||function3(flag)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);}std::panic::set_hook(hook);
        "#,
        ALLOCATOR,
    );
    // Alternative moves start from the same availability; two moves of one owner are not sequential.
    assert!(check("function main(){const text=\"owned\";const selected=true?text:text;console.log(selected);}").is_ok());
    assert!(check("type Box={text:string;};function main(){const box:Box={text:\"owned\"};const value=true?box.text:box.text;box.text=\"restored\";console.log(box.text);}").is_ok());
}

#[test]
fn strict_branch_types_ownership_diagnostics_and_normal_stack_limits_are_preserved() {
    for (source, code) in [
        ("function main(){console.log(1?2:3);}", "type"),
        ("function main(){let n=1;console.log(true?n:1.0);}", "type"),
        ("function main(){console.log(true?1:false);}", "type"),
        ("function main(){console.log(true?\"text\":1);}", "type"),
        (
            "type A={n:int;};type B={n:int;};function main(){const a:A={n:1};const b:B={n:2};const x=true?a:b;}",
            "type",
        ),
        (
            "function main(){const text=\"owned\";const result=true?text:\"other\";console.log(text);}",
            "transferred",
        ),
        (
            "type Box={text:string;n:int;};function main(){const box:Box={text:\"owned\",n:1};const text=true?box.text:\"other\";console.log(box.text);}",
            "partial-transfer",
        ),
        (
            "function read(v:int[],n:int):int{return v.length+n;}function grow(v:int[]):int{v.push(7);return 0;}function test(flag:boolean):int{const a=[1];const b=[2];return read(flag?a:b,grow(b));}function main(){}",
            "conflicting-access",
        ),
        (
            "function read(v:int[],n:int):int{return v.length+n;}function grow(v:int[]):int{v.push(7);return 0;}function test(flag:boolean,other:boolean):int{const a=[1];const b=[2];const invoke=read;return invoke(flag?(other?a:b):b,grow(a));}function main(){}",
            "conflicting-access",
        ),
        (
            "function take(s:string):string{s;return \"x\";}function test(flag:boolean):boolean{const a=\"a\";const b=\"b\";return (flag?a:b)===take(b);}function main(){}",
            "conflicting-access",
        ),
        (
            "function take(s:string):string{s;return \"x\";}function test(flag:boolean):string{const a=\"a\";const b=\"b\";return (flag?a:b)+take(a);}function main(){}",
            "conflicting-access",
        ),
        (
            "function bad(f:Mutable<Function<int>>,flag:boolean):()=>int{return flag?f:f;}function main(){}",
            "callback-escape",
        ),
        (
            "function main(){let x=1;let y=2;(true?x:y)=3;}",
            "unsupported",
        ),
        (
            "function main(){let x=1;let y=2;(true?x:y)++;}",
            "unsupported",
        ),
        (
            "function main(){let x=1;console.log((true?x:x)??=2);}",
            "unsupported",
        ),
        (
            "function main(){let x=1;console.log(x?.length);}",
            "type",
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
    // Restore on one arm cannot make a moved field available on every path.
    for source in [
        "type Box={text:string;n:int;};function restore(b:Box):int{b.text=\"new\";return 1;}function main(){const b:Box={text:\"old\",n:0};const moved=b.text;const n=true?restore(b):0;console.log(b.text);}",
        "function main(){const a=[1];const b=[2];console.log((true?a:b)[0]);console.log(b.length);}",
    ] {
        assert!(check(source).is_err());
    }
    for source in [
        "function main(){console.log(true?1);}",
        "function main(){console.log(true?:2);}",
        "function main(){console.log(true?1:);}",
    ] {
        assert_eq!(check(source).err().unwrap().code, "syntax");
    }
    for depth in [100, 200] {
        for expression in [
            format!("{}1{}", "true?".repeat(depth), ":0".repeat(depth)),
            format!("{}1", "false?0:".repeat(depth)),
        ] {
            let source = format!("function main(){{console.log({expression});}}");
            if depth == 100 {
                check(&source).unwrap().emit_rust();
            } else {
                assert_eq!(check(&source).err().unwrap().code, "unsupported");
            }
        }
    }
    for depth in [100, 200] {
        for source in [
            format!(
                "function main(){{const a=[0];console.log({}0{});}}",
                "a[".repeat(depth),
                "]".repeat(depth)
            ),
            format!(
                "function main(){{const a={}0{};}}",
                "[".repeat(depth),
                "]".repeat(depth)
            ),
        ] {
            if depth == 100 {
                check(&source).unwrap().emit_rust();
            } else {
                assert_eq!(check(&source).err().unwrap().code, "unsupported");
            }
        }
    }
}

#[test]
fn conditional_selection_allocates_nothing_and_owned_results_dispose_on_success_and_unwind() {
    native(
        r#"
        type Box={n:int;number:number;text:string;values:int[];};
        function integer(flag:boolean,left:int,right:int):int{return flag?left:right;}
        function number(flag:boolean,left:number,right:number):number{return flag?left:right;}
        function owned(flag:boolean,left:Box,right:Box):Box{return flag?left:right;}
        function fresh(flag:boolean):Box{return flag?{n:1,number:-0,text:"owned😀".repeat(2),values:[1,2,3,4,5]}:{n:2,number:Number.NaN,text:"é\u0000".repeat(3),values:[]};}
        function fail(flag:boolean,left:Box,right:Box):Box{return flag?left:(1/0===0?right:right);}
        function main(){}
    "#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for flag in [false,true]{for value in [i64::MIN,-7,0,i64::MAX]{let count=ALLOCATIONS.load(Ordering::SeqCst);let live=LIVE.load(Ordering::SeqCst);let result=std::hint::black_box(function0)(std::hint::black_box(flag),std::hint::black_box(value),std::hint::black_box(23));assert_eq!(result,if flag{value}else{23});std::hint::black_box(result);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),live);}for bits in [0_u64,0x8000000000000000,1,0x7ff0000000000000,0x7ff8000000000042]{let count=ALLOCATIONS.load(Ordering::SeqCst);let result=std::hint::black_box(function1)(std::hint::black_box(flag),std::hint::black_box(f64::from_bits(bits)),std::hint::black_box(-0.0));assert_eq!(result.to_bits(),if flag{bits}else{(-0.0_f64).to_bits()});assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);}}
        fn owner(n:i64,text:&str)->Record0{Record0{field0:n,field1:-0.0,field2:String::Owned(text.to_owned()),field3:NumericArray::from([1,2,3,4,5])}}
        for flag in [false,true]{for text in ["","owned\0😀e\u{301}"]{
            let left=owner(1,text);let right=owner(2,"otheré");let count=ALLOCATIONS.load(Ordering::SeqCst);let selected=std::hint::black_box(function2)(flag,left,right);assert_eq!(selected.field0,if flag{1}else{2});assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);drop(selected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            let selected=function3(flag);assert_eq!(selected.field0,if flag{1}else{2});assert_eq!(selected.field2.as_ref(),if flag{"owned😀owned😀"}else{"é\0é\0é\0"});drop(selected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            assert!(std::panic::catch_unwind(||{let selected=function2(flag,owner(1,text),owner(2,"other"));std::hint::black_box(selected);panic!("dispose selected result");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            if flag {let selected=function4(true,owner(1,text),owner(2,"other"));drop(selected);}else{assert!(std::panic::catch_unwind(||function4(false,owner(1,text),owner(2,"other"))).is_err());}assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }}std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
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

#[test]
fn grouped_zero_parameter_arrows_in_conditional_arms() {
    support::execute("type Handler={run:Function<int>;};function main(){const chosen:Handler={run:true?(():int=>7):(():int=>9)};console.log(chosen.run());const other:Handler={run:false?(():int=>11):(():int=>13)};console.log(other.run());}", "7\n13\n", None);
}

#[test]
fn unannotated_conditional_arrows_infer_common_callable_carrier() {
    support::execute("function main(){const chosen=true?(():int=>7):(():int=>9);console.log(chosen());}", "7\n", None);
}

#[test]
fn unannotated_owned_conditional_carriers_keep_parameters_capture_values_and_failures() {
    support::execute("function main(){const offset=3;const chosen=false?((value:int):int=>value+offset):((value:int):int=>value*offset);console.log(chosen(4));const safe=true?((value:int):int=>value+1):((value:int):int=>value/0);console.log(safe(4));const failed=false?((value:int):int=>value+1):((value:int):int=>value/0);let caught=false;try{failed(4);}catch(error){caught=true;}console.log(caught);}","12\n5\ntrue\n",None);
}

#[test]
fn unannotated_owned_conditional_carriers_preserve_once_consumption() {
    support::execute("function first(text:string){return ():string=>text;}function second(text:string){return ():string=>text;}function main(){const chosen=false?first('one'):second('two');console.log(chosen());}","two\n",None);
    let source=tsrust_compiler::format("function first(text:string){return ():string=>text;}function second(text:string){return ():string=>text;}function main(){const chosen=false?first('one'):second('two');console.log(chosen());console.log(chosen());}").unwrap();
    let error=tsrust_compiler::check(&source).err().expect("consuming carrier cannot be invoked twice");assert_eq!(error.code,"closure-consumed","{}",error.render("carrier.tsr",&source));
}

#[test]
fn unannotated_owned_conditional_carriers_reject_incompatible_signatures() {
    for source in ["function main(){const chosen=true?((value:int):int=>value):(():int=>7);}","function main(){const chosen=true?(():int=>7):(():string=>'wrong');}"] {
        let source=tsrust_compiler::format(source).unwrap();assert!(tsrust_compiler::check(&source).is_err(),"different contracts cannot share a carrier");
    }
}

#[test]
fn unannotated_owned_conditional_aliases_move_selected_origin() {
    support::execute("function main(){const f=():int=>1;const g=():int=>2;const chosen=true?f:g;console.log(chosen());}","1\n",None);
}

#[test]
fn common_owned_conditional_origins_dispose_original_owners_once() {
    let source=support::format_round_trip("type Item={id:int;};function first(item:Item){return ():int=>item.id;}function second(item:Item){return ():int=>item.id+10;}function failed(item:Item){return ():int=>{throw item.id;};}function main(){{const chosen=true?first({id:1}):second({id:2});if(chosen()!==1||chosen()!==1){throw 99;}}{const chosen=false?first({id:3}):second({id:4});if(chosen()!==14){throw 99;}}{const abandoned=false?first({id:5}):second({id:6});}{const chosen=true?failed({id:7}):first({id:8});let caught=false;try{chosen();}catch(error){caught=error===7;}if(!caught){throw 99;}}}");
    let rust=tsrust_compiler::check(&source).unwrap().emit_rust().replace("fn main()","fn source_main()")+r#"
static DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
static SEEN:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);
impl Drop for Record0 {fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);SEEN.fetch_or(1<<self.field0,std::sync::atomic::Ordering::SeqCst);}}
fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),4);assert_eq!(SEEN.load(std::sync::atomic::Ordering::SeqCst),210);}
"#;
    let directory=support::TempDirectory::new();let path=directory.0.join("conditional-owned-origins.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {let binary=directory.0.join(format!("conditional-owned-origins-{level}"));support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&binary),60));support::success(&support::run(&mut std::process::Command::new(binary),10));}
}

#[test]
fn borrowed_conditional_carriers_preserve_selected_local_capture_mutation() {
    support::execute(r#"
        function readSelected(flag:boolean):int {
            const left=[3];const right=[7,9];
            const action=flag?(()=>left.length):(()=>right.length);
            return action();
        }
        function mutateSelected(flag:boolean):int {
            let left=0;let right=10;let calls=0;
            {
                const first=():int=>{left+=1;return left;};
                const second=():int=>{right+=2;return right;};
                const selected=flag?first:second;
                calls=selected()+selected();
            }
            return calls+left+right;
        }
        function main(){
            console.log(readSelected(true),readSelected(false));
            console.log(mutateSelected(true),mutateSelected(false));
        }
    "#, "1 2\n15 40\n", None);
}

#[test]
fn borrowed_conditional_carriers_preserve_defaults_and_callback_failures() {
    support::execute(r#"
        function defaults(flag:boolean):int {
            const left=[1];const right=[2];
            const first=(value:int=3):int=>left[0]+value;
            const second=(value:int=7):int=>right[0]+value;
            const selected=flag?first:second;
            return selected()+selected(5);
        }
        function failures(flag:boolean,fail:boolean):int {
            const values=[4];
            const selected=flag?((value:int):int=>values[0]+value):((value:int):int=>values[0]/value);
            try{return selected(fail?0:2);}catch(error){return -1;}
        }
        function main(){console.log(defaults(true),defaults(false));console.log(failures(true,false),failures(false,false),failures(false,true));}
    "#, "10 16\n6 2 -1\n", None);
}

#[test]
fn borrowed_conditional_carriers_reject_conflicts_and_scoped_escape() {
    for (source,code) in [
        ("function main(){let left=0;let right=10;const first=():int=>{left+=1;return left;};const second=():int=>{right+=1;return right;};const selected=true?first:second;left=3;console.log(selected());}","capture-conflict"),
        ("function factory(flag:boolean){const left=[1];const right=[2];const first=():int=>left.length;const second=():int=>right.length;const selected=flag?first:second;return selected;}function main(){}","callback-escape"),
        ("function take(action:Function<int>):int{return action();}function main(){const left=[1];const right=[2];const selected=false?(()=>left.length):(()=>right.length);console.log(take(selected));}","callback-contract"),
        ("function main(){const left=[1];const right=[2];const selected=false?(()=>left.length):(()=>right.length);try{throw selected;}catch{}}","closure-escape"),
    ] {support::reject(source,code);}
}

#[test]
fn borrowed_conditional_carriers_pass_scoped_callback_contracts() {
    support::execute(r#"
        function read(action:ReadOnly<Function<int>>):int{return action();}
        function change(action:Mutable<Function<int>>):int{return action();}
        function main(){
            const left=[1];const right=[2,3];
            const selected=false?(()=>left.length):(()=>right.length);
            console.log(read(selected));
            let first=0;let second=10;
            {const selected=false?(()=>{first+=1;return first;}):(()=>{second+=2;return second;});console.log(change(selected),change(selected));}
            console.log(first,second);
        }
    "#, "2\n12 14\n0 14\n", None);
}

#[test]
fn borrowed_conditional_direct_invocation_joins_both_failure_contracts() {
    support::execute(r#"
        function choose(flag:boolean,value:int):int {
            const values=[4];
            try{return (flag?((input:int):int=>values[0]+input):((input:int):int=>values[0]/input))(value);}catch(error){return -1;}
        }
        function main(){console.log(choose(true,2),choose(false,2),choose(false,0));}
    "#, "6 2 -1\n", None);
}

#[test]
fn borrowed_conditional_source_owners_dispose_once_after_scoped_handles() {
    let source=r#"type Owner={id:int;};function main(){const first:Owner={id:1};const second:Owner={id:2};{const selected=false?(()=>first.id):(()=>second.id);console.log(selected(),selected());}console.log(first.id,second.id);}"#;
    let formatted=support::format_round_trip(source);
    let checked=check(&formatted).unwrap();
    let rust=checked.emit_rust().replace("fn main()","fn source_main()")+
        "\nstatic DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);impl Drop for Record0{fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),2);}";
    let directory=support::TempDirectory::new();let path=directory.0.join("borrowed.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let executable=directory.0.join(format!("borrowed-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&executable),60));
        let output=support::run(&mut std::process::Command::new(&executable),10);support::success(&output);assert_eq!(output.stdout,b"2 2\n1 2\n");
    }
}

#[test]
fn borrowed_conditional_callback_arguments_preserve_failures_through_parameters() {
    support::execute(r#"
        function apply(action:ReadOnly<Function<int,[int]>>,value:int):int{return action(value);}
        function choose(flag:boolean,value:int):int{
            const values=[4];
            const selected=flag?((input:int):int=>values[0]+input):((input:int):int=>values[0]/input);
            try{return apply(selected,value);}catch(error){return -1;}
        }
        function main(){console.log(choose(true,2),choose(false,2),choose(false,0));}
    "#, "6 2 -1\n", None);
}

#[test]
fn borrowed_conditional_carriers_join_read_mutate_and_owning_alternatives() {
    support::execute(r#"
        function ownership(flag:boolean):int{
            const values=[3];const selected=flag?(()=>values.length):(()=>7);return selected();
        }
        function access(flag:boolean):int{
            const values=[1,2];let changed=0;let calls=0;
            {const first=():int=>values.length;const second=():int=>{changed+=1;return changed;};const selected=flag?first:second;calls=selected()+selected();}
            return calls+changed;
        }
        function main(){console.log(ownership(true),ownership(false),access(true),access(false));}
    "#, "1 7 4 5\n", None);
}

#[test]
fn consuming_borrowed_conditional_carriers_move_owned_captures_once() {
    support::execute(r#"
        function choose(flag:boolean):string {
            const values=[4];const left="left".repeat(1);const right="right".repeat(1);
            const selected=flag?(():string=>{console.log(values.length);return left;}):(():string=>{console.log(values.length);return right;});
            return selected();
        }
        function main(){console.log(choose(true));console.log(choose(false));}
    "#, "1\nleft\n1\nright\n", None);
}

#[test]
fn consuming_borrowed_conditional_carriers_reject_reuse_and_owning_escape() {
    let setup="const values=[4];const text='left'.repeat(1);const selected=true?(():string=>{console.log(values.length);return text;}):(():string=>String(values[0]));";
    support::reject(&format!("function main(){{{setup}selected();selected();}}"),"closure-consumed");
    support::reject(&format!("function take(action:Function<string>):string{{return action();}}function main(){{{setup}take(selected);}}"),"callback-contract");
    support::reject(&format!("function factory(){{{setup}return selected;}}function main(){{}}"),"callback-escape");
}

#[test]
fn consuming_borrowed_conditional_carriers_preserve_defaults_and_failures() {
    support::execute(r#"
        function choose(flag:boolean,input:int):string{
            const values=[4];const text="left".repeat(1);
            const selected=flag?((value:int):string=>{console.log(values[0]/value);return text;}):((value:int):string=>String(values[0]+value));
            try{return selected(input);}catch(error){return "failed";}
        }
        function defaults(flag:boolean):string{
            const values=[4];const text="left".repeat(1);
            const selected=flag?((value:int=3):string=>{console.log(values[0]+value);return text;}):((value:int=7):string=>String(values[0]+value));
            return selected();
        }
        function main(){console.log(choose(true,2),choose(false,2),choose(true,0));console.log(defaults(true),defaults(false));}
    "#, "2\nleft 6 failed\n7\nleft 11\n", None);
}

#[test]
fn once_parameters_accept_scoped_consuming_environments_and_join_failures() {
    support::execute(r#"
        function apply(action:Once<Function<string,[int]>>,input:int):string{return action(input);}
        function choose(flag:boolean,input:int):string{
            const values=[4];const text="left".repeat(1);
            const selected=flag?((value:int):string=>{console.log(values[0]/value);return text;}):((value:int):string=>String(values[0]+value));
            try{return apply(selected,input);}catch(error){return "failed";}
        }
        function main(){console.log(choose(true,2),choose(false,2),choose(true,0));}
    "#, "2\nleft 6 failed\n", None);
}

#[test]
fn consuming_borrowed_conditional_carriers_dispose_invoked_and_abandoned_owners_once() {
    let source=r#"
        type Owner={id:int;};
        function choose(flag:boolean,invoke:boolean){
            const values=[4];const left:Owner={id:1};const right:Owner={id:2};
            const first=():Owner=>{console.log(values.length);return left;};
            const second=():Owner=>{console.log(values.length);return right;};
            const selected=flag?first:second;
            if(invoke){const result=selected();console.log(result.id);}
        }
        function main(){choose(true,true);choose(false,true);choose(true,false);choose(false,false);}
    "#;
    let formatted=support::format_round_trip(source);
    let rust=check(&formatted).unwrap().emit_rust().replace("fn main()","fn source_main()")+
        "\nstatic DROPS:std::sync::atomic::AtomicUsize=std::sync::atomic::AtomicUsize::new(0);impl Drop for Record0{fn drop(&mut self){DROPS.fetch_add(1,std::sync::atomic::Ordering::SeqCst);}}fn main(){source_main();assert_eq!(DROPS.load(std::sync::atomic::Ordering::SeqCst),8);}";
    let directory=support::TempDirectory::new();let path=directory.0.join("consuming-borrowed.rs");std::fs::write(&path,rust).unwrap();
    for level in ["0","3"] {
        let executable=directory.0.join(format!("consuming-borrowed-{level}"));
        support::success(&support::run(support::rustc().args(["--edition=2024","-C",&format!("opt-level={level}")]).arg(&path).arg("-o").arg(&executable),60));
        let output=support::run(&mut std::process::Command::new(&executable),10);support::success(&output);assert_eq!(output.stdout,b"1\n1\n1\n2\n");
    }
}
