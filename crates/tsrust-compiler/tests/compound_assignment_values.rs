mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis = check(source)
        .unwrap_or_else(|error| panic!("{}", error.render("compound-value.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("compound-value.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("compound-value-{level}"));
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

const OPERATORS: [&str; 12] = [
    "+=", "-=", "*=", "/=", "%=", "**=", "&=", "|=", "^=", "<<=", ">>=", ">>>=",
];

fn numeric_source(record: bool) -> String {
    let mut source = if record {
        "type Box={integer:int;number:number;text:string;};".to_owned()
    } else {
        String::new()
    };
    for (index, operator) in OPERATORS.iter().enumerate() {
        if record {
            source += &format!(
                "function integer{index}(box:Box,y:int):int{{return box.integer{operator}y;}}function number{index}(box:Box,y:number):number{{return box.number{operator}y;}}"
            );
        } else {
            source += &format!(
                "function integer{index}(x:int,y:int):int[]{{let target=x;const result=target{operator}y;return[result,target];}}function number{index}(x:number,y:number):number[]{{let target=x;const result=target{operator}y;return[result,target];}}"
            );
        }
    }
    source
}

#[test]
fn all_numeric_operators_return_committed_values_in_contexts_and_preserve_number_bits() {
    let mut source = numeric_source(false);
    source += r#"
        type Cell={n:int;};
        function pair(left:int,right:int):int{return left*100+right;}
        function contexts():int[]{
            let a=1;let b=2;const first=a+=b*=3;const second=a*=b+=1;
            const third=a+=b=4;const fourth=a=b+=3;
            const values=[a+=1,a*=2];const row:Cell={n:a%=3};const called=pair(a+=2,a-=1);
            const input=[1,2];let sum=0;const mapped=input.map((value:int):int=>{return sum+=value;});
            let captured=0;{const add=():int=>{return a+=2;};captured=add();}
            const read=():int=>{return a;};const changed=a*=3;const snapshot=read();
            let count=0;let flag=false;const skipped=flag&&=(count+=1)>0;const taken=flag||=(count+=2)>0;
            while((count-=1)>0){}let index=0;sum=0;for(index=0;index<3;sum+=index++){continue;}
            const selected=[4,9];const selectedValue=selected[a%=2]+=3;
            return[first,second,third,fourth,b,values[0],values[1],row.n,called,mapped[0],mapped[1],captured,changed,snapshot,count,sum,index,selectedValue,a];
        }
        function contextual():number{let target:number=-0;return target*=1;}
        function numberCallback():number[]{let target:number=2;const input=[1.0,2.0];return input.map((value:number):number=>{return target+=value;});}
        function factory():(value:int)=>int{let target=2;return (value:int):int=>{return target**=value;};}
        function self():int[]{let value=2;const result=value+=value;return[result,value];}
        function main(){let text="old\u0000😀";text+=text;console.log(text);}
    "#;
    let mut body = r#"
        fn integer(operation:fn(i64,i64)->NumericArray<i64>,left:i64,right:i64,expected:i64){let values=operation(std::hint::black_box(left),std::hint::black_box(right));assert_eq!(values.as_slice(),&[expected,expected]);}
        fn number(operation:fn(f64,f64)->Vec<f64>,left:f64,right:f64,expected:f64){let values=operation(std::hint::black_box(left),std::hint::black_box(right));assert_eq!(values.len(),2);assert_eq!(values[0].to_bits(),values[1].to_bits());if expected.is_nan(){assert!(values[0].is_nan());}else{assert_eq!(values[0].to_bits(),expected.to_bits());}}
        // Independent 32-bit conversion: truncate, reduce modulo 2^32, then reinterpret signed bits.
        fn uint32(value:f64)->u32{if !value.is_finite()||value==0.0{return 0;}let reduced=value.trunc()%4294967296.0;let positive=if reduced<0.0{reduced+4294967296.0}else{reduced};positive as u32}
    "#.to_owned();
    let integers = [17, 13, 30, 7, 1, 225, 2, 15, 13, 60, 3, 3];
    let numbers = [
        17.5, 13.5, 31.0, 7.75, 1.5, 240.25, 2.0, 15.0, 13.0, 60.0, 3.0, 3.0,
    ];
    for (index, (integer, number)) in integers.iter().zip(numbers).enumerate() {
        body += &format!(
            "integer(function{},15,2,{integer});number(function{},15.5,2.0,{number:?});",
            index * 2,
            index * 2 + 1
        );
    }
    body += r#"
        for (left,right) in [(1_i64<<40,3),(-7,2),(i64::MIN,0),(i64::MAX,0)] {
            integer(function12,left,right,left&right);integer(function14,left,right,left|right);integer(function16,left,right,left^right);
        }
        for (left,count) in [(i64::MIN,0),(i64::MIN,63),(-1,1),(-1,63),(i64::MAX,1),(1,63),(1,0)]{
            integer(function18,left,count,left.wrapping_shl(count as u32));integer(function20,left,count,left>>count);integer(function22,left,count,((left as u64)>>count) as i64);
        }
        integer(function22,-1,0,-1);integer(function22,-1,1,i64::MAX);integer(function10,-2,63,i64::MIN);integer(function10,0,0,1);
        for (left,right) in [(-5.75,2.75),(4294967297.0,33.0),(f64::INFINITY,f64::NAN),(f64::NAN,-1.0),(-0.0,0.0)]{
            let a=uint32(left);let b=uint32(right);let shift=b&31;
            number(function13,left,right,(a&b) as i32 as f64);number(function15,left,right,(a|b) as i32 as f64);number(function17,left,right,(a^b) as i32 as f64);
            number(function19,left,right,a.wrapping_shl(shift) as i32 as f64);number(function21,left,right,((a as i32)>>shift) as f64);number(function23,left,right,(a>>shift) as f64);
        }
        for (left,right) in [(-0.0,0.0),(-0.0,-0.0),(f64::INFINITY,f64::NEG_INFINITY),(f64::from_bits(0x7ff8000000000042),1.0),(1.0,0.0),(-1.0,0.0)]{
            number(function1,left,right,left+right);number(function3,left,right,left-right);number(function5,left,right,left*right);number(function7,left,right,left/right);number(function9,left,right,left%right);
        }
        for (left,right,expected) in [(-0.0,3.0,-0.0),(-0.0,-3.0,f64::NEG_INFINITY),(0.0,0.0,1.0),(1.0,f64::INFINITY,f64::NAN),(-1.0,f64::NEG_INFINITY,f64::NAN),(-2.0,0.5,f64::NAN),(f64::NAN,0.0,1.0),(f64::NAN,1.0,f64::NAN),(2.0,f64::NEG_INFINITY,0.0),(2.0,f64::INFINITY,f64::INFINITY)]{number(function11,left,right,expected);}
        assert_eq!(function25().as_slice(),&[7,49,53,7,7,8,16,1,302,1,3,4,12,4,0,3,3,7,0]);
        assert_eq!(function26().to_bits(),(-0.0_f64).to_bits());assert_eq!(function27(),vec![3.0,5.0]);let mut power=function28();assert_eq!(power.call(3),8);assert_eq!(power.call(2),64);assert_eq!(function29().as_slice(),&[4,4]);
    "#;
    native(&source, "old\0😀old\0😀\n", &body, "");
}

#[test]
fn old_snapshots_selection_reacquisition_captures_and_growth_follow_source_order() {
    let analysis = native(
        r#"
        function mutate(values:int[],answer:int):int{values[0]=100;values.push(42);return answer;}
        function growth(values:int[],answer:int):int{return values[0]+=mutate(values,answer);}
        function reversed(values:int[]):int{values.reverse();return 3;}
        function reverseApply(values:int[]):int{return values[0]+=reversed(values);}
        function removed(values:int[]):int{values.remove(0);return 3;}
        function removeApply(values:int[]):int{return values[0]+=removed(values);}
        function lostApply(values:int[]):int{return values[1]+=removed(values);}
        type Row={n:int;number:number;text:string;};
        function grow(rows:Row[],answer:int):int{rows.push({n:77,number:1.0,text:"extra"});return answer;}
        function statement(rows:Row[],answer:int):int{rows[0].n+=grow(rows,answer);return rows[0].n;}
        function expression(rows:Row[],answer:int):int{return rows[0].n+=grow(rows,answer);}
        function chain(rows:Row[],counts:int[]):int{return rows[counts[0]++].n+=rows[counts[1]++].n*=grow(rows,3);}
        function growNumber(rows:Row[],answer:number):number{rows.push({n:77,number:1.0,text:"extra"});return answer;}
        function number(rows:Row[],answer:number):number{return rows[0].number*=growNumber(rows,answer);}
        function nested(rows:Row[][],other:Row[]):int{return rows[0][0].n+=grow(other,3);}
        function counter():(value:int)=>int{let target=1;return (value:int):int=>{return target+=value;};}
        function update(row:Row,value:int):int{return row.n+=value;}
        function mark():int{console.log("index");return 0;}
        function rhs():int{console.log("rhs");return 3;}
        function main(){
            const values=[7];console.log(values[mark()]+=rhs());console.log(values[0]);values[mark()]+=rhs();console.log(values[0]);
            const rows=[[1]];let i=0;let j=0;console.log(rows[i++][j++]*=3);console.log(i);console.log(j);console.log(rows[0][0]);
            const row:Row={n:1,number:2.0,text:"owner😀"};const text=row.text;console.log(row.n+=2);row.text=text;console.log(update(row,4));console.log(row.text);
            let target=1;{const change=():int=>{return target*=3;};console.log(change());}console.log(target);
            let snapshot=3;const read=():int=>{return snapshot;};console.log(snapshot+=1);console.log(read());console.log(snapshot);
            const retained=counter();console.log(retained(2));console.log(retained(3));
        }
    "#,
        "index\nrhs\n10\n10\nindex\nrhs\n13\n3\n1\n1\n3\n3\n7\nowner😀\n3\n3\n4\n3\n4\n3\n6\n",
        r#"
        for answer in [-3,0,23]{let mut values=NumericArray::from([7,2,3,4]);assert_eq!(function1(&mut values,answer),7+answer);assert_eq!(values.as_slice(),&[7+answer,2,3,4,42]);}
        let mut values=NumericArray::from([7,2]);assert_eq!(function3(&mut values),10);assert_eq!(values.as_slice(),&[10,7]);
        let mut values=NumericArray::from([7,2]);assert_eq!(function5(&mut values),10);assert_eq!(values.as_slice(),&[10]);
        fn row(n:i64)->Record0{Record0{field0:n,field1:-0.0,field2:String::Owned("owned\0😀e\u{301}".to_owned())}}
        for operation in [function8 as fn(&mut Array<Record0>,i64)->i64,function9]{let mut rows=Array::from([row(7),row(2),row(3),row(4)]);assert_eq!(operation(&mut rows,3),10);assert_eq!(rows.len(),5);assert_eq!(rows.get(0).unwrap().field0,10);assert_eq!(rows.get(0).unwrap().field2.as_ref(),"owned\0😀e\u{301}");}
        let mut rows=Array::from([row(7),row(2),row(3),row(4)]);let mut counts=NumericArray::from([0,1]);assert_eq!(function10(&mut rows,&mut counts),13);assert_eq!(counts.as_slice(),&[1,2]);assert_eq!(rows.len(),5);assert_eq!(rows.get(0).unwrap().field0,13);assert_eq!(rows.get(1).unwrap().field0,6);
        for value in [1.0,-1.0,f64::INFINITY,f64::NAN]{let mut rows=Array::from([row(7),row(2),row(3),row(4)]);let result=function12(&mut rows,value);let expected=-0.0*value;assert_eq!(result.to_bits(),rows.get(0).unwrap().field1.to_bits());if expected.is_nan(){assert!(result.is_nan());}else{assert_eq!(result.to_bits(),expected.to_bits());}assert_eq!(rows.len(),5);}
        let mut rows=vec![Array::from([row(7)])];let mut other=Array::from([row(2),row(3),row(4),row(5)]);assert_eq!(function13(&mut rows,&mut other),10);assert_eq!(rows.get(0).unwrap().get(0).unwrap().field0,10);assert_eq!(other.len(),5);
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let mut values=NumericArray::from([7,2]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function6(&mut values))).is_err());assert_eq!(values.as_slice(),&[2]);std::panic::set_hook(hook);
    "#,
        "",
    );
    for name in [
        "mutate.values",
        "growth.values",
        "expression.rows",
        "update.row",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Mutate));
    }
    let fixed=check("function main(){const a=[1,2,3];console.log(a[1]+=2);const n=[1.0,2.0];console.log(n[0]*=1);}").unwrap().emit_rust();
    assert!(fixed.contains(": [i64; 3] = ["));
    assert!(fixed.contains(": [f64; 2] = ["));
}

#[test]
fn strict_types_checked_failures_and_depth_guards_preserve_targets() {
    for operator in OPERATORS {
        for source in [
            format!("function main(){{const n=1;console.log(n{operator}1);}}"),
            format!("function f(n:int):int{{return n{operator}1;}}function main(){{}}"),
            format!("function f(n:number):number{{return n{operator}1;}}function main(){{}}"),
            format!("function main(){{let n=1;const x=1.0;console.log(n{operator}x);}}"),
            format!("function main(){{let n=1.0;const x=1;console.log(n{operator}x);}}"),
            format!(
                "function main(){{let n=1;let other=1.0;console.log(n{operator}other{operator}1);}}"
            ),
            format!("function f():number{{let n=1;return n{operator}1;}}function main(){{}}"),
            format!("function main(){{console.log(1{operator}2);}}"),
            format!("function main(){{let flag=true;console.log(flag{operator}1);}}"),
        ] {
            assert!(check(&source).is_err(), "{source}");
        }
    }
    for (source, code) in [
        (
            "function main(){let text=\"old\";console.log(text+=\"new\");}",
            "unsupported",
        ),
        (
            "function read(a:int[],value:int):int{return a[0];}function main(){const a=[1];read(a,a[0]+=1);}",
            "conflicting-access",
        ),
        (
            "function grow(a:int[][]):int{a.push([2]);return 3;}function main(){const a=[[1]];console.log(a[0][0]+=grow(a));}",
            "conflicting-access",
        ),
        (
            "type Box={n:int;values:int[];};function main(){const box:Box={n:1,values:[1]};console.log(box.values[0]+=(box.n+=1));}",
            "conflicting-access",
        ),
        (
            "type Box={n:int;};function main(){const box:Box={n:1};const read=():int=>{return box.n;};console.log(box.n+=1);}",
            "capture-conflict",
        ),
        (
            "type Row={n:int;};function take(a:Row[]):int{a;return 1;}function main(){const a:Row[]=[{n:1}];console.log(a[0].n+=take(a));}",
            "transferred",
        ),
        (
            "type Box={n:int;text:string;};function take(s:string):int{s;return 1;}function main(){const b:Box={n:1,text:\"owned\"};console.log(b.n+=take(b.text));console.log(b.text);}",
            "partial-transfer",
        ),
        (
            "function main(){const a=[1];for(const value of a){console.log(a[0]+=1);}}",
            "iteration-conflict",
        ),
        (
            "function main(){const a=[1];console.log(a.length+=1);}",
            "unsupported",
        ),
        (
            "function main(){let n=1;console.log((true?n:n)??=2);}",
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
    let deep = format!(
        "function main(){{let n=1;console.log({}1);}}",
        "n+=".repeat(200)
    );
    assert_eq!(check(&deep).err().unwrap().code, "unsupported");
    let mut source = String::new();
    for (index, operator) in OPERATORS.iter().enumerate() {
        source +=
            &format!("function trap{index}(a:int[],rhs:int):int{{return a[0]{operator}rhs;}}");
    }
    source += r#"
        function mutate(a:int[]):int{a[0]=9;return 1;}
        function overflow(a:int[]):int{return a[0]+=mutate(a);}
        function failingRhs(a:int[]):int{a[0]=99;a.push(42);return 1/0;}
        function noCommit(a:int[]):int{return a[0]*=failingRhs(a);}
        function bounds(rows:int[][],counts:int[]):int{return rows[counts[0]++][counts[1]++] += counts[2]+=1;}
        function marker():int{console.log("unexpected-rhs");return 1;}
        function bad(a:int[]):int{return a[-1]+=marker();}
        function statement(a:int[]){a[-1]+=marker();}
        function main(){}
    "#;
    let mut body=r#"let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));fn trap(operation:fn(&mut NumericArray<i64>,i64)->i64,left:i64,right:i64){let mut values=NumericArray::from([left]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||operation(&mut values,right))).is_err());assert_eq!(values.as_slice(),&[left]);}"#.to_owned();
    for (index, left, right) in [
        (0, "i64::MAX", "1"),
        (1, "i64::MIN", "1"),
        (2, "i64::MAX", "2"),
        (3, "1", "0"),
        (3, "i64::MIN", "-1"),
        (4, "1", "0"),
        (4, "i64::MIN", "-1"),
        (5, "2", "63"),
        (5, "2", "-1"),
        (5, "2", "4294967296"),
    ] {
        body += &format!("trap(function{index},{left},{right});");
    }
    for index in [9, 10, 11] {
        for right in ["-1", "64", "i64::MIN", "i64::MAX"] {
            body += &format!("trap(function{index},1,{right});");
        }
    }
    body += r#"
        let mut values=NumericArray::from([i64::MAX]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function13(&mut values))).is_err());assert_eq!(values.as_slice(),&[9]);
        let mut values=NumericArray::from([7,2,3,4]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function15(&mut values))).is_err());assert_eq!(values.as_slice(),&[99,2,3,4,42]);
        for first in [-1,1]{let mut rows=Array::from([NumericArray::from([1])]);let mut counts=NumericArray::from([first,0,0]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function16(&mut rows,&mut counts))).is_err());assert_eq!(counts.as_slice(),&[first+1,0,0]);}
        let mut rows=Array::from([NumericArray::from([1])]);let mut counts=NumericArray::from([0,1,0]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function16(&mut rows,&mut counts))).is_err());assert_eq!(counts.as_slice(),&[1,2,0]);
        for statement in [false,true]{let mut values=NumericArray::from([1]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||{if statement{function19(&mut values);}else{function18(&mut values);}})).is_err());assert_eq!(values.as_slice(),&[1]);}std::panic::set_hook(hook);
    "#;
    native(&source, "", &body, "");
}

#[test]
fn numeric_values_allocate_nothing_and_owned_storage_disposes_on_success_and_unwind() {
    let mut source = numeric_source(true);
    source += "function failing(box:Box):int{return box.integer+=1/0;}function main(){}";
    let mut body=r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for text in ["","owned\0😀e\u{301}"]{
            let mut owner=Record0{field0:15,field1:15.5,field2:String::Owned(text.to_owned())};
    "#.to_owned();
    let integers = [17, 13, 30, 7, 1, 225, 2, 15, 13, 60, 3, 3];
    let numbers = [
        17.5, 13.5, 31.0, 7.75, 1.5, 240.25, 2.0, 15.0, 13.0, 60.0, 3.0, 3.0,
    ];
    for (index, (integer, number)) in integers.iter().zip(numbers).enumerate() {
        body += &format!(
            r#"
        owner.field0=std::hint::black_box(15);let count=ALLOCATIONS.load(Ordering::SeqCst);let live=LIVE.load(Ordering::SeqCst);let result=std::hint::black_box(function{})(std::hint::black_box(&mut owner),std::hint::black_box(2));assert_eq!(result,{integer});assert_eq!(owner.field0,result);std::hint::black_box(result);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        owner.field1=std::hint::black_box(15.5);let count=ALLOCATIONS.load(Ordering::SeqCst);let live=LIVE.load(Ordering::SeqCst);let result=std::hint::black_box(function{})(std::hint::black_box(&mut owner),std::hint::black_box(2.0));assert_eq!(result.to_bits(),({number:?}_f64).to_bits());assert_eq!(owner.field1.to_bits(),result.to_bits());std::hint::black_box(result);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),live);
        "#,
            index * 2,
            index * 2 + 1
        );
    }
    body += r#"
        assert_eq!(owner.field2.as_ref(),text);drop(owner);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:9,field1:0.0,field2:String::Owned(text.to_owned())};function24(&mut owner);}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:9,field1:0.0,field2:String::Owned(text.to_owned())};let result=function0(&mut owner,23);assert_eq!(owner.field0,result);std::hint::black_box(result);panic!("dispose committed owner");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }std::panic::set_hook(hook);
    "#;
    let analysis = native(&source, "", &body, ALLOCATOR);
    for index in 0..12 {
        for leaf in ["integer", "number"] {
            assert_eq!(
                analysis.contracts().get(&format!("{leaf}{index}.box")),
                Some(&Mode::Mutate)
            );
        }
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
