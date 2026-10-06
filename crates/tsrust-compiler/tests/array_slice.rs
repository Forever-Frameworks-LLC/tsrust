mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, assertions: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("array-slice.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-slice-{level}"));
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
fn scalar_slices_normalize_all_arity_forms_and_extreme_bounds() {
    let analysis = native(
        r#"
        function all(values:int[]):int[]{return values.slice();}
        function tail(values:int[],start:int):int[]{return values.slice(start);}
        function part(values:int[],start:int,end:int):int[]{return values.slice(start,end);}
        function bools(values:boolean[],start:int,end:int):boolean[]{return values.slice(start,end);}
        function numbers(values:number[],start:int,end:int):number[]{return values.slice(start,end);}
        function main(){
            const values=[1,2,3,4,5];const full=values.slice();full.push(6);
            console.log(values.length);console.log(full.length);const tail=values.slice(-2);console.log(tail[0]);
            console.log(values.slice(3,1).length);console.log(values.slice(0,-1).length);
            console.log(values.slice(-9223372036854775808,9223372036854775807).length);
            console.log(values.slice(9223372036854775807).length);
            console.log(values.slice(0,-9223372036854775808).length);
            const empty:int[]=[];console.log(empty.slice().length);
            const flags=[true,false,true];const selectedFlags=flags.slice(-2);console.log(selectedFlags[0]);
            const decimals=[1.5,2.5];const selectedDecimals=decimals.slice();console.log(selectedDecimals[1]);
        }
        "#,
        "5\n6\n4\n0\n4\n5\n0\n0\n0\nfalse\n2.5\n",
        r#"
        fn bound(len:usize,index:i64)->usize {
            let len=len as i128;let index=index as i128;
            (if index<0{len+index}else{index}).clamp(0,len) as usize
        }
        let indices=[i64::MIN,-i64::MAX,-9,-3,-1,0,1,2,7,i64::MAX];
        for len in 0..=7 {
            let original=(0..len).map(|n|n as i64+10).collect::<Vec<_>>();
            let mut storages=vec![NumericArray::Heap(original.clone())];
            if len<=4 {
                let mut slots=[0i64;4];slots[..len].copy_from_slice(&original);
                storages.push(NumericArray::Small{len,values:slots});
            }
            let flags=(0..len).map(|n|n%2==0).collect::<Vec<_>>();
            let flags=NumericArray::Heap(flags);
            let decimals=(0..len).map(|n|if n==0{-0.0}else if n==1{f64::NAN}else{n as f64+0.5}).collect::<Vec<_>>();
            for values in storages {
                assert_eq!(function0(&values).as_slice(),original.as_slice());
                for start in indices {
                    let begin=bound(len,start);
                    assert_eq!(function1(&values,start).as_slice(),&original[begin..]);
                    for end in indices {
                        let finish=bound(len,end).max(begin);
                        assert_eq!(function2(&values,start,end).as_slice(),&original[begin..finish]);
                        assert_eq!(function3(&flags,start,end).as_slice(),&flags.as_slice()[begin..finish]);
                        let actual=function4(&decimals,start,end);
                        assert_eq!(actual.len(),finish-begin);
                        for (a,b) in actual.iter().zip(&decimals[begin..finish]){assert_eq!(a.to_bits(),b.to_bits());}
                    }
                }
                assert_eq!(values.as_slice(),original.as_slice());
            }
        }
        "#,
    );
    for parameter in [
        "all.values",
        "tail.values",
        "part.values",
        "bools.values",
        "numbers.values",
    ] {
        assert_eq!(analysis.contracts().get(parameter), Some(&Mode::Read));
    }
}

#[test]
fn owned_strings_nested_arrays_and_record_forests_slice_independent_values() {
    native(
        r#"
        type Leaf={name:string;values:int[];};
        type Node={name:string;children:Node[];};
        type Small={n:int;};
        function rows(values:Leaf[],start:int,end:int):Leaf[]{return values.slice(start,end);}
        function forest(values:Node[]):Node[]{return values.slice();}
        function main(){
            const text=" First ".trim();const words=[text,"second"];const copied=words.slice();
            words[0]="changed";copied.push("third");console.log(copied[0]);console.log(words.length);console.log(copied.length);
            const ints:int[][]=[[1,2],[3]];const saved=ints.slice(0,1);ints[0][0]=9;console.log(saved[0][0]);
            const triples:int[][][]=[[[4]]];const tripleCopy=triples.slice();triples[0][0][0]=8;console.log(tripleCopy[0][0][0]);
            const flags:boolean[][]=[[true]];const flagCopy=flags.slice();flags[0][0]=false;console.log(flagCopy[0][0]);
            const decimals:number[][]=[[1.5]];const decimalCopy=decimals.slice();decimals[0][0]=3.5;console.log(decimalCopy[0][0]);
            const strings:string[][]=[["left"]];const stringCopy=strings.slice();strings[0][0]="right";console.log(stringCopy[0][0]);
            const values:Leaf[]=[{name:" ROW ".trim().toLowerCase(),values:[1,2]}];const rowCopy=rows(values,0,1);
            values[0].name="new";values[0].values[0]=7;console.log(rowCopy[0].name);console.log(rowCopy[0].values[0]);
            const grid:Leaf[][]=[[{name:"cell",values:[3]}]];const gridCopy=grid.slice();grid[0][0].values[0]=9;console.log(gridCopy[0][0].values[0]);
            const small:Small[]=[{n:1},{n:2}];const smallCopy=small.slice();small[0].n=5;console.log(smallCopy[0].n);
            const nodes:Node[]=[{name:"root",children:[{name:"child",children:[]}]}];const nodeCopy=forest(nodes);
            nodes[0].children[0].name="replaced";console.log(nodeCopy[0].children[0].name);
        }
        "#,
        "First\n2\n3\n1\n4\ntrue\n1.5\nleft\nrow\n1\n3\n1\nchild\n",
        r#"
        let owned=String::Owned("owned".repeat(32));
        let values=Array::Heap(vec![owned]);let copied=values.tsr_slice(0,1);
        let a=values.get(0).unwrap();let b=copied.get(0).unwrap();
        assert_eq!(a,b);assert_ne!(a.as_ptr(),b.as_ptr());drop(values);assert_eq!(copied.get(0).unwrap().len(),160);
        let mut root=Record1{field0:String::Owned("node".into()),field1:RecordArray1::from([])};
        for _ in 0..4096 {root=Record1{field0:String::Owned("node".into()),field1:RecordArray1::from([root])};}
        let source=RecordArray1::from([root]);let mut copied=function1(&source);
        assert_ne!(source.as_ptr(),copied.as_ptr());
        copied[0].field0=String::Owned("changed".into());assert_eq!(source[0].field0.as_ref(),"node");
        drop(source);let mut count=0;let mut cursor=&copied[0];
        while !cursor.field1.is_empty(){count+=1;cursor=&cursor.field1[0];}
        assert_eq!(count,4096);drop(copied);
        "#,
    );
}

#[test]
fn slice_receiver_and_bounds_run_once_in_order_with_capture_and_fixed_storage_contracts() {
    let analysis = native(
        r#"
        type Box={values:int[];n:int;};
        type Grid={values:int[][];n:int;};
        function receiver(box:Box):int[]{console.log("receiver");return clone(box.values);}
        function start(box:Box):int{console.log("start");box.values.push(4);return 1;}
        function end(box:Box):int{console.log("end");box.values.push(5);return box.values.length;}
        function index(counter:Box):int{console.log("index");counter.n=counter.n+1;return 0;}
        function bound():int{console.log("bound");return 0;}
        function saved(values:int[]):()=>int[]{return ():int[]=>{return values.slice();};}
        function main(){
            const box:Box={values:[1,2,3],n:0};const result=receiver(box).slice(start(box),end(box));
            console.log(result.length);console.log(result[0]);console.log(result[1]);console.log(box.values.length);
            const grid:Grid={values:[[4,5]],n:0};const chosen=grid.values[index(box)].slice(bound(),1);
            console.log(chosen[0]);console.log(box.n);
            const action=saved([7,8]);const first=action();const second=action();first[0]=9;console.log(second[0]);
            const values=[1,2];
            {
                const change=():int[]=>{values[0]=values[0]+1;return values.slice();};
                const before=change();const after=change();console.log(before[0]);console.log(after[0]);
            }
            console.log(values[0]);values.push(3);console.log(values.length);
        }
        "#,
        "receiver\nstart\nend\n2\n2\n3\n5\nindex\nbound\n4\n1\n7\n2\n3\n3\n3\n",
        "",
    );
    assert_eq!(analysis.contracts().get("receiver.box"), Some(&Mode::Read));
    assert_eq!(analysis.contracts().get("start.box"), Some(&Mode::Mutate));
    assert_eq!(analysis.contracts().get("end.box"), Some(&Mode::Mutate));
    assert_eq!(
        analysis.contracts().get("index.counter"),
        Some(&Mode::Mutate)
    );
    let fixed = native(
        r#"
        type Point={x:int;};
        function main(){
            const values=[1,2,3];const copy=values.slice(1);copy.push(4);console.log(copy[0]);console.log(copy.length);console.log(values.length);
            const words=["a","b"];const tail=words.slice(1);tail.push("c");console.log(tail[0]);console.log(words[0]);
            const points:Point[]=[{x:1},{x:2}];const selected=points.slice(-1);selected.push({x:3});console.log(selected[0].x);console.log(points[0].x);
        }
        "#,
        "2\n3\n3\nb\na\n2\n1\n",
        "",
    );
    let emitted = fixed.emit_rust();
    assert!(emitted.contains(": [i64; 3] = ["));
    assert!(emitted.contains("LiteralStringArray"));
    assert!(emitted.contains("[Record0; 2]"));
}

#[test]
fn array_slice_diagnostics_loans_temporaries_and_string_overload_remain_sound() {
    native(
        r#"
        function empty():int[]{return [];}
        function main(){
            const selected=[1,2,3].slice(1);console.log(selected[0]);console.log(empty().slice().length);
            console.log("AéZ".slice(1,3));console.log("abc".slice(2,1).length);
        }
        "#,
        "2\n0\né\n0\n",
        r#"
        let text=String::Borrowed("é");let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        assert!(std::panic::catch_unwind(||tsr_slice(&text,1,None)).is_err());std::panic::set_hook(hook);
        "#,
    );
    for (source, code) in [
        (
            "type Grid={values:int[][];n:int;};function index(grid:Grid):int{grid.n=grid.n+1;return 0;}function main(){const grid:Grid={values:[[1]],n:0};grid.values[0].slice(index(grid));}",
            "conflicting-access",
        ),
        (
            "function main(){const values=[1];values.slice(0,1,2);}",
            "arity",
        ),
        (
            "function main(){const values=[1];values.slice(true);}",
            "type",
        ),
        (
            "function main(){const values=[1];values.slice(0,1.5);}",
            "type",
        ),
        (
            "function main(){const values=[1];values.charAt(0);}",
            "type",
        ),
        ("function main(){const value=1;value.slice();}", "type"),
        ("function main(){[].slice();}", "array-type"),
        ("function main(){[1,2,3].slice(1)[false];}", "type"),
        (
            "function main(){const values=[1];const result:int=values.slice();}",
            "type",
        ),
        (
            "function grow(values:int[]):int{values.push(2);return 0;}function main(){const values=[1];values.slice(grow(values));}",
            "conflicting-access",
        ),
        (
            "function grow(values:int[]):int{values.push(2);return 0;}function main(){const values=[1];values.slice(0,grow(values));}",
            "conflicting-access",
        ),
        (
            "type Box={values:int[];};function mutate(box:Box):int{box.values.push(2);return 0;}function main(){const box:Box={values:[1]};box.values.slice(mutate(box));}",
            "conflicting-access",
        ),
        (
            "function main(){const values=[1];const moved=values;values.slice();}",
            "transferred",
        ),
        (
            "function main(){const values=[1];const change=():int=>{values[0]=2;return 0;};values.slice();}",
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
}
