mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    body: &str,
    globals: &str,
    failure: bool,
) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("last-index.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("last-index.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("last-index-{level}"));
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
        assert_eq!(output.stdout, expected.as_bytes());
        if failure {
            assert!(!output.status.success());
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("array index out of bounds at source bytes")
            );
        } else {
            support::success(&output);
            assert!(output.stderr.is_empty());
        }
    }
    analysis
}

#[test]
fn primitive_results_normalize_extremes_and_use_strict_number_and_utf8_equality() {
    let analysis = native(
        r#"
        function ints(values:int[],value:int,from:int):int{return values.lastIndexOf(value,from);}
        function numbers(values:number[],value:number,from:int):int{return values.lastIndexOf(value,from);}
        function flags(values:boolean[],value:boolean,from:int):int{return values.lastIndexOf(value,from);}
        function words(values:string[],value:string,from:int):int{return values.lastIndexOf(value,from);}
        function defaultInt(values:int[],value:int):int{return values.lastIndexOf(value);}
        function defaultNumber(values:number[],value:number):int{return values.lastIndexOf(value);}
        function defaultFlag(values:boolean[],value:boolean):int{return values.lastIndexOf(value);}
        function defaultWord(values:string[],value:string):int{return values.lastIndexOf(value);}
        function main(){
            const values=[7,9,7];const found:int=values.lastIndexOf(7);console.log(found);
            console.log(values.lastIndexOf(7,0));console.log(values.lastIndexOf(7,-1));console.log(values.lastIndexOf(7,-4));
            console.log(values.lastIndexOf(7,9223372036854775807));console.log(values.lastIndexOf(7,-9223372036854775808));
            const names=["é","e\u0301","😀","\u0000","","é"];console.log(names.lastIndexOf("é"));console.log(names.lastIndexOf("e\u0301"));console.log(names.lastIndexOf("😀"));console.log(names.lastIndexOf("\u0000"));console.log(names.lastIndexOf(""));
            const decimals:number[]=[Number("NaN"),-0.0,0.0,Number("Infinity")];console.log(decimals.lastIndexOf(Number("NaN")));console.log(decimals.lastIndexOf(-0.0));console.log(decimals.lastIndexOf(Number("Infinity")));
            const empty:boolean[]=[];console.log(empty.lastIndexOf(false));const flags=[true];console.log(flags.lastIndexOf(false));
        }
    "#,
        "2\n0\n2\n-1\n2\n-1\n5\n1\n2\n3\n4\n-1\n2\n3\n-1\n-1\n",
        r#"
        // Independent forward scan: retain the greatest eligible matching index.
        fn expected<T:PartialEq>(values:&[T],needle:&T,from:Option<i64>)->i64{
            let bound=from.map_or(values.len() as i128-1,|index|if index<0{values.len() as i128+index as i128}else{index as i128});
            let mut result=-1;for(index,value)in values.iter().enumerate(){if index as i128<=bound && value==needle{result=index as i64;}}result
        }
        for len in [0,1,4,5,17,80] {
            let ints=(0..len).map(|i|i as i64%3-1).collect::<Vec<_>>();let mut intArray=NumericArray::from([]);for value in &ints{intArray.push(*value);}
            let flags=(0..len).map(|i|i%3!=0).collect::<Vec<_>>();let mut flagArray=NumericArray::from([]);for value in &flags{flagArray.push(*value);}
            let bits=[0x7ff8000000000123,0x8000000000000000,0,0x7ff0000000000000,0xfff0000000000000];let numbers=(0..len).map(|i|f64::from_bits(bits[i%5])).collect::<Vec<_>>();
            let mut words=Array::from([]);for i in 0..len{words.push(if i%3==0{String::Borrowed("é")}else if i%3==1{String::Owned("é".to_owned())}else{String::Borrowed("e\u{301}")});}let model=words.iter().cloned().collect::<Vec<_>>();
            for from in [i64::MIN,-(len as i64)-1,-(len as i64),-1,0,1,3,len as i64,i64::MAX] {
                for needle in [-2,-1,0,1,2]{assert_eq!(function0(&intArray,needle,from),expected(&ints,&needle,Some(from)));}
                for needle in [false,true]{assert_eq!(function2(&flagArray,needle,from),expected(&flags,&needle,Some(from)));}
                for needle in bits.map(f64::from_bits){assert_eq!(function1(&numbers,needle,from),expected(&numbers,&needle,Some(from)));}
                for needle in [String::Borrowed("é"),String::Owned("e\u{301}".to_owned()),String::Borrowed("")]{assert_eq!(function3(&words,&needle,from),expected(&model,&needle,Some(from)));}
            }
            assert_eq!(function4(&intArray,0),expected(&ints,&0,None));assert_eq!(function5(&numbers,-0.0),expected(&numbers,&-0.0,None));assert_eq!(function6(&flagArray,false),expected(&flags,&false,None));assert_eq!(function7(&words,&String::Borrowed("é")),expected(&model,&String::Borrowed("é"),None));
        }
    "#,
        "",
        false,
    );
    for name in [
        "ints.values",
        "numbers.values",
        "flags.values",
        "words.values",
        "words.value",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
}

#[test]
fn storage_selection_order_temporaries_and_readonly_forwarding_preserve_existing_owners() {
    let analysis = native(
        r#"
        type Box={rows:int[][];words:string[];};type Node={name:string;words:string[];children:Node[];};
        function mark(tag:int,value:int):int{console.log(tag);return value;}
        function word(value:string):string{console.log(3);return clone(value);}
        function make():int[]{console.log(1);return [7,8,7];}
        function change(values:int[]):int{values.push(7);return 7;}
        function query(words:string[],needle:string):int{return words.lastIndexOf(needle);}
        function invoke(f:(values:ReadOnly<int[]>)=>int,values:int[]):int{return f(values);}
        function owned(values:int[]):()=>int{return ():int=>{return values.lastIndexOf(7);};}
        function main(){
            const box:Box={rows:[[7,8,7]],words:["a","b","a"]};console.log(box.rows[mark(1,0)].lastIndexOf(mark(2,7),mark(4,1)));console.log(box.words.lastIndexOf(word("a"),mark(4,2)));
            const empty:int[]=[];console.log(empty.lastIndexOf(mark(2,7),mark(4,-99)));console.log(make().lastIndexOf(mark(2,7),mark(4,-1)));
            const values=[7];console.log(clone(values).lastIndexOf(change(values)));console.log(values.length);console.log([7,8,7].lastIndexOf(7));
            const words=["a","b","a"];console.log(query(words,"a"));const read=():int=>{return values.lastIndexOf(7);};console.log(read());console.log(read());
            console.log(invoke((rows:int[]):int=>{return rows.lastIndexOf(7);},values));const retained=owned([7,8,7]);console.log(retained());
            const rows=[[1,2,1],[3,4,3]];for(const row of rows){console.log(row.lastIndexOf(row[0]));}
            const node:Node={name:"root",words:[],children:[{name:"child",words:["yes","no","yes"],children:[]}]};console.log(node.children[0].words.lastIndexOf("yes"));
            const moved=node.name;console.log(node.children[0].words.lastIndexOf("no"));node.name=moved;
            const dynamic=[" x ".trim()," x ".trim(),"y"," x ".trim()];dynamic.remove(1);console.log(dynamic.lastIndexOf("x"));dynamic.push("x");dynamic.push("z");console.log(dynamic.lastIndexOf("x"));
            while(dynamic.length>0){dynamic.remove(0);}console.log(dynamic.lastIndexOf(""));dynamic.push("");console.log(dynamic.lastIndexOf(""));
        }
    "#,
        "1\n2\n4\n0\n3\n4\n2\n2\n4\n-1\n1\n2\n4\n2\n0\n2\n2\n2\n1\n1\n1\n2\n2\n2\n2\n1\n2\n3\n-1\n0\n",
        "",
        "",
        false,
    );
    let emitted = analysis.emit_rust();
    assert!(emitted.contains(".iter().rev().skip("));
    // Existing fixed-storage proof conservatively excludes bodies with captures.
    // Exercise literal and record forwarding in a separate eligible body.
    let fixed = native(
        r#"type Row={values:int[];};type Box={rows:Row[];};function query(words:string[],needle:string):int{return words.lastIndexOf(needle);}function queryBox(box:Box):int{return box.rows[0].values.lastIndexOf(1);}function main(){const words=["a","b","a"];console.log(query(words,"a"));const box:Box={rows:[{values:[1,2,1]}]};console.log(queryBox(box));}"#,
        "2\n2\n", "", "", false,
    ).emit_rust();
    assert!(fixed.contains("LiteralStringArray"));
    assert!(fixed.contains("FixedRecord"));
    native(
        r#"function make():int[][]{console.log(1);return [[7]];}function mark(tag:int,value:int):int{console.log(tag);return value;}function main(){console.log(make()[mark(2,9)].lastIndexOf(mark(3,7),mark(4,0)));}"#,
        "1\n2\n",
        "",
        "",
        true,
    );
}

#[test]
fn diagnostics_keep_primitive_identity_boundaries_and_receiver_needle_parent_loans() {
    for (source, code) in [
        ("function main(){[1].lastIndexOf();}", "arity"),
        ("function main(){[1].lastIndexOf(1,0,0);}", "arity"),
        ("function main(){[1].lastIndexOf(false);}", "type"),
        ("function main(){[1].lastIndexOf(1,0.0);}", "type"),
        (
            "function main(){const result:boolean=[1].lastIndexOf(1);}",
            "type",
        ),
        ("function main(){\"abc\".lastIndexOf(1);}", "type"),
        ("function main(){1.lastIndexOf(1);}", "type"),
        ("function main(){[[1]].lastIndexOf([1]);}", "unsupported"),
        (
            "type Row={n:int;};function main(){const rows:Row[]=[{n:1}];rows.lastIndexOf({n:1});}",
            "unsupported",
        ),
        (
            "function main(){const values=[1];const moved=values;values.lastIndexOf(1);}",
            "transferred",
        ),
        (
            "type Box={values:int[];};function main(){const box:Box={values:[1]};const moved=box.values;box.values.lastIndexOf(1);}",
            "partial-transfer",
        ),
        (
            "function main(){const values=[1];const read=():int=>{return values.lastIndexOf(1);};values.push(2);console.log(read());}",
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
    for source in [
        "function change(values:int[]):int{values.push(1);return 1;}function main(){const values=[1];values.lastIndexOf(change(values));}",
        "function change(values:int[]):int{values.push(1);return 0;}function main(){const values=[1];values.lastIndexOf(1,change(values));}",
        "function take(values:int[]):int{const moved=values;return 1;}function main(){const values=[1];values.lastIndexOf(take(values));}",
        "type Box={rows:int[][];n:int;};function change(box:Box):int{box.rows[0]=[2];return 0;}function main(){const box:Box={rows:[[1]],n:0};box.rows[0].lastIndexOf(1,change(box));}",
        "function take(word:string):int{const moved=word;return 0;}function main(){const words=[\"yes\"];const word=\"yes\";words.lastIndexOf(word,take(word));}",
    ] {
        let error = check(source)
            .err()
            .unwrap_or_else(|| panic!("Expected conflicting access: {source}"));
        assert_eq!(error.code, "conflicting-access");
        assert!(!error.related.is_empty());
    }
    native(
        "function take(word:string):string{return word;}function main(){const words=[\"yes\"];const word=\"yes\";console.log(words.lastIndexOf(word));console.log(take(word));words.push(\"no\");console.log(words.lastIndexOf(\"no\"));}",
        "0\nyes\n1\n",
        "",
        "",
        false,
    );
}

#[test]
fn searches_allocate_nothing_and_dispose_owned_input_bytes_normally_and_on_unwind() {
    native(
        r#"function query(values:string[],needle:string,from:int):int{return values.lastIndexOf(needle,from);}function main(){}"#,
        "",
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());let baseline=LIVE.load(Ordering::SeqCst);
        for len in [0,1,4,5,17,80]{for spare in [false,true]{
            let needle=String::Owned("é😀\0".repeat(97));let mut values=Array::from([]);for index in 0..len{values.push(if index%3==0{needle.clone()}else{String::Owned("e\u{301}".repeat(97))});}
            if spare{values.push(needle.clone());drop(values.remove(len));}
            let input=LIVE.load(Ordering::SeqCst);let pointer=needle.as_ptr();let count=ALLOCATIONS.load(Ordering::SeqCst);
            for from in [i64::MIN,-(len as i64)-1,-1,0,1,len as i64,i64::MAX]{let bound=if from<0{len as i128+from as i128}else{from as i128};let expected=(0..len).filter(|index|*index as i128<=bound&&index%3==0).last().map_or(-1,|i|i as i64);assert_eq!(function0(&values,&needle,from),expected);}
            assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),count,"search cloned or allocated");assert_eq!(needle.as_ptr(),pointer);assert_eq!(LIVE.load(Ordering::SeqCst),input);
            assert!(std::panic::catch_unwind(||{let values=values.clone();let needle=needle.clone();std::hint::black_box(function0(&values,&needle,i64::MAX));panic!("dispose search inputs");}).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),input);
            drop(values);drop(needle);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
        }}std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
        false,
    );
}

// Test-only forwarding allocator; generated implementation stays entirely safe.
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
