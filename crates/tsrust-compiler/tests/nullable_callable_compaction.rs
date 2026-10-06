mod support;

#[test]
fn nullable_recursive_result_compaction_keeps_projected_native_origin() {
    let source = "function make(n:int):()=>int{const unused=():()=>int=>{return make(n);};let value=n;if(n>0){return make(n-1);}return ():int=>{return value;};}function main(){const action:(ReadOnly<Function<int>>)|null=make(3);if(action!==null){console.log(action());}}";
    let analysis = tsrust_compiler::check(source).unwrap();
    assert_eq!(
        analysis.to_json(),
        tsrust_compiler::check(source).unwrap().to_json()
    );
    let generated = analysis.emit_rust();
    assert_eq!(
        generated,
        tsrust_compiler::check(source).unwrap().emit_rust()
    );
    let temporary_evidence = support::TempDirectory::new();
    let evidence = std::env::var_os("TSR_GATE97_EVIDENCE_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| temporary_evidence.0.clone());
    std::fs::create_dir_all(&evidence).unwrap();
    std::fs::write(evidence.join("input.tsr"), source).unwrap();
    std::fs::write(evidence.join("generated.rs"), &generated).unwrap();
    std::fs::write(evidence.join("checked.json"), analysis.to_json()).unwrap();
    let directory = support::TempDirectory::new();
    let rust = directory.0.join("source.rs");
    std::fs::write(&rust, generated).unwrap();
    for level in ["0", "3"] {
        let binary = directory.0.join(format!("compaction-{level}"));
        support::success(&support::run(
            support::rustc()
                .args(["--edition=2024", "-C", &format!("opt-level={level}")])
                .arg(&rust)
                .arg("-o")
                .arg(&binary),
            60,
        ));
        let output = support::run(&mut std::process::Command::new(binary), 15);
        std::fs::write(
            evidence.join(format!("stdout-O{level}.txt")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            evidence.join(format!("stderr-O{level}.txt")),
            &output.stderr,
        )
        .unwrap();
        support::success(&output);
        assert_eq!(output.stdout, b"0\n");
        assert!(output.stderr.is_empty());
    }
}
