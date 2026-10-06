mod support;
use tsrust_compiler::check;

fn nested(receiver: &str, leaf: &str, depth: usize) -> String {
    format!(
        "function main(){{const x={}{leaf}{};}}",
        format!("{receiver}.concat(").repeat(depth),
        ")".repeat(depth)
    )
}

#[test]
fn nested_string_and_array_concat_keep_normal_stack_and_depth_diagnostics() {
    for (receiver, leaf) in [("\"x\"", "String(7)"), ("[1]", "[2]")] {
        let source = nested(receiver, leaf, 100).replace(";}", ";console.log(x.length);}");
        let analysis = check(&source)
            .unwrap_or_else(|error| panic!("{}", error.render("concat-depth.tsr", &source)));
        let emitted = analysis.emit_rust();
        if std::env::var_os("TSR_SKIP_CONCAT_DEPTH_NATIVE").is_none() {
            let directory = support::TempDirectory::new();
            let rust = directory.0.join("concat-depth.rs");
            let executable = directory.0.join("concat-depth");
            std::fs::write(&rust, emitted).unwrap();
            support::success(&support::run(
                support::rustc()
                    .args(["--edition=2024", "-C", "opt-level=0"])
                    .arg(&rust)
                    .arg("-o")
                    .arg(&executable),
                60,
            ));
            let output = support::run(&mut std::process::Command::new(&executable), 10);
            support::success(&output);
            assert_eq!(output.stdout, b"101\n");
            assert!(output.stderr.is_empty());
        }
        let source = nested(receiver, leaf, 200);
        let error = check(&source).err().expect("expected diagnostic");
        assert_eq!(error.code, "unsupported");
        assert!(
            error.message.contains("nesting") && error.message.contains("128"),
            "{}",
            error.message
        );
    }
    for source in [
        "function main(){const x:int[]=[].concat(1,[2]);console.log(x.join(\"|\"));}",
        "function main(){const x:number[]=[].concat(1, [2.5]);console.log(x.join(\"|\"));}",
        "function main(){const x=[[1]].concat([[2]]);console.log(x.join(\"|\"));}",
        "function main(){const x=\"é\".concat(String(7),\"😀\");console.log(x);}",
    ] {
        assert!(!check(source).unwrap().emit_rust().is_empty());
    }
}
