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
        check(source).unwrap_or_else(|error| panic!("{}", error.render("spliced.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + globals
        + &format!("\nfn main(){{source_main();{body}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("to-spliced.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("to-spliced-{level}"));
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
fn all_arity_forms_normalize_signed_extremes_and_preserve_exact_number_bits() {
    let analysis = native(
        r#"
        function all(values:int[]):int[]{return values.toSpliced();}
        function prefix(values:int[],start:int):int[]{return values.toSpliced(start);}
        function edit(values:int[],start:int,count:int,value:int):int[]{return values.toSpliced(start,count,value);}
        function numbers(values:number[],start:int,count:int,value:number):number[]{return values.toSpliced(start,count,value);}
        function main(){
            const literal=[1,2,3];const result=literal.toSpliced(-2,1,7,8);for(const value of result){console.log(value);}
            console.log(literal.length);console.log(literal[1]);
            const empty:int[]=[].toSpliced();console.log(empty.length);
            const inferred=[].toSpliced(0,0,4);console.log(inferred[0]);
            const decimals:number[]=[].toSpliced(0,0,1,2.5);console.log(decimals[1]);
            console.log(literal.toSpliced(0).length);console.log(literal.toSpliced(9223372036854775807).length);
        }
    "#,
        "1\n7\n8\n3\n3\n2\n0\n4\n2.5\n0\n3\n",
        r#"
        fn start(len:usize,index:i64)->usize{let size=len as i128;let from=index as i128;(if from<0{size+from}else{from}).clamp(0,size) as usize}
        for len in [0,1,4,5] {
            let mut source=NumericArray::from([]);for index in 0..len{source.push(index as i64);}
            let original=source.iter().copied().collect::<Vec<_>>();
            assert_eq!(function0(&source).iter().copied().collect::<Vec<_>>(),original);
            for from in [i64::MIN,-(len as i64)-1,-1,0,len as i64,(len+1) as i64,i64::MAX] {
                let begin=start(len,from);assert_eq!(function1(&source,from).iter().copied().collect::<Vec<_>>(),original[..begin]);
                for count in [i64::MIN,-1,0,1,len as i64,i64::MAX] {
                    let end=begin+((count as i128).max(0).min((len-begin) as i128) as usize);
                    // Independent oracle removes the range then inserts one value.
                    let mut expected=original.clone();expected.drain(begin..end);expected.insert(begin,99);
                    assert_eq!(function2(&source,from,count,99).iter().copied().collect::<Vec<_>>(),expected);
                }
            }
            assert_eq!(source.iter().copied().collect::<Vec<_>>(),original);
        }
        let original=[0x8000000000000000,0x7ff8000000000123,0x7ff0000000000000,0xfff0000000000000];
        let values=original.map(f64::from_bits).to_vec();let inserted=f64::from_bits(0xfff8000000000456);
        let copied=function3(&values,-2,1,inserted);
        assert_eq!(copied.iter().map(|value|value.to_bits()).collect::<Vec<_>>(),vec![original[0],original[1],inserted.to_bits(),original[3]]);
        assert_eq!(values.iter().map(|value|value.to_bits()).collect::<Vec<_>>(),original);
    "#,
        "",
        false,
    );
    for name in [
        "all.values",
        "prefix.values",
        "edit.values",
        "numbers.values",
    ] {
        assert_eq!(analysis.contracts().get(name), Some(&Mode::Read));
    }
}

#[test]
fn inserted_owners_transfer_and_retained_descendants_remain_independent_across_leaves_and_ranks() {
    let mut source = String::from(
        "type Row={name:string;items:int[];};type Node={name:string;children:Node[];};",
    );
    for leaf in ["int", "number", "boolean", "string", "Row", "Node"] {
        for rank in 1..=3 {
            let array = format!("{leaf}{}", "[]".repeat(rank));
            let element = format!("{leaf}{}", "[]".repeat(rank - 1));
            source.push_str(&format!("function edit{leaf}{rank}(values:{array},value:{element}):{array}{{return values.toSpliced(0,0,value);}}\n"));
        }
    }
    source.push_str(r#"
        function main(){
            const word=" saved ".trim();const words=[" first ".trim(),"middle","last"];const copy=words.toSpliced(1,1,clone(word),"new");
            words[0]="changed";console.log(copy[0]);console.log(copy[1]);console.log(copy[2]);console.log(copy[3]);console.log(word);
            const literals=["a","b","c"];const saved=literals.toSpliced(1,1,"z");console.log(saved[0]);console.log(saved[1]);console.log(saved[2]);
            const rows:int[][]=[[1],[2],[3]];const insert=[8,9];const inserted=rows.toSpliced(1,1,clone(insert));
            rows[0][0]=7;inserted[1][0]=4;console.log(inserted[0][0]);console.log(insert[0]);console.log(rows[1][0]);
            const records:Row[]=[{name:"one",items:[1]},{name:"two",items:[2]}];
            const recordCopy=records.toSpliced(1,1,{name:"new",items:[3]});records[0].items[0]=9;console.log(recordCopy[0].items[0]);console.log(recordCopy[1].items[0]);
            const nodes:Node[]=[{name:"root",children:[{name:"child",children:[]}]}];
            const nodeCopy=nodes.toSpliced(1,0,{name:"new",children:[]});nodes[0].children[0].name="changed";
            console.log(nodeCopy[0].children[0].name);console.log(nodeCopy[1].name);
            const triples:int[][][]=[[[1]],[[2]]];const deep=triples.toSpliced(0,1,[[3]]);triples[1][0][0]=9;console.log(deep[0][0][0]);console.log(deep[1][0][0]);
            const selected=rows[0].toSpliced(0,1,5);console.log(selected[0]);console.log(rows[0][0]);
            const temporary=[1,2].toSpliced(1,1,6).toSpliced(0,0,7);for(const value of temporary){console.log(value);}
        }
    "#);
    native(
        &source,
        "first\nsaved\nnew\nlast\nsaved\na\nz\nc\n1\n8\n2\n1\n3\nchild\nnew\n3\n2\n5\n7\n7\n1\n6\n",
        "",
        "",
        false,
    );
}

#[test]
fn source_order_and_diagnostics_preserve_receiver_and_selected_parent_loans() {
    native(
        r#"
        function receiver():int[]{console.log(1);return [1,2,3];}
        function mark(tag:int,value:int):int{console.log(tag);return value;}
        function main(){const result=receiver().toSpliced(mark(2,1),mark(3,1),mark(4,7),mark(5,8));for(const value of result){console.log(value);}}
    "#,
        "1\n2\n3\n4\n5\n1\n7\n8\n3\n",
        "",
        "",
        false,
    );
    native(
        r#"
        function receiver():int[][]{console.log(1);return [[1]];}
        function mark(tag:int,value:int):int{console.log(tag);return value;}
        function main(){const result=receiver()[mark(2,9)].toSpliced(mark(3,0),mark(4,0),mark(5,7));console.log(result.length);}
    "#,
        "1\n2\n",
        "",
        "",
        true,
    );
    for (source, code) in [
        (
            "function main(){const values=[1];values.toSpliced(true);}",
            "type",
        ),
        (
            "function main(){const values=[1];values.toSpliced(0,1.5);}",
            "type",
        ),
        (
            "function main(){const values=[1];values.toSpliced(0,0,false);}",
            "type",
        ),
        ("function main(){const value=1;value.toSpliced();}", "type"),
        ("function main(){[].toSpliced();}", "array-type"),
        (
            "function main(){const values=[1];const moved=values;values.toSpliced();}",
            "transferred",
        ),
        (
            "function main(){const values:string[]=[];const word=\" owned \".trim();const result=values.toSpliced(0,0,word);console.log(word);}",
            "transferred",
        ),
        (
            "function change(values:int[]):int{values.push(2);return 0;}function main(){const values=[1];values.toSpliced(change(values),0);}",
            "conflicting-access",
        ),
        (
            "function change(values:int[]):int{values.push(2);return 0;}function main(){const values=[1];values.toSpliced(0,change(values));}",
            "conflicting-access",
        ),
        (
            "function change(values:int[]):int{values.push(2);return 0;}function main(){const values=[1];values.toSpliced(0,0,change(values));}",
            "conflicting-access",
        ),
        (
            "type Grid={values:int[][];n:int;};function change(grid:Grid):int{grid.n=1;return 0;}function main(){const grid:Grid={values:[[1]],n:0};grid.values[0].toSpliced(0,0,change(grid));}",
            "conflicting-access",
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
}

#[test]
fn deleted_payloads_are_never_cloned_and_every_owned_byte_is_disposed_normally_and_on_unwind() {
    native(
        r#"
        type Small={name:string;};type Node={name:string;children:Node[];};
        function words(values:string[],start:int,count:int,value:string):string[]{return values.toSpliced(start,count,value);}
        function rows(values:Small[],start:int,count:int,value:Small):Small[]{return values.toSpliced(start,count,value);}
        function trees(values:Node[],start:int,count:int,value:Node):Node[]{return values.toSpliced(start,count,value);}
        function main(){}
    "#,
        "",
        r#"
        fn owned(size:usize)->String{String::Owned("x".repeat(size))}
        fn small(size:usize)->Record0{Record0{field0:owned(size)}}
        fn node(size:usize)->Record1{Record1{field0:owned(size),field1:RecordArray1::from([Record1{field0:owned(79),field1:RecordArray1::from([])}])}}
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));assert!(std::panic::catch_unwind(||panic!("warm")).is_err());
        let baseline=LIVE.load(Ordering::SeqCst);
        for len in [0,1,4,5] {for spare in [false,true] {
            let mut words=Array::from([]);let mut rows=Array::from([]);let mut trees=RecordArray1::from([]);
            for index in 0..len {let size=if index==0{4093}else{97};words.push(owned(size));rows.push(small(size));trees.push(node(size));}
            if spare{words.push(owned(97));rows.push(small(97));trees.push(node(97));drop(words.remove(len));drop(rows.remove(len));drop(trees.remove(len));}
            let inputs=LIVE.load(Ordering::SeqCst);
            let word=owned(131);let row=small(131);let tree=node(131);let pointers=(word.as_ptr(),row.field0.as_ptr(),tree.field0.as_ptr());
            SELECTED.store(0,Ordering::SeqCst);
            let copied=function0(&words,0,1,word);let copied_rows=function1(&rows,0,1,row);let copied_trees=function2(&trees,0,1,tree);
            assert_eq!(SELECTED.load(Ordering::SeqCst),0,"deleted payload cloned");
            assert_eq!(copied.len(),len.max(1));assert_eq!(copied_rows.len(),len.max(1));assert_eq!(copied_trees.len(),len.max(1));
            assert_eq!(copied.get(0).unwrap().as_ptr(),pointers.0);assert_eq!(copied_rows.get(0).unwrap().field0.as_ptr(),pointers.1);assert_eq!(copied_trees[0].field0.as_ptr(),pointers.2);
            for index in 1..len{assert_eq!(copied.get(index).unwrap(),words.get(index).unwrap());assert_ne!(copied_trees[index].field1[0].field0.as_ptr(),trees[index].field1[0].field0.as_ptr());}
            if len>0{assert_eq!(words.get(0).unwrap().len(),4093);assert_eq!(rows.get(0).unwrap().field0.len(),4093);assert_eq!(trees[0].field0.len(),4093);}
            drop(copied);drop(copied_rows);drop(copied_trees);assert_eq!(LIVE.load(Ordering::SeqCst)-inputs,0);
            assert!(std::panic::catch_unwind(||{
                let words=words.clone();let rows=rows.clone();let trees=trees.clone();
                let a=function0(&words,-1,i64::MAX,owned(131));let b=function1(&rows,-1,i64::MAX,small(131));let c=function2(&trees,-1,i64::MAX,node(131));
                std::hint::black_box((&a,&b,&c));panic!("dispose source owners and spliced outputs");
            }).is_err());assert_eq!(LIVE.load(Ordering::SeqCst)-inputs,0);
            drop(words);drop(rows);drop(trees);assert_eq!(LIVE.load(Ordering::SeqCst)-baseline,0);
        }}
        std::panic::set_hook(hook);
    "#,
        ALLOCATOR,
        false,
    );
}

// Test-only System allocator instrumentation; absent from emitted implementation.
const ALLOCATOR: &str = r#"
use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,AtomicUsize,Ordering};
static LIVE:AtomicIsize=AtomicIsize::new(0);static SELECTED:AtomicUsize=AtomicUsize::new(0);struct Observer;
unsafe impl GlobalAlloc for Observer{
    unsafe fn alloc(&self,layout:Layout)->*mut u8{let p=unsafe{System.alloc(layout)};if !p.is_null(){LIVE.fetch_add(layout.size() as isize,Ordering::SeqCst);if layout.size()==4093{SELECTED.fetch_add(1,Ordering::SeqCst);}}p}
    unsafe fn dealloc(&self,p:*mut u8,layout:Layout){LIVE.fetch_sub(layout.size() as isize,Ordering::SeqCst);unsafe{System.dealloc(p,layout)}}
    unsafe fn realloc(&self,p:*mut u8,layout:Layout,size:usize)->*mut u8{let q=unsafe{System.realloc(p,layout,size)};if !q.is_null(){LIVE.fetch_add(size as isize-layout.size() as isize,Ordering::SeqCst);if size==4093{SELECTED.fetch_add(1,Ordering::SeqCst);}}q}
}
#[global_allocator]static GLOBAL:Observer=Observer;
"#;
