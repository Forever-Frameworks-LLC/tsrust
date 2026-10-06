// Prepared Gate56 fixtures only. No source parsing or native execution yet.
mod support;
use tsrust_compiler::{Mode, check};

fn native(source: &str, expected: &str, assertions: &str) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    assert!(!rust.contains("unsafe"));
    let directory = support::TempDirectory::new();
    let path = directory.0.join("fill-copy-within.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("native-{level}"));
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
fn range_mutation_all_leaves_ranks_extreme_bounds_live_prefixes_and_number_bits() {
    let source = r#"
        type Node = { code: int; text: string; children: Node[]; };
        function fillInts(values:int[],value:int,start:int,end:int){values.fill(value,start,end);}
        function copyInts(values:int[],target:int,start:int,end:int){values.copyWithin(target,start,end);}
        function fillNumbers(values:number[],value:number,start:int,end:int){values.fill(value,start,end);}
        function copyNumbers(values:number[],target:int,start:int,end:int){values.copyWithin(target,start,end);}
        function fillBools(values:boolean[],value:boolean,start:int,end:int){values.fill(value,start,end);}
        function copyBools(values:boolean[],target:int,start:int,end:int){values.copyWithin(target,start,end);}
        function fillStrings(values:string[],value:string,start:int,end:int){values.fill(value,start,end);}
        function copyStrings(values:string[],target:int,start:int,end:int){values.copyWithin(target,start,end);}
        function fillRows(values:int[][],value:int[],start:int,end:int){values.fill(value,start,end);}
        function copyRows(values:int[][],target:int,start:int,end:int){values.copyWithin(target,start,end);}
        function fillNodes(values:Node[],value:Node,start:int,end:int){values.fill(value,start,end);}
        function copyNodes(values:Node[],target:int,start:int,end:int){values.copyWithin(target,start,end);}
        function main(){
            const numbers:number[][]=[[1.25],[],[-0.0]];numbers.fill([2.5],-2);numbers.copyWithin(0,1);for(const row of numbers){console.log(row.length);for(const value of row){console.log(value);}}
            const bools:boolean[][]=[[false],[],[true]];bools.fill([true,false],1,2);bools.copyWithin(2,0,1);for(const row of bools){console.log(row.length);for(const value of row){console.log(value);}}
            const strings:string[][]=[["é"],[],["😀"]];strings.fill(["a\u0000b"],-2);strings.copyWithin(0,1);for(const row of strings){console.log(row.length);for(const value of row){console.log(value);}}
            const nodes:Node[][]=[[{code:1,text:"a",children:[]}],[],[{code:3,text:"c",children:[]}]];nodes.fill([{code:2,text:"é",children:[{code:9,text:"leaf",children:[]}]}],1);nodes.copyWithin(0,1,2);for(const row of nodes){console.log(row.length);for(const value of row){console.log(value.code);console.log(value.text);console.log(value.children.length);}}
            const deep:int[][][]=[[[1]],[[2]],[[3]]];deep.fill([[8]],0,2);deep.copyWithin(2,0,1);deep[0][0].push(9);console.log(deep[1][0].length);console.log(deep[2][0].length);
        }
    "#;
    let expected = "1\n2.5\n1\n2.5\n1\n2.5\n1\nfalse\n2\ntrue\nfalse\n1\nfalse\n1\na\0b\n1\na\0b\n1\na\0b\n1\n2\né\n1\n1\n2\né\n1\n1\n2\né\n1\n1\n1\n";
    let analysis = native(
        source,
        expected,
        r#"
        fn relative(len:usize,value:i64)->usize { let x=if value<0 {len as i128+value as i128}else{value as i128};x.clamp(0,len as i128)as usize }
        let bounds=[i64::MIN,-99,-5,-1,0,1,3,5,99,i64::MAX];
        for len in 0..=5 {
            for &start in &bounds { for &end in &bounds {
                let mut actual=NumericArray::from([]);for value in 0..len{actual.push(value as i64);}
                let capacity=match &actual{NumericArray::Small{..}=>4,NumericArray::Heap(v)=>v.capacity()};let mut expected=(0..len as i64).collect::<Vec<_>>();let a=relative(len,start);let z=relative(len,end).max(a);for slot in &mut expected[a..z]{*slot=77;}
                function0(&mut actual,77,start,end);assert_eq!(actual.iter().copied().collect::<Vec<_>>(),expected);assert_eq!(actual.len(),len);
                match &actual{NumericArray::Small{len,values}=>assert!(values[*len..].iter().all(|v|*v==0)),NumericArray::Heap(v)=>assert_eq!(v.capacity(),capacity)};
            }}
            for &target in &bounds{for &start in &bounds{for &end in &bounds{
                let mut actual=NumericArray::from([]);for value in 0..len{actual.push(value as i64);}
                let before=(0..len as i64).collect::<Vec<_>>();let mut expected=before.clone();let t=relative(len,target);let a=relative(len,start);let z=relative(len,end).max(a);let count=(z-a).min(len-t);for offset in 0..count{expected[t+offset]=before[a+offset];}
                function1(&mut actual,target,start,end);assert_eq!(actual.iter().copied().collect::<Vec<_>>(),expected);assert_eq!(actual.len(),len);
                match &actual{NumericArray::Small{len,values}=>assert!(values[*len..].iter().all(|v|*v==0)),NumericArray::Heap(_)=>{}};
            }}}
        }
        let payload=f64::from_bits(0x7ff8_0000_0000_1234);let mut values=vec![0.0,-0.0,1.25,payload];function2(&mut values,-0.0,1,3);assert_eq!(values.iter().map(|n|n.to_bits()).collect::<Vec<_>>(),vec![0,1u64<<63,1u64<<63,payload.to_bits()]);function3(&mut values,0,1,4);assert_eq!(values.iter().map(|n|n.to_bits()).collect::<Vec<_>>(),vec![1u64<<63,1u64<<63,payload.to_bits(),payload.to_bits()]);function2(&mut values,payload,0,4);assert!(values.iter().all(|v|v.to_bits()==payload.to_bits()));
        let mut flags=NumericArray::from([false,true,false]);function4(&mut flags,true,0,2);function5(&mut flags,1,0,2);assert_eq!(flags.iter().copied().collect::<Vec<_>>(),vec![true,true,true]);
        for len in [0,1,3,4,5] {let mut strings=Array::from([]);for index in 0..len{strings.push(String::Owned(index.to_string()));}let capacity=match &strings{Array::Small{..}=>4,Array::Heap(v)=>v.capacity()};function6(&mut strings,String::Owned("é\0😀".into()),0,len as i64);function7(&mut strings,1,0,len as i64);assert!(strings.iter().all(|s|s.as_ref()=="é\0😀"));assert_eq!(strings.len(),len);match &strings{Array::Small{len,values}=>{assert!(values[..*len].iter().all(Option::is_some));assert!(values[*len..].iter().all(Option::is_none));},Array::Heap(v)=>assert_eq!(v.capacity(),capacity)}}
        let mut rows=Array::from([NumericArray::from([1]),NumericArray::from([2]),NumericArray::from([3])]);function8(&mut rows,NumericArray::from([8,9]),0,3);rows.get_mut(0).unwrap().push(10);assert_eq!(rows.get(1).unwrap().len(),2);function9(&mut rows,1,0,2);rows.get_mut(1).unwrap().push(11);assert_eq!(rows.get(0).unwrap().len(),3);assert_eq!(rows.get(2).unwrap().len(),2);
        let leaf=||Record0{field0:9,field1:String::Owned("leaf".into()),field2:RecordArray0::from([])};let node=||Record0{field0:1,field1:String::Owned("é\0😀".into()),field2:RecordArray0::from([leaf()])};let mut nodes=RecordArray0::from([node(),node(),node()]);function10(&mut nodes,node(),0,3);nodes.get_mut(0).unwrap().field2.get_mut(0).unwrap().field0=7;assert_eq!(nodes.get(1).unwrap().field2.get(0).unwrap().field0,9);function11(&mut nodes,1,0,2);nodes.get_mut(1).unwrap().field2.get_mut(0).unwrap().field0=6;assert_eq!(nodes.get(0).unwrap().field2.get(0).unwrap().field0,7);assert_eq!(nodes.get(2).unwrap().field2.get(0).unwrap().field0,9);
    "#,
    );
    assert_eq!(
        analysis.contracts().get("fillInts.values"),
        Some(&Mode::Mutate)
    );
    assert_eq!(
        analysis.contracts().get("fillStrings.value"),
        Some(&Mode::Consume)
    );
}

#[test]
fn range_mutation_argument_order_final_length_defaults_clones_and_selected_receivers() {
    let source = r#"
        type Probe={state:int;};type Holder={values:int[];};
        function mark(value:int):int{console.log(value);return value;}
        function grow(values:int[],value:int):int{console.log(value);values.push(value);return -2;}
        function ending(values:int[],value:int):int{console.log(value);values.push(value);return -1;}
        function choose(probe:Probe):int{console.log("choose");probe.state=probe.state+1;return 0;}
        function holder():Holder{console.log("holder");return {values:[1,2,3]};}
        function mutate(values:int[]){values.fill(8);values.copyWithin(1,0);}
        function apply(action:(values:Mutable<int[]>)=>void,values:int[]){action(values);}
        function main(){
            const values=[1,2];values.fill(mark(5),grow(values,9),ending(values,8));console.log(values.length);for(const value of values){console.log(value);}
            const rows=[[1,2,3]];let probe:Probe={state:0};rows[choose(probe)].copyWithin(mark(1),mark(0),mark(2));for(const value of rows[0]){console.log(value);}console.log(probe.state);
            holder().values.fill(mark(4),mark(1));
            const callback=mutate;apply(callback,values);for(const value of values){console.log(value);}
            const owned=[1,2];const nested:int[][]=[[],[]];nested.fill(clone(owned));nested[0].push(3);console.log(nested[1].length);console.log(owned.length);
            const empty:string[]=[];const text="é".toUpperCase();empty.fill(text,9,-9);console.log(empty.length);
            const single=[7];single.copyWithin(0,0);single.fill(9);console.log(single[0]);
        }
    "#;
    native(
        source,
        "5\n9\n8\n4\n1\n2\n5\n8\nchoose\n1\n0\n2\n1\n1\n2\n1\nholder\n4\n1\n8\n8\n8\n8\n2\n2\n0\n9\n",
        "",
    );
    let failure = r#"function index():int{console.log("index");return 5;}function value():int{console.log("later");return 8;}function main(){const rows=[[1,2]];rows[index()].fill(value());}"#;
    let rust = check(failure).unwrap().emit_rust();
    let directory = support::TempDirectory::new();
    let path = directory.0.join("failure.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("failure-{level}"));
        support::success(&support::run(
            support::rustc()
                .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                .arg(&path)
                .arg("-o")
                .arg(&executable),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(executable), 10);
        assert!(!output.status.success());
        assert_eq!(output.stdout, b"index\n");
        assert!(
            String::from_utf8_lossy(&output.stderr)
                .contains("array index out of bounds at source bytes")
        );
    }
}

#[test]
fn range_mutation_type_access_ownership_diagnostics_and_fixed_abi_proof() {
    for (body, code) in [
        ("const a=[1];a.fill();", "arity"),
        ("const a=[1];a.fill(1,0,1,2);", "arity"),
        ("const a=[1];a.copyWithin(0);", "arity"),
        ("const a=[1];a.copyWithin(0,0,1,2);", "arity"),
        ("const a=[1];a.fill(true);", "type"),
        ("const a=[1];a.fill(2,0.0);", "type"),
        ("const a=[1];a.copyWithin(true,0);", "type"),
        ("const a:number[]=[1.0];const b=2;a.fill(b);", "type"),
        ("const a=[[1]];a.fill(1);", "type"),
        ("const a=[1];const b=a;a.fill(2);", "transferred"),
        (
            "const a=[\"x\".toUpperCase()];const b=\"y\".toUpperCase();a.fill(b);console.log(b);",
            "transferred",
        ),
        ("const a=[\"x\"];a.fill(a[0]);", "borrowed-element"),
        (
            "const a=[[1]];for(const row of a){row.fill(8);}",
            "borrowed-element",
        ),
        ("const a=[1];const result=a.fill(2);", "type"),
    ] {
        let source = format!("function main(){{{body}}}");
        assert_eq!(check(&source).err().unwrap().code, code, "{source}");
    }
    for source in [
        "function edit(values:int[]){values.fill(1);}function use(action:(values:ReadOnly<int[]>)=>void){}function main(){use(edit);}",
        "type Holder={items:int[];};function value(root:Holder):int{root.items.push(3);return 1;}function main(){let root:Holder={items:[1,2]};root.items.fill(value(root));}",
        "function change(values:int[][]):int{values.push([4]);return 0;}function main(){const a=[[1,2]];a[0].copyWithin(change(a),0);}",
        "type Left={n:int;};type Right={n:int;};function main(){const a:Left[]=[{n:1}];const b:Right={n:2};a.fill(b);}",
    ] {
        assert!(check(source).is_err(), "{source}");
    }
    let source = "function main(){const ints=[1,2,3];ints.fill(8);ints.copyWithin(0,1);const numbers:number[]=[1.25,2.5,-0.0];numbers.fill(-0.0);numbers.copyWithin(1,0);const flags=[true,false];flags.fill(false);flags.copyWithin(1,0);const words=[\"a\",\"b\"];words.fill(\"c\");console.log(words[0]);}";
    let analysis = native(source, "c\n", "");
    let rust = analysis.emit_rust();
    assert!(rust.contains("[i64; 3]"));
    assert!(rust.contains("[f64; 3]"));
    assert!(rust.contains("[bool; 2]"));
    assert!(!rust.contains("LiteralStringArray"));
    let record = "type Row={n:int;};function main(){const values:Row[]=[{n:1},{n:2}];const replacement:Row={n:9};values.fill(replacement);console.log(values[0].n);}";
    native(record, "9\n", "");
}

#[test]
fn compact_owned_snapshots_clone_once_drop_replaced_values_preserve_capacity_and_unwind() {
    let source = "function fill(values:string[],value:string){values.fill(value);}function copy(values:string[]){values.copyWithin(1,0);}function main(){}";
    native(
        source,
        "",
        r#"
        use std::sync::{Arc,atomic::{AtomicUsize,Ordering}};
        #[derive(Default)]struct Counts{live:AtomicUsize,clones:AtomicUsize,fail_at:AtomicUsize}
        struct Witness{n:usize,counts:Arc<Counts>}
        impl Witness{fn new(n:usize,counts:&Arc<Counts>)->Self{counts.live.fetch_add(1,Ordering::SeqCst);Self{n,counts:counts.clone()}}}
        impl Clone for Witness{fn clone(&self)->Self{let call=self.counts.clones.fetch_add(1,Ordering::SeqCst)+1;assert!(call!=self.counts.fail_at.load(Ordering::SeqCst),"planned clone failure");Self::new(self.n,&self.counts)}}
        impl Drop for Witness{fn drop(&mut self){self.counts.live.fetch_sub(1,Ordering::SeqCst);}}
        for len in [0,1,3,4,5,9]{let counts=Arc::new(Counts::default());let mut values=Array::from([]);for n in 0..len{values.push(Witness::new(n,&counts));}let capacity=match &values{Array::Small{..}=>4,Array::Heap(v)=>v.capacity()};let before=counts.clones.load(Ordering::SeqCst);values.tsr_copy_within(0,len.saturating_sub(1),1.min(len));assert_eq!(counts.clones.load(Ordering::SeqCst)-before,len.saturating_sub(1));assert_eq!(values.len(),len);assert_eq!(counts.live.load(Ordering::SeqCst),len);let no_op=counts.clones.load(Ordering::SeqCst);values.tsr_copy_within(0,len,0);values.tsr_copy_within(0,0,0);assert_eq!(counts.clones.load(Ordering::SeqCst),no_op);values.tsr_fill(Witness::new(8,&counts),0,len);assert_eq!(counts.live.load(Ordering::SeqCst),len);match &values{Array::Small{len,values}=>{assert!(values[..*len].iter().all(Option::is_some));assert!(values[*len..].iter().all(Option::is_none));},Array::Heap(v)=>assert_eq!(v.capacity(),capacity)};drop(values);assert_eq!(counts.live.load(Ordering::SeqCst),0);}
        let counts=Arc::new(Counts::default());let mut values=Array::from([Witness::new(0,&counts),Witness::new(1,&counts),Witness::new(2,&counts),Witness::new(3,&counts)]);counts.fail_at.store(2,Ordering::SeqCst);let old_hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let panicked=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||values.tsr_copy_within(0,3,1)));std::panic::set_hook(old_hook);assert!(panicked.is_err());assert_eq!(values.iter().map(|w|w.n).collect::<Vec<_>>(),vec![0,1,2,3]);assert_eq!(counts.live.load(Ordering::SeqCst),4);drop(values);assert_eq!(counts.live.load(Ordering::SeqCst),0);
    "#,
    );
}
