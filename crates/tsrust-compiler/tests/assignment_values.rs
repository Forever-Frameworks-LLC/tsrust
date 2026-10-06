mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("assignment.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("assignment.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("assignment-{level}"));
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
fn scalar_types_chains_contexts_and_number_bits_return_exact_committed_values() {
    let analysis = native(
        r#"
        function integer(value:int):int{let target=0;return target=value;}
        function number(value:number):number{let target:number=0;return target=value;}
        function boolean(value:boolean):boolean{let target=false;return target=value;}
        function contextual():number{let target:number=0;return target=-0;}
        function literal():number{let target:number=0;return target=7;}
        type Box={n:int;flag:boolean;number:number;text:string;};
        function numberChain(box:Box,value:number):number{let target:number=1;return target=box.number=value;}
        function pair(left:int,right:int){console.log(left);console.log(right);}
        function main(){
            let a=0;let b=0;console.log(a=b=7);console.log(a);console.log(b);
            const values=[a=2,a=4];console.log(values[0]);console.log(values[1]);console.log(a);
            let flag=false;let n:number=0;const box:Box={n:a=6,flag:flag=true,number:n=-0.0,text:"owned"};console.log(box.n);console.log(box.flag);
            pair(a=8,a=9);console.log(a);console.log((a=2)<<1);
            const input=[1,2];const result=input.map((value:int):int=>{return a=value+10;});console.log(result[0]);console.log(result[1]);console.log(a);
            {const set=(value:int):int=>{return a=value;};console.log(set(3));}console.log(a);
            flag=false;console.log(flag&&=(flag=true));console.log(flag);console.log(flag||=(flag=false));console.log(flag);
            let other=false;console.log(flag=other||=true);console.log(flag);console.log(other);
            flag=false;console.log(false&&(flag=true));console.log(true||(flag=false));console.log(flag);
            a=0;while(flag=a<2){a++;}console.log(a);console.log(flag);
            let count=0;for(a=0;a<2;a=a+1){count++;continue;}console.log(count);console.log(a);
            do{a=a-1;}while(flag=a>0);console.log(a);console.log(flag);
            const numbers=[n=1,n=-0.0];console.log(numbers.length);
        }
    "#,
        "7\n7\n7\n2\n4\n4\n6\ntrue\n8\n9\n9\n4\n11\n12\n12\n3\n3\nfalse\nfalse\nfalse\nfalse\ntrue\ntrue\ntrue\nfalse\ntrue\nfalse\n2\nfalse\n2\n2\n0\nfalse\n2\n",
        r#"
        for value in [i64::MIN,-7,0,9,i64::MAX]{assert_eq!(function0(std::hint::black_box(value)),value);}
        for value in [false,true]{assert_eq!(function2(std::hint::black_box(value)),value);}
        let bits=[0_u64,0x8000000000000000,1,0x0010000000000000,0x3ff0000000000000,0x7fefffffffffffff,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000042,0xfff8000000000001];
        for bits in bits {let value=f64::from_bits(bits);assert_eq!(function1(std::hint::black_box(value)).to_bits(),bits);let mut owner=Record0{field0:0,field1:false,field2:1.0,field3:String::Borrowed("owner")};let result=function5(&mut owner,std::hint::black_box(value));assert_eq!(result.to_bits(),bits);assert_eq!(owner.field2.to_bits(),bits);}
        assert_eq!(function3().to_bits(),(-0.0_f64).to_bits());assert_eq!(function4().to_bits(),7.0_f64.to_bits());
    "#,
        "",
    );
    assert_eq!(
        analysis.contracts().get("numberChain.box"),
        Some(&Mode::Mutate)
    );
}

#[test]
fn selection_growth_unconditional_effects_and_captures_preserve_storage_rules() {
    let analysis = native(
        r#"
        function quiet(values:int[],answer:int):int{values[0]=100;values.push(42);return answer;}
        function growth(values:int[],answer:int):int{return values[0]=quiet(values,answer);}
        function reversed(values:int[]):int{values.reverse();return 7;}
        function reverseApply(values:int[]):int{return values[0]=reversed(values);}
        function removed(values:int[]):int{values.remove(0);return 11;}
        function removeApply(values:int[]):int{return values[0]=removed(values);}
        function lostApply(values:int[]):int{return values[1]=removed(values);}
        function lostStatement(values:int[]):int{values[1]=removed(values);return values.length;}
        function chain(values:int[],counts:int[]):int{return values[counts[0]++]=values[counts[1]++]=quiet(values,7);}
        function counter():(value:int)=>int{let target=0;return (value:int):int=>{return target=value;};}
        type Box={n:int;text:string;};
        function update(box:Box,value:int):int{return box.n=value;}
        function take(text:string):int{text;return 13;}
        function mark():int{console.log("index");return 0;}
        function rhs():int{console.log("rhs");return 5;}
        function main(){
            const values=[1];console.log(values[mark()]=rhs());console.log(values[0]);values[mark()]=rhs();console.log(values[0]);
            const rows=[[1]];let i=0;let j=0;console.log(rows[i++][j++]=8);console.log(i);console.log(j);console.log(rows[0][0]);
            const box:Box={n:1,text:"owner😀"};console.log(update(box,4));const text=box.text;console.log(box.n=6);box.text=text;console.log(box.text);
            const replacement=clone(box.text);console.log(box.n=take(box.text));box.text=replacement;console.log(box.text);
            let target=1;{const set=(value:int):int=>{return target=value;};console.log(set(2));}console.log(target);
            let snapshot=3;const read=():int=>{return snapshot;};console.log(snapshot=4);console.log(read());console.log(snapshot);
            const retained=counter();console.log(retained(7));console.log(retained(9));
            const flags=[true,false];for(let flag of flags){console.log(flag=false);}console.log(flags[0]);
        }
    "#,
        "index\nrhs\n5\n5\nindex\nrhs\n5\n8\n1\n1\n8\n4\n6\nowner😀\n13\nowner😀\n2\n2\n4\n3\n4\n7\n9\nfalse\nfalse\ntrue\n",
        r#"
        for answer in [i64::MIN,0,23,i64::MAX]{let mut values=NumericArray::from([1,2,3,4]);assert_eq!(function1(&mut values,answer),answer);assert_eq!(values.as_slice(),&[answer,2,3,4,42]);}
        let mut values=NumericArray::from([1,2]);assert_eq!(function3(&mut values),7);assert_eq!(values.as_slice(),&[7,1]);
        let mut values=NumericArray::from([1,2]);assert_eq!(function5(&mut values),11);assert_eq!(values.as_slice(),&[11]);
        let mut values=NumericArray::from([1,2,3,4]);let mut counts=NumericArray::from([0,1]);assert_eq!(function8(&mut values,&mut counts),7);assert_eq!(counts.as_slice(),&[1,2]);assert_eq!(values.as_slice(),&[7,7,3,4,42]);
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));for operation in [function6 as fn(&mut NumericArray<i64>)->i64,function7]{let mut values=NumericArray::from([1,2]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||operation(&mut values))).is_err());assert_eq!(values.as_slice(),&[2]);}std::panic::set_hook(hook);
    "#,
        "",
    );
    assert_eq!(analysis.contracts().get("update.box"), Some(&Mode::Mutate));
    assert_eq!(
        analysis.contracts().get("quiet.values"),
        Some(&Mode::Mutate)
    );
    assert_eq!(analysis.contracts().get("take.text"), Some(&Mode::Consume));
    native(
        r#"
        type Row={n:int;number:number;flag:boolean;text:string;};
        function grow(rows:Row[],answer:int):int{rows.push({n:77,number:1.0,flag:false,text:"extra"});return answer;}
        function statement(rows:Row[],answer:int):int{rows[0].n=grow(rows,answer);return rows[0].n;}
        function expression(rows:Row[],answer:int):int{return rows[0].n=grow(rows,answer);}
        function chain(rows:Row[],counts:int[]):int{return rows[counts[0]++].n=rows[counts[1]++].n=grow(rows,13);}
        function growNumber(rows:Row[],answer:number):number{rows.push({n:77,number:1.0,flag:false,text:"extra"});return answer;}
        function number(rows:Row[],answer:number):number{return rows[0].number=growNumber(rows,answer);}
        function growBoolean(rows:Row[],answer:boolean):boolean{rows.push({n:77,number:1.0,flag:false,text:"extra"});return answer;}
        function boolean(rows:Row[],answer:boolean):boolean{return rows[0].flag=growBoolean(rows,answer);}
        function nested(rows:Row[][],other:Row[]):int{return rows[0][0].n=grow(other,23);}
        function main(){}
    "#,
        "",
        r#"
        fn row(n:i64)->Record0{Record0{field0:n,field1:0.0,field2:false,field3:String::Owned("owned\0😀e\u{301}".to_owned())}}
        for operation in [function1 as fn(&mut Array<Record0>,i64)->i64,function2]{for answer in [i64::MIN,0,11,i64::MAX]{let mut rows=Array::from([row(1),row(2),row(3),row(4)]);assert_eq!(operation(&mut rows,answer),answer);assert_eq!(rows.len(),5);assert_eq!(rows.get(0).unwrap().field0,answer);assert_eq!(rows.get(1).unwrap().field0,2);assert_eq!(rows.get(4).unwrap().field0,77);assert_eq!(rows.get(0).unwrap().field3.as_ref(),"owned\0😀e\u{301}");}}
        let mut rows=Array::from([row(1),row(2),row(3),row(4)]);let mut counts=NumericArray::from([0,1]);assert_eq!(function3(&mut rows,&mut counts),13);assert_eq!(counts.as_slice(),&[1,2]);assert_eq!(rows.len(),5);assert_eq!(rows.get(0).unwrap().field0,13);assert_eq!(rows.get(1).unwrap().field0,13);
        for bits in [0_u64,0x8000000000000000,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000042]{let mut rows=Array::from([row(1),row(2),row(3),row(4)]);let result=function5(&mut rows,f64::from_bits(bits));assert_eq!(result.to_bits(),bits);assert_eq!(rows.get(0).unwrap().field1.to_bits(),bits);assert_eq!(rows.len(),5);}
        for answer in [false,true]{let mut rows=Array::from([row(1),row(2),row(3),row(4)]);assert_eq!(function7(&mut rows,answer),answer);assert_eq!(rows.get(0).unwrap().field2,answer);assert_eq!(rows.len(),5);}
        let mut rows=vec![Array::from([row(1)])];let mut other=Array::from([row(2),row(3),row(4),row(5)]);assert_eq!(function8(&mut rows,&mut other),23);assert_eq!(rows.get(0).unwrap().get(0).unwrap().field0,23);assert_eq!(other.len(),5);
    "#,
        "",
    );
    for source in [
        "type Row={n:int;};function hold(rows:Row[],n:int){console.log(rows.length);}function main(){const rows:Row[]=[{n:1}];hold(rows,rows[0].n=2);}",
        "type Row={n:int;};function grow(rows:Row[][]):int{rows[0].push({n:2});return 7;}function main(){const rows:Row[][]=[[{n:1}]];console.log(rows[0][0].n=grow(rows));}",
        "type Row={n:int;};type Box={rows:Row[];n:int;};function main(){const box:Box={rows:[{n:1}],n:0};console.log(box.rows[0].n=(box.n=2));}",
        "type Row={n:int;};function grow(rows:Row[][]):int{rows.push([{n:2}]);return 0;}function main(){const rows:Row[][]=[[{n:1}]];console.log(rows[0][rows[0][0].n=grow(rows)].n=3);}",
    ] {
        assert_eq!(
            check(source).err().unwrap().code,
            "conflicting-access",
            "{source}"
        );
    }
    assert_eq!(
        check("type Row={n:int;};function take(rows:Row[]):int{rows;return 7;}function main(){const rows:Row[]=[{n:1}];console.log(rows[0].n=take(rows));}").err().unwrap().code,
        "transferred",
    );
    let fixed=check("function main(){const values=[1,2,3];console.log(values[1]=9);const numbers=[1.0,2.0];console.log(numbers[0]=-0.0);}").unwrap().emit_rust();
    assert!(fixed.contains(": [i64; 3] = ["));
    assert!(fixed.contains(": [f64; 2] = ["));
}

#[test]
fn strict_diagnostics_depth_bounds_and_failed_rhs_never_commit_outer_assignment() {
    for (source, code) in [
        (
            "function main(){const n=1;console.log(n=2);}",
            "unsupported",
        ),
        (
            "function f(n:int):int{return n=2;}function main(){}",
            "unsupported",
        ),
        (
            "function main(){const f=(n:number):number=>{return n=2;};}",
            "unsupported",
        ),
        ("function main(){let n=1;console.log(n=1.0);}", "type"),
        (
            "function main(){let n=1.0;const integer=1;console.log(n=integer);}",
            "type",
        ),
        (
            "function main(){let flag=false;console.log(flag=1);}",
            "type",
        ),
        (
            "function main(){let n=0;let other=0.0;console.log(n=other=1);}",
            "type",
        ),
        (
            "function main(){let n=0;let other=0.0;console.log(other=n=1);}",
            "type",
        ),
        (
            "function f():number{let n=0;return n=1;}function main(){}",
            "type",
        ),
        ("function main(){console.log(1=2);}", "unsupported"),
        (
            "function f():int{return 1;}function main(){f()=2;}",
            "unsupported",
        ),
        (
            "function main(){const values=[1];console.log(values.length=1);}",
            "unsupported",
        ),
        (
            "function main(){let text=\"old\";console.log(text=\"new\");}",
            "unsupported",
        ),
        (
            "function main(){let values=[1];console.log(values=[2]);}",
            "unsupported",
        ),
        (
            "type Box={n:int;};function main(){let box:Box={n:1};console.log(box={n:2});}",
            "unsupported",
        ),
        (
            "function read(values:int[],value:int):int{return values[0];}function main(){const values=[1];read(values,values[0]=7);}",
            "conflicting-access",
        ),
        (
            "type Box={n:int;values:int[];};function main(){const box:Box={n:1,values:[1]};console.log(box.values[0]=(box.n=2));}",
            "conflicting-access",
        ),
        (
            "function grow(rows:int[][]):int{rows.push([2]);return 3;}function main(){const rows=[[1]];console.log(rows[0][0]=grow(rows));}",
            "conflicting-access",
        ),
        (
            "type Box={n:int;};function main(){const box:Box={n:1};const read=():int=>{return box.n;};console.log(box.n=2);}",
            "capture-conflict",
        ),
        (
            "type Box={n:int;};function take(box:Box):int{box;return 2;}function main(){const box:Box={n:1};console.log(box.n=take(box));}",
            "transferred",
        ),
        (
            "type Box={n:int;text:string;};function take(text:string):int{text;return 2;}function main(){const box:Box={n:1,text:\"owned\"};console.log(box.n=take(box.text));console.log(box.text);}",
            "partial-transfer",
        ),
        (
            "function main(){const values=[1];for(const value of values){console.log(values[0]=2);}}",
            "iteration-conflict",
        ),
        (
            "function main(){const values=[1];for(const value of values){console.log(value=2);}}",
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
    assert!(check("function main(){let text=\"old\";console.log(text+=\"new\");}").is_err());
    for source in [
        format!(
            "function main(){{let n=0;console.log({}1);}}",
            "n=".repeat(200)
        ),
        format!(
            "function main(){{const a=[1];console.log({}0{});}}",
            "a[".repeat(200),
            "]".repeat(200)
        ),
        format!(
            "function main(){{const a={}0{};}}",
            "[".repeat(200),
            "]".repeat(200)
        ),
    ] {
        assert_eq!(check(&source).err().unwrap().code, "unsupported");
    }
    for source in [
        format!(
            "function main(){{const a=[0];console.log({}0{});}}",
            "a[".repeat(100),
            "]".repeat(100)
        ),
        format!(
            "function main(){{const a={}0{};}}",
            "[".repeat(100),
            "]".repeat(100)
        ),
    ] {
        check(&source).unwrap().emit_rust();
    }
    native(
        r#"
        function rhs(values:int[]):int{values[0]=99;values.push(42);const bad=1/0;return bad;}
        function apply(values:int[]):int{return values[0]=rhs(values);}
        function statement(values:int[]){values[0]=rhs(values);}
        function bounds(rows:int[][],counts:int[]):int{return rows[counts[0]++][counts[1]++]=counts[2]=9;}
        function scalarRhs(values:int[]):int{console.log("unexpected-rhs");return 7;}
        function valueBounds(values:int[]):int{return values[-1]=scalarRhs(values);}
        function statementBounds(values:int[]){values[-1]=scalarRhs(values);}
        function ownedRhs(values:string[]):string{console.log("owned-rhs");return "replacement";}
        function ownedBounds(values:string[]){values[-1]=ownedRhs(values);}
        function main(){let text="old";const moved=text;text="new";console.log(text);console.log(moved);const words=["old"];words[0]="new";console.log(words[0]);}
    "#,
        "new\nold\nnew\nowned-rhs\n",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        for statement in [false,true]{let mut values=NumericArray::from([1,2,3,4]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{if statement{function2(&mut values);}else{function1(&mut values);}})).is_err());assert_eq!(values.as_slice(),&[99,2,3,4,42]);}
        for first in [-1,1]{let mut rows=Array::from([NumericArray::from([1])]);let mut counts=NumericArray::from([first,0,0]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut rows,&mut counts))).is_err());assert_eq!(counts.as_slice(),&[first+1,0,0]);}
        let mut rows=Array::from([NumericArray::from([1])]);let mut counts=NumericArray::from([0,1,0]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut rows,&mut counts))).is_err());assert_eq!(counts.as_slice(),&[1,2,0]);
        for statement in [false,true]{let mut values=NumericArray::from([1]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{if statement{function6(&mut values);}else{function5(&mut values);}})).is_err());assert_eq!(values.as_slice(),&[1]);}
        let mut words=Array::from([String::Owned("old".to_owned())]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function8(&mut words))).is_err());assert_eq!(words.get(0).unwrap().as_ref(),"old");std::panic::set_hook(hook);
    "#,
        "",
    );
}

#[test]
fn scalar_assignment_allocates_nothing_and_owned_storage_disposes_normally_and_on_unwind() {
    let analysis = native(
        r#"
        type Box={n:int;number:number;flag:boolean;text:string;};
        function integer(box:Box,value:int):int{return box.n=value;}
        function number(box:Box,value:number):number{return box.number=value;}
        function boolean(box:Box,value:boolean):boolean{return box.flag=value;}
        function failing(box:Box):int{return box.n=1/0;}
        function main(){}
    "#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for text in ["","owned\0😀e\u{301}"]{
            let mut owner=Record0{field0:0,field1:0.0,field2:false,field3:String::Owned(text.to_owned())};
            for value in [i64::MIN,-7,0,23,i64::MAX]{let count=ALLOCATIONS.load(Ordering::SeqCst);let live=LIVE.load(Ordering::SeqCst);let result=std::hint::black_box(function0)(std::hint::black_box(&mut owner),std::hint::black_box(value));assert_eq!(result,value);assert_eq!(owner.field0,value);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),live);}
            for bits in [0_u64,0x8000000000000000,1,0x7ff0000000000000,0xfff0000000000000,0x7ff8000000000042]{let value=f64::from_bits(bits);let count=ALLOCATIONS.load(Ordering::SeqCst);let live=LIVE.load(Ordering::SeqCst);let result=std::hint::black_box(function1)(std::hint::black_box(&mut owner),std::hint::black_box(value));assert_eq!(result.to_bits(),bits);assert_eq!(owner.field1.to_bits(),bits);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),live);}
            for value in [false,true]{let count=ALLOCATIONS.load(Ordering::SeqCst);let result=std::hint::black_box(function2)(std::hint::black_box(&mut owner),std::hint::black_box(value));assert_eq!(result,value);assert_eq!(owner.field2,value);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);}
            assert_eq!(owner.field3.as_ref(),text);drop(owner);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:9,field1:0.0,field2:false,field3:String::Owned(text.to_owned())};function3(&mut owner);}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
            assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:9,field1:0.0,field2:false,field3:String::Owned(text.to_owned())};let result=function0(&mut owner,23);assert_eq!(owner.field0,result);std::hint::black_box(result);panic!("dispose committed owner");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
    );
    for name in ["integer.box", "number.box", "boolean.box"] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Mutate));
    }
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
