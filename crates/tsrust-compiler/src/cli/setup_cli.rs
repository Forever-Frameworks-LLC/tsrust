//! Explicit machine preparation. Ordinary build commands never enter this path.
use std::{fs, io::{self, IsTerminal, Write}, process::Command};
use serde_json::{Value, json};
use tsrust_compiler::RUST_BACKEND_VERSION;

const DOCTOR_USAGE: &str = "Usage: tsrust doctor [--json] [--native]";
const SETUP_USAGE: &str = "Usage: tsrust setup [--toolchain] [--dependencies] [--offline] [--yes]";

fn flags(args: impl Iterator<Item=String>, allowed: &[&str], usage: &str) -> Result<Vec<String>, String> {
    let mut flags = Vec::new();
    for arg in args {
        if !allowed.contains(&arg.as_str()) || flags.contains(&arg) { return Err(usage.into()); }
        flags.push(arg);
    }
    Ok(flags)
}

fn version(mut command: Command, expected: &str) -> Value {
    match command.arg("--version").output() {
        Ok(output) if output.status.success() => {
            let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            let words = text.split_whitespace().collect::<Vec<_>>();
            let ok = words.first() == Some(&expected)
                && (expected != "rustc" || words.get(1) == Some(&RUST_BACKEND_VERSION));
            json!({"status":if ok {"ok"} else {"error"}, "version":text})
        }
        Ok(output) => json!({"status":"error", "message":String::from_utf8_lossy(&output.stderr).trim()}),
        Err(error) => json!({"status":"error", "message":error.to_string()}),
    }
}

fn native_probe() -> Result<(), String> {
    let directory = super::TempDirectory::new().map_err(|e| e.to_string())?;
    let source = directory.0.join("probe.rs");
    let binary = directory.0.join(format!("probe{}", std::env::consts::EXE_SUFFIX));
    fs::write(&source, "fn main() { println!(\"tsrust-native-probe\"); }\n").map_err(|e| e.to_string())?;
    let output = super::backend_command().arg("--edition=2024").arg(&source).arg("-o").arg(&binary)
        .env("CARGO_INCREMENTAL", "0").output().map_err(|e| e.to_string())?;
    if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).trim().into()); }
    let output = Command::new(binary).output().map_err(|e| e.to_string())?;
    if !output.status.success() || output.stdout != b"tsrust-native-probe\n" && output.stdout != b"tsrust-native-probe\r\n" {
        return Err("Native probe compiled but did not execute successfully".into());
    }
    Ok(())
}

pub(super) fn doctor(args: impl Iterator<Item=String>) -> Result<u8, String> {
    let flags = flags(args, &["--json", "--native"], DOCTOR_USAGE)?;
    let rustc = version(super::backend_command(), "rustc");
    let cargo = version(super::backend_cargo_command(), "cargo");
    let tools_ok = rustc["status"] == "ok" && cargo["status"] == "ok";
    let native = if flags.iter().any(|a| a == "--native") && tools_ok {
        match native_probe() {
            Ok(()) => json!({"status":"ok", "scope":"minimal host compile, link and execution; no application dependencies"}),
            Err(error) => json!({"status":"error", "message":error}),
        }
    } else { json!({"status":"unchecked", "message":"Run tsrust doctor --native after preparing the pinned tools"}) };
    let ok = tools_ok && native["status"] != "error";
    let report = json!({"schemaVersion":1,"compilerVersion":env!("CARGO_PKG_VERSION"),
        "host":{"os":std::env::consts::OS,"architecture":std::env::consts::ARCH},
        "requiredRust":RUST_BACKEND_VERSION,"tools":{"rustc":rustc,"cargo":cargo},
        "toolchainReady":tools_ok,"nativeProbe":native,
        "applicationDependencies":{"status":"unchecked","message":"This probe does not resolve or validate an application's dependency graph"},
        "remediation":"Install rustup if absent; run tsrust setup --toolchain --yes. Platform linker/build tools must be installed separately.",
        "networkUsed":false});
    if flags.iter().any(|a| a == "--json") { println!("{}", serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?); }
    else {
        println!("Tsrust {} on {}/{}; required Rust {}", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH, RUST_BACKEND_VERSION);
        for name in ["rustc", "cargo"] {
            let tool = &report["tools"][name];
            println!("{name}: {} — {}", tool["status"].as_str().unwrap(), tool["version"].as_str().or(tool["message"].as_str()).unwrap_or("unknown"));
        }
        println!("Native probe: {}", report["nativeProbe"]);
        println!("Application dependencies: unchecked. check/emit need no Rust toolchain.");
        if !ok { println!("{}", report["remediation"].as_str().unwrap()); }
    }
    Ok(if ok {0} else {1})
}

fn install_command() -> Command {
    let backend = super::backend_cargo_command();
    let mut command = Command::new(backend.get_program());
    command.args(["toolchain", "install", RUST_BACKEND_VERSION, "--profile", "minimal", "--no-self-update"]);
    command
}

pub(super) fn setup(args: impl Iterator<Item=String>) -> Result<u8, String> {
    let flags = flags(args, &["--toolchain", "--dependencies", "--offline", "--yes"], SETUP_USAGE)?;
    let toolchain=flags.iter().any(|a|a=="--toolchain");
    let dependencies=flags.iter().any(|a|a=="--dependencies");
    let offline=flags.iter().any(|a|a=="--offline");
    if !toolchain && !dependencies { return Err(SETUP_USAGE.into()); }
    if offline && toolchain { return Err("--offline prepares cached dependencies only; omit --toolchain".into()); }
    if !offline && !flags.iter().any(|a| a == "--yes") {
        if !io::stdin().is_terminal() { return Err("Toolchain setup may download files. Pass --yes for unattended setup.".into()); }
        eprint!("Prepare the selected Rust toolchain/dependency sources (downloads may be required)? [y/N] ");
        io::stderr().flush().map_err(|e| e.to_string())?;
        let mut answer = String::new(); io::stdin().read_line(&mut answer).map_err(|e| e.to_string())?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") { return Err("Setup cancelled; no toolchain installation started".into()); }
    }
    if toolchain {
        let status = install_command().status().map_err(|e| format!("Cannot invoke rustup: {e}. Install rustup from https://rustup.rs first; setup does not run downloaded installer scripts."))?;
        if !status.success() { return Err("Pinned toolchain installation failed; rerun tsrust doctor for current state".into()); }
    }
    super::verify_backend()?;
    if dependencies {
        let temporary=super::TempDirectory::new().map_err(|e|e.to_string())?;
        let package=temporary.0.join("backend-sources");
        super::backend_sources::materialize(&package)?;
        let root=super::backend_sources::build_root()?;
        fs::create_dir_all(&root).map_err(|e|e.to_string())?;
        let mut command=super::backend_cargo_command();
        command.current_dir(root).args(["fetch","--locked","--manifest-path"]).arg(package.join("Cargo.toml"))
            .env("CARGO_HOME",super::backend_sources::cargo_home()?);
        if offline{command.arg("--offline");}
        let status=command.status().map_err(|e|e.to_string())?;
        if !status.success(){return Err("Backend source preparation failed; existing shared Cargo sources were preserved. Rerun setup --dependencies --yes with network access, or provide a populated Cargo home for --offline.".into());}
        println!("Locked backend sources are available in the shared Cargo cache. No project vendor directory was created.");
    }
    println!("Pinned Rust {RUST_BACKEND_VERSION} is available. Run tsrust doctor --native to test host linking.");
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preparation_requires_explicit_unique_flags_and_never_changes_defaults() {
        for args in [vec!["--fetch"], vec!["--yes", "--yes"]] {
            assert!(flags(args.into_iter().map(str::to_owned), &["--toolchain", "--yes"], SETUP_USAGE).is_err());
        }
        let command = install_command();
        let args = command.get_args().map(|a| a.to_str().unwrap()).collect::<Vec<_>>();
        assert_eq!(args, ["toolchain", "install", RUST_BACKEND_VERSION, "--profile", "minimal", "--no-self-update"]);
        assert!(setup(["--yes".into()].into_iter()).is_err());
        assert!(doctor(["--native".into(), "--native".into()].into_iter()).is_err());
    }
}
