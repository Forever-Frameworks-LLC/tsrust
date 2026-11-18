// Fault-unwinding behavior that crosses the process boundary: the uncaught
// report on stderr, the exit status and native backtraces. Each program runs
// with the fault mode set explicitly, whatever mode the suite runs in.
mod support;

fn run(mode: &str, fixture: &str, backtrace: bool) -> std::process::Output {
    let directory = support::TempDirectory::new();
    let source = std::fs::read_to_string(format!("tests/fixtures/fault-unwinding/{fixture}")).unwrap();
    let path = directory.source(&support::format_round_trip(&source));
    let mut command = support::cli();
    command.env("TSRUST_FAULT_MODE", mode).env("TSRUST_COLOR", "never").arg("run").arg(&path);
    if backtrace { command.env("RUST_BACKTRACE", "1"); } else { command.env_remove("RUST_BACKTRACE"); }
    support::run(&mut command, 300)
}

#[test]
fn task_faults_leave_pool_channel_and_scope_locks_usable() {
    for mode in ["unwind", "value"] {
        let output = run(mode, "task-fault-locks.tsr", false);
        support::success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "worker\n9223372036854775807\nworker\n1\nworker\n9223372036854775807\n0\n", "{mode}");
    }
}

#[test]
fn capacity_limits_raise_catchable_range_error_faults() {
    let output = run("unwind", "capacity-faults.tsr", false);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "RangeError capacity\n");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("Uncaught RangeError: string repeat capacity overflow at source bytes "), "{stderr}");
    assert!(stderr.contains(".tsr:12:15\n") && !stderr.contains("native backtrace"), "{stderr}");
    let traced = run("unwind", "capacity-faults.tsr", true);
    assert_eq!(traced.status.code(), Some(1));
    assert!(String::from_utf8(traced.stderr).unwrap().contains("native backtrace:\n"));
}
