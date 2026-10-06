mod support;

#[test]
fn assertions_remove_absence_and_preserve_postfix_chains_and_formatting() {
    let source = r#"type Row={name:string;};function main(){
        const users=['Alice','Bob','Charlie'];const lastUser=users.pop()!;console.log(lastUser);
        console.log(users.pop()!.toUpperCase());
        const rows:Row[]=[{name:'Pat'}];const row=rows.pop()!;console.log(row.name);
        const n:int|null=7;console.log(n!+1,n!,!false);
        const text:string|null='repeat';console.log(text!.toUpperCase());console.log(text!);
        const values:int[]|null=[2,3];console.log(values!.length);
        const ordinary=4;console.log(ordinary!);
    }"#;
    support::execute(source, "Charlie\nBOB\nPat\n8 7 true\nREPEAT\nrepeat\n2\n4\n", None);
    support::execute("function main(){const users=['Pat'];console.log(users.pop()!);}","Pat\n",None);
    support::execute("function main(){const action:(()=>int)|null=():int=>7;console.log(action!());}","7\n",None);
    support::execute("function main(){const values:int[]|null=[2,3];values!.push(4);console.log(values!.join('|'),values![0]);}","2|3|4 2\n",None);
    let formatted=tsrust_compiler::format(source).unwrap();
    assert!(formatted.contains("users.pop()!"), "{formatted}");
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(),formatted);
    support::execute(&formatted, "Charlie\nBOB\nPat\n8 7 true\nREPEAT\nrepeat\n2\n4\n", None);
}

#[test]
fn assertions_do_not_clone_move_twice_or_own_borrowed_payloads() {
    for source in ["function main(){const x=null!;}","function main(){const x=undefined!;}"] {
        let error=tsrust_compiler::check(source).err().unwrap();
        assert_eq!(error.code,"non-null");
        assert!(error.message.contains("fallback"));
        assert_eq!(&source[error.span.start..error.span.end],"!");
    }
    let source="function main(){const words=['Pat'];const selected=words.at(0)!;}";
    assert!(tsrust_compiler::check(source).is_err());
    let moved="function main(){const value:string|null='Pat';const taken=value!;console.log(value);}";
    assert_eq!(tsrust_compiler::check(moved).err().unwrap().code,"transferred");
    support::execute("function main(){const words=['Pat'];const selected=clone(words.at(0)!);console.log(selected,words[0]);}","Pat Pat\n",None);
}

#[test]
fn absent_assertions_fail_with_a_clear_native_message() {
    let source="function main(){const users:string[]=[];const lastUser=users.pop()!;console.log(lastUser);}";
    let analysis=tsrust_compiler::check(source).unwrap();
    let dir=support::TempDirectory::new();
    let input=dir.0.join("assertion.rs");
    let binary=dir.0.join("assertion");
    std::fs::write(&input,analysis.emit_rust()).unwrap();
    support::success(&support::run(support::rustc().args(["--edition=2024","-Awarnings"]).arg(input).arg("-o").arg(&binary),60));
    let output=support::run(&mut std::process::Command::new(binary),15);
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr).unwrap().contains("Non-null assertion failed: value is null or undefined"));
}

#[test]
fn repeated_assertions_keep_normal_depth_limits() {
    let ordinary=format!("function main(){{const x=7{};console.log(x);}}","!".repeat(64));
    tsrust_compiler::check(&ordinary).unwrap().emit_rust();
    let excessive=format!("function main(){{const x=7{};}}","!".repeat(200));
    let error=tsrust_compiler::check(&excessive).err().unwrap();
    assert_eq!(error.code,"unsupported");
    assert!(error.message.contains("nesting"));
}
