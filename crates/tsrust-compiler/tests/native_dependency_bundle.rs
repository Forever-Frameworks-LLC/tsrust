mod support;
#[path = "../src/cli/native_dependency_bundle.rs"]
mod native_dependency_bundle;

#[test]
fn bundled_dependencies_build_without_registry_or_repository_access() {
    let dir = support::TempDirectory::new();
    let root = dir.0.join("support");
    native_dependency_bundle::materialize(&root).unwrap();
    assert!(root.join("vendor/serde-1.0.229/LICENSE-MIT").is_file());
    std::fs::write(root.join("src/main.rs"), r##"
fn main() {
    let value: tsrust_native_dependencies::serde_json::Value =
        tsrust_native_dependencies::serde_json::from_str(r#"{"answer":42}"#).unwrap();
    assert_eq!(value["answer"], 42);
    let input=String::from(r#"{ "answer":42 }"#);
    let raw:&tsrust_native_dependencies::serde_json::value::RawValue=
        tsrust_native_dependencies::serde_json::from_str(&input).unwrap();
    assert_eq!(raw.get(),input);
    assert_eq!(raw.get().as_ptr(),input.as_ptr());
    println!("bundled JSON passed");
}
"##).unwrap();
    let mut command = std::process::Command::new("rustup");
    command.args(["run", "1.96.1", "cargo", "run", "--offline", "--locked"])
        .arg("--manifest-path").arg(root.join("Cargo.toml"))
        .arg("--config").arg(root.join(".cargo/config.toml"))
        .current_dir(&dir.0)
        .env("CARGO_HOME", dir.0.join("empty-cargo"))
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_TARGET_DIR", std::env::var_os("CARGO_TARGET_DIR").expect("use shared Cargo target"));
    let output = support::run(&mut command, 60);
    support::success(&output);
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "bundled JSON passed\n");
    // Never overwrite an existing build directory, including a caller's files.
    assert!(native_dependency_bundle::materialize(&root).is_err());
    let checksum = root.join("vendor/serde_json-1.0.151/.cargo-checksum.json");
    let mut metadata: serde_json::Value = serde_json::from_slice(&std::fs::read(&checksum).unwrap()).unwrap();
    metadata["package"] = serde_json::Value::String("0".repeat(64));
    std::fs::write(checksum, serde_json::to_vec(&metadata).unwrap()).unwrap();
    let output = support::run(&mut command, 60);
    assert!(!output.status.success());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("checksum") && error.contains("changed"), "{error}");
}
