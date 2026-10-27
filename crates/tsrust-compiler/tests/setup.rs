mod support;
use std::fs;

#[test]
fn doctor_reports_missing_pinned_tools_without_installing_them() {
    let directory = support::TempDirectory::new();
    let home = directory.0.join("rustup");
    fs::create_dir(&home).unwrap();
    let output = support::run(support::cli().args(["doctor", "--json", "--native"])
        .env("RUSTUP_HOME", &home).env("RUSTUP_AUTO_INSTALL", "0"), 30);
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schemaVersion"], 1);
    assert_eq!(report["toolchainReady"], false);
    assert_eq!(report["networkUsed"], false);
    assert_eq!(report["nativeProbe"]["status"], "unchecked");
    assert_eq!(report["applicationDependencies"]["status"], "unchecked");
    assert!(!home.join("toolchains").exists(), "doctor must not install a toolchain");
    let declined = support::run(support::cli().args(["setup", "--toolchain"])
        .env("RUSTUP_HOME", &home).stdin(std::process::Stdio::null()), 10);
    assert!(!declined.status.success());
    assert!(String::from_utf8_lossy(&declined.stderr).contains("--yes"));
    assert!(!home.join("toolchains").exists());
}

#[test]
fn explicit_setup_delegates_to_rustup_and_checks_the_installed_pin() {
    use std::process::Command;
    let directory = support::TempDirectory::new();
    let source = directory.0.join("rustup.rs");
    let stub = directory.0.join(format!("rustup{}", std::env::consts::EXE_SUFFIX));
    let log = directory.0.join("calls.txt");
    let pin = tsrust_compiler::RUST_BACKEND_VERSION;
    // A real process-boundary fixture: no network, global toolchain or config writes.
    fs::write(&source, format!(r#"
use std::io::Write;
fn main() {{
    let args=std::env::args().skip(1).collect::<Vec<_>>();
    let mut log=std::fs::OpenOptions::new().create(true).append(true)
        .open(std::env::var_os("TSRUST_TEST_RUSTUP_LOG").unwrap()).unwrap();
    writeln!(log,"{{}}",args.join("|")).unwrap();
    if args.first().map(String::as_str)==Some("toolchain") {{
        if std::env::var_os("TSRUST_TEST_FAIL_INSTALL").is_some() {{std::process::exit(8);}}
    }} else {{println!("rustc {pin} (test fixture)");}}
}}
"#)).unwrap();
    support::success(&support::run(Command::new("rustup").args(["run", pin, "rustc", "--edition=2024"])
        .arg(&source).arg("-o").arg(&stub), 30));
    let mut command = support::cli();
    command.args(["setup", "--toolchain", "--yes"]).env("PATH", &directory.0)
        .env("TSRUST_TEST_RUSTUP_LOG", &log);
    support::success(&support::run(&mut command, 15));
    let install = format!("toolchain|install|{pin}|--profile|minimal|--no-self-update\n");
    let actual = fs::read_to_string(&log).unwrap().replace("\r\n", "\n");
    assert_eq!(actual, format!("{install}run|{pin}|rustc|--version\n"));
    fs::write(&log, "").unwrap();
    let failed = support::run(command.env("TSRUST_TEST_FAIL_INSTALL", "1"), 15);
    assert!(!failed.status.success());
    assert_eq!(fs::read_to_string(&log).unwrap().replace("\r\n", "\n"), install);
    assert!(String::from_utf8_lossy(&failed.stderr).contains("installation failed"));
}
