mod support;
use std::process::Command;
use support::{TempDirectory,run,rustc,success};
use tsrust_compiler::check;

fn execute(source:&str,expected:&str,suffix:&str)->String {
    let source=support::format_round_trip(source);
    let rust=check(&source).unwrap_or_else(|e|panic!("{}",e.render("input.tsr",&source))).emit_rust();
    assert!(!rust.contains("unsafe"));
    let program=if suffix.is_empty(){rust.clone()}else{rust.replace("fn main()","fn source_main()")+suffix};
    let dir=TempDirectory::new();let path=dir.0.join("program.rs");std::fs::write(&path,program).unwrap();
    for level in ["0","3"] {
        let binary=dir.0.join(format!("program-{level}"));
        success(&run(rustc().arg("--edition=2024").arg("-A").arg("warnings").arg("-D").arg("unused_assignments").arg("-C").arg(format!("opt-level={level}")).arg("-C").arg("overflow-checks=on").arg(&path).arg("-o").arg(&binary),60));
        let out=run(&mut Command::new(binary),30);success(&out);assert_eq!(String::from_utf8(out.stdout).unwrap(),expected);
    }
    rust
}

#[test]
fn constant_counted_rows_preserve_values_small_storage_and_final_counter() {
    let rust=execute(r#"
        function labels(values:null[]):string[]{const out:string[]=[];let i=0;while(i<values.length){const kind=typeof values[i];out.push(kind.concat(":",String(values[i])));i=i+1;}console.log(i);return out;}
        function inputs(n:int):null[]{const out:null[]=[];let i=0;while(i<n){out.push(null);i+=1;}return out;}
        function absent(values:undefined[]):string[]{const out:string[]=[];let i=0;while(i<values.length){out.push(String(values[i]));i+=1;}return out;}
        function main(){const empty=labels(inputs(0));console.log(empty.length);const one=labels(inputs(1));console.log(one[0]);const four=labels(inputs(4));console.log(four.length);const five=labels(inputs(5));console.log(five[4]);const many=labels(inputs(17));console.log(many.length);const a=absent([undefined,undefined]);console.log(a[1]);}
    "#,"0\n0\n1\nobject:null\n4\n4\n5\nobject:null\n17\n17\nundefined\n","");
    assert_eq!(rust.matches("let counted_length =").count(),2);
    assert!(rust.contains("Array::Heap(vec![String::Borrowed(\"object:null\")"));
}

#[test]
fn unproved_builders_keep_effects_bounds_and_existing_items() {
    for body in [
        "function f(src:null[]):string[]{const out:string[]=[];out.push(\"seed\");let i=0;while(i<src.length){out.push(\"x\");i+=1;}return out;}",
        "function f(src:null[]):string[]{const out:string[]=[];let i=1;while(i<src.length){out.push(\"x\");i+=1;}return out;}",
        "function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){out.push(\"x\");i+=2;}return out;}",
        "function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){out.push(typeof src[i+1]);i+=1;}return out;}",
        "function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){console.log(i);out.push(\"x\");i+=1;}return out;}",
        "function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){out.push(\"x\");out.push(\"y\");i+=1;}return out;}",
        "function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){if(i===0){out.push(\"x\");}i+=1;}return out;}",
        "function f(src:null[]):string[]{const out:string[]=[];const read=()=>src.length;let i=0;while(i<src.length){out.push(\"x\");i+=1;}console.log(read());return out;}",
        "function effect():null{console.log(\"effect\");return null;}function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){out.push(String(effect()));i+=1;}return out;}",
        "function f(src:null[]):string[]{let out:string[]=[];let i=0;while(i<src.length){out.push(\"x\");i+=1;}out=[];return out;}",
    ] {
        let source=support::format_round_trip(&format!("{body}function main(){{}}"));
        let rust=check(&source).unwrap_or_else(|e|panic!("{}",e.render("input.tsr",&source))).emit_rust();
        assert!(!rust.contains("let counted_length ="),"unproved builder: {body}");
    }
    execute(r#"
        function effect():null{console.log("effect");return null;}
        function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){out.push(String(effect()));i+=1;}return out;}
        function bad(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){out.push(typeof src[i+1]);i+=1;}return out;}
        function main(){const out=f([null,null]);console.log(out.length);try{const missing=bad([null]);console.log(missing.length);}catch{console.log("caught");}}
    "#,"effect\neffect\n2\ncaught\n","");
}

#[test]
fn sized_rows_keep_rust_length_limits_and_allocate_nothing_for_small_results() {
    let allocator=include_str!("../../../benchmarks/runtime/allocation-driver.rs").split("fn main()").next().unwrap();
    let suffix=format!(r#"
        {allocator}
        fn main(){{
            source_main();
            for n in 0..=4{{let input=vec![TsrNull;n];let before=ALLOCS.load(Relaxed);let result=function0(&input);assert_eq!(result.len(),n);assert_eq!(ALLOCS.load(Relaxed),before);assert!(matches!(result,Array::Small{{..}}));}}
            for _ in 0..3{{let input=vec![TsrNull;17];let before=LIVE.load(Relaxed);let result=function0(&input);assert_eq!(result.get(16).unwrap().as_ref(),"é🙂");drop(result);assert_eq!(LIVE.load(Relaxed),before);}}
            if usize::BITS>63{{// SAFETY: this Copy ZST needs no allocation; its dangling pointer is aligned,
            // non-null, and len equals capacity. The test checks the signed-length guard.
            let input=unsafe{{Vec::from_raw_parts(std::ptr::NonNull::<TsrNull>::dangling().as_ptr(),usize::MAX,usize::MAX)}};let error=std::panic::catch_unwind(||function0(&input)).err().expect("length rejected");let message=error.downcast_ref::<std::string::String>().map(|text|text.as_str()).or_else(||error.downcast_ref::<&str>().copied()).unwrap();assert!(message.contains("array length exceeds int range"));}}
        }}
    "#);
    let rust=execute("function f(src:null[]):string[]{const out:string[]=[];let i=0;while(i<src.length){out.push(\"é🙂\");i+=1;}return out;}function main(){}","",&suffix);
    assert!(rust.contains("let counted_length ="));
}

#[test]
fn exact_nullish_spread_workloads_select_both_constant_builders() {
    for source in [
        include_str!("../../../benchmarks/runtime/workloads/array_literal_spread_nullish_packs.tsr"),
        include_str!("../../../benchmarks/runtime/workloads/call_spread_nullish_packs.tsr"),
    ] {
        let source=support::format_round_trip(source);
        let rust=check(&source).unwrap().emit_rust();
        assert_eq!(rust.matches("let counted_length =").count(),2);
        let directory=TempDirectory::new();let path=directory.0.join("workload.rs");std::fs::write(&path,rust).unwrap();
        for level in ["0","3"] {
            success(&run(rustc().arg("--edition=2024").arg("-A").arg("warnings").arg("-D").arg("unused_assignments").arg("-C").arg(format!("opt-level={level}")).arg("--emit=metadata").arg(&path).arg("-o").arg(directory.0.join("workload.rmeta")),60));
        }
    }
}
