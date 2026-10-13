mod support;
use tsrust_compiler::{check, Mode};

fn chain(length: usize, scalar: &str, leaf: &str, inferred: bool, values: bool) -> String {
    let result = if inferred { String::new() } else { format!(": {scalar}") };
    let mut source = String::new();
    for index in 0..length {
        let body = if index + 1 == length { leaf.to_owned() }
            else if values { format!("const action = step{}; return action(value);", index + 1) }
            else { format!("return step{}(value);", index + 1) };
        source.push_str(&format!("function step{index}(value: {scalar}){result} {{ {body} }}\n"));
    }
    source
}

#[test]
fn long_function_dependencies_check_emit_and_drop_on_normal_worker_stack() {
    let mut source = chain(2048, "int", "return value;", false, false);
    source.push_str("function main() { console.log(step0(7)); }");
    let source = support::format_round_trip(&source);
    let checked = check(&source).unwrap();
    assert_eq!(checked.contracts()["step0.value"], Mode::Copy);
    let emitted = checked.emit_rust();
    assert!(emitted.contains("fn function2047"));
    drop(emitted);
    drop(checked);
}

#[test]
fn inferred_results_and_function_value_dependencies_share_the_worklist() {
    for values in [false, true] {
        let mut source = chain(512, "int", "return value;", true, values);
        source.push_str("function main() { console.log(step0(7)); }");
        support::execute(&source, "7\n", None);
        assert_eq!(check(&source).unwrap().contracts()["step0.value"], Mode::Copy);
    }
}

#[test]
fn long_owned_chain_preserves_transfer_and_error_location() {
    let mut source = chain(512, "string", "return value;", false, false);
    source.push_str("function main() { console.log(step0(\"kept\")); }");
    support::execute(&source, "kept\n", None);
    assert_eq!(check(&source).unwrap().contracts()["step0.value"], Mode::Consume);
    let mut source = chain(512, "string", "return value;", false, false);
    source.push_str("function main() { const text = \"moved\"; const result = step0(text); console.log(text); }");
    let source = support::format_round_trip(&source);
    let error = check(&source).err().expect("moved string must remain unavailable");
    assert_eq!(error.code, "transferred");
    assert_eq!(&source[error.span.start..error.span.end], "text");
    assert!(error.span.start > source.find("step0(text)").unwrap());
}

#[test]
fn dependency_scheduling_preserves_argument_order_and_throw_payloads() {
    let mut source = chain(256, "int", "throw value;", false, false);
    source.push_str(r#"
function mark(value: int): int { console.log(value); return value; }
function main() {
    try { step0(mark(9)); } catch (error) { console.log(error); }
    console.log(mark(1), mark(2));
}
"#);
    support::execute(&source, "9\n9\n1\n2\n1 2\n", None);
}

#[test]
fn recursive_group_entry_keeps_the_original_inferred_return_seed() {
    support::execute(r#"
function first(n: int) { if (n === 0) { return 7; } return second(n - 1); }
function second(n: int) { return first(n); }
function main() { console.log(first(2)); }
"#, "7\n", None);
}

#[test]
fn imported_function_chain_uses_project_identity_and_native_execution() {
    let directory = support::TempDirectory::new();
    let module = format!("export {}", chain(512, "int", "return value;", false, false));
    std::fs::write(directory.0.join("chain.tsr"), &module).unwrap();
    let entry = directory.source("import { step0 } from './chain.tsr'; function main() { console.log(step0(7)); }");
    support::success(&support::run(support::cli().arg("format").arg(&directory.0), 60));
    let formatted = std::fs::read_to_string(directory.0.join("chain.tsr")).unwrap();
    support::success(&support::run(support::cli().arg("format").arg(&directory.0), 60));
    assert_eq!(std::fs::read_to_string(directory.0.join("chain.tsr")).unwrap(), formatted);
    let output = support::run(support::cli().arg("run").arg(entry), 60);
    support::success(&output);
    assert_eq!(output.stdout, b"7\n");
}

fn recursive_chain(length: usize, owned: bool) -> String {
    let mut source = String::new();
    let ty = if owned { "string" } else { "int" };
    for index in 0..length {
        let next = (index + 1) % length;
        let result = if index + 1 == length { "if (remaining === 0) { return value; }" } else { "" };
        let remaining = if index + 1 == length { "remaining - 1" } else { "remaining" };
        source.push_str(&format!("function cycle{index}(value: {ty}, remaining: int): {ty} {{ {result} return cycle{next}(value, {remaining}); }}\n"));
    }
    source
}

#[test]
fn large_recursive_group_uses_normal_stack_for_check_emit_and_disposal() {
    let mut source = recursive_chain(2048, false);
    source.push_str("function main() { console.log(cycle0(7, 0)); }");
    let source = support::format_round_trip(&source);
    let checked = check(&source).unwrap();
    assert_eq!(checked.contracts()["cycle0.value"], Mode::Copy);
    assert!(checked.emit_rust().contains("fn function2047"));
    drop(checked);
}

#[test]
fn recursive_group_propagates_owned_access_without_changing_runtime_order() {
    let mut source = recursive_chain(256, true);
    source.push_str("function main() { const text = cycle0(\"kept\", 1); console.log(text); }");
    support::execute(&source, "kept\n", None);
    let contracts = check(&source).unwrap().contracts();
    for index in 0..256 { assert_eq!(contracts[&format!("cycle{index}.value")], Mode::Consume); }
    let mut source = recursive_chain(256, true);
    source.push_str("function main() { const text = \"moved\"; const result = cycle0(text, 0); console.log(text); }");
    let source = support::format_round_trip(&source);
    let error = check(&source).err().expect("recursive ownership must propagate");
    assert_eq!(error.code, "transferred");
    assert_eq!(&source[error.span.start..error.span.end], "text");
    assert!(error.span.start > source.find("cycle0(text").unwrap());
}

fn inferred_cycle(length: usize) -> String {
    (0..length).map(|index| {
        let seed = if index == 0 { "if (remaining === 0) { return value; }" } else { "" };
        let remaining = if index + 1 == length { "remaining - 1" } else { "remaining" };
        format!("function cycle{index}(value: int, remaining: int) {{ {seed} return cycle{}(value, {remaining}); }}\n", (index + 1) % length)
    }).collect()
}

#[test]
fn deep_inferred_recursive_group_preserves_entry_seed_and_shared_dependencies() {
    let mut source = inferred_cycle(2048);
    source.push_str("function main() { console.log(cycle0(7, 1)); }");
    let source = support::format_round_trip(&source);
    let checked = check(&source).unwrap();
    assert!(checked.emit_rust().contains("fn function2047"));
    drop(checked);
    let mut source = inferred_cycle(256);
    source.push_str("function main() { console.log(cycle0(7, 1)); }");
    support::execute(&source, "7\n", None);
    support::execute("function root(n:int){if(n===0){return 7;}return left(n-1)+right(n-1);}function left(n:int){return root(n);}function right(n:int){return left(n);}function main(){console.log(root(1));}", "14\n", None);
}

#[test]
fn inferred_cycle_cannot_take_its_entry_seed_from_another_function() {
    let source = "function first(n:int){return second(n);}function second(n:int){if(n===0){return 7;}return first(n-1);}function main(){}";
    let error = check(source).err().expect("recursive entry has no independent result");
    assert_eq!(error.code, "result-inference");
    assert_eq!(&source[error.span.start..error.span.end], "first");
    let source = "function first(n:int){const answer=7;if(n===0){return answer;}return second(n-1);}function second(n:int){return first(n);}function main(){}";
    let error = check(source).err().expect("locals remain outside provisional inference");
    assert_eq!(error.code, "result-inference");
    assert_eq!(&source[error.span.start..error.span.end], "first");
}

fn callable_cycle(length: usize) -> String {
    (0..length).map(|index| {
        let seed = if index == 0 { "if (remaining === 0) { return (): int => { return 7; }; }" } else { "" };
        let remaining = if index == 0 { "remaining - 1" } else { "remaining" };
        format!("function cycle{index}(remaining: int): () => int {{ {seed} return cycle{}({remaining}); }}\n", (index + 1) % length)
    }).collect()
}

#[test]
fn callable_result_dependencies_resolve_deep_cycles_and_native_environments() {
    let mut source = callable_cycle(2048);
    source.push_str("function main() { const action = cycle0(1); console.log(action()); }");
    let source = support::format_round_trip(&source);
    let checked = check(&source).unwrap();
    assert!(checked.emit_rust().contains("fn function2047"));
    drop(checked);
    let mut source = callable_cycle(256);
    source.push_str("function main() { const action = cycle0(1); console.log(action()); }");
    support::execute(&source, "7\n", None);
}

fn record_chain(length: usize, recursive: bool, mutate: bool) -> String {
    let mut source = "type Item = { value: int; };\n".to_owned();
    let result = if mutate { "int" } else { "Item" };
    let terminal = if mutate { "value.value += 1; return value.value;" } else { "return value;" };
    for index in 0..length {
        let next = (index + 1) % length;
        let body = if index + 1 == length {
            if recursive { format!("if (remaining === 0) {{ {terminal} }} return step0(value, remaining - 1);") }
            else { terminal.to_owned() }
        } else { format!("return step{next}(value, remaining);") };
        source.push_str(&format!("function step{index}(value: Item, remaining: int): {result} {{ {body} }}\n"));
    }
    source
}

#[test]
fn data_record_dependencies_check_emit_and_drop_long_chains_and_cycles() {
    for recursive in [false, true] {
        let mut source = record_chain(2048, recursive, false);
        source.push_str("function main() { const result = step0({ value: 7 }, 0); console.log(result.value); }");
        let source = support::format_round_trip(&source);
        let checked = check(&source).unwrap();
        assert_eq!(checked.contracts()["step0.value"], Mode::Consume);
        assert!(checked.emit_rust().contains("fn function2047"));
        drop(checked);
    }
}

#[test]
fn recursive_data_records_preserve_transfer_mutation_and_callsite_diagnostics() {
    let mut source = record_chain(256, true, false);
    source.push_str("function main() { const item: Item = { value: 7 }; const result = step0(item, 1); console.log(result.value); }");
    let rust = support::execute(&source, "7\n", None);
    assert!(!rust.contains("Clone::clone("));
    let mut source = record_chain(256, true, true);
    source.push_str("function main() { const item: Item = { value: 7 }; console.log(step0(item, 1)); console.log(item.value); }");
    support::execute(&source, "8\n8\n", None);
    assert_eq!(check(&source).unwrap().contracts()["step0.value"], Mode::Mutate);
    let mut source = record_chain(256, true, false);
    source.push_str("function main() { const item: Item = { value: 7 }; const result = step0(item, 0); console.log(item.value); }");
    let source = support::format_round_trip(&source);
    let error = check(&source).err().expect("record moved through cycle");
    assert_eq!(error.code, "transferred");
    assert_eq!(&source[error.span.start..error.span.end], "item");
    assert!(error.span.start > source.find("step0(item").unwrap());
}

fn generic_chain(length: usize, scalar: &str, leaf: &str) -> String {
    let mut source = String::new();
    for index in 0..length {
        let body = if index + 1 == length { leaf.to_owned() }
            else { format!("return step{}<T>(value);", index + 1) };
        source.push_str(&format!("function step{index}<T>(value:T):T{{{body}}}\n"));
    }
    source.push_str(&format!("function main(){{console.log(step0<{scalar}>({}));}}", if scalar == "string" { "\"kept\"" } else { "7" }));
    source
}

#[test]
fn generic_materialization_chain_uses_pending_work_instead_of_depth() {
    let source = support::format_round_trip(&generic_chain(128, "int", "return value;"));
    let checked = check(&source).unwrap();
    let emitted = checked.emit_rust();
    assert!(emitted.contains("fn function128"));
    drop(emitted);
    drop(checked);
}

#[test]
fn generic_materialization_chain_preserves_native_transfer_and_failure() {
    support::execute(&generic_chain(64, "string", "return value;"), "kept\n", None);
    let source = generic_chain(64, "int", "throw value;")
        .replace("console.log(step0<int>(7));", "try{step0<int>(7);}catch(error){console.log(error);}");
    support::execute(&source, "7\n", None);
    let source = generic_chain(64, "string", "return value;")
        .replace("console.log(step0<string>(\"kept\"));", "const value=\"kept\";step0<string>(value);console.log(value);");
    let source = support::format_round_trip(&source);
    let error = check(&source).err().expect("transferred string remains unavailable");
    assert_eq!(error.code, "transferred");
    assert_eq!(&source[error.span.start..error.span.end], "value");
    assert!(error.span.start > source.find("step0<string>(value)").unwrap());
}
