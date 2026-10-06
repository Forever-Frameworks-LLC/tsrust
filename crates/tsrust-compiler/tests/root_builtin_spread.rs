mod support;
use tsrust_compiler::check;

fn recorded(
    directory: &std::path::Path,
    label: &str,
    command: &mut std::process::Command,
    timeout: u64,
) -> std::process::Output {
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
    let stamp = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    };
    let argv = std::iter::once(command.get_program())
        .chain(command.get_args())
        .map(|value| format!("{:?}", value.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(",");
    let start = stamp();
    let elapsed = Instant::now();
    let stdout = directory.join(format!("{label}.stdout"));
    let stderr = directory.join(format!("{label}.stderr"));
    command
        .stdout(std::fs::File::create(&stdout).unwrap())
        .stderr(std::fs::File::create(&stderr).unwrap());
    let mut child = command.spawn().unwrap();
    let pid = child.id();
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if elapsed.elapsed() > Duration::from_secs(timeout) {
            child.kill().unwrap();
            panic!("native child timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    std::fs::write(directory.join(format!("{label}.json")),format!("{{\"argv\":[{argv}],\"pid\":{pid},\"startUtcEpochMillis\":{start},\"endUtcEpochMillis\":{},\"exitCode\":{}}}\n",stamp(),status.code().map_or("null".into(),|value|value.to_string()))).unwrap();
    std::process::Output {
        status,
        stdout: std::fs::read(stdout).unwrap(),
        stderr: std::fs::read(stderr).unwrap(),
    }
}

fn native(source: &str, expected: &str) {
    static CASE_ID: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let persistent = std::env::var_os("TSR_GATE100_NATIVE_DIR");
    let temporary = persistent.is_none().then(support::TempDirectory::new);
    let directory = if let Some(root) = persistent {
        let root = std::path::PathBuf::from(root);
        let case = CASE_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        if case == 0 {
            root
        } else {
            root.join(format!("case-{case}"))
        }
    } else {
        temporary.as_ref().unwrap().0.clone()
    };
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(directory.join("input.tsr"), source).unwrap();
    std::fs::write(directory.join("expected.txt"), expected).unwrap();
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    let rust = analysis.emit_rust();
    std::fs::write(directory.join("generated.rs"), &rust).unwrap();
    assert!(!rust.contains("unsafe"));
    if std::env::var_os("TSR_GATE99_EMIT_ONLY").is_some() {
        return;
    }
    for level in ["0", "3"] {
        let path = directory.join(format!("native-O{level}"));
        let build = recorded(
            &directory,
            &format!("build-O{level}"),
            support::rustc()
                .args([
                    "--edition=2024",
                    "-C",
                    &format!("opt-level={level}"),
                    "-C",
                    "overflow-checks=on",
                ])
                .arg(directory.join("generated.rs"))
                .arg("-o")
                .arg(&path),
            60,
        );
        std::fs::write(
            directory.join(format!("build-O{level}-stdout.txt")),
            &build.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("build-O{level}-stderr.txt")),
            &build.stderr,
        )
        .unwrap();
        support::success(&build);
        let output = recorded(
            &directory,
            &format!("run-O{level}"),
            &mut std::process::Command::new(path),
            15,
        );
        std::fs::write(
            directory.join(format!("stdout-O{level}.txt")),
            &output.stdout,
        )
        .unwrap();
        std::fs::write(
            directory.join(format!("stderr-O{level}.txt")),
            &output.stderr,
        )
        .unwrap();
        support::success(&output);
        assert_eq!(output.stdout, expected.as_bytes());
        assert!(output.stderr.is_empty());
    }
}


#[test]
fn root_affected_callback_native() { native(include_str!("fixtures/call-spread/callback-builtins.tsr"),include_str!("fixtures/call-spread/callback-builtins.stdout")); }

#[test]
fn root_affected_edges_native() { native(include_str!("fixtures/call-spread/spread-edges.tsr"),include_str!("fixtures/call-spread/spread-edges.stdout")); }

#[test]
fn root_affected_interop_native() { native(include_str!("fixtures/call-spread/peer-interop.tsr"),include_str!("fixtures/call-spread/peer-interop.stdout")); }

#[test]
fn root_affected_literalstrings_native() { native(include_str!("fixtures/call-spread/literalstrings-builtins.tsr"),include_str!("fixtures/call-spread/literalstrings-builtins.stdout")); }

#[test]
fn root_affected_protected_native() { native(include_str!("fixtures/call-spread/protected-builtins.tsr"),include_str!("fixtures/call-spread/protected-builtins.stdout")); }
