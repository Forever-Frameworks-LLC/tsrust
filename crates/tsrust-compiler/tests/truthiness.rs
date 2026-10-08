mod support;

#[test]
fn project_truthiness_assertions_preserve_skipped_effects() {
    let source = support::format_round_trip(include_str!("fixtures/truthiness/values.tsr"));
    let directory = support::TempDirectory::new();
    let output = support::run(support::cli().arg("run").arg(directory.source(&source)), 60);
    support::success(&output);
    assert!(output.stdout.is_empty());
}

#[test]
fn logical_assignment_results_preserve_move_and_type_diagnostics() {
    for (source, code) in [
        ("function main(){let text=\"x\";const moved=(text||=\"y\");console.log(text);}", "transferred"),
        ("function main(){const text=\"\";text||=\"y\";}", "unsupported"),
        ("function main(){let text=\"\";text||=1;}", "type"),
        ("function main(){console.log(!console.log(1));}", "type"),
    ] {
        let error = tsrust_compiler::check(source).err().unwrap_or_else(|| panic!("Expected {code}: {source}"));
        assert_eq!(error.code, code, "{source}: {}", error.message);
    }
}

#[test]
fn conditions_and_negation_use_scalar_and_collection_truthiness() {
    support::execute(
        r#"
        type Value = { value: int; };
        function main() {
            console.log(!"", !" ", !0, !1, !NaN, !null, !undefined);
            if (1) console.log(1 ? 2 : 3);
            const values: int[] = [];
            const record: Value = { value: 0 };
            if (values) console.log("array");
            if (record) console.log("record");
            let count = 3;
            while (count) count -= 1;
            do { count += 1; } while (0);
            for (; count; count -= 1) console.log(count);
            const text: string | undefined = "yes";
            if (text) console.log(text.length);
            console.log("" ? 1 : 2, "yes" ? 3 : 4);
            let scalar = 0;
            scalar ||= 5;
            scalar &&= 6;
            console.log(scalar);
        }
        "#,
        "true false true false true true true\n2\narray\nrecord\n1\n3\n2 3\n6\n",
        None,
    );
}

#[test]
fn owned_logical_assignment_preserves_target_and_short_circuits() {
    support::execute(
        r#"
        type Profile = { name: string; };
        function main() {
            let text = "";
            text ||= "default";
            text ||= "ignored";
            console.log(text);
            text &&= "changed";
            console.log(text);
            let nullable: string | undefined = undefined;
            nullable ||= "present";
            console.log(nullable);
            const profile: Profile = { name: "" };
            profile.name ||= "anonymous";
            console.log(profile.name);
            console.log(text ||= "ignored", text);
        }
        "#,
        "default\nchanged\npresent\nanonymous\nchanged changed\n",
        None,
    );
}
