mod support;
use tsrust_compiler::{Mode, check};

// This fixture is prepared for Gate57. It has not been run during source preparation.
fn native(
    source: &str,
    expected: &str,
    assertions: &str,
    failure: bool,
) -> tsrust_compiler::Analysis {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis
        .emit_rust()
        .replace("fn main()", "fn source_main()")
        + &format!("\nfn main() {{ source_main(); {assertions} }}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("array-copying.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("array-copying-{level}"));
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
fn owned_results_preserve_sources_all_leaves_and_nested_record_fields() {
    let mut source = String::from(
        r#"
        type Leaf={name:string;values:int[];};
        type Node={name:string;children:Node[];};
        function reversed(values:number[]):number[]{return values.toReversed();}
        function replaced(values:number[],index:int,value:number):number[]{return values.with(index,value);}
        function ints(values:int[],index:int,value:int):int[]{return values.with(index,value);}
    "#,
    );
    // Exercise both methods' checked types and Rust ABIs across three ranks of
    // every current leaf, including recursive records. Runtime cases below
    // observe ownership independence rather than only successful compilation.
    for leaf in ["int", "number", "boolean", "string", "Leaf", "Node"] {
        for rank in 1..=3 {
            let array = format!("{leaf}{}", "[]".repeat(rank));
            let element = format!("{leaf}{}", "[]".repeat(rank - 1));
            source.push_str(&format!(
                "function rev{leaf}{rank}(values:{array}):{array}{{return values.toReversed();}}\nfunction set{leaf}{rank}(values:{array},index:int,value:{element}):{array}{{return values.with(index,value);}}\n"
            ));
        }
    }
    source.push_str(r#"
        function main(){
            const empty:int[]=[];console.log(empty.toReversed().length);
            const singleton=[7];const singleCopy=singleton.with(-1,8);console.log(singleCopy[0]);console.log(singleton[0]);
            const values=[1,2,3];const reverse=values.toReversed();const replace=values.with(-2,9);
            reverse[0]=8;replace[0]=7;console.log(values[0]);console.log(values[2]);console.log(replace[1]);
            const flags=[true,false];const flagCopy=flags.toReversed();const flagSet=flags.with(0,false);
            console.log(flagCopy[0]);console.log(flagSet[0]);console.log(flags[0]);
            const nums:number[]=[1.5,2.5];const numReverse=nums.toReversed();const numSet=nums.with(1,3.5);
            console.log(numReverse[0]);console.log(numSet[1]);console.log(nums[1]);
            const word=" First ".trim();const words=[word,"second"];
            const wordReverse=words.toReversed();const retained=" Saved ".trim();const wordSet=words.with(1,clone(retained));
            words[0]="changed";console.log(wordReverse[1]);console.log(wordSet[1]);console.log(retained);
            words.remove(1);const liveWords=words.toReversed();console.log(liveWords.length);console.log(liveWords[0]);
            const literals=["one","two"];const literalReverse=literals.toReversed();const literalSet=literals.with(-1,"last");
            console.log(literalReverse[0]);console.log(literalSet[1]);
            const nested:int[][]=[[1,2],[3]];const nestedReverse=nested.toReversed();const row=[6];const nestedSet=nested.with(1,clone(row));
            nested[0][0]=8;nestedSet[1][0]=9;console.log(nestedReverse[1][0]);console.log(nestedSet[0][0]);console.log(row[0]);
            const triples:int[][][]=[[[4]]];const tripleReverse=triples.toReversed();const tripleSet=triples.with(0,[[5]]);
            triples[0][0][0]=9;console.log(tripleReverse[0][0][0]);console.log(tripleSet[0][0][0]);
            const rows:Leaf[]=[{name:"leaf",values:[1]}];const rowsReverse=rows.toReversed();const rowsSet=rows.with(0,{name:"new",values:[2]});
            rows[0].values[0]=9;console.log(rowsReverse[0].values[0]);console.log(rowsSet[0].values[0]);
            const tree:Node[]=[{name:"root",children:[{name:"child",children:[]}]}];
            const treeReverse=tree.toReversed();const treeSet=tree.with(0,{name:"fresh",children:[]});
            tree[0].children[0].name="changed";console.log(treeReverse[0].children[0].name);console.log(treeSet[0].name);
            const selected=nested[0].with(-1,10);console.log(selected[1]);console.log(nested[0][1]);
            const temporary=[1,2].toReversed().with(0,3);console.log(temporary[0]);console.log(temporary[1]);
            const contextual:int[]=[].toReversed();console.log(contextual.length);
        }
    "#);
    let analysis = native(
        &source,
        "0\n8\n7\n1\n3\n9\nfalse\nfalse\ntrue\n2.5\n3.5\n2.5\nFirst\nSaved\nSaved\n1\nchanged\ntwo\nlast\n1\n1\n6\n4\n5\n1\n2\nchild\nfresh\n10\n2\n3\n1\n0\n",
        r#"
        let bits=[0x8000000000000000,0x7ff8000000001234,0x7ff0000000000000,0xfff0000000000000];
        let values=bits.map(f64::from_bits).to_vec();
        assert_eq!(function0(&values).iter().map(|x|x.to_bits()).collect::<Vec<_>>(),bits.into_iter().rev().collect::<Vec<_>>());
        let replacement=f64::from_bits(0xfff8000000005678);
        let replaced=function1(&values,-2,replacement);
        assert_eq!(replaced.iter().map(|x|x.to_bits()).collect::<Vec<_>>(),vec![bits[0],bits[1],replacement.to_bits(),bits[3]]);
        assert_eq!(values.iter().map(|x|x.to_bits()).collect::<Vec<_>>(),bits);
        let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));
        for len in [0,1,3,4,5] {
            let mut values=NumericArray::from([]);for index in 0..len { values.push(index as i64); }
            let before=values.iter().copied().collect::<Vec<_>>();
            assert_eq!(function3(&values).iter().copied().collect::<Vec<_>>(),before.iter().copied().rev().collect::<Vec<_>>());
            for index in [i64::MIN,i64::MAX,-(len as i64)-1,len as i64] {
                assert!(std::panic::catch_unwind(||function2(&values,index,99)).is_err());
            }
            if len>0 {
                assert_eq!(*function2(&values,-(len as i64),99).get(0).unwrap(),99);
                assert_eq!(*function2(&values,(len-1) as i64,99).get(len-1).unwrap(),99);
            }
            assert_eq!(values.iter().copied().collect::<Vec<_>>(),before);
        }
        std::panic::set_hook(hook);
        "#,
        false,
    );
    assert_eq!(
        analysis.contracts().get("reversed.values"),
        Some(&Mode::Read)
    );
    assert_eq!(
        analysis.contracts().get("replaced.values"),
        Some(&Mode::Read)
    );
    assert_eq!(analysis.contracts().get("ints.values"), Some(&Mode::Read));
}

#[test]
fn receiver_and_argument_effects_precede_replacement_bounds_failure() {
    native(
        r#"
        function receiver():int[]{console.log(1);return [10];}
        function mark(value:int):int{console.log(value);return value;}
        function main(){const result=receiver().with(mark(3),mark(4));console.log(result.length);}
    "#,
        "1\n3\n4\n",
        "",
        true,
    );
    native(
        r#"
        function receiver():int[][]{console.log(1);return [[10]];}
        function mark(value:int):int{console.log(value);return value;}
        function main(){const result=receiver()[mark(2)].with(mark(3),mark(4));console.log(result.length);}
    "#,
        "1\n2\n",
        "",
        true,
    );
}

#[test]
fn copying_diagnostics_protect_types_ownership_and_selected_parent_loans() {
    for (source, code) in [
        (
            "function main(){const values=[1];values.toReversed(1);}",
            "arity",
        ),
        ("function main(){const values=[1];values.with(0);}", "arity"),
        (
            "function main(){const values=[1];values.with(0,1,2);}",
            "arity",
        ),
        (
            "function main(){const values=[1];values.with(1.5,2);}",
            "type",
        ),
        (
            "function main(){const values=[1];values.with(0,true);}",
            "type",
        ),
        ("function main(){const value=1;value.toReversed();}", "type"),
        (
            "function main(){const values=[1];const moved=values;values.toReversed();}",
            "transferred",
        ),
        (
            "function main(){const values:int[][]=[[1]];const row=[2];const copy=values.with(0,row);console.log(row.length);}",
            "transferred",
        ),
        (
            "function grow(values:int[]):int{values.push(2);return 0;}function main(){const values=[1];values.with(grow(values),3);}",
            "conflicting-access",
        ),
        (
            "function grow(values:int[]):int{values.push(2);return 3;}function main(){const values=[1];values.with(0,grow(values));}",
            "conflicting-access",
        ),
        (
            "type Grid={values:int[][];n:int;};function change(grid:Grid):int{grid.n=1;return 0;}function main(){const grid:Grid={values:[[1]],n:0};grid.values[0].with(change(grid),2);}",
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
