#![allow(dead_code)] // Integration-test binaries use different parts of this shared support.

pub mod closure_corpus;
pub mod corpus;

use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tsrust_compiler::{Diagnostic, Mode, check};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

pub struct TempDirectory(pub PathBuf);
impl TempDirectory {
    pub fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for _ in 0..100 {
            let id = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("tsrust-test-{}-{stamp}-{id}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Self(path),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => panic!("Cannot create test directory: {error}"),
            }
        }
        panic!("Cannot allocate a test directory");
    }
    pub fn source(&self, source: &str) -> PathBuf {
        let path = self.0.join("input.tsr");
        std::fs::write(&path, source).unwrap();
        path
    }
}
impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/ownership")
}

pub fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_tsrust"))
}

pub fn rustc() -> Command {
    let mut command = Command::new("rustup");
    command.args(["run", tsrust_compiler::RUST_BACKEND_VERSION, "rustc"]);
    command
}

/// Preserve the old harness's timeouts without adding runtime dependencies.
pub fn run(command: &mut Command, seconds: u64) -> Output {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn test process");
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let err = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).unwrap();
        bytes
    });
    let start = Instant::now();
    let mut timed_out = false;
    let status = loop {
        if let Some(status) = child.try_wait().expect("poll test process") {
            break status;
        }
        if start.elapsed() >= Duration::from_secs(seconds) {
            timed_out = true;
            let _ = child.kill();
            break child.wait().expect("reap timed-out process");
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let output = Output {
        status,
        stdout: out.join().unwrap(),
        stderr: err.join().unwrap(),
    };
    assert!(!timed_out, "Test process exceeded {seconds}s: {command:?}");
    output
}

pub fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn reject(source: &str, code: &str) -> Diagnostic {
    let diagnostic = check(source).err().unwrap_or_else(|| panic!("source must be rejected: {source}"));
    assert_eq!(diagnostic.code, code, "{source}\n{}", diagnostic.message);
    let directory = TempDirectory::new();
    let path = directory.source(source);
    let output = run(cli().arg("check").arg(path).arg("--json"), 10);
    assert_eq!(output.status.code(), Some(1));
    let diagnostics=tsrust_compiler::diagnostics(source);
    assert!(diagnostics.iter().any(|error|error.to_json(source)==diagnostic.to_json(source)), "Recovery must retain the strict diagnostic");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{{\"ok\":false,\"diagnostics\":[{}]}}\n",diagnostics.iter().map(|error|error.to_json(source)).collect::<Vec<_>>().join(","))
    );
    assert!(output.stderr.is_empty());
    diagnostic
}

/// Format in memory and require a stable second pass. Native fixtures execute
/// this result so feature acceptance exercises the editor's whitespace output.
pub fn format_round_trip(source: &str) -> String {
    let formatted = tsrust_compiler::format(source)
        .unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    assert_eq!(tsrust_compiler::format(&formatted).unwrap(), formatted,
        "Formatter must be idempotent for executable fixture");
    formatted
}

pub fn execute(source: &str, expected: &str, contracts: Option<&[(&str, Mode)]>) -> String {
    let formatted = format_round_trip(source);
    let source = formatted.as_str();
    let analysis =
        check(source).unwrap_or_else(|error| panic!("{}", error.render("input.tsr", source)));
    if let Some(contracts) = contracts {
        let expected: BTreeMap<_, _> = contracts
            .iter()
            .map(|(name, mode)| (name.to_string(), *mode))
            .collect();
        assert_eq!(analysis.contracts(), expected);
    }
    let directory = TempDirectory::new();
    let path = directory.source(source);
    let checked = run(cli().arg("check").arg(&path).arg("--json"), 10);
    success(&checked);
    assert_eq!(
        String::from_utf8(checked.stdout).unwrap(),
        format!("{}\n", analysis.to_json())
    );
    assert!(checked.stderr.is_empty());
    let emitted = run(cli().arg("emit").arg(&path), 10);
    success(&emitted);
    assert!(emitted.stderr.is_empty());
    let rust = String::from_utf8(emitted.stdout).unwrap();
    assert_eq!(rust, analysis.emit_rust());
    let rust_path = directory.0.join("program.rs");
    std::fs::write(&rust_path, &rust).unwrap();
    let binary = directory
        .0
        .join(format!("program{}", std::env::consts::EXE_SUFFIX));
    let compiled = run(
        rustc()
            .arg("--edition=2024")
            .arg(&rust_path)
            .arg("-o")
            .arg(&binary),
        60,
    );
    assert!(
        compiled.status.success(),
        "{}\n{rust}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let result = run(&mut Command::new(binary), 10);
    success(&result);
    assert_eq!(result.stdout, expected.as_bytes());
    assert!(result.stderr.is_empty());
    rust
}
