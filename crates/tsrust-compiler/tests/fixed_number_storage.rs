mod support;
use tsrust_compiler::{Mode, check};
fn native(
    source: &str,
    expected: &str,
    observer: &str,
    assertions: &str,
) -> (tsrust_compiler::Analysis, String) {
    let analysis = check(source).unwrap_or_else(|e| panic!("{}", e.render("input.tsr", source)));
    let emitted = analysis.emit_rust();
    assert!(!emitted.contains("unsafe"));
    let rust = emitted.replace("fn main()", "fn source_main()")
        + observer
        + &format!("\nfn main(){{source_main();{assertions}}}\n");
    let directory = support::TempDirectory::new();
    let path = directory.0.join("fixed-number.rs");
    std::fs::write(&path, rust).unwrap();
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("fixed-number-{level}"));
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
    (analysis, emitted)
}
#[test]
fn number_literals_keep_standard_arrays_stable_calls_clones_search_and_ieee_values() {
    let (analysis, rust) = native(
        r#"
    function item(value:number):number{console.log(value);return value;}
    function edit(values:number[],delta:number){values[1]=values[0]+delta;}
    function forward(values:number[],delta:number){edit(values,delta);}
    function sum(values:number[],depth:int):number{if(depth>0){return sum(values,depth-1);}let total=0.0;for(const value of values){total=total+value;}return total;}
    function bits(value:number,index:int):number{const values=[-0.0,value,Number("NaN"),Number("Infinity")];return values[index];}
    function main(){
        const values=[item(1.5),item(2.5),item(3.5)];forward(values,2);console.log(sum(values,2));
        console.log(values.includes(3.5,-1));console.log(values.indexOf(3.5,1));
        const copy=clone(values);copy[0]=9;console.log(values[0]);console.log(copy[0]);copy.reverse();console.log(copy[2]);
        const section=values.slice(1);section.push(8);console.log(section.length);console.log(values.length);
        const growth=clone(values);growth.push(9);console.log(growth.length);
        const empty:number[]=[];console.log(empty.length);console.log(empty.includes(Number("NaN")));
        const special=[-0.0,Number("NaN"),Number("Infinity")];console.log(special.includes(0));console.log(special.includes(Number("NaN")));console.log(special.indexOf(Number("NaN")));
    }
    "#,
        "1.5\n2.5\n3.5\n8.5\ntrue\n1\n1.5\n9\n9\n3\n3\n4\n0\nfalse\ntrue\ntrue\n-1\n",
        "",
        r#"
    for value in [-0.0,0.0,f64::NAN,f64::INFINITY,f64::NEG_INFINITY,f64::from_bits(1),1.25]{assert_eq!(function4(value,0).to_bits(),(-0.0f64).to_bits());assert_eq!(function4(value,1).to_bits(),value.to_bits());assert!(function4(value,2).is_nan());assert_eq!(function4(value,3),f64::INFINITY);}
    "#,
    );
    assert!(rust.contains(": [f64; 3] = ["));
    assert!(rust.contains(": [f64; 0] = []"));
    assert!(rust.contains(": &mut [f64; 3]"));
    assert!(rust.contains(": &[f64; 3]"));
    assert!(rust.contains("Vec::from(Clone::clone("));
    assert_eq!(
        analysis.contracts().get("forward.values"),
        Some(&Mode::Mutate)
    );
    assert_eq!(analysis.contracts().get("sum.values"), Some(&Mode::Read));
}
#[test]
fn temporary_scalar_literals_select_without_allocating_and_preserve_evaluation_order() {
    let (_, rust) = native(
        r#"
    function number(value:number,index:int):number{return [value,-0.0,-value,Number("NaN")][index];}
    function integer(value:int,index:int):int{return [value,-value,value+1][index];}
    function boolean(value:boolean,index:int):boolean{return [value,!value][index];}
    function observe(values:number[],index:int):number{return values[index];}
    function item(value:number):number{console.log(value);return value;}
    function index():int{console.log("index");return 1;}
    function named(value:number):number{const values=[value,-0.0,-value];return values[2];}
    function main(){console.log([item(1.5),item(2.5),item(3.5)][index()]);console.log(observe([1.5,2.5],1));}
    function escaped(value:number):number[]{const values=[value,-0.0,-value];values.push(9);return values;}
    function closurePick(value:number,index:int):number{const pick=(position:int):number=>{return [value,-0.0,-value][position];};return pick(index);}
    function emptyPick(index:int):number{return [][index];}
    "#,
        "1.5\n2.5\n3.5\nindex\n2.5\n2.5\n",
        r#"
    use std::alloc::{GlobalAlloc,Layout,System};use std::sync::atomic::{AtomicUsize,Ordering};static ALLOCATIONS:AtomicUsize=AtomicUsize::new(0);static LIVE:AtomicUsize=AtomicUsize::new(0);struct Observer;
    unsafe impl GlobalAlloc for Observer{unsafe fn alloc(&self,l:Layout)->*mut u8{ALLOCATIONS.fetch_add(1,Ordering::SeqCst);let pointer=unsafe{System.alloc(l)};if !pointer.is_null(){LIVE.fetch_add(1,Ordering::SeqCst);}pointer}unsafe fn dealloc(&self,p:*mut u8,l:Layout){unsafe{System.dealloc(p,l)};LIVE.fetch_sub(1,Ordering::SeqCst);}}
    #[global_allocator]static ALLOCATOR:Observer=Observer;
    "#,
        r#"
    let baseline=ALLOCATIONS.load(Ordering::SeqCst);
    for value in [f64::from_bits(1),-0.0,0.0,f64::INFINITY,f64::NEG_INFINITY,f64::NAN,1.5]{assert_eq!(function0(value,0).to_bits(),value.to_bits());assert_eq!(function0(value,1).to_bits(),(-0.0f64).to_bits());assert_eq!(function0(value,2).to_bits(),(-value).to_bits());assert!(function0(value,3).is_nan());assert_eq!(function6(value).to_bits(),(-value).to_bits());assert_eq!(function9(value,0).to_bits(),value.to_bits());assert_eq!(function9(value,1).to_bits(),(-0.0f64).to_bits());assert_eq!(function9(value,2).to_bits(),(-value).to_bits());}
    for value in [-10,0,20]{assert_eq!(function1(value,0),value);assert_eq!(function1(value,1),-value);assert_eq!(function1(value,2),value+1);}for value in [false,true]{assert_eq!(function2(value,0),value);assert_eq!(function2(value,1),!value);}
    assert_eq!(ALLOCATIONS.load(Ordering::SeqCst),baseline,"direct literal selection or stable named array allocated");
    let live=LIVE.load(Ordering::SeqCst);let escaped=std::hint::black_box(function8(1.5));assert_eq!(escaped.len(),4);assert_eq!(escaped[3],9.0);assert!(ALLOCATIONS.load(Ordering::SeqCst)>baseline);drop(escaped);assert_eq!(LIVE.load(Ordering::SeqCst),live);
    let hook=std::panic::take_hook();std::panic::set_hook(Box::new(|_|{}));let live=LIVE.load(Ordering::SeqCst);
    for index in [i64::MIN,-1,4,i64::MAX]{assert!(std::panic::catch_unwind(||function0(1.5,index)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);}
    for index in [i64::MIN,-1,0,1,i64::MAX]{assert!(std::panic::catch_unwind(||function10(index)).is_err());assert_eq!(LIVE.load(Ordering::SeqCst),live);}
    std::panic::set_hook(hook);
    "#,
    );
    assert!(rust.contains("temporary_array: [f64; 4] = ["));
    assert!(rust.contains("temporary_array: [i64; 3] = ["));
    assert!(rust.contains("temporary_array: [bool; 2] = ["));
    assert!(rust.contains("temporary_array: [f64; 0] = []"));
    assert!(rust.contains("temporary_array: [f64; 3] = ["));
    // Ordinary calls retain their ordinary array ABI; only direct literal-index bases change.
    assert!(rust.contains("fn function3(value0: &Vec<f64>"));
    for source in [
        "function main(){[1.5][0]=2;}",
        "function main(){const moved=[1.5];const values=moved;console.log(moved[0]);}",
    ] {
        assert!(check(source).is_err());
    }
}
#[test]
fn number_storage_falls_back_for_resize_rebind_moves_captures_and_callback_receiver_abi() {
    for (source, expected) in [
        (
            "function main(){const values=[1.5,2.5];values.push(3.5);console.log(values.length);}",
            "3\n",
        ),
        (
            "function main(){const values=[1.5,2.5];console.log(values.remove(0));console.log(values.length);}",
            "1.5\n1\n",
        ),
        (
            "function main(){let values=[1.5,2.5];values=[3.5];console.log(values[0]);}",
            "3.5\n",
        ),
        (
            "function grow(values:number[]){values.push(3.5);}function relay(values:number[]){grow(values);}function main(){const values=[1.5,2.5];relay(values);console.log(values.length);}",
            "3\n",
        ),
        (
            "function main(){const values=[1.5,2.5];const moved=values;console.log(moved[0]);}",
            "1.5\n",
        ),
        (
            "function main(){const values=[1.5,2.5];const action=():number=>{return values[0];};console.log(action());}",
            "1.5\n",
        ),
        (
            "function main(){const values=[1.5,2.5];const mapped=values.map((value:number,index:int,source:number[]):number=>{return value+Number(source.length);});console.log(mapped[0]);const kept=values.filter((value:number):boolean=>{return value>2;});console.log(kept[0]);}",
            "3.5\n2.5\n",
        ),
    ] {
        let (_, rust) = native(source, expected, "", "");
        assert!(!rust.contains(": [f64; 2] = ["), "{source}");
    }
    let source = "function mapped(values:number[],action:(value:number)=>number):number[]{return values.map(action);}function main(){const action=(value:number):number=>{return value+1;};const values=[1.5,2.5];console.log(mapped(values,action)[0]);}";
    native(source, "2.5\n", "", "");
}
#[test]
fn standard_array_planning_respects_stack_and_proof_budgets_and_bounds_failure_order() {
    let oversized = (0..513).map(|_| "value").collect::<Vec<_>>().join(",");
    let source = format!(
        "function pick(value:number,index:int):number{{return [{oversized}][index];}}function main(){{console.log(pick(1.5,512));}}"
    );
    let (_, rust) = native(&source, "1.5\n", "", "");
    assert!(rust.contains("temporary_array = vec!["));
    assert!(!rust.contains("temporary_array: [f64; 513]"));
    let full = (0..512).map(|_| "value").collect::<Vec<_>>().join(",");
    let source = format!(
        "function pick(value:number):number{{const values=[{full}];console.log(values.length);return [value][0];}}function main(){{}}"
    );
    let rust = check(&source).unwrap().emit_rust();
    assert!(rust.contains(": [f64; 512] = ["));
    assert!(rust.contains("temporary_array = vec!["));
    let source = format!(
        "function main(){{console.log({}[true][0]);}}",
        "!".repeat(70)
    );
    let rust = check(&source).unwrap().emit_rust();
    assert!(!rust.contains("temporary_array: [bool; 1]"));
    let source = format!(
        "function main(){{{} }}",
        (0..2100)
            .map(|_| "console.log([true][0]);")
            .collect::<String>()
    );
    let rust = check(&source).unwrap().emit_rust();
    assert!(
        !rust.contains("temporary_array: [bool; 1]"),
        "proof step limit must fail closed"
    );
    for (source, expected) in [
        (
            "function item(value:number):number{console.log(value);return value;}function index():int{console.log(\"index\");return -1;}function main(){console.log([item(1.5),item(2.5)][index()]);}",
            "1.5\n2.5\nindex\n",
        ),
        (
            "function item():number{console.log(\"item\");const failures=[1];failures.remove(-1);return 1.5;}function index():int{console.log(\"index\");return 0;}function main(){console.log([item()][index()]);}",
            "item\n",
        ),
    ] {
        let rust = check(source).unwrap().emit_rust();
        let directory = support::TempDirectory::new();
        let path = directory.0.join("bounds.rs");
        std::fs::write(&path, rust).unwrap();
        for level in ["0", "3"] {
            let executable = directory.0.join(format!("bounds-{level}"));
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
            assert!(!output.status.success());
            assert_eq!(output.stdout, expected.as_bytes());
            assert!(String::from_utf8_lossy(&output.stderr).contains("array index out of bounds"));
        }
    }
}
