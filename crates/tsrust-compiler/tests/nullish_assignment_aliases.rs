mod support;

use tsrust_compiler::check;

fn native(source: &str, expected: &str) {
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    assert!(!rust.contains("unsafe"));
    let directory = support::TempDirectory::new();
    let file = directory.0.join("assignment-aliases.rs");
    std::fs::write(&file, &rust).unwrap();
    let evidence = std::env::var_os("TSR_GATE95_ALIAS_EVIDENCE_DIR").map(std::path::PathBuf::from);
    if let Some(root) = &evidence {
        std::fs::create_dir_all(root).unwrap();
        std::fs::write(root.join("input.tsr"), source).unwrap();
        std::fs::write(root.join("generated.rs"), &rust).unwrap();
        std::fs::write(root.join("expected.txt"), expected).unwrap();
    }
    for level in ["0", "3"] {
        let executable = directory.0.join(format!("assignment-aliases-O{level}"));
        let build = support::run(
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(&file)
                .arg("-o")
                .arg(&executable),
            60,
        );
        if let Some(root) = &evidence {
            std::fs::write(root.join(format!("build-O{level}.stderr")), &build.stderr).unwrap();
        }
        support::success(&build);
        let output = support::run(&mut std::process::Command::new(executable), 15);
        if let Some(root) = &evidence {
            std::fs::write(root.join(format!("stdout-O{level}.txt")), &output.stdout).unwrap();
            std::fs::write(root.join(format!("stderr-O{level}.txt")), &output.stderr).unwrap();
        }
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn assignment_aliases_forward_read_contracts_and_explicit_clones_own_data() {
    native(
        r#"
type Box<T> = { value: T; };
function inspect<T>(box: Box<T>): int { return 1; }
function textLength(text: string): int { return text.length; }
function main() {
    let text: string | null = null;
    const selected = (text ??= "é".repeat(2));
    const forwarded = selected;
    console.log(textLength(forwarded), selected, text);
    let rows: Box<string>[] | undefined = undefined;
    const items = (rows ??= [{ value: "row".repeat(1) }]);
    const alias = items;
    console.log(alias.length, inspect(alias[0]), alias[0].value, rows.length);
    let independentTarget: int[] | null = null;
    const independent = clone(independentTarget ??= [3, 4]);
    independentTarget = [9];
    if (independentTarget !== null) {
        console.log(independent.join("|"), independentTarget.join("|"));
    }
    let outer: string | null = null;
    let inner: string | null = null;
    const chain = (outer ??= clone(inner ??= "pair".repeat(1)));
    console.log(chain, String(inner), String(outer));
    let scoped: string | null = null;
    if (true) {
        const local = (scoped ??= "first".repeat(1));
        const localAlias = local;
        console.log(localAlias, scoped);
    }
    scoped = "second".repeat(1);
    console.log(String(scoped));
}
"#,
        "4 éé éé\n1 1 row 1\n3|4 9\npair pair pair\nfirst first\nsecond\n",
    );
}

#[test]
fn assignment_aliases_reject_transfer_mutation_and_owner_conflicts() {
    for (source, codes) in [
        (
            "function bad():string{let value:string|null=null;const alias=(value??=\"x\");return alias;}function main(){}",
            &["borrowed-place"][..],
        ),
        (
            "function main(){let values:int[]|null=null;const alias=(values??=[1]);alias.push(2);}",
            &["borrowed-place"][..],
        ),
        (
            "function main(){let value:string|null=null;const alias=(value??=\"x\");value=\"other\";console.log(alias);}",
            &["view-conflict"][..],
        ),
        (
            "function bad():string{let value:string|null=null;return value??=\"x\";}function main(){}",
            &["borrowed-place"][..],
        ),
    ] {
        let error = check(source)
            .err()
            .expect("unsafe alias use must be rejected by the compiler");
        assert!(
            codes.contains(&error.code),
            "{}",
            error.render("input.tsr", source)
        );
    }
}
