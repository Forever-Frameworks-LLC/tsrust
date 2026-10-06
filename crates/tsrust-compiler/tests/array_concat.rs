mod support;
use tsrust_compiler::check;

fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    assert!(!rust.contains("unsafe"));
    let rust = rust.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("array-concat.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-concat-{level}"));
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
        let output = support::run(&mut std::process::Command::new(executable), 10);
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
    analysis
}

#[test]
fn concat_preserves_homogeneous_leaves_ranks_contents_and_variadic_order() {
    let cases = [
        ("int", "[1,2,3,4,5]", "v", vec!["1", "2", "3", "4", "5"]),
        ("number", "[1.25,-0.0,2.5]", "v", vec!["1.25", "-0", "2.5"]),
        ("boolean", "[true,false]", "v", vec!["true", "false"]),
        (
            "string",
            r#"["é","😀","a\u0000b","e\u0301"]"#,
            "v",
            vec!["é", "😀", "a\0b", "e\u{301}"],
        ),
        ("Row", "[{n:1},{n:2}]", "v.n", vec!["1", "2"]),
        ("int[]", "[[1,2],[],[3]]", "v.length", vec!["2", "0", "1"]),
        (
            "number[]",
            "[[1.25],[],[2.5,3.75]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "boolean[]",
            "[[true],[],[false,true]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "string[]",
            r#"[["é"],[],["😀","x"]]"#,
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "Row[]",
            "[[{n:1}],[],[{n:2},{n:3}]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "int[][]",
            "[[[1]],[],[[2],[3]]]",
            "v.length",
            vec!["1", "0", "2"],
        ),
        (
            "Node",
            "[{n:1,children:[]},{n:2,children:[{n:3,children:[]}]}]",
            "v.n",
            vec!["1", "2"],
        ),
    ];
    let mut source =
        String::from("type Row={n:int;};type Node={n:int;children:Node[];};function main(){");
    let mut expected = String::new();
    for (index, (element, values, observe, items)) in cases.iter().enumerate() {
        source.push_str(&format!("const values{index}:{element}[]={values};const output{index}=values{index}.concat(values{index},[],values{index});for(const v of output{index}){{console.log({observe});}}console.log(values{index}.length);"));
        for _ in 0..3 {
            for item in items {
                expected.push_str(&format!("{item}\n"));
            }
        }
        expected.push_str(&format!("{}\n", items.len()));
    }
    source.push_str("const scalar=[1].concat(2,[3,4],5,[]);for(const v of scalar){console.log(v);}const words=[\"é\"].concat(\"😀\",[\"x\"]);for(const v of words){console.log(v);}const records:Row[]=[{n:1}];const more=records.concat({n:2},[{n:3}]);for(const v of more){console.log(v.n);}const nested=[[1]].concat([[2,3]]);console.log(nested[1][1]);const copy=scalar.concat();copy[0]=9;console.log(scalar[0]);console.log(copy[0]);}");
    expected.push_str("1\n2\n3\n4\n5\né\n😀\nx\n1\n2\n3\n3\n1\n9\n");
    native(&source, &expected, "", "");
}

#[test]
fn concat_contextual_empty_receivers_sources_and_once_operand_effects_preserve_contracts() {
    native(
        r#"
    type Row={n:int;};type Box={n:int;values:int[];};
    function ints():int[]{return [].concat();}
    function numbers():number[]{return [].concat(1,[2,3]);}
    function nested():int[][]{return [].concat([[1],[]]);}
    function receiver():int[]{console.log("receiver");return [1];}
    function arrayArgument():int[]{console.log("array");return [2,3];}
    function scalarArgument():int{console.log("scalar");return 4;}
    function snapshot(box:Box):int{console.log("snapshot");return box.n;}
    function change(box:Box):int{console.log("change");box.n=9;return box.n;}
    function main(){
      const empty:int[]=[].concat();console.log(empty.length);console.log(ints().length);
      const inferred=[].concat([],1,[2]);console.log(inferred[0]);console.log(inferred[1]);
      const nums:number[]=[].concat(1,[2]);console.log(nums[0]);console.log(numbers()[2]);
      const rows:Row[]=[].concat({n:3},[{n:4}]);console.log(rows[1].n);console.log(nested()[1].length);
      const result=receiver().concat(arrayArgument(),scalarArgument());for(const v of result){console.log(v);}
      const box:Box={n:7,values:[5]};const source=[5];const copied=source.concat(snapshot(box),change(box));for(const v of copied){console.log(v);}console.log(box.n);const fieldCopy=box.values.concat([6]);console.log(fieldCopy[0]);
      const grid=[[6],[7]];const selected=grid[1].concat([8]);console.log(selected[0]);console.log(selected[1]);
      let trace=0;{const action=():int[]=>{trace=trace*10+1;return [3];};const actionResult=[2].concat(action(),action());console.log(actionResult.length);}console.log(trace);
      const names=["a","b"];const more=names.concat("c",["d"]);console.log(more[3]);
      const inferredNumbers=[].concat(1,[2],3.25);console.log(inferredNumbers[0]);console.log(inferredNumbers[2]);
      const literalNumbers:number[]=[1,2].concat(3.0);console.log(literalNumbers[0]);console.log(literalNumbers[2]);
      const reused=[1,2];const tail=[3];{const intBuild=():int[]=>{return reused.concat(tail,4);};console.log(intBuild()[3]);console.log(intBuild()[0]);}console.log(reused[0]);
      const borrowedWords=["owned".repeat(2)];const extra="tail".repeat(2);{const wordBuild=():string[]=>{return borrowedWords.concat(extra);};console.log(wordBuild()[1]);console.log(wordBuild()[0]);}console.log(extra);
    }
    "#,
        "0\n0\n1\n2\n1\n3\n4\n0\nreceiver\narray\nscalar\n1\n2\n3\n4\nsnapshot\nchange\n5\n7\n9\n9\n5\n7\n8\n3\n11\nd\n1\n3.25\n1\n3\n4\n1\n1\ntailtail\nownedowned\ntailtail\n",
        "",
        "",
    );
    let fixed=native("function main(){const values=[1,2,3];const output=values.concat([4]);values[0]=9;console.log(output[0]);console.log(values[0]);}","1\n9\n","","").emit_rust();
    assert!(fixed.contains("[i64; 3]"));

    let recursive = native(
        r#"
    type Node={n:int;text:string;children:Node[];};type Holder={node:Node;};
    function returned():Node{return {n:7,text:"returned",children:[]};}
    function main(){
      const base:Node[]=[{n:1,text:"é",children:[]},{n:2,text:"😀",children:[]}];
      const one:Node={n:3,text:"a\u0000b",children:[{n:30,text:"e\u0301",children:[]}]};
      const zero:Node={n:4,text:"zero",children:[]};
      const holder:Holder={node:{n:5,text:"field",children:[{n:50,text:"child",children:[]}]}};
      const indexed:Node[]=[{n:6,text:"index",children:[]}];
      const tail:Node[]=[{n:9,text:"array",children:[]}];
      const output=base.concat(one,zero,holder.node,indexed[0],returned(),{n:8,text:"literal",children:[]},tail);
      for(const node of output){console.log(node.n);console.log(node.text);console.log(node.children.length);for(const child of node.children){console.log(child.n);console.log(child.text);console.log(child.children.length);}}
      output[2].children[0].n=99;output[4].children[0].text="changed";
      console.log(one.children[0].n);console.log(holder.node.children[0].text);console.log(base.length);console.log(tail.length);
      const words=["left","right"];const word="owned".repeat(2);const wordCopy=words.concat(word);wordCopy[2]="changed";console.log(word);
    }
    "#,
        "1\né\n0\n2\n😀\n0\n3\na\0b\n1\n30\ne\u{301}\n0\n4\nzero\n0\n5\nfield\n1\n50\nchild\n0\n6\nindex\n0\n7\nreturned\n0\n8\nliteral\n0\n9\narray\n0\n30\nchild\n2\n1\nownedowned\n",
        "",
        "",
    ).emit_rust();
    // Read-only concat arrays still retain their fixed native representation.
    assert!(recursive.contains(": [Record0; 2] = ["));
    assert!(recursive.contains(": [Record0; 1] = ["));
}

#[test]
fn concat_rejects_rank_changes_nominal_mismatch_coercion_and_overlapping_operand_effects() {
    for (body, code) in [
        ("[].concat();", "array-type"),
        ("(1).concat(2);", "type"),
        ("true.concat();", "type"),
        (
            "const values=[[1]];const row=[2,3];values.concat(row);",
            "type",
        ),
        (
            "const values=[[[1]]];const matrix=[[2]];values.concat(matrix);",
            "type",
        ),
        ("[1].concat(true);", "type"),
        (
            "const values:number[]=[1];const n=2;values.concat(n);",
            "type",
        ),
        (
            "const values:number[]=[1];const ints=[2];values.concat(ints);",
            "type",
        ),
        ("const n=1;[].concat(n,2.5);", "type"),
        (
            "const ints=[1,2];const result:number[]=ints.concat(3.0);",
            "type",
        ),
        ("const n=2;const result:number[]=[1,n].concat(3.0);", "type"),
        ("const a:A[]=[{n:1}];const b:B={n:2};a.concat(b);", "type"),
        (
            "const values=[1];values.concat(changing(values));",
            "conflicting-access",
        ),
        (
            "const values=[1];const other=[2];values.concat(other,changing(other));",
            "conflicting-access",
        ),
        (
            "const words=[\"x\"];const box:Words={text:\"owned\"};words.concat(box.text,changeWords(box));",
            "conflicting-access",
        ),
    ] {
        support::reject(
            &format!(
                "type A={{n:int;}};type B={{n:int;}};type Words={{text:string;}};function changing(a:int[]):int[]{{a.push(2);return clone(a);}}function changeWords(a:Words):string{{a.text=\"changed\";return clone(a.text);}}function main(){{{body}}}"
            ),
            code,
        );
    }
    support::reject(
        "function main(){const moved=\"owned\";const x=moved.trim();const take=():string=>{return moved;};[\"x\"].concat(take(),take());}",
        "closure-consumed",
    );
}

#[test]
fn concat_fresh_owners_ieee_bits_recursive_copies_and_unwind_disposal_are_standard() {
    native(
        r#"
    type Row={text:string;values:int[];};type Node={text:string;children:Node[];};
    function words(values:string[],extra:string):string[]{return values.concat(extra,values);}
    function rows(values:int[][]):int[][]{return values.concat(values);}
    function records(values:Row[],extra:Row):Row[]{return values.concat(extra,values);}
    function forest(values:Node[]):Node[]{return values.concat(values);}
    function numbers(values:number[],extra:number):number[]{return values.concat(extra,values);}
    function failed(values:string[]):string[]{return values.concat("temporary".repeat(16),fail());}
    function fail():string[]{const text="temporary".repeat(16);const bad=[1];bad.remove(-1);return [text];}
    function main(){}
    "#,
        "",
        r#"
    use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicIsize,Ordering};
    static LIVE:AtomicIsize=AtomicIsize::new(0);struct Observer;
    unsafe impl GlobalAlloc for Observer{unsafe fn alloc(&self,l:Layout)->*mut u8{let p=unsafe{System.alloc(l)};if !p.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);}p}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(1,Ordering::SeqCst);}}
    #[global_allocator]static OBSERVER:Observer=Observer;
    "#,
        r#"
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let baseline=LIVE.load(Ordering::SeqCst);
    let words=Array::Heap(vec![String::Owned("source".repeat(16))]);let extra=String::Owned("extra".repeat(16));let live=LIVE.load(Ordering::SeqCst);let output=function0(&words,&extra);assert_eq!(output.len(),3);assert_ne!(output.get(0).unwrap().as_ptr(),words.get(0).unwrap().as_ptr());assert_ne!(output.get(1).unwrap().as_ptr(),extra.as_ptr());drop(output);assert_eq!(LIVE.load(Ordering::SeqCst),live);assert!(std::panic::catch_unwind(||function5(&words)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);drop(words);drop(extra);
    let rows=Array::Heap(vec![NumericArray::Heap(vec![1i64,2,3,4,5])]);let mut output=function1(&rows);*output.get_mut(0).unwrap().get_mut(0).unwrap()=9;assert_eq!(*rows.get(0).unwrap().get(0).unwrap(),1);assert_eq!(*output.get(1).unwrap().get(0).unwrap(),1);drop(output);drop(rows);
    let row=Record0{field0:String::Owned("text".repeat(16)),field1:NumericArray::Heap(vec![1i64;5])};let records=vec![row.clone()];let output=function2(&records,&row);assert_ne!(output.get(0).unwrap().field0.as_ptr(),records.get(0).unwrap().field0.as_ptr());drop(output);drop(records);drop(row);
    let root=Record1{field0:String::Owned("root".repeat(16)),field1:RecordArray1::from([Record1{field0:String::Owned("leaf".repeat(16)),field1:RecordArray1::from([])}])};let forest=RecordArray1::from([root]);let mut output=function3(&forest);output.get_mut(0).unwrap().field0=String::Borrowed("changed");assert_eq!(forest.get(0).unwrap().field0.as_ref(),"root".repeat(16));assert_eq!(output.get(1).unwrap().field0.as_ref(),"root".repeat(16));drop(output);drop(forest);
    for value in [-0.0f64,0.0,f64::INFINITY,f64::NEG_INFINITY,f64::from_bits(0x7ff8_0000_0000_0123)]{let source=vec![value];let output=function4(&source,value);assert_eq!(output.len(),3);for item in &output{assert_eq!(item.to_bits(),value.to_bits());}drop(output);drop(source);}
    assert_eq!(LIVE.load(Ordering::SeqCst),baseline);std::panic::set_hook(hook);
    "#,
    );
}
