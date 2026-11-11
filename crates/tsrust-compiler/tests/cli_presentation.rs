mod support;

#[test]
fn help_lists_commands_and_test_help_without_discovery() {
    for args in [vec!["--help"], vec!["test", "--help"]] {
        let output = support::run(support::cli().args(args), 10);
        support::success(&output);
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("Commands:"));
        assert!(text.contains("tsrust test [files/directories]"));
        assert!(!text.contains('\u{1b}'));
    }
}

#[test]
fn usage_errors_are_concise_and_color_policy_applies() {
    let output = support::run(support::cli().args(["unknown", "--color", "always"]), 10);
    assert!(!output.status.success());
    let text = String::from_utf8(output.stderr).unwrap();
    assert!(text.contains("Unknown command: unknown"));
    assert!(text.contains("tsrust --help"));
    assert!(!text.contains("error[toolchain]"));
    assert!(text.contains('\u{1b}'));
    let output = support::run(support::cli().args(["--help", "--color=always"]), 10);
    support::success(&output);
    assert!(output.stdout.contains(&27));
}

#[test]
fn native_test_report_respects_explicit_color() {
    let directory = support::TempDirectory::new();
    let path = directory.source("import {test,expect} from \"std:testing\"; test(\"sum\",():void=>{expect(2).toBe(2);});");
    for mode in ["always", "never"] {
        let output = support::run(support::cli().arg("test").arg(&path).args(["--color",mode]),60);
        support::success(&output);
        assert_eq!(output.stdout.contains(&27), mode == "always");
    }
}


#[test]
fn project_check_and_format_respect_color_without_styling_machine_output() {
    let directory = support::TempDirectory::new();
    let source = directory.source("function main() {}\n");
    for command in ["check", "format"] {
        for mode in ["always", "never"] {
            let output = support::run(support::cli().arg(command).arg(&directory.0).args(["--color", mode, "--progress"]).env("NO_COLOR", "1"), 10);
            support::success(&output);
            assert_eq!(output.stdout.contains(&27), mode == "always", "{command} stdout");
            assert_eq!(output.stderr.contains(&27), mode == "always", "{command} progress");
        }
    }
    let json = support::run(support::cli().arg("check").arg(&directory.0).args(["--json", "--color=always"]), 10);
    support::success(&json);
    assert!(!json.stdout.contains(&27));
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&json.stdout).unwrap()["ok"], true);
    let text = support::run(support::cli().arg("format").arg(source).arg("--color=always"), 10);
    support::success(&text);
    assert!(!text.stdout.contains(&27));
    assert!(String::from_utf8(text.stdout).unwrap().starts_with("function main()"));
}
