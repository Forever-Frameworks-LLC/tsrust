mod support;
use support::{execute as execute_default, reject};

fn execute(
    source: &str,
    expected: &str,
    contracts: Option<&[(&str, tsrust_compiler::Mode)]>,
) -> String {
    let rust = execute_default(source, expected, contracts);
    let directory = support::TempDirectory::new();
    let path = directory.0.join("optimized.rs");
    std::fs::write(&path, &rust).unwrap();
    let binary = directory.0.join("optimized");
    support::success(&support::run(
        support::rustc()
            .arg("--edition=2024")
            .arg("-Copt-level=3")
            .arg(&path)
            .arg("-o")
            .arg(&binary),
        60,
    ));
    let output = support::run(&mut std::process::Command::new(&binary), 10);
    support::success(&output);
    assert_eq!(output.stdout, expected.as_bytes());
    assert!(output.stderr.is_empty());
    rust
}

#[test]
fn typed_local_callables_preserve_owned_mutation_and_named_moves() {
    execute(
        include_str!("fixtures/callable-locals/typed_counter.tsr"),
        "1 2\n3\n5\n",
        None,
    );
}

#[test]
fn typed_once_locals_enforce_consumption_for_inline_and_named_environments() {
    execute(
        r#"function increment(value: int): int { return value + 1; }
function main() {
    const action: Once<Function<int, [value: int]>> = (value: int): int => { return value + 1; };
    const named: Once<Function<int, [value: int]>> = increment;
    console.log(action(4), named(7));
}"#,
        "5 8\n",
        None,
    );
    let source = "function main() { const action: Once<Function<int>> = (): int => { return 7; }; action(); action(); }";
    let error = tsrust_compiler::check(source)
        .err()
        .expect("second call rejected");
    assert_eq!(error.code, "closure-consumed");
}

#[test]
fn typed_borrowing_locals_keep_receiver_loans_and_once_lifetimes() {
    execute(
        r#"type State = { value: int; };
function main() {
    const state: State = { value: 3 };
    const read: ReadOnly<Function<int>> = (): int => { return state.value; };
    console.log(read(), read());
    const snapshot = clone(state);
    const single: Once<Function<int>> = (): int => { return snapshot.value; };
    console.log(single());
}"#,
        "3 3\n3\n",
        None,
    );
}

#[test]
fn typed_local_contracts_reject_mismatch_and_preserve_owned_capture_transfer() {
    reject(
        "function main() { const action: (value: int) => int = (value: string): int => { return value.length; }; }",
        "type",
    );
    reject(
        "function main() { const action: (value: ReadOnly<string>) => int = (value: string): int => { const moved = value; return moved.length; }; }",
        "callback-access",
    );
    reject(
        "type User = { name: string; }; function main() { const user: User = { name: \"Pat\" }; const action: Function<int> = (): int => { return user.name.length; }; console.log(user.name); }",
        "transferred",
    );
}

#[test]
fn once_local_moves_own_capture_storage_and_explicit_copies_preserve_originals() {
    execute(
        r#"type State = { value: int; };
function main() {
    const state: State = { value: 3 };
    {
        const snapshot = clone(state);
        const original = (): int => { return snapshot.value; };
        const single: Once<Function<int>> = original;
        console.log(single());
    }
    state.value = 8;
    console.log(state.value);
}"#,
        "3\n8\n",
        None,
    );
    reject(
        "type State = { value: int; }; function main() { const state: State = { value: 3 }; const original = (): int => { return state.value; }; const single: Once<Function<int>> = original; single(); state.value = 8; }",
        "transferred",
    );
}

#[test]
fn generic_local_annotations_keep_owned_nominal_results() {
    execute(
        r#"type User = { name: string; };
function make<T>(value: T): () => T {
    const action: Once<Function<T>> = (): T => { return value; };
    return action;
}
function main() {
    const action = make<User>({ name: "Pat" });
    const user = action();
    console.log(user.name);
}"#,
        "Pat\n",
        None,
    );
}

#[test]
fn inferred_record_fields_move_callable_bindings_and_retain_program_function_values(){
 support::execute_batch(&[
  ("function work(value:int):int{return value+1;}function main(){const alias=work;const stored={alias};console.log(stored.alias(1));}","2\n"),
  ("function work(value:int):int{return value+1;}const alias=work;const stored={alias};function main(){console.log(alias(2),stored.alias(1));}","3 2\n"),
  ("function work<T>(value:T):T{return value;}const alias=work;const stored={alias};function main(){console.log(alias(2),stored.alias('kept'));}","2 kept\n"),
  ("function main(){const original:Once<Function<int>>=():int=>{return 4;};const stored={original};console.log(stored.original());}","4\n"),
 ],None);
 for source in [
  "function main(){const original:Once<Function<int>>=():int=>{return 4;};const stored={original};original();}",
  "function main(){const original:Once<Function<int>>=():int=>{return 4;};const stored={original};stored.original();stored.original();}",
 ] {support::reject(source,"closure-consumed");}
 support::reject("type State={value:int;};function main(){const state:State={value:3};const original=():int=>{return state.value;};const stored={original};state.value=8;console.log(stored.original());}","capture-conflict");
 support::reject("function inspect(action:ReadOnly<Function<int>>):void{const stored={action};}function main(){}","callback-escape");
 support::reject("function work(value:int):int{return value+1;}let alias=work;const stored={alias};function main(){console.log(stored.alias(1));}","module-state-consume");
 support::reject("function work(value:int):int{return value+1;}const alias=work;function observe<T>(action:Mutable<Function<int,[value:int]>>,stored:T):void{}function main(){observe(alias,{alias});}","conflicting-access");

 support::reject("type State={value:int;};const state:State={value:3};const original=():int=>{return state.value;};const stored={original};function main(){console.log(stored.original());}","module-state-consume");
}
