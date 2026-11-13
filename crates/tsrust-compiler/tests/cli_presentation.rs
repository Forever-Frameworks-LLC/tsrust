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

#[cfg(unix)]
#[test]
fn studio_check_stops_when_its_launching_process_exits() {
    use std::io::{BufRead, BufReader, Read};
    use std::process::{Command, Stdio};
    use std::time::Duration;

    let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/studio/src/main.tsrx");
    let mut owner = Command::new("sh")
        .args(["-c", r#""$1" check "$2" --json --progress & echo $!; read hold"#, "owner"])
        .arg(env!("CARGO_BIN_EXE_tsrust")).arg(source)
        .env("TSRUST_CHECK_WATCH_PARENT", "1")
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped())
        .spawn().unwrap();
    let mut pid = String::new();
    BufReader::new(owner.stdout.take().unwrap()).read_line(&mut pid).unwrap();
    let mut progress = BufReader::new(owner.stderr.take().unwrap());
    let mut line = String::new();
    progress.read_line(&mut line).unwrap();
    assert!(line.contains("Checking"), "{line}");
    drop(owner.stdin.take());
    owner.wait().unwrap();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || { let _ = tx.send(progress.read_to_end(&mut Vec::new())); });
    if rx.recv_timeout(Duration::from_secs(3)).is_err() {
        let _ = Command::new("kill").arg(pid.trim()).status();
        panic!("orphaned Studio check kept its diagnostic pipe open");
    }
}
