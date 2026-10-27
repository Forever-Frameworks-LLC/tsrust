mod dev_cli;
mod dev_web;
mod runtime_build;
mod native_workspace;
mod rust_bindings;
mod package_cli;
mod package_validation;
mod rust_binding_packages;
mod package_export;
mod native_dependency_bundle;
#[cfg(any(unix,windows))]
mod dev_frontend;
#[cfg(any(unix,windows))]
mod dev_dependencies;
#[cfg(any(unix,windows))]
mod dev_watch;
#[cfg(unix)]
mod dev_events;
#[cfg(windows)]
#[path="dev_windows_events.rs"]
mod dev_events;
#[cfg(windows)]
mod dev_windows;
mod project_config;
mod new_cli;
mod application_scaffold;
mod ui_cli;
mod cache_cli;
mod setup_cli;
mod backend_sources;
mod docs_cli;
mod api_cli;
mod http_client;
mod docs_model;
mod format_cli;
mod check_cli;
mod interactive_cli;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};
use tsrust_compiler::RUST_BACKEND_VERSION;

const USAGE: &str = "tsrust â€” native TypeScript-style programs

Usage:
  tsrust                       Open the interactive command menu in a terminal
  tsrust <command> [input.tsr] [options]
  tsrust add <crate[@version]> [--project directory] [--features feature,...]
  tsrust update <crate> [--to version] [--dry-run]
  tsrust features <crate> [--enable feature,... | --disable feature,... | --features feature,...]
  tsrust packages [--project directory]
  tsrust remove <crate> [--dry-run]
  tsrust bindings resolve <file.bindings.json> [--offline]
  tsrust new [name-or-path] [--app [--target backend] | --ui [--target native|web]...]
  tsrust ui <list --bundled|init|add name...> [--cwd directory] [--path directory] [--target native|web]... [--dry-run]
  tsrust ui attributions [--cwd directory] [--dry-run]
  tsrust docs content [--cwd directory]
  tsrust api generate [--config file | --description file | --url HTTPS_URL] [--language typescript|rust] [--out directory] [--header-env Header=ENV] [--check]
  tsrust cache <info|clean>
  tsrust doctor [--json] [--native]
  tsrust setup --toolchain [--yes]
  tsrust dev [--target native|web] [--port 4317] [--release] [--verbose] [--color auto|always|never] [-- program-arguments]
  tsrust run [input.tsr] [--release] [-- program-arguments]
  tsrust test [files/directories] [options]
  tsrust format [files/directories] [--check | --write]
  tsrust check [files/directories] [--json]

Commands:
  add      Add a Cargo crate; validate adapters, callers and tests before applying
  update   Validate a compatible upgrade, or an explicit --to version
  features Inspect or change Cargo features; --features replaces the explicit set
  packages Show resolved versions, effective features and available bindings
  remove   Validate dependency removal without rewriting adapters
  bindings Resolve and vendor declared Rust dependencies
  doctor   Inspect pinned tools; --native also tests the host compiler and linker offline
  setup    Explicitly install the pinned Rust toolchain without changing defaults
  docs     Compile checked documentation content and exact example/API data offline
  api      Generate a checked named TypeScript client without adopting frontend tooling
  dev      Watch a configured native program or web UI
  new      Create a project with source, tests and configuration
  check    Check files/directories recursively; defaults to the current directory
  format   Format all .tsr project files; one explicit file prints source
  emit     Emit Rust source or a standalone Cargo package
  build    Build a native executable using config output or -o
  run      Build and run a program
  test     Discover and run .test.tsr / .spec.tsr files

Options:
  -h, --help                 Show this help
  -V, --version              Show compiler version
  -o, --output <path>        Output file for emit/build
  --color auto|always|never  Color help, diagnostics, dev status and test reports
  --package                 emit: write a standalone Cargo package to -o directory
  --json                    Machine-readable check results
  --release                 Optimize build/run/test/dev; overflow checks remain enabled
  --write                   Update source with format
  --check                   Check formatting without writing

Examples:
  tsrust check src/main.tsr
  tsrust run src/main.tsr --release
  tsrust test
  tsrust test tests --color always
  tsrust emit src/main.tsr --package -o exported
  tsrust format
  tsrust format --check
  tsrust format src

check/emit need no external compiler. build/run/test/dev use the pinned Rust backend.";

fn color_enabled(terminal: bool) -> bool {
    let args: Vec<String> = env::args().collect();
    let mut mode = "auto";
    for (index, arg) in args.iter().enumerate() {
        if arg == "--" { break; }
        if let Some(value) = arg.strip_prefix("--color=") { mode = value; }
        else if arg == "--color" { mode = args.get(index + 1).map(String::as_str).unwrap_or("auto"); }
    }
    mode == "always" || (mode == "auto" && terminal && env::var_os("NO_COLOR").is_none() && env::var("TERM").as_deref() != Ok("dumb"))
}
fn styled(text: &str, code: &str, color: bool) -> String {
    if color { format!("\x1b[{code}m{text}\x1b[0m") } else { text.to_owned() }
}
fn print_help() {
    let color = color_enabled(std::io::stdout().is_terminal());
    for line in USAGE.lines() {
        if line.ends_with(':') || line.starts_with("tsrust â€”") { println!("{}", styled(line,"1;36",color)); }
        else { println!("{line}"); }
    }
}

struct TempDirectory(PathBuf);
impl TempDirectory {
    fn new() -> std::io::Result<Self> {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        for attempt in 0..100 {
            let path =
                env::temp_dir().join(format!("tsrust-{}-{stamp}-{attempt}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => return Ok(Self(path)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "Could not create a private build directory",
        ))
    }
}
impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Separate program flags before config discovery and compiler option parsing.
fn split_program_arguments(command:&str,mut arguments:Vec<String>)->(Vec<String>,Vec<String>) {
    if command=="run" && let Some(index)=arguments.iter().position(|arg|arg=="--") {
        let program=arguments.split_off(index+1);
        arguments.pop();
        (arguments,program)
    } else {(arguments,Vec::new())}
}

#[cfg(test)]
mod argument_tests {
    use super::*;
    #[test]
    fn run_arguments_preserve_order_empty_values_and_option_spelling() {
        let input=["main.tsr","--release","--","--help","","two words","--color=invalid","--","Î»"];
        let (compiler,program)=split_program_arguments("run",input.map(str::to_owned).to_vec());
        assert_eq!(compiler,["main.tsr","--release"]);
        let mut child=Command::new("program");child.args(&program);
        assert_eq!(child.get_args().collect::<Vec<_>>(),input[3..].iter().map(std::ffi::OsStr::new).collect::<Vec<_>>());
        let (compiler,program)=split_program_arguments("run",vec!["--".into()]);
        assert!(compiler.is_empty() && program.is_empty());
        let (compiler,program)=split_program_arguments("build",vec!["--".into(),"--help".into()]);
        assert_eq!(compiler,["--","--help"]);assert!(program.is_empty());
    }
}

fn run() -> Result<u8, String> {
    let arguments=env::args().skip(1).collect::<Vec<_>>();
    interactive_cli::colors();
    let arguments=if interactive_cli::launcher(&arguments)? {
        if !interactive_cli::available() {print_help();return Ok(0);}
        let Some(arguments)=interactive_cli::command()? else{return Ok(0);};arguments
    }else{arguments};
    let mut args = arguments.into_iter();
    let Some(command) = args.next() else {
        return Err("Missing command. Run `tsrust --help` for usage.".into());
    };
    if command == "--help" || command == "-h" || env::args().skip(2).take_while(|arg|arg!="--").any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(0);
    }
    if command == "--version" || command == "-V" { println!("tsrust {}", env!("CARGO_PKG_VERSION")); return Ok(0); }
    if matches!(command.as_str(),"add"|"update"|"features"|"remove"|"packages") {return package_cli::run(&command,args);}
    if command=="bindings" {return rust_bindings::resolve(args);}
    if command=="dev" {return dev_cli::run(args);}
    if command=="docs" {return docs_cli::run(args);}
    if command=="api" {return api_cli::run(args);}
    if command=="cache" {return cache_cli::run(args);}
    if command=="doctor" {return setup_cli::doctor(args);}
    if command=="setup" {return setup_cli::setup(args);}
    if command=="ui" {return ui_cli::run(args);}
    if command=="new" {return new_cli::run(args);}
    let (arguments,program_arguments)=split_program_arguments(&command,args.collect());
    let (arguments,project_root)=project_config::defaults(&command,arguments)?;
    let mut args=arguments.into_iter();
    if !["check", "format", "emit", "build", "run", "test"].contains(&command.as_str()) {
        return Err(format!("Unknown command: {command}. Run `tsrust --help` for usage."));
    }
    if command == "format" { return format_cli::run(args); }
    if command == "check" { return check_cli::run(args); }
    let mut test_paths = Vec::new();
    let mut test_options = Vec::new();
    let source_path = if command == "test" {
        while let Some(arg) = args.next() {
            if arg == "--color" {
                test_options.push(arg);
                test_options.push(args.next().ok_or("--color requires auto, always or never")?);
            } else if arg.starts_with('-') { test_options.push(arg); }
            else { test_paths.push(PathBuf::from(arg)); }
        }
        if test_paths.is_empty() {test_paths.push(project_config::default_root()?);}
        test_paths = tsrust_compiler::project_cli::discover_test_inputs(&test_paths)?;
        test_paths[0].clone()
    } else { PathBuf::from(args.next().ok_or("Missing input file. Run `tsrust --help` for usage.")?) };
    let mut args: Box<dyn Iterator<Item=String>> = if command == "test" { Box::new(test_options.into_iter()) } else { Box::new(args) };
    let mut output: Option<PathBuf> = None;
    let mut json = false;
    let mut color_mode = "auto".to_owned();
    let mut release = false;
    let mut package = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" if output.is_none() => {
                output = Some(PathBuf::from(args.next().ok_or("Missing output path")?))
            }
            "--json" if !json => json = true,
            "--color" => {
                color_mode = args
                    .next()
                    .ok_or("--color requires auto, always or never")?
            }
            _ if arg.starts_with("--color=") => color_mode = arg[8..].to_owned(),
            "--release" if !release => release = true,
            "--package" if !package => package = true,
            _ => return Err(format!("Unknown or repeated option: {arg}. Run `tsrust --help` for usage.")),
        }
    }
    if !matches!(color_mode.as_str(), "auto" | "always" | "never") {
        return Err("--color requires auto, always or never".into());
    }
    let color = color_mode == "always"
        || (color_mode == "auto"
            && std::io::stderr().is_terminal()
            && env::var_os("NO_COLOR").is_none()
            && env::var("TERM").as_deref() != Ok("dumb"));
    let display_path = env::current_dir()
        .ok()
        .and_then(|cwd| {
            fs::canonicalize(&source_path)
                .ok()
                .and_then(|path| path.strip_prefix(cwd).ok().map(Path::to_owned))
        })
        .unwrap_or_else(|| source_path.clone())
        .display()
        .to_string();
    if release && command != "build" && command != "run" && command != "test" {
        return Err("--release applies only to build/run/test/dev".into());
    }
    if json && command != "check" {
        return Err("--json applies only to check".into());
    }
    if output.is_some() && command != "emit" && command != "build" {
        return Err("--output applies only to emit/build".into());
    }
    if package && (command != "emit" || output.is_none()) {return Err("--package requires emit and --output <new-directory>".into());}
    if command == "build" && output.is_none() {
        return Err("build requires --output".into());
    }
    let canonical_source = fs::canonicalize(&source_path).map_err(|e| e.to_string())?;
    if let Some(output) = &output
        && fs::canonicalize(output).ok().as_ref() == Some(&canonical_source)
    {
        return Err("output must not overwrite the source".into());
    }
    if matches!(command.as_str(),"build"|"run") {docs_cli::prepare_entry(&source_path)?;ui_cli::check_setup(&source_path)?;}
    let source = fs::read_to_string(&source_path).map_err(|e| e.to_string())?;

    let check_started=std::time::Instant::now();
    let checked = if command == "test" {
        tsrust_compiler::project_cli::check_test_inputs(&test_paths)
    } else { tsrust_compiler::project_cli::check_input(&source_path, &source) };
    let analysis = match checked {
        Ok(analysis) => analysis,
        Err(diagnostic) => {
            if json {
                println!(
                    "{{\"ok\":false,\"diagnostics\":[{}]}}",
                    diagnostic.to_json(&source)
                );
            } else {
                eprint!("{}", diagnostic.render(&display_path, &source, color));
            }
            return Ok(1);
        }
    };
    if matches!(command.as_str(), "build" | "run" | "test") {
        for warning in analysis.warnings() {
            let (text, span) = match &warning {
                tsrust_compiler::project_cli::InputDiagnostic::Single(diagnostic) => (source.clone(), diagnostic.span),
                tsrust_compiler::project_cli::InputDiagnostic::Project(diagnostic) => (fs::read_to_string(&diagnostic.location.source.0).unwrap_or_default(), diagnostic.location.span),
            };
            if !tsrust_compiler::DiagnosticSuppressions::new(&text).suppresses(span) {
                eprint!("{}", warning.render(&display_path, &source, color));
            }
        }
    }
    if let Some(output) = &output
        && let tsrust_compiler::project_cli::CheckedInput::Project(project) = &analysis
        && let Ok(output) = fs::canonicalize(output)
        && project
            .graph
            .modules
            .keys()
            .any(|id| Path::new(&id.0) == output)
    {
        return Err("output must not overwrite a project source file".into());
    }
    let check_elapsed=check_started.elapsed();
    let emit_started=std::time::Instant::now();
    let (rust, native_features) = analysis.emit_rust_with_native_features();
    let emit_elapsed=emit_started.elapsed();
    if command == "emit" {
        if package {
            let output=output.as_ref().expect("package output");
            let web_only=if native_features.contains(&"ui-native") {
                let targets=tsrust_compiler::ui_project::configured_targets(&canonical_source,|path|match fs::read_to_string(path){Ok(text)=>Ok(Some(text)),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(e)=>Err(e.to_string())})?.ok_or("UI package export needs ui.target. Run tsrust ui add tokens@1.0.0 with --target native or --target web")?;
                !targets.native && targets.web
            }else{false};
            let notices=if native_features.iter().any(|feature|feature.starts_with("ui")){ui_cli::attribution_files(&canonical_source)?}else{vec![]};
            package_export::export(output,&rust,web_only,&notices).map_err(|error|format!("Unable to export package {}: {error}",output.display()))?;
            println!("Emitted package {}",output.display());
            return Ok(0);
        }
        if let Some(output) = output {
            fs::write(output, rust).map_err(|e| e.to_string())?;
        } else {
            print!("{rust}");
        }
        return Ok(0);
    }
    if native_features.contains(&"ui-native"){tsrust_compiler::ui_project::require_target(&canonical_source,"native",|path|match fs::read_to_string(path){Ok(text)=>Ok(Some(text)),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Ok(None),Err(e)=>Err(e.to_string())})?;}
    let backend_started=std::time::Instant::now();
    let backend_identity = verify_backend()?;
    let backend_elapsed=backend_started.elapsed();
    let directory = TempDirectory::new().map_err(|e| e.to_string())?;
    let executable = if command == "build" {
        absolute(output.as_ref().expect("build output"))?
    } else {
        directory
            .0
            .join(format!("program{}", env::consts::EXE_SUFFIX))
    };
    let run_project = if command == "run" { project_config::discover()? } else { None };
    let cache_root = match env::var_os("TSRUST_CACHE_DIR") {
        Some(path) => PathBuf::from(path),
        None if command == "test" || (command == "run" && run_project.is_none()) => directory.0.join("cache"),
        None => run_project.map(|project| project.root).unwrap_or(env::current_dir().map_err(|error| error.to_string())?)
            .join("target/tsrust-native"),
    };
    let mut graph_identity = Vec::new();
    if let tsrust_compiler::project_cli::CheckedInput::Project(project) = &analysis {
        for (id, module) in &project.graph.modules {
            for value in [id.0.as_bytes(), module.source.text.as_bytes()] {
                graph_identity.extend_from_slice(&(value.len() as u64).to_le_bytes());
                graph_identity.extend_from_slice(value);
            }
        }
    } else {
        let path = absolute(&source_path)?;
        let path = path.to_string_lossy();
        graph_identity.extend_from_slice(&(path.len() as u64).to_le_bytes());
        graph_identity.extend_from_slice(path.as_bytes());
        graph_identity.extend_from_slice(source.as_bytes());
    }
    let features = native_features.join(",");
    let shared_inputs = backend_sources::application_identity(&rust, &cache_root)?;
    let mut cache_parts: Vec<&[u8]> = vec![
            b"native-cache-abi-4-ui-dev-profile",
            native_dependency_bundle::IDENTITY.as_bytes(),
            backend_identity.as_bytes(),
            env!("CARGO_PKG_VERSION").as_bytes(),
            env::consts::OS.as_bytes(),
            env::consts::ARCH.as_bytes(),
            if release { b"release" } else { b"debug" },
            b"edition=2024;overflow-checks=on",
            rust.as_bytes(),
            &graph_identity,
            features.as_bytes(),
        ];
    if let Some(inputs) = &shared_inputs { cache_parts.push(inputs.as_bytes()); }
    let cache = tsrust_compiler::native_cache::NativeCache::new(&cache_root, &cache_parts);
    let cache_started=std::time::Instant::now();
    let reused = cache.copy_to(&executable);
    let cache_elapsed=cache_started.elapsed();
    let native_started=std::time::Instant::now();
    if !reused {
        let mut prepared=runtime_build::Prepared::new(&rust,&backend_identity,&cache_root,&directory.0,release)?;
        if let Some(mut runtime)=prepared.command.take() {
            let result=runtime.output().map_err(|e|e.to_string())?;
            if !result.status.success(){return Err(format!("Runtime compilation failed:\n{}",String::from_utf8_lossy(&result.stderr)));}
            prepared.publish();
        }
        if env::var("TSRUST_BUILD_TIMINGS").as_deref()==Ok("1"){eprintln!("[build] runtime={}",prepared.state);}
        let mut command=backend_command();prepared.link(&mut command)?;
        let result = command
            .arg("--edition=2024")
            .args([
                "-C",
                if release {
                    "opt-level=3"
                } else {
                    "opt-level=0"
                },
            ])
            .args(["-C", "overflow-checks=on"])
            .arg(&prepared.application)
            .arg("-o")
            .arg(&executable)
            .output()
            .map_err(|e| format!("Cannot invoke pinned Rust {RUST_BACKEND_VERSION}: {e}"))?;
        if !result.status.success() {
            let stderr = String::from_utf8_lossy(&result.stderr);
            let missing_linker = stderr.lines().any(|line| {
                let line = line.trim();
                line.starts_with("error: linker `") && line.ends_with("` not found")
            });
            let explanation = if missing_linker {
                "Native linker unavailable. Install the platform build tools and ensure the linker is on PATH. check/emit do not require a linker."
            } else {
                "Rust rejected generated code. This is a compiler defect; inspect with the emit command."
            };
            return Err(format!("{explanation}\n{}", stderr.trim()));
        }
        // Cache failure must not turn a successful native build into a failure.
        let _ = cache.publish(&executable);
    }
    if env::var("TSRUST_BUILD_TIMINGS").as_deref()==Ok("1") {
        eprintln!("[build] check={:.1}ms emit={:.1}ms backend={:.1}ms cache={} ({:.1}ms) native+link={:.1}ms",check_elapsed.as_secs_f64()*1000.0,emit_elapsed.as_secs_f64()*1000.0,backend_elapsed.as_secs_f64()*1000.0,if reused {"hit"}else{"miss"},cache_elapsed.as_secs_f64()*1000.0,if reused {0.0}else{native_started.elapsed().as_secs_f64()*1000.0});
    }
    if command == "run" || command == "test" {
        let mut executable_command=Command::new(&executable);
        executable_command.args(&program_arguments);
        if command == "test" && std::env::var_os("TSRUST_TEST_REPORTER").is_none() { executable_command.env("TSRUST_TEST_REPORTER", "pretty"); }
        if let Some(root)=&project_root {executable_command.current_dir(root);}
        let status = executable_command
            .env("TSRUST_COLOR", &color_mode)
            .status()
            .map_err(|e| e.to_string())?;
        return Ok(if status.success() {
            0
        } else {
            status
                .code()
                .and_then(|c| u8::try_from(c).ok())
                .unwrap_or(1)
        });
    }
    println!("{} {}", styled("Built", "1;32", color_enabled(std::io::stdout().is_terminal())), executable.display());
    Ok(0)
}

fn backend_command() -> Command { backend_tool("rustc") }
fn backend_cargo_command() -> Command { backend_tool("cargo") }
fn backend_tool(tool:&str) -> Command {
    let executable = format!("rustup{}", env::consts::EXE_SUFFIX);
    // GUI editors may omit Cargo's bin directory from their inherited PATH.
    // Keep PATH first so explicit wrappers and custom installations still work.
    let on_path = env::var_os("PATH").and_then(|path| {
        env::split_paths(&path)
            .map(|directory| directory.join(&executable))
            .find(|candidate| candidate.is_file())
    });
    let cargo_home = env::var_os("CARGO_HOME").map(PathBuf::from).or_else(|| {
        env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
            .map(|home| PathBuf::from(home).join(".cargo"))
    });
    let installed = cargo_home
        .map(|home| home.join("bin").join(&executable))
        .filter(|candidate| candidate.is_file());
    let mut command = Command::new(on_path.or(installed).unwrap_or_else(|| executable.into()));
    // No --install: compiling a program never implicitly downloads a toolchain.
    command.args(["run", RUST_BACKEND_VERSION, tool]);
    command
}

fn verify_backend() -> Result<String, String> {
    let setup = format!(
        "Install rustup and run `rustup toolchain install {RUST_BACKEND_VERSION} --profile minimal`. check/emit do not require it."
    );
    let result = backend_command()
        .arg("--version")
        .output()
        .map_err(|error| {
            format!("Cannot invoke pinned Rust {RUST_BACKEND_VERSION}: {error}. {setup}")
        })?;
    validate_backend_output(result)
}

fn validate_backend_output(result:std::process::Output)->Result<String,String> {
    let setup=format!("Install rustup and run `rustup toolchain install {RUST_BACKEND_VERSION} --profile minimal`. check/emit do not require it.");
    if !result.status.success() {
        return Err(format!(
            "Pinned Rust {RUST_BACKEND_VERSION} is unavailable. {setup}\n{}",
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    let version = String::from_utf8_lossy(&result.stdout);
    let mut words = version.split_whitespace();
    if words.next() != Some("rustc") || words.next() != Some(RUST_BACKEND_VERSION) {
        return Err(format!(
            "Expected Rust {RUST_BACKEND_VERSION}, but the selected backend reports {:?}. {setup}",
            version.trim()
        ));
    }
    Ok(version.trim().to_owned())
}

fn absolute(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(env::current_dir().map_err(|e| e.to_string())?.join(path))
    }
}

/// Let a closed output pipe end the command quietly, as other command-line
/// tools do. Rust ignores the signal by default, which turns the next print
/// into a panic.
#[cfg(unix)]
fn restore_pipe_signal() {
    unsafe extern "C" {
        fn signal(signal: i32, handler: usize) -> usize;
    }
    const SIGPIPE: i32 = 13;
    const DEFAULT_HANDLER: usize = 0;
    // SAFETY: called once before any thread starts; it selects the system's
    // default disposition for one signal and installs no handler code.
    unsafe {
        signal(SIGPIPE, DEFAULT_HANDLER);
    }
}
#[cfg(not(unix))]
fn restore_pipe_signal() {}

fn main() -> ExitCode {
    restore_pipe_signal();
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let color = color_enabled(std::io::stderr().is_terminal());
            eprintln!("{}: {error}", styled("error", "1;31", color));
            ExitCode::FAILURE
        }
    }
}
