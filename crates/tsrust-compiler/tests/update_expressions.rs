mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, body: &str, globals: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("update.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("update.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("update-{level}"));
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
fn prefix_postfix_values_contexts_and_ieee_bits_are_exact() {
    let mut source = String::new();
    for (id, expression) in ["++value", "value++", "--value", "value--"]
        .into_iter()
        .enumerate()
    {
        for (offset, ty) in [(0, "int"), (4, "number")] {
            source += &format!(
                "function update{}(input:{ty}):{ty}[]{{let value=input;const result={expression};return [result,value];}}\n",
                id + offset
            );
        }
    }
    source += r#"
    type Pair={integer:int;number:number;};
    function show(value:int){console.log(value);}
    function main(){let x=1;const values=[x++,++x,x--,--x];console.log(values[0]);console.log(values[1]);console.log(values[2]);console.log(values[3]);console.log(x);
        const pair:Pair={integer:++x,number:1.5};console.log(pair.integer);show(x++);console.log(x);
        let number:number=2;console.log(++number ** 2);console.log(number-- ** 2);console.log(-number++);console.log(number);
        const inputs=[1,2];let count=0;const mapped=inputs.map((value:int):int=>{return value+count++;});console.log(mapped[0]);console.log(mapped[1]);console.log(count);
    }"#;
    native(
        &source,
        "1\n3\n3\n1\n1\n2\n2\n3\n9\n9\n-2\n3\n1\n3\n2\n",
        r#"
        // Declarations alternate int and Number functions; inspect both returned and committed values.
        let integers:[fn(i64)->NumericArray<i64>;4]=[function0,function2,function4,function6];
        for (operation,expected) in integers.into_iter().zip([[18,18],[17,18],[16,16],[17,16]]) {let output=operation(17);assert_eq!(output.get(0),Some(&expected[0]));assert_eq!(output.get(1),Some(&expected[1]));}
        assert_eq!(function0(i64::MAX-1).get(0),Some(&i64::MAX));assert_eq!(function4(i64::MIN+1).get(0),Some(&i64::MIN));
        let numbers:[fn(f64)->Vec<f64>;4]=[function1,function3,function5,function7];
        let cells:[(f64,f64,f64);8]=[(-0.0,1.0,-1.0),(0.0,1.0,-1.0),(1.5,2.5,0.5),(f64::MAX,f64::MAX,f64::MAX),(f64::INFINITY,f64::INFINITY,f64::INFINITY),(f64::NEG_INFINITY,f64::NEG_INFINITY,f64::NEG_INFINITY),(f64::from_bits(1),1.0,-1.0),(9007199254740992.0,9007199254740992.0,9007199254740991.0)];
        for (input,incremented,decremented) in cells {for (id,operation) in numbers.into_iter().enumerate() {let output=operation(input);let new=if id<2{incremented}else{decremented};let result=if id%2==0{new}else{input};assert_eq!(output.get(0).unwrap().to_bits(),result.to_bits());assert_eq!(output.get(1).unwrap().to_bits(),new.to_bits());}}
        for operation in numbers {let output=operation(f64::NAN);assert!(output.get(0).unwrap().is_nan());assert!(output.get(1).unwrap().is_nan());}
        let payload=f64::from_bits(0xfff8000000000042);assert_eq!(function3(payload).get(0).unwrap().to_bits(),payload.to_bits());assert_eq!(function7(payload).get(0).unwrap().to_bits(),payload.to_bits());
    "#,
        "",
    );
}

#[test]
fn target_selection_effect_order_loans_captures_and_fixed_storage_are_preserved() {
    let analysis = native(
        r#"
        type Box={n:int;text:string;};
        function pair(a:int,b:int){console.log(a);console.log(b);}
        function mark():int{console.log("index");return 0;}
        function update(box:Box):int{return box.n++;}
        function counter():(step:int)=>int{let count=5;return (step:int):int=>{if(step>0){return count++;}return --count;};}
        function main(){
            let x=1;pair(x++,++x);console.log(x);console.log(false && (++x>0));console.log(true || (x++>0));console.log(x);
            const values=[0,10];const old=values[values[0]++]++;console.log(old);console.log(values[0]);console.log(values[mark()]++);console.log(values[0]);console.log(values[0]+++values[0]);
            values.push(values[0]++);console.log(values[0]);console.log(values[2]);
            const rows=[[7,8]];let i=0;let j=0;console.log(rows[i++][j++]++);console.log(rows[0][0]);console.log(i);console.log(j);
            const nested=[[0,10]];console.log(nested[nested[0][0]++][0]++);console.log(nested[0][0]);
            const box:Box={n:7,text:"owner"};console.log(update(box));const moved=box.text;console.log(box.n++);console.log(box.n);box.text=moved;console.log(box.text);
            let captured=0;{const next=():int=>{return captured++;};console.log(next());console.log(next());}console.log(captured);
            const retained=counter();console.log(retained(1));console.log(retained(1));console.log(retained(0));
            let loop=0;while(loop++<3){}console.log(loop);let sum=0;for(let k=0;k<3;k++){if(k===1){continue;}sum+=k;}console.log(sum);
            let init=0;for(init++;init<2;++init){}console.log(init);do{--init;}while(init>0);console.log(init);
            const iterable=[3,4];for(let element of iterable){console.log(element++);}console.log(iterable[0]);
        }
    "#,
        "1\n3\n3\nfalse\ntrue\n3\n1\n2\nindex\n2\n3\n7\n5\n4\n7\n8\n1\n1\n1\n2\n7\n8\n9\nowner\n0\n1\n2\n5\n6\n6\n4\n2\n2\n0\n3\n4\n3\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("update.box"), Some(&Mode::Mutate));
    let fixed = native(
        "function main(){const values=[1,2,3];console.log(values[1]++);console.log(++values[1]);const words=[\"a\",\"b\"];let index=0;console.log(words[index++]);console.log(index);}",
        "2\n4\na\n1\n",
        "",
        "",
    );
    assert!(fixed.emit_rust().contains(": [i64; 3] = ["));
    assert!(fixed.emit_rust().contains("LiteralStringArray"));
}

#[test]
fn grammar_diagnostics_bounds_and_checked_traps_preserve_targets() {
    let valid = native(
        "function main(){let a=1;const b=5;console.log(a+++b);console.log(a);console.log(a---b);console.log(a);console.log((a\n)++);console.log(a/* same line */++);console.log(++\na);let count=1;const read=():int=>{return count;};count++;console.log(read());console.log(count);}",
        "6\n2\n-3\n1\n1\n2\n4\n1\n2\n",
        "",
        "",
    );
    assert!(!valid.emit_rust().contains("unsafe"));
    for terminator in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        for gap in [
            terminator.to_owned(),
            format!("/*{terminator}*/"),
            format!("// comment{terminator}"),
        ] {
            let source = format!("function main(){{let n=1;n{gap}++;}}");
            let error = check(&source).err().unwrap();
            assert_eq!(error.code, "syntax");
            assert_eq!(&source[error.span.start..error.span.end], "++");
        }
    }
    for (source, code) in [
        ("function main(){const n=1;n++;}", "unsupported"),
        ("function f(n:int){++n;}function main(){}", "unsupported"),
        ("function main(){let text=\"x\";text++;}", "type"),
        ("function main(){let flag=true;--flag;}", "type"),
        ("function main(){let values=[1];values++;}", "type"),
        (
            "function main(){const values=[1];values.length++;}",
            "unsupported",
        ),
        ("function main(){Math.PI++;}", "type"),
        (
            "function main(){const values=[1];for(const value of values){value++;}}",
            "unsupported",
        ),
        (
            "function main(){const values=[1];for(const value of values){values[0]++;}}",
            "iteration-conflict",
        ),
        (
            "type Box={n:int;};function main(){const values:Box[]=[{n:1}];for(const value of values){value.n++;}}",
            "borrowed-element",
        ),
        (
            "type Box={n:int;};function hold(box:Box,n:int){console.log(box.n);}function main(){const box:Box={n:1};hold(box,box.n++);}",
            "conflicting-access",
        ),
        (
            "function hold(values:int[],n:int){console.log(values.length);}function main(){const values=[1];hold(values,values[0]++);}",
            "conflicting-access",
        ),
        (
            "type Box={n:int;values:int[];};function main(){const box:Box={n:0,values:[1]};box.values[box.n++]++;}",
            "conflicting-access",
        ),
        (
            "function main(){const rows=[[0]];rows[0][rows[0][0]++]++;}",
            "conflicting-access",
        ),
        (
            "type Box={n:int;};function main(){const box:Box={n:1};const moved=box;box.n++;}",
            "transferred",
        ),
        (
            "type Box={part:Part;};type Part={n:int;};function main(){const box:Box={part:{n:1}};const moved=box.part;box.part.n++;}",
            "partial-transfer",
        ),
        (
            "function main(){let n=1;const next=():int=>{return n++;};n++;console.log(next());}",
            "capture-conflict",
        ),
        (
            "function f():number{let n=1;return n++;}function main(){}",
            "type",
        ),
        (
            "function f():int{let n=1.0;return ++n;}function main(){}",
            "type",
        ),
        ("function main(){console.log(++1);}", "unsupported"),
        (
            "function f():int{return 1;}function main(){f()++;}",
            "unsupported",
        ),
        ("function main(){let n=1;++(n++);}", "unsupported"),
        ("function main(){let n=1;console.log(-n++ ** 2);}", "syntax"),
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
        "function main(){let n=1;n++++;}",
        "function main(){let n=1;n++=2;}",
        "function main(){let text=\"old\";console.log(text+=\"new\");}",
        "function main(){let flag=1;flag&&=false;}",
    ] {
        assert!(check(source).is_err());
    }
    // `++++n` increments a value, not a place; deep grouping has no limit.
    assert!(check(&format!("function main(){{let n=1;{}n;}}", "++".repeat(200))).is_err());
    support::accept_deep(&format!("function main(){{let n=1;{}n++{}; }}", "(".repeat(200), ")".repeat(200)));
    native(
        r#"
        type Box={n:int;text:string;};
        function increment(values:int[]):int{return values[0]++;}
        function decrement(values:int[]):int{return --values[0];}
        function record(box:Box):int{return ++box.n;}
        function bounds(rows:int[][],counts:int[]):int{return rows[counts[0]++][counts[1]++]++;}
        function rows():int[][]{return [[7]];}
        function main(){}
    "#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        let mut values=NumericArray::from([i64::MAX]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function0(&mut values))).is_err());assert_eq!(values.get(0),Some(&i64::MAX));
        let mut values=NumericArray::from([i64::MIN]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function1(&mut values))).is_err());assert_eq!(values.get(0),Some(&i64::MIN));
        let mut owner=Record0{field0:i64::MAX,field1:String::Owned("owned😀".to_owned())};assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function2(&mut owner))).is_err());assert_eq!(owner.field0,i64::MAX);assert_eq!(owner.field1.as_ref(),"owned😀");
        for first in [-1,1] {let mut rows=function4();let mut counts=NumericArray::from([first,0]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut rows,&mut counts))).is_err());assert_eq!(counts.get(0),Some(&(first+1)));assert_eq!(counts.get(1),Some(&0));assert_eq!(rows.get(0).unwrap().get(0),Some(&7));}
        let mut rows=function4();let mut counts=NumericArray::from([0,1]);assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(||function3(&mut rows,&mut counts))).is_err());assert_eq!(counts.get(0),Some(&1));assert_eq!(counts.get(1),Some(&2));assert_eq!(rows.get(0).unwrap().get(0),Some(&7));
        std::panic::set_hook(hook);
    "#,
        "",
    );
}

#[test]
fn updates_allocate_nothing_and_release_owned_storage_normally_and_on_unwind() {
    let analysis = native(
        r#"
        type Box={n:int;number:number;text:string;};
        function integer0(box:Box):int{return ++box.n;}
        function integer1(box:Box):int{return box.n++;}
        function integer2(box:Box):int{return --box.n;}
        function integer3(box:Box):int{return box.n--;}
        function number0(box:Box):number{return ++box.number;}
        function number1(box:Box):number{return box.number++;}
        function number2(box:Box):number{return --box.number;}
        function number3(box:Box):number{return box.number--;}
        function main(){}
    "#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for text in ["","owned\0😀e\u{301}"] {
            let mut owner=Record0{field0:0,field1:0.0,field2:String::Owned(text.to_owned())};
            let integers:[fn(&mut Record0)->i64;4]=[function0,function1,function2,function3];
            for (input,increased,decreased) in [(7,8,6),(-4,-3,-5)] {for (id,operation) in integers.into_iter().enumerate(){
                owner.field0=std::hint::black_box(input);let before=LIVE.load(Ordering::SeqCst);let count=ALLOCATIONS.load(Ordering::SeqCst);let result=std::hint::black_box(operation)(std::hint::black_box(&mut owner));let new=if id<2{increased}else{decreased};assert_eq!(result,if id%2==0{new}else{input});assert_eq!(owner.field0,new);assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),before);
            }}
            let numbers:[fn(&mut Record0)->f64;4]=[function4,function5,function6,function7];
            let cells:[(f64,f64,f64);3]=[(-0.0,1.0,-1.0),(1.5,2.5,0.5),(9007199254740992.0,9007199254740992.0,9007199254740991.0)];
            for (input,increased,decreased) in cells {for (id,operation) in numbers.into_iter().enumerate(){
                owner.field1=std::hint::black_box(input);let before=LIVE.load(Ordering::SeqCst);let count=ALLOCATIONS.load(Ordering::SeqCst);let result=std::hint::black_box(operation)(std::hint::black_box(&mut owner));let new=if id<2{increased}else{decreased};assert_eq!(result.to_bits(),(if id%2==0{new}else{input}).to_bits());assert_eq!(owner.field1.to_bits(),new.to_bits());assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count);assert_eq!(LIVE.load(Ordering::SeqCst),before);
            }}
            assert_eq!(owner.field2.as_ref(),text);drop(owner);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
            assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:i64::MAX,field1:0.0,field2:String::Owned(text.to_owned())};function0(&mut owner);}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
            assert!(std::panic::catch_unwind(||{let mut owner=Record0{field0:7,field1:-0.0,field2:String::Owned(text.to_owned())};let result=function5(&mut owner);std::hint::black_box((result,owner.field1));panic!("dispose result and owner");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
        }std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
    );
    for name in [
        "integer0.box",
        "integer1.box",
        "integer2.box",
        "integer3.box",
        "number0.box",
        "number1.box",
        "number2.box",
        "number3.box",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Mutate));
    }
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
"#;
