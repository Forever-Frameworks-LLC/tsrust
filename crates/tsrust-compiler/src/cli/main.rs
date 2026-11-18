mod dev_cli;
mod workspace_cli;
mod runtime_build;
mod native_workspace;
mod rust_bindings;
mod package_cli;
mod package_validation;
mod rust_binding_packages;
mod package_export;
mod native_dependency_bundle;
#[cfg(any(unix,windows))]
mod dev_dependencies;
mod dev_frontend;
mod app_package_cli;
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
mod api_cli;
mod http_client;
mod toolchain_cli;
mod toolchain_install;
mod toolchain_release;
mod format_cli;
mod check_cli;
mod interactive_cli;
use std::env;
use std::fs;
use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
mod temp_directory;
use temp_directory::TempDirectory;
use tsrust_compiler::RUST_BACKEND_VERSION;

const USAGE: &str = "tsrust — native TypeScript-style programs

Usage:
  tsrust                       Open the interactive command menu in a terminal
  tsrust <command> [input.tsr] [options]
  tsrust add <crate[@version]> [--project directory] [--features feature,...]
  tsrust update <crate> [--to version] [--dry-run]
  tsrust features <crate> [--enable feature,... | --disable feature,... | --features feature,...]
  tsrust packages [--project directory]
  tsrust remove <crate> [--dry-run]
  tsrust bindings resolve <file.bindings.json> [--offline] [--portable]
  tsrust bindings generate <file.bindings.json> --crate name --source crate-root.rs --select name,name [--rustdoc crate.json]
  tsrust new [name-or-path] [--ui | --app [--target backend]]
  tsrust ui list | ui add [Component...] [--all] [--force]
  tsrust package [entry.tsrx] [--companion PROJECT] [--name NAME] [--version 1.0.0] [--icon icon.png] [--repo owner/name] [--sign ID --notarize PROFILE]
  tsrust docs content [--cwd directory]
  tsrust api generate [entry.tsr | --entry file | --config file | --description file | --url HTTPS_URL] [--language typescript|rust] [--out directory] [--header-env Header=ENV] [--check]
  tsrust cache <info|clean>
  tsrust doctor [--json] [--native]
  tsrust toolchain install --root NEW_DIRECTORY
  tsrust toolchain check [--json] | update | rollback
  tsrust setup [--toolchain] [--dependencies] [--portable] [--offline] [--yes]
  tsrust dev [--release] [--verbose] [--color auto|always|never] [-- program-arguments]
  tsrust run [input.tsr] [--release] [-- program-arguments]
  tsrust test [files/directories] [options]
  tsrust format [files/directories] [--check | --write] [--text-files-up-to bytes]
    Skip binary (invalid UTF-8 or NUL) and larger files when a byte limit is given
  tsrust format --stdin-filepath <source-path> [--options <JSON>]
  tsrust check|format [--progress]   Print a task-start message before processing
  tsrust check [files/directories] [--json] [--diagnostics-only]
  tsrust check|build|test|format [--project name]   At a tsrust.workspace.json root: every member, clients first

Commands:
  add      Add a Cargo crate; validate adapters, callers and tests before applying
  update   Validate a compatible upgrade, or an explicit --to version
  features Inspect or change Cargo features; --features replaces the explicit set
  packages Show resolved versions, effective features and available bindings
  remove   Validate dependency removal without rewriting adapters
  bindings Prepare shared Rust dependency sources; --portable also prepares an export pack
  doctor   Inspect the CLI/LSP pair and pinned tools; --native tests the host linker offline
  toolchain Install, check, update or roll back the verified prebuilt CLI/LSP pair
  setup    Explicitly prepare pinned Rust tools or shared backend dependency sources
  docs     Compile checked documentation content and exact example/API data offline
  api      Generate checked TypeScript or async Rust clients from an API description
  dev      Rebuild and restart the configured program on source changes
  new      Create a project with source, tests and configuration; --ui creates a desktop app
  ui       List GPUI Kit components or copy editable wrappers into src/components
  check    Check files/directories recursively; defaults to the current directory
  format   Format .tsr/.tsrx project files; one explicit file prints source
  emit     Emit Rust source or a standalone Cargo package
  build    Build a native executable using config output or -o
  run      Build and run a program
  package  Package a std:ui application for this platform (.app/DMG, zip, tarball/.deb)
  test     Discover and run .test.tsr / .spec.tsr files

Options:
  --filter <text>            Test names containing literal full suite/name text
  --reporter <mode>          Test reporter: pretty, plain or json (NDJSON)
  --timeout <milliseconds>   Explicit async test-body deadline; no default
  --profile                  Test setup/build timings, separate from test execution
  -h, --help                 Show this help
  -V, --version              Show compiler version
  -o, --output <path>        Output file for emit/build
  --color auto|always|never  Color help, diagnostics, dev status and test reports
  --debug                   build: retain mapped Rust source beside the executable and emit debugger symbols
  --source-locations        emit: annotate Rust statements with Tsrust file and byte range
  --crate-plan              emit: print each generated function's and closure's source module as JSON
  --stable-names LEDGER     emit: name generated items through LEDGER as incremental builds do
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
        if line.ends_with(':') || line.starts_with("tsrust —") { println!("{}", styled(line,"1;36",color)); }
        else { println!("{line}"); }
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
    let command_started=std::time::Instant::now();
    if let Some(code)=toolchain_install::dispatch()?{return Ok(code);}
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
    if command == "--help"
        || command == "-h"
        || env::args()
            .skip(2)
            .take_while(|arg| arg != "--")
            .any(|arg| arg == "--help" || arg == "-h")
    {
        print_help();
        return Ok(0);
    }
    if command == "--version" || command == "-V" { println!("tsrust {}", env!("CARGO_PKG_VERSION")); return Ok(0); }
    if matches!(command.as_str(),"add"|"update"|"features"|"remove"|"packages") {return package_cli::run(&command,args);}
    if command=="bindings" {return rust_bindings::resolve(args);}
    if command=="dev" {return dev_cli::run(args);}
    if command=="api" {return api_cli::run(args);}
    if command=="cache" {return cache_cli::run(args);}
    if command=="doctor" {return setup_cli::doctor(args);}
    if command=="toolchain" {return toolchain_cli::run(args);}
    if command=="setup" {return setup_cli::setup(args);}
    if command=="ui" {return ui_cli::run(args);}
    if command=="new" {return new_cli::run(args);}
    if command=="package" {return app_package_cli::run(args);}
    let args=args.collect::<Vec<_>>();
    if let Some(result)=workspace_cli::dispatch(&command,&args) {return result;}
    let (arguments,program_arguments)=split_program_arguments(&command,args);
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
            if matches!(arg.as_str(),"--color"|"--filter"|"--reporter"|"--timeout") {
                let message=format!("{arg} requires a value");test_options.push(arg);
                test_options.push(args.next().ok_or(message)?);
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
    let mut test_filter: Option<String> = None;
    let mut test_reporter: Option<String> = None;
    let mut test_timeout: Option<u64> = None;
    let mut test_profile=false;
    let mut release = false;
    let mut package = false;
    let mut source_locations = false;
    let mut crate_plan = false;
    let mut stable_names: Option<PathBuf> = None;
    let mut debug = false;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-o" | "--output" if output.is_none() => {
                output = Some(PathBuf::from(args.next().ok_or("Missing output path")?))
            }
            "--json" if !json => json = true,
            "--filter" if command == "test" && test_filter.is_none() => {
                test_filter = Some(
                    args.next()
                        .ok_or("--filter requires a literal name substring")?,
                )
            }
            _ if command == "test" && arg.starts_with("--filter=") && test_filter.is_none() => {
                test_filter = Some(arg[9..].to_owned())
            }
            "--reporter" if command == "test" && test_reporter.is_none() => {
                test_reporter = Some(
                    args.next()
                        .ok_or("--reporter requires pretty, plain or json")?,
                )
            }
            _ if command == "test" && arg.starts_with("--reporter=") && test_reporter.is_none() => {
                test_reporter = Some(arg[11..].to_owned())
            }
            "--profile" if command == "test" && !test_profile => test_profile = true,
            "--timeout" if command == "test" && test_timeout.is_none() => {
                test_timeout = Some(test_timeout_value(
                    &args.next().ok_or("--timeout requires milliseconds")?,
                )?)
            }
            _ if command == "test" && arg.starts_with("--timeout=") && test_timeout.is_none() => {
                test_timeout = Some(test_timeout_value(&arg[10..])?)
            }
            "--color" => {
                color_mode = args
                    .next()
                    .ok_or("--color requires auto, always or never")?
            }
            _ if arg.starts_with("--color=") => color_mode = arg[8..].to_owned(),
            "--release" if !release => release = true,
            "--debug" if command == "build" && !debug => debug = true,
            "--source-locations" if command == "emit" && !source_locations => {
                source_locations = true
            }
            "--crate-plan" if command == "emit" && !crate_plan => crate_plan = true,
            "--stable-names" if command == "emit" && stable_names.is_none() => {
                stable_names = Some(PathBuf::from(
                    args.next().ok_or("--stable-names requires a ledger path")?,
                ))
            }
            "--package" if !package => package = true,
            _ => {
                return Err(format!(
                    "Unknown or repeated option: {arg}. Run `tsrust --help` for usage."
                ));
            }
        }
    }
    if test_reporter
        .as_deref()
        .is_some_and(|value| !matches!(value, "pretty" | "plain" | "json"))
    {
        return Err("--reporter requires pretty, plain or json".into());
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
    if release && debug { return Err("--debug and --release select different build profiles".into()); }
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
    if crate_plan {
        let tsrust_compiler::project_cli::CheckedInput::Project(project) = &analysis else {
            return Err("--crate-plan needs a project entry (a tsrust.config.json project)".into());
        };
        let plan = project.crate_plan();
        match &output {Some(output) => fs::write(output, plan).map_err(|e| e.to_string())?, None => println!("{plan}")}
        return Ok(0);
    }
    let check_elapsed=check_started.elapsed();
    let emit_started=std::time::Instant::now();
    let (rust, native_features) = if source_locations || debug {
        analysis.emit_rust_with_locations(canonical_source.to_str().ok_or("Source path must be valid UTF-8")?, &source)
    } else { analysis.emit_rust_for_source(canonical_source.to_str().ok_or("Source path must be valid UTF-8")?, &source) };
    let emit_elapsed=emit_started.elapsed();
    if command == "emit" {
        if package {
            let output=output.as_ref().expect("package output");
            package_export::export(output,&rust).map_err(|error|format!("Unable to export package {}: {error}",output.display()))?;
            println!("Emitted package {}",output.display());
            return Ok(0);
        }
        // Diagnostic aid: show the generated program as incremental native builds name it.
        let rust = match &stable_names {
            Some(ledger_path) => {
                let mut ledger = tsrust_compiler::native_stable_names::Ledger::load(ledger_path);
                let renamed = tsrust_compiler::native_stable_names::stabilize(
                    &rust,
                    &analysis.name_keys(),
                    &mut ledger,
                );
                ledger.save(ledger_path);
                renamed
            }
            None => rust,
        };
        if let Some(output) = output {
            fs::write(output, rust).map_err(|e| e.to_string())?;
        } else {
            print!("{rust}");
        }
        return Ok(0);
    }
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
        None if command == "run" && run_project.is_none() => directory.0.join("cache"),
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
    let optimization = runtime_build::application_optimization(release,false);
    let shared_inputs = backend_sources::application_identity(&rust, &cache_root)?;
    let mut cache_parts: Vec<&[u8]> = vec![
            b"native-cache-abi-4-ui-dev-profile",
            native_dependency_bundle::dependency_identity().as_bytes(),
            backend_identity.as_bytes(),
            env!("CARGO_PKG_VERSION").as_bytes(),
            env::consts::OS.as_bytes(),
            env::consts::ARCH.as_bytes(),
            if release { b"release" } else { b"debug" },
            b"edition=2024;overflow-checks=on",
            rust.as_bytes(),
            &graph_identity,
            features.as_bytes(),
            optimization.as_bytes(),
        ];
    if let Some(inputs) = &shared_inputs { cache_parts.push(inputs.as_bytes()); }
    if let Some(identity)=runtime_build::application_cache_identity(&rust){cache_parts.push(identity);}
    cache_parts.push(runtime_build::profile_identity(&rust).as_bytes());
    if !release && !debug && runtime_build::crate_split_enabled() { cache_parts.push(b"crate-split-1"); }
    let cache = tsrust_compiler::native_cache::NativeCache::new(&cache_root, &cache_parts);
    let cache_started=std::time::Instant::now();
    // Debug symbols can include platform sidecars outside the executable cache.
    let reused = !debug && cache.copy_to(&executable);
    let cache_elapsed=cache_started.elapsed();
    let native_started=std::time::Instant::now();
    if !reused {
        let incremental_entry = absolute(&source_path)?.join(&command);
        let naming = (!release && !debug).then(|| {
            runtime_build::Naming::for_session(
                analysis.name_keys(),
                &cache_root,
                &incremental_entry,
            )
        });
        let mut prepared = runtime_build::Prepared::new_named(
            &rust,
            &backend_identity,
            &cache_root,
            &directory.0,
            release,
            naming.as_ref(),
        )?;
        if let Some(mut runtime)=prepared.command.take() {
            prepared.acquire_target(true)?;
            let result=runtime.output().map_err(|e|e.to_string())?;
            if !result.status.success(){return Err(prepared.compilation_error(&String::from_utf8_lossy(&result.stderr)));}
            prepared.finish()?;prepared.publish();
        }
        if !test_profile&&env::var("TSRUST_BUILD_TIMINGS").as_deref()==Ok("1"){eprintln!("[build] runtime={}",prepared.state);}
        if let Some(split) = &prepared.split {
            let mut probe = backend_command();
            prepared.link(&mut probe)?;
            let base = || {
                let mut command = backend_command();
                let _ = prepared.link(&mut command);
                command.args(["--cap-lints", "allow", "--edition=2024", "-C", optimization, "-C", "overflow-checks=on"]);
                command
            };
            let summary = split.build(&base, &executable, prepared.library.as_deref(), &Default::default(), false)?.ok_or("Native build cancelled")?;
            if !test_profile&&env::var("TSRUST_BUILD_TIMINGS").as_deref()==Ok("1"){eprintln!("[build] split {summary}");}
            if backend_sources::unchanged(shared_inputs.as_deref(), &cache_root, false) { let _ = cache.publish(&executable); }
        } else {
        let mut command = backend_command();
        prepared.link(&mut command)?;
        prepared.application=runtime_build::application_backend_args(&mut command,&cache_root,&incremental_entry,release||debug,&prepared.application);
        if debug {
            prepared.application = retain_debug_source(&prepared.application, &executable)?;
            command.args(["--crate-name", "program", "-g"]);
            #[cfg(target_os = "macos")]
            command.arg("-Csplit-debuginfo=packed");
        }
        let result = command
            .arg("--edition=2024")
            .args([
                "-C",
                optimization,
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
        if debug { record_debug_unit(&prepared.application, &executable).map_err(|e|e.to_string())?; }
        // Cache failure must not turn a successful native build into a failure.
        if !debug && backend_sources::unchanged(shared_inputs.as_deref(), &cache_root,false) { let _ = cache.publish(&executable); }
        }
    }
    if !test_profile&&env::var("TSRUST_BUILD_TIMINGS").as_deref()==Ok("1") {
        eprintln!("[build] check={:.1}ms emit={:.1}ms backend={:.1}ms cache={} ({:.1}ms) native+link={:.1}ms",check_elapsed.as_secs_f64()*1000.0,emit_elapsed.as_secs_f64()*1000.0,backend_elapsed.as_secs_f64()*1000.0,if reused {"hit"}else{"miss"},cache_elapsed.as_secs_f64()*1000.0,if reused {0.0}else{native_started.elapsed().as_secs_f64()*1000.0});
    }
    if test_profile {
        let profile=serde_json::json!({
            "event":"build", "durationMs":command_started.elapsed().as_secs_f64()*1000.0,
            "checkMs":check_elapsed.as_secs_f64()*1000.0,
            "emitMs":emit_elapsed.as_secs_f64()*1000.0,
            "backendMs":backend_elapsed.as_secs_f64()*1000.0,
            "cacheMs":cache_elapsed.as_secs_f64()*1000.0,
            "nativeBuildMs":if reused {0.0}else{native_started.elapsed().as_secs_f64()*1000.0},
            "cache":if reused {"hit"}else{"miss"}, "testFiles":test_paths.len()
        });
        let reporter=test_reporter.clone().or_else(||env::var("TSRUST_TEST_REPORTER").ok());
        if reporter.as_deref()==Some("json") {
            use std::io::Write;
            let mut stdout=std::io::stdout().lock();
            writeln!(stdout,"{profile}").and_then(|()|stdout.flush()).map_err(|e|e.to_string())?;
        } else {
            eprintln!("[build] total={:.1}ms check={:.1}ms emit={:.1}ms backend={:.1}ms cache={} ({:.1}ms) native+link={:.1}ms",profile["durationMs"].as_f64().unwrap(),profile["checkMs"].as_f64().unwrap(),profile["emitMs"].as_f64().unwrap(),profile["backendMs"].as_f64().unwrap(),if reused {"hit"}else{"miss"},profile["cacheMs"].as_f64().unwrap(),profile["nativeBuildMs"].as_f64().unwrap());
        }
    }
    if command == "run" || command == "test" {
        let mut executable_command=Command::new(&executable);
        executable_command.args(&program_arguments);
        if command=="test"{if let Some(reporter)=&test_reporter{executable_command.env("TSRUST_TEST_REPORTER",reporter);}else if std::env::var_os("TSRUST_TEST_REPORTER").is_none(){executable_command.env("TSRUST_TEST_REPORTER","pretty");}if let Some(filter)=&test_filter{executable_command.env("TSRUST_TEST_FILTER",filter);}if let Some(timeout)=test_timeout{executable_command.env("TSRUST_TEST_TIMEOUT_MS",timeout.to_string());}}
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

/// Compile the exact post-runtime-split source from a stable path recorded in DWARF/PDB.
fn retain_debug_source(application: &Path, executable: &Path) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    let source = fs::read(application).map_err(|error| error.to_string())?;
    let mut directory = executable.as_os_str().to_owned();
    directory.push(".tsrust-debug");
    let directory = PathBuf::from(directory);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let digest: String = Sha256::digest(&source).iter().map(|byte| format!("{byte:02x}")).collect();
    let path = directory.join(format!("program_{digest}.rs"));
    if !path.exists() {
        use std::io::Write;
        let mut staged = tempfile::NamedTempFile::new_in(&directory).map_err(|e|e.to_string())?;
        staged.write_all(&source).map_err(|e|e.to_string())?;
        match staged.persist_noclobber(&path) {
            Ok(_) => return Ok(path),
            Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {},
            Err(error) => return Err(error.to_string()),
        }
    }
    if fs::read(&path).map_err(|e|e.to_string())? != source {
        return Err(format!("Generated debugger source differs at {}", path.display()));
    }
    Ok(path)
}

// Bind a completed executable to its retained unit even when the output path is rebuilt.
fn record_debug_unit(unit: &Path, executable: &Path) -> std::io::Result<()> {
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};
    let mut file = fs::File::open(executable)?;
    let mut hash = Sha256::new();
    let mut bytes = [0; 65536];
    loop {
        let count = file.read(&mut bytes)?;
        if count == 0 { break; }
        hash.update(&bytes[..count]);
    }
    let key: String = hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect();
    let directory = unit.parent().expect("debug unit directory");
    let mut staged = tempfile::NamedTempFile::new_in(directory)?;
    staged.write_all(unit.file_name().expect("debug unit name").as_encoded_bytes())?;
    staged.persist(directory.join(format!("{key}.unit")))?;
    Ok(())
}

fn backend_command() -> Command {
    let mut command=backend_tool("rustc");
    if cfg!(all(windows,target_env="msvc")){command.arg(runtime_build::MSVC_LINKER_FLAG);}
    command
}
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
    // rustc parses and lowers nested code recursively; its default stack stops
    // near 1,000 nested blocks. Source nesting has no limit, so give rustc's
    // compiler thread room. The reservation is virtual: only pages in use are
    // committed. An explicit RUST_MIN_STACK from the user is kept.
    if env::var_os("RUST_MIN_STACK").is_none() {
        command.env("RUST_MIN_STACK", BACKEND_STACK_BYTES);
    }
    command
}

/// Stack reserved for the backend compiler thread (1 GiB, committed on use).
const BACKEND_STACK_BYTES: &str = "1073741824";

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
    let mut identity=version.trim().to_owned();
    if cfg!(all(windows,target_env="msvc")){identity.push_str(";");identity.push_str(runtime_build::MSVC_LINKER_FLAG);}
    Ok(identity)
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
    let worker = std::thread::Builder::new().name("tsrust".into()).stack_size(tsrust_compiler::COMPILER_STACK_BYTES).spawn(run);
    let result = match worker {
        Ok(worker) => worker.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic)),
        Err(_) => run(),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            let color = color_enabled(std::io::stderr().is_terminal());
            eprintln!("{}: {error}", styled("error", "1;31", color));
            ExitCode::FAILURE
        }
    }
}

fn test_timeout_value(text:&str)->Result<u64,String>{let value=text.parse::<u64>().map_err(|_|"--timeout requires a nonnegative integer millisecond duration".to_owned())?;std::time::Instant::now().checked_add(std::time::Duration::from_millis(value)).ok_or_else(||"--timeout exceeds the backend Instant range".to_owned())?;Ok(value)}
