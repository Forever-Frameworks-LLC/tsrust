mod support;

#[test]
fn schema_arrays_build_final_storage_and_preserve_cleanup_at_o0_o3() {
    let directory = support::TempDirectory::new();
    for (source, call) in [
        (include_str!("../../../benchmarks/runtime/workloads/native_schema_owned_array.tsr"), "validate_array_into(input.tsr_owned_items()"),
        (include_str!("../../../benchmarks/runtime/workloads/native_schema_json_array.tsr"), "validate_array_into(values,|index,value|"),
    ] {
        let entry = directory.source(&support::format_round_trip(source));
        let project = tsrust_compiler::project::load_project(&tsrust_compiler::project::FileSourceProvider, entry.to_str().unwrap()).unwrap();
        let emitted = tsrust_compiler::project::check_graph(project).unwrap().emit_rust();
        assert!(emitted.contains(call), "Missing lowering: {call}");
    }
    let carriers = support::format_round_trip(r#"
function keepNumbers(values:int[]):int[]{values.push(1);return values;}
function keepStrings(values:string[]):string[]{values.push("tail");return values;}
function main(){}
"#);
    let carriers = tsrust_compiler::check(&carriers).unwrap().emit_rust();
    assert!(!carriers.contains("unsafe"));
    let plan = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/stdlib/schema/runtime/plan.rs");
    let allocator = include_str!("../../../benchmarks/runtime/allocation-driver.rs").split("fn main()").next().unwrap();
    let harness = r#"
fn main(){
 let baseline=LIVE.load(Relaxed);let allocations=ALLOCS.load(Relaxed);let reallocations=REALLOCS.load(Relaxed);
 let output=tsrust_schema_plan::validate_array_into(std::hint::black_box([2i64,3]),|_,value|Some(value),NumericArray::from([]),NumericArray::push).unwrap();
 assert_eq!(output.as_slice(),[2,3]);assert_eq!(ALLOCS.load(Relaxed),allocations);assert_eq!(REALLOCS.load(Relaxed),reallocations);
 drop(output);assert_eq!(LIVE.load(Relaxed),baseline);
 for length in [0,1,4,5,17] {
  let baseline=LIVE.load(Relaxed);
  let input=(0..length).map(|index|String::Owned(format!("owned-{index}"))).collect::<Vec<_>>();
  let pointers=input.iter().map(|value|AsRef::<str>::as_ref(value).as_ptr() as usize).collect::<Vec<_>>();
  let output=tsrust_schema_plan::validate_array_into(std::hint::black_box(input),|_,value|Some(value),Array::from([]),Array::push).unwrap();
  assert_eq!(output.len(),length);
  for (index,value) in output.iter().enumerate(){assert_eq!(AsRef::<str>::as_ref(value).as_ptr() as usize,pointers[index]);}
  drop(output);drop(pointers);assert_eq!(LIVE.load(Relaxed),baseline);
 }
}
"#;
    let path = directory.0.join("schema-array-storage.rs");
    std::fs::write(&path, carriers.replace("fn main()", "fn source_main()") + &format!("\n#[path={plan:?}] mod tsrust_schema_plan;\n") + allocator + harness).unwrap();
    let units = directory.0.join("schema-array-plan.rs");
    std::fs::write(&units, format!("#[path={plan:?}] mod plan;\n")).unwrap();
    for level in ["0", "3"] {
        for unit_tests in [false, true] {
            let executable = directory.0.join(format!("schema-array-{level}-{unit_tests}"));
            let mut compiler = support::rustc();
            compiler.args(["--edition=2024", "-C", &format!("opt-level={level}"), "-C", "overflow-checks=on", "-A", "warnings"]);
            if unit_tests { compiler.args(["--test", "-F", "unsafe_code"]); }
            support::success(&support::run(compiler.arg(if unit_tests { &units } else { &path }).arg("-o").arg(&executable), 60));
            let mut command = std::process::Command::new(executable);
            if unit_tests { command.arg("--test-threads=3"); }
            support::success(&support::run(&mut command, 30));
        }
    }
}
