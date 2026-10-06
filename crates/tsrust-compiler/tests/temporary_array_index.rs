mod support;
use tsrust_compiler::{Mode, check};

fn native(
    source: &str,
    expected: &str,
    definitions: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\n{definitions}\nfn main() {{ source_main(); {assertions} }}\n");
    assert!(!analysis.emit_rust().contains("unsafe"));
    let directory = support::TempDirectory::new();
    let path = directory.0.join("temporary-index.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("temporary-index-{level}"));
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

#[test]
fn temporary_indices_cover_literals_return_values_all_leaf_types_ranks_and_record_fields() {
    native(
        r#"
        type Item={name:string;n:int;};
        type Row={value:number;};
        type Box={items:Item[];};
        function ints():int[]{return [1,2,3];}
        function flags():boolean[]{return [false,true];}
        function numbers():number[]{return [-0.0,1.5];}
        function words():string[]{return ["ß".toUpperCase(),"é"] ;}
        function rows():int[][]{return [[4,5],[6]];}
        function triples():number[][][]{return [[[2.5]]];}
        function items():Item[]{return [{name:"name".toUpperCase(),n:7}];}
        function boxed():Box{return {items:[{name:"boxed",n:8}]};}
        function echo(item:Item):Item{return item;}
        function main(){
            console.log([1,2,3][1]);console.log([false,true][1]);
            const contextual:number=[1,2][0];console.log(contextual);
            const item:Item=[{name:"literal",n:9}][0];console.log(item.name);
            const row:Row=[[{value:1}]][0][0];console.log(row.value);
            const typedRows:Row[]=[[[{value:2}]]][0][0];console.log(typedRows[0].value);
            const scalar:number=[[[3]]][0][0][0];console.log(scalar);
            const decimals:number[]=[[[4,5]]][0][0];console.log(decimals[1]);
            console.log(ints()[2]);console.log(flags()[1]);console.log(numbers()[1]);
            console.log(1.0/numbers()[0]);console.log(words()[0]);
            console.log(rows()[0][1]);console.log(triples()[0][0][0]);
            const selected=items()[0];console.log(selected.name);console.log(selected.n);
            console.log(items()[0].name);console.log(items()[0].n);
            const moved=echo(items()[0]);console.log(moved.name);
            console.log(boxed().items[0].name);
            const values=[7,8,9];console.log(values.slice(1)[0]);console.log(values[0]);
            console.log(["abcd"][0].slice(1));console.log([[true]][0][0]);
            console.log([["owned".toUpperCase()]][0][0]);
            const nested:Item[][]=[[{name:"nested",n:10}]];
            console.log(nested.slice()[0][0].name);
        }
        "#,
        "2\ntrue\n1\nliteral\n1\n2\n3\n5\n3\ntrue\n1.5\n-Infinity\nSS\n5\n2.5\nNAME\n7\nNAME\n7\nNAME\nboxed\n8\n7\nbcd\ntrue\nOWNED\nnested\n",
        "",
        "",
    );
}

#[test]
fn temporary_receiver_index_effects_borrowed_calls_and_captures_evaluate_once() {
    let analysis = native(
        r#"
        type Box={values:int[];count:int;};
        function receiver(box:Box):int[]{console.log("receiver");return clone(box.values);}
        function index(box:Box):int{console.log("index");box.values.push(4);box.count=box.count+1;return 1;}
        function inspect(values:int[],other:int):int{return values[0]+other;}
        function append(values:int[]):int{values.push(4);return values.length-1;}
        function owned(values:int[]):()=>int{return ():int=>{return clone(values)[0];};}
        function main(){
            const box:Box={values:[1,2,3],count:0};console.log(receiver(box)[index(box)]);console.log(box.count);console.log(box.values.length);
            const values=[[7],[8]];console.log(inspect(values.slice()[0],values[1][0]));console.log(values[0][0]);
            const action=owned([9]);console.log(action());console.log(action());
            const get=():string[]=>{console.log("closure");return [" hello ".trim()];};console.log(get()[0]);
            const rows=[[1,2],[3]];rows.slice()[0].push(9);rows.slice()[0].reverse();console.log(rows[0][0]);console.log(rows[0].length);
            console.log(rows.slice()[0].remove(0));console.log(rows[0][0]);
            let current=1;
            {
                const increment=():int=>{current=current+1;return [current][0];};
                console.log(increment());console.log(increment());
            }
            console.log(current);
            const direct=[1];console.log(direct[append(direct)]);console.log(direct.length);
        }
        "#,
        "receiver\nindex\n2\n1\n4\n15\n7\n9\n9\nclosure\nhello\n1\n2\n1\n1\n2\n3\n3\n4\n2\n",
        "",
        "",
    );
    assert_eq!(analysis.contracts().get("receiver.box"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("index.box"), Some(&Mode::Mutate));
    assert_eq!(
        analysis.contracts().get("inspect.values"),
        Some(&Mode::Read)
    );
    assert_eq!(
        analysis.contracts().get("owned.values"),
        Some(&Mode::Consume)
    );
}

#[test]
fn owning_temporary_selection_transfers_buffers_and_disposes_unselected_and_panicking_owners() {
    native(
        r#"
        type Node={text:string;children:Node[];};
        function strings(values:string[]):string[]{return values;}
        function pickString(values:string[],index:int):string{return strings(values)[index];}
        function rows(values:int[][]):int[][]{return values;}
        function pickRow(values:int[][],index:int):int[]{return rows(values)[index];}
        function nodes(values:Node[]):Node[]{return values;}
        function pickNode(values:Node[],index:int):Node{return nodes(values)[index];}
        function pickText(values:Node[],index:int):string{return nodes(values)[index].text;}
        function length(values:string[]):int{return strings(values)[0].length;}
        function main(){console.log(pickString(["a","b"],1));console.log(pickRow([[1],[2]],1)[0]);}
        "#,
        "b\n2\n",
        r#"
        use std::alloc::{GlobalAlloc,Layout,System};
        use std::sync::atomic::{AtomicIsize,Ordering};
        static LIVE:AtomicIsize=AtomicIsize::new(0);
        struct Allocations;
        unsafe impl GlobalAlloc for Allocations {
            unsafe fn alloc(&self,layout:Layout)->*mut u8 {
                let pointer=unsafe{System.alloc(layout)};
                if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);}pointer
            }
            unsafe fn dealloc(&self,pointer:*mut u8,layout:Layout) {
                unsafe{System.dealloc(pointer,layout)};LIVE.fetch_sub(1,Ordering::SeqCst);
            }
        }
        #[global_allocator]static ALLOCATOR:Allocations=Allocations;
        "#,
        r#"
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        let baseline=LIVE.load(Ordering::SeqCst);
        let make=||Array::Heap(vec![String::Owned("left".repeat(32)),String::Owned("selected".repeat(32)),String::Owned("right".repeat(32))]);
        let input=make();let pointer=input.get(1).unwrap().as_ptr();let selected=function1(input,1);
        assert_eq!(pointer,selected.as_ptr());assert_eq!(LIVE.load(Ordering::SeqCst),baseline+1);
        drop(selected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        for index in [i64::MIN,-1,3,i64::MAX] {
            let input=make();assert!(std::panic::catch_unwind(||function1(input,index)).is_err());
            assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        }
        let input=Array::Heap(vec![NumericArray::Heap(vec![1,2]),NumericArray::Heap(vec![3,4])]);
        let pointer=input.get(1).unwrap().as_slice().as_ptr();let selected=function3(input,1);
        assert_eq!(pointer,selected.as_slice().as_ptr());assert_eq!(selected.as_slice(),&[3,4]);
        drop(selected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let leaf=||Record0{field0:String::Owned("leaf".into()),field1:RecordArray0::from([])};
        let mut root=leaf();for _ in 0..4096{root=Record0{field0:String::Owned("node".into()),field1:RecordArray0::from([root])};}
        let input=RecordArray0::from([leaf(),root,leaf()]);let pointer=input[1].field0.as_ptr();let selected=function5(input,1);
        assert_eq!(pointer,selected.field0.as_ptr());let mut depth=0;let mut cursor=&selected;
        while !cursor.field1.is_empty(){depth+=1;cursor=&cursor.field1[0];}assert_eq!(depth,4096);
        drop(selected);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let branch=Record0{field0:String::Owned("selected text".into()),field1:RecordArray0::from([leaf(),leaf()])};
        let input=RecordArray0::from([branch,leaf()]);let pointer=input[0].field0.as_ptr();
        let selected=function6(input,0);assert_eq!(selected.as_ptr(),pointer);
        assert_eq!(LIVE.load(Ordering::SeqCst),baseline+1);drop(selected);
        assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        assert_eq!(function7(make()),128);assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        let input=RecordArray0::from([leaf(),leaf()]);assert!(std::panic::catch_unwind(||function5(input,2)).is_err());
        assert_eq!(LIVE.load(Ordering::SeqCst),baseline);
        std::panic::set_hook(hook);
        "#,
    );
}

#[test]
fn temporary_index_failures_keep_receiver_index_bounds_order_and_named_storage_restrictions() {
    // Indexed const bindings borrow their source; this restriction was lifted in gate 125.
    for source in [
        "function main(){const values=[\"owned\"];const item=values[0];}",
        "type Item={name:string;};function main(){const values:Item[]=[{name:\"owned\"}];const item=values[0].name;}",
        "function main(){const values=[[1]];const nested=values[0];}",
    ] {
        assert!(check(source).is_ok(), "{source}");
    }
    for (source, code) in [
        ("function main(){console.log([1][true]);}", "type"),
        ("function main(){console.log([1][1.5]);}", "type"),
        ("function main(){console.log(1[0]);}", "type"),
        ("function main(){[][0];}", "array-type"),
        (
            "function mutate(values:int[][]):int{values.push([2]);return 0;}function main(){const values=[[1]];values[0][mutate(values)];}",
            "conflicting-access",
        ),
        (
            "function main(){const values=[1];const moved=values;clone(values)[0];}",
            "transferred",
        ),
        (
            "function main(){const values=[1];const change=():int=>{values[0]=2;return 0;};clone(values)[0];}",
            "capture-conflict",
        ),
        ("function main(){[1][0]=2;}", "unsupported"),
        (
            "function make():int[][]{return [[1]];}function main(){make()[0][0]=2;}",
            "unsupported",
        ),
        (
            "type Item={name:string;};function make():Item[]{return [{name:\"x\"}];}function main(){make()[0].name=\"changed\";}",
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
    for (source, expected) in [
        (
            "function make():int[]{console.log(\"receiver\");return [1];}function index():int{console.log(\"index\");return -1;}function main(){console.log(make()[index()]);}",
            "receiver\nindex\n",
        ),
        (
            "function make():int[]{console.log(\"receiver\");const values=[1];const failure=values.remove(-1);return values;}function index():int{console.log(\"index\");return 0;}function main(){console.log(make()[index()]);}",
            "receiver\n",
        ),
        (
            "function make():int[][]{console.log(\"receiver\");return [[1]];}function first():int{console.log(\"first\");return 9;}function second():int{console.log(\"second\");return 0;}function main(){console.log(make()[first()][second()]);}",
            "receiver\nfirst\n",
        ),
    ] {
        let analysis =
            check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
        let directory = support::TempDirectory::new();
        let path = directory.0.join("temporary-index-failure.rs");
        std::fs::write(&path, analysis.emit_rust()).unwrap();
        for level in ["0", "3"] {
            let executable = directory.0.join(format!("failure-{level}"));
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
            assert!(!output.status.success());
            assert_eq!(output.stdout, expected.as_bytes());
            assert!(
                String::from_utf8_lossy(&output.stderr)
                    .contains("array index out of bounds at source bytes")
            );
        }
    }
}
