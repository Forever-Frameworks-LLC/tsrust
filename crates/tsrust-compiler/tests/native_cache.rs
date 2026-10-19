mod support;
use tsrust_compiler::native_cache::NativeCache;
#[test]
#[ignore = "subprocess worker, invoked with isolated paths by the process contention test"]
fn cache_publication_process_worker() {
    use std::io::Read;
    let root = std::env::var_os("TSRUST_TEST_PUBLICATION_ROOT").unwrap();
    let artifact = std::env::var_os("TSRUST_TEST_PUBLICATION_ARTIFACT").unwrap();
    std::io::stdin().read_exact(&mut [0]).unwrap();
    let cache = NativeCache::new(std::path::Path::new(&root), &[b"process build"]);
    cache.publish(std::path::Path::new(&artifact)).unwrap();
    assert_eq!(
        std::fs::read(cache.lookup().unwrap()).unwrap(),
        vec![37; 256 * 1024]
    );
}
#[test]
fn separate_processes_publish_and_repair_one_complete_entry() {
    use std::{
        io::Write,
        process::{Command, Stdio},
        time::{Duration, Instant},
    };
    for damaged in [false, true] {
        let directory = support::TempDirectory::new();
        let root = directory.0.join("cache");
        let artifact = directory.0.join("compiled");
        let contents = vec![37; 256 * 1024];
        std::fs::write(&artifact, &contents).unwrap();
        let cache = NativeCache::new(&root, &[b"process build"]);
        if damaged {
            cache.publish(&artifact).unwrap();
            std::fs::write(cache.lookup().unwrap(), b"damaged").unwrap();
        }
        let mut children = Vec::new();
        for _ in 0..8 {
            children.push(
                Command::new(std::env::current_exe().unwrap())
                    .args(["--ignored", "--exact", "cache_publication_process_worker"])
                    .env("TSRUST_TEST_PUBLICATION_ROOT", &root)
                    .env("TSRUST_TEST_PUBLICATION_ARTIFACT", &artifact)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
        }
        for child in &mut children {
            child.stdin.take().unwrap().write_all(&[1]).unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(15);
        let mut failed = false;
        for child in &mut children {
            loop {
                if let Some(status) = child.try_wait().unwrap() {
                    failed |= !status.success();
                    break;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    child.wait().unwrap();
                    failed = true;
                    break;
                }
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        assert!(
            !failed,
            "publication subprocess failed or timed out (damaged={damaged})"
        );
        assert_eq!(std::fs::read(cache.lookup().unwrap()).unwrap(), contents);
        let entries = std::fs::read_dir(root.join("v2"))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .all(|entry| !entry.file_name().to_string_lossy().starts_with(".staging-"))
        );
    }
}
#[test]
fn concurrent_publishers_leave_one_complete_artifact_and_no_staging_directories() {
    concurrent_publication(false);
}
#[test]
fn concurrent_publishers_repair_a_damaged_entry_without_removing_the_winner() {
    concurrent_publication(true);
}
fn concurrent_publication(damaged: bool) {
    use std::sync::{Arc, Barrier};
    let directory = support::TempDirectory::new();
    let root = directory.0.join("cache");
    let artifact = directory.0.join("compiled");
    let contents = vec![42; 256 * 1024];
    std::fs::write(&artifact, &contents).unwrap();
    if damaged {
        let cache = NativeCache::new(&root, &[b"same build"]);
        cache.publish(&artifact).unwrap();
        std::fs::write(cache.lookup().unwrap(), b"damaged").unwrap();
        assert!(cache.lookup().is_none());
    }
    let barrier = Arc::new(Barrier::new(8));
    std::thread::scope(|scope| {
        let mut publishers = Vec::new();
        for _ in 0..8 {
            let barrier = barrier.clone();
            let root = &root;
            let artifact = &artifact;
            publishers.push(scope.spawn(move || {
                let cache = NativeCache::new(root, &[b"same build"]);
                barrier.wait();
                cache.publish(artifact).unwrap();
                assert_eq!(
                    std::fs::read(cache.lookup().unwrap()).unwrap(),
                    vec![42; 256 * 1024]
                );
            }));
        }
        for publisher in publishers {
            publisher.join().unwrap();
        }
    });
    let entries = std::fs::read_dir(root.join("v2"))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(entries.len(), 2); // One complete artifact directory and its persistent lock file.
    assert!(
        entries
            .iter()
            .all(|entry| !entry.file_name().to_string_lossy().starts_with(".staging-"))
    );
    assert_eq!(
        std::fs::read(NativeCache::new(&root, &[b"same build"]).lookup().unwrap()).unwrap(),
        contents
    );
}
#[test]
fn cache_validates_identity_and_publishes_complete_artifacts() {
    let directory = support::TempDirectory::new();
    let artifact = directory.0.join("compiled");
    std::fs::write(&artifact, b"native artifact").unwrap();
    let root = directory.0.join("cache");
    let cache = NativeCache::new(&root, &[b"source", b"debug", b"backend"]);
    assert!(cache.lookup().is_none());
    cache.publish(&artifact).unwrap();
    let path_features = NativeCache::new(&root, &[b"source", b"debug", b"backend", b"path"]);
    path_features.publish(&artifact).unwrap();
    assert!(path_features.lookup().is_some());
    assert!(
        NativeCache::new(&root, &[b"source", b"debug", b"backend", b"fs"])
            .lookup()
            .is_none()
    );
    assert!(
        NativeCache::new(&root, &[b"source", b"debug", b"backend", b""])
            .lookup()
            .is_none()
    );
    assert_eq!(
        std::fs::read(cache.lookup().unwrap()).unwrap(),
        b"native artifact"
    );
    assert!(
        NativeCache::new(&root, &[b"source", b"release", b"backend"])
            .lookup()
            .is_none()
    );
    assert!(
        NativeCache::new(&root, &[b"changed", b"debug", b"backend"])
            .lookup()
            .is_none()
    );
    cache.publish(&artifact).unwrap();
    let damaged = cache.lookup().unwrap();
    std::fs::write(damaged, b"corrupt artifact").unwrap();
    assert!(cache.lookup().is_none());
    cache.publish(&artifact).unwrap();
    assert_eq!(
        std::fs::read(cache.lookup().unwrap()).unwrap(),
        b"native artifact"
    );
    let executable = cache.lookup().unwrap();
    std::fs::write(
        executable.parent().unwrap().join("identity"),
        b"wrong identity",
    )
    .unwrap();
    assert!(cache.lookup().is_none());
}
#[cfg(unix)]
#[test]
fn warm_cli_build_reuses_native_compilation_and_invalidates_changed_inputs() {
    use std::{os::unix::fs::PermissionsExt, process::Command};
    let directory = support::TempDirectory::new();
    let wrappers = directory.0.join("bin");
    std::fs::create_dir(&wrappers).unwrap();
    let real = Command::new("which").arg("rustup").output().unwrap();
    assert!(real.status.success());
    let real = String::from_utf8(real.stdout).unwrap().trim().to_owned();
    let log = directory.0.join("compile.log");
    let wrapper = wrappers.join("rustup");
    // Paths are passed via environment, so shell metacharacters are never interpolated.
    std::fs::write(&wrapper,"#!/bin/sh\nif [ \"$4\" != \"--version\" ]; then echo compile >> \"$TSRUST_TEST_LOG\"; fi\nexec \"$TSRUST_TEST_RUSTUP\" \"$@\"\n").unwrap();
    std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755)).unwrap();
    let input=directory.source("import { basename } from 'std:path';function main(){console.log(basename('/tmp/report.txt'));}");
    let cache = directory.0.join("cache");
    let build = |release: bool| {
        let mut command = support::cli();
        command
            .arg("run")
            .arg(&input)
            .env(
                "PATH",
                std::env::join_paths(
                    std::iter::once(wrappers.clone())
                        .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
                )
                .unwrap(),
            )
            .env("TSRUST_TEST_RUSTUP", &real)
            .env("TSRUST_TEST_LOG", &log)
            .env("TSRUST_CACHE_DIR", &cache);
        if release {
            command.arg("--release");
        }
        let output = support::run(&mut command, 60);
        support::success(&output);
        String::from_utf8(output.stdout).unwrap()
    };
    // Cold profiles compile runtime plus application; edits reuse matching runtime.
    assert_eq!(build(false), "report.txt\n");
    assert_eq!(build(false), "report.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 2);
    assert_eq!(build(true), "report.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 4);
    std::fs::write(&input,"import { basename } from 'std:path';function main(){console.log(basename('/tmp/other.txt'));}").unwrap();
    assert_eq!(build(false), "other.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 5);
    let dependency = directory.0.join("helper.tsr");
    let original = "import { basename } from 'std:path';export function name():string{return basename('/tmp/report.txt');}";
    std::fs::write(&dependency, original).unwrap();
    std::fs::write(
        &input,
        "import { name } from './helper.tsr';function main(){console.log(name());}",
    )
    .unwrap();
    assert_eq!(build(false), "report.txt\n");
    assert_eq!(build(false), "report.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 6);
    let changed = original.replace("report.txt", "changed.txt");
    std::fs::write(&dependency, &changed).unwrap();
    assert_eq!(build(false), "changed.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 7);
    // Even unchanged generated Rust must not erase source-graph identity.
    std::fs::write(&dependency, format!("// dependency revision\n{changed}")).unwrap();
    assert_eq!(build(false), "changed.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 8);
    assert_eq!(build(false), "changed.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 8);
    std::fs::write(
        &dependency,
        "export function name():string{return 'changed.txt';}",
    )
    .unwrap();
    assert_eq!(build(false), "changed.txt\n");
    assert_eq!(std::fs::read_to_string(&log).unwrap().lines().count(), 10);
    let emitted = support::run(support::cli().arg("emit").arg(&input), 10);
    support::success(&emitted);
    assert!(
        !String::from_utf8(emitted.stdout)
            .unwrap()
            .contains("tsrust_native_path")
    );
}

#[cfg(unix)]
#[test]
fn std_project_frontend_is_backend_independent_and_backend_errors_leave_no_artifact() {
    use std::os::unix::fs::PermissionsExt;
    let directory = support::TempDirectory::new();
    let dependency = directory.0.join("helper.tsr");
    std::fs::write(&dependency, "import { basename } from 'std:path';export function name(path:string):string{return basename(path);}").unwrap();
    let source = directory.source("import { name } from './helper.tsr';function main(){console.log(name('/tmp/report.txt'));}");
    for operation in ["check", "emit"] {
        let output = support::run(
            support::cli().arg(operation).arg(&source).env("PATH", ""),
            10,
        );
        support::success(&output);
        assert!(output.stderr.is_empty());
        if operation == "emit" {
            assert!(
                String::from_utf8(output.stdout)
                    .unwrap()
                    .contains("tsrust_std_path_basename")
            );
        }
    }
    let wrappers = directory.0.join("bin");
    std::fs::create_dir(&wrappers).unwrap();
    let mock = wrappers.join("rustup");
    let log = directory.0.join("backend.log");
    std::fs::write(&mock, "#!/bin/sh\nprintf '%s\\n' \"$@\" >> \"$TSRUST_TEST_BACKEND_LOG\"\nif [ -n \"$TSRUST_TEST_BACKEND_UNAVAILABLE\" ]; then printf 'toolchain unavailable\\n' >&2; exit 1; fi\nprintf 'rustc 0.0.0 (wrong backend)\\n'\n").unwrap();
    std::fs::set_permissions(&mock, std::fs::Permissions::from_mode(0o755)).unwrap();
    for (path, expected, unavailable) in [
        (std::path::Path::new(""), "Cannot invoke pinned Rust", ""),
        (wrappers.as_path(), "Expected Rust", ""),
        (wrappers.as_path(), "is unavailable", "1"),
    ] {
        let binary = directory.0.join("program");
        let output = support::run(
            support::cli()
                .arg("build")
                .arg(&source)
                .arg("-o")
                .arg(&binary)
                .env("PATH", path)
                .env("CARGO_HOME", directory.0.join("missing-cargo-home"))
                .env("TSRUST_TEST_BACKEND_LOG", &log)
                .env("TSRUST_TEST_BACKEND_UNAVAILABLE", unavailable),
            10,
        );
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("check/emit do not require it"), "{error}");
        assert!(!binary.exists());
    }
    assert_eq!(
        std::fs::read_to_string(&log).unwrap(),
        format!(
            "run\n{}\nrustc\n--version\n",
            tsrust_compiler::RUST_BACKEND_VERSION
        )
        .repeat(2)
    );
}

#[cfg(unix)]
#[test]
fn cli_run_finds_rustup_in_cargo_home_when_editor_path_omits_it() {
    let directory = support::TempDirectory::new();
    let source = directory.source("function main(){console.log(42);}");
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| {
            std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join(".cargo")
        });
    let home = directory.0.join("editor home");
    std::fs::create_dir(&home).unwrap();
    std::os::unix::fs::symlink(&cargo_home, home.join(".cargo")).unwrap();
    for explicit_cargo_home in [true, false] {
        let mut command = support::cli();
        command.arg("run").arg(&source).env("PATH", "/usr/bin:/bin");
        if explicit_cargo_home {
            command.env("CARGO_HOME", &cargo_home);
        } else {
            command.env_remove("CARGO_HOME").env("HOME", &home);
        }
        let output = support::run(&mut command, 60);
        support::success(&output);
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "42\n");
    }
}

#[test]
fn default_run_cleans_artifacts_after_success_and_runtime_failure() {
    let directory = support::TempDirectory::new();
    let temporary = directory.0.join("temporary builds");
    std::fs::create_dir(&temporary).unwrap();
    for (source, successful) in [
        (
            "import { basename } from 'std:path'; function main(){console.log(basename('/tmp/report.txt'));}",
            true,
        ),
        (
            "function main(){let value: int = 9223372036854775807; console.log(value + 1);}",
            false,
        ),
    ] {
        let input = directory.source(source);
        let output = support::run(
            support::cli()
                .arg("run")
                .arg(&input)
                .current_dir(&directory.0)
                .env_remove("TSRUST_CACHE_DIR")
                .env("TMPDIR", &temporary)
                .env("TMP", &temporary)
                .env("TEMP", &temporary),
            60,
        );
        assert_eq!(output.status.success(), successful, "{output:?}");
        if successful {
            assert_eq!(output.stdout, b"report.txt\n");
        } else {
            assert!(
                String::from_utf8_lossy(&output.stderr).contains("overflow"),
                "{output:?}"
            );
        }
        assert!(std::fs::read_dir(&temporary).unwrap().all(|entry| {
            !entry
                .unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("tsrust-")
        }));
        let mut entries = std::fs::read_dir(&directory.0)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect::<Vec<_>>();
        entries.sort();
        assert_eq!(
            entries,
            vec![
                std::ffi::OsString::from("input.tsr"),
                std::ffi::OsString::from("temporary builds")
            ]
        );
    }
}

#[test]
fn callback_error_layout_identity_is_stable_across_ui_compiler_processes() {
    let entry=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/ui-conditional-mounting/src/main.tsr");
    let mut reference=None;
    for _ in 0..4 {
        let output=support::run(support::cli().arg("emit").arg(&entry),30);support::success(&output);
        if let Some(reference)=&reference {assert_eq!(&output.stdout,reference,"unchanged UI source must produce identical Rust across fresh processes");}
        else {reference=Some(output.stdout);}
    }
}
