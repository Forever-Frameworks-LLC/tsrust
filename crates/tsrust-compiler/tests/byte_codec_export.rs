mod support;
use tsrust_compiler::{native_runtime::separate, project::{FileSourceProvider, check_graph, load_project}};

#[test]
fn base64_dependencies_follow_used_codec_calls_without_affecting_std_only_codecs() {
    let directory = support::TempDirectory::new();
    for (source, dependency) in [
        ("import {toBase64} from 'std:bytes';function main(){console.log('unused');}", false),
        ("import {fromArray,toHex} from 'std:bytes';function main(){console.log(unwrap(toHex(unwrap(fromArray([0,255])))));}", false),
        ("import {fromUtf16,toUtf16} from 'std:bytes';function main(){console.log(unwrap(toUtf16(unwrap(fromUtf16('A😀','big')),'big')));}", false),
        ("import {fromUtf8,toBase64 as encode} from 'std:bytes';function main(){const stored=encode;console.log(unwrap(stored(unwrap(fromUtf8('f')))));}", true),
    ] {
        let path = directory.source(&support::format_round_trip(source));
        let program = check_graph(load_project(&FileSourceProvider, path.to_str().unwrap()).unwrap()).unwrap();
        let emitted = program.emit_rust();
        assert!(!emitted.contains("unsafe"));
        assert_eq!(separate(&emitted).dependencies, dependency, "{source}");
        assert_eq!(tsrust_compiler::native_runtime::requires_byte_codecs(&emitted), dependency, "{source}");
    }
}

#[test]
fn codec_export_is_safe_and_builds_offline_after_relocation_at_o0_and_o3() {
    let directory = support::TempDirectory::new();
    let path = directory.source(&support::format_round_trip("import {fromUtf8,toBase64,fromUtf16,toUtf16} from 'std:bytes';function main(){console.log(unwrap(toBase64(unwrap(fromUtf8('f')))),unwrap(toUtf16(unwrap(fromUtf16('A😀','big')),'big')));}"));
    support::success(&support::run(support::cli().current_dir(&directory.0).args(["setup", "--portable", "--offline"]), 180));
    let package = directory.0.join("package");
    support::success(&support::run(support::cli().arg("emit").arg(&path).args(["--package", "-o"]).arg(&package), 60));
    std::fs::remove_file(&path).unwrap();
    let relocated = directory.0.join("relocated"); std::fs::rename(package, &relocated).unwrap();
    assert!(relocated.join("vendor/base64-0.23.1/LICENSE-MIT").is_file());
    let manifest = std::fs::read_to_string(relocated.join("Cargo.toml")).unwrap();
    assert!(manifest.lines().any(|line| line.starts_with("default = [") && line.contains("byte-codecs")));
    let source = relocated.join("src/main.rs"); let emitted = std::fs::read_to_string(&source).unwrap();
    assert!(!emitted.contains("unsafe")); std::fs::write(source, format!("#![forbid(unsafe_code)]\n{emitted}")).unwrap();
    let target = std::env::var_os("CARGO_TARGET_DIR").map(std::path::PathBuf::from).unwrap_or_else(||std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target"));
    assert!(target.is_absolute());
    for release in [false, true] {
        let mut command = std::process::Command::new("rustup");
        command.args(["run", tsrust_compiler::RUST_BACKEND_VERSION, "cargo", "build", "--offline", "--locked"])
            .current_dir(&relocated).env("CARGO_HOME", directory.0.join("empty-cargo-home"))
            .env("CARGO_TARGET_DIR", &target).env("CARGO_INCREMENTAL", "0").env("CARGO_BUILD_JOBS", "1");
        if release { command.arg("--release"); }
        support::success(&support::run(&mut command, 180));
        let executable = target.join(if release {"release"} else {"debug"}).join(format!("program{}",std::env::consts::EXE_SUFFIX));
        let output = support::run(std::process::Command::new(executable).current_dir(&relocated), 30);
        support::success(&output); assert_eq!(output.stdout, "Zg== A😀\n".as_bytes());
    }
}
