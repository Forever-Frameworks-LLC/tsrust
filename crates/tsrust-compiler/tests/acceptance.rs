mod support;

use support::{execute, reject};
use tsrust_compiler::Mode;

#[test]
fn forward_calls_and_transitive_consumption() {
    execute(
        r#"type User = { name: string; };

function main() {
    const user: User = { name: "Pat" };
    const saved = outer(user);
    console.log(saved.name);
}
function outer(user: User): User { return inner(user); }
function inner(user: User): User { return user; }
"#,
        r#"Pat
"#,
        Some(&[
            (r#"outer.user"#, Mode::Consume),
            (r#"inner.user"#, Mode::Consume),
        ]),
    );
}

#[test]
fn consuming_parameter_can_mutate_before_return() {
    execute(
        r#"type User = { name: string; };

function change(user: User): User { user.name = "Sam"; return user; }
function main() {
    const user: User = { name: "Pat" };
    const result = change(user);
    console.log(result.name);
}
"#,
        r#"Sam
"#,
        Some(&[(r#"change.user"#, Mode::Consume)]),
    );
}

#[test]
fn string_parameters_and_owned_assignment() {
    execute(
        r#"
function read(value: string) { console.log(value); }
function take(value: string): string { return value; }
function main() {
    const value = "Pat";
    read(value);
    const moved = take(value);
    console.log(moved);
}
"#,
        r#"Pat
Pat
"#,
        Some(&[
            (r#"read.value"#, Mode::Read),
            (r#"take.value"#, Mode::Consume),
        ]),
    );
}

#[test]
fn no_hidden_clone_on_discarded_owned_expression() {
    reject(
        r#"type User = { name: string; };
function main() {
const user: User = { name: "Pat" }; user; console.log(user.name);
}"#,
        r#"transferred"#,
    );
}

#[test]
fn conflicting_nested_argument() {
    reject(
        r#"type User = { name: string; };

function change(user: User): string { user.name = "Sam"; return "done"; }
function both(user: User, message: string) { console.log(user.name); console.log(message); }
function main() { const user: User = { name: "Pat" }; both(user, change(user)); }
"#,
        r#"conflicting-access"#,
    );
}

#[test]
fn nested_owned_result_releases_input_access() {
    execute(
        r#"type User = { name: string; };

function change(user: User) { user.name = "Sam"; }
function both(message: string, user: User) { console.log(message); change(user); }
function main() {
    const user: User = { name: " Pat " };
    both(user.name.trim(), user);
    console.log(user.name);
}
"#,
        r#"Pat
Sam
"#,
        None,
    );
}

#[test]
fn two_mutable_arguments_rejected() {
    reject(
        r#"type User = { name: string; };

function both(first: User, second: User) { first.name = "A"; second.name = "B"; }
function main() { const user: User = { name: "Pat" }; both(user, user); }
"#,
        r#"conflicting-access"#,
    );
}

#[test]
fn field_loan_conflicts_with_record_mutation() {
    reject(
        r#"type User = { name: string; };

function change(user: User) { user.name = "Sam"; }
function both(name: string, user: User) { console.log(name); change(user); }
function main() { const user: User = { name: "Pat" }; both(user.name, user); }
"#,
        r#"conflicting-access"#,
    );
}

#[test]
fn move_inside_consuming_function_is_checked() {
    reject(
        r#"type User = { name: string; };

function invalid(user: User): User {
    const saved = user;
    console.log(user.name);
    return saved;
}
function main() {}
"#,
        r#"transferred"#,
    );
}

#[test]
fn assignment_cannot_consume_its_target() {
    reject(
        r#"type User = { name: string; };

function take(user: User): string { const saved = user; return "Sam"; }
function main() { const user: User = { name: "Pat" }; user.name = take(user); }
"#,
        r#"transferred"#,
    );
}

#[test]
fn replace_from_own_field_transform() {
    execute(
        r#"type User = { name: string; };

function change(user: User) { user.name = user.name.trim(); }
function main() { const user: User = { name: " Pat " }; change(user); console.log(user.name); }
"#,
        r#"Pat
"#,
        Some(&[(r#"change.user"#, Mode::Mutate)]),
    );
}

#[test]
fn contextual_record_literal_and_identifier_hygiene() {
    execute(
        r#"
type String = { match: string; };
function display(type: String) { console.log(type.match); }
function main() { display({ match: "ok" }); }
"#,
        r#"ok
"#,
        None,
    );
}

#[test]
fn frontend_rejections() {
    let cases = &[
        (r#"function main() { console.log(missing); }"#, r#"name"#),
        (r#"function main() { absent(); }"#, r#"name"#),
        (r#"function main() { Math.abs(); }"#, r#"arity"#),
        (r#"function main() { const x: Unknown = "a"; }"#, r#"type"#),
        (
            r#"type User = { name: string; };
function main() { const x: User = {}; }"#,
            r#"type"#,
        ),
        (
            r#"type User = { name: string; };
function main() { const x: User = { other: "a" }; }"#,
            r#"type"#,
        ),
        (
            r#"type User = { name: string; };
function main() { const x: User = { name: "a", name: "b" }; }"#,
            r#"duplicate"#,
        ),
        (
            r#"type User = { name: string; };
function main() { const x: User = { name: "a" }; console.log(x.other); }"#,
            r#"type"#,
        ),
        (
            r#"function main() { const x = "a"; const x = "b"; }"#,
            r#"duplicate"#,
        ),
        (
            r#"function f(x: string, x: string) {} function main() {}"#,
            r#"duplicate"#,
        ),
        (
            r#"function main() { const x = console.log("a"); }"#,
            r#"type"#,
        ),
        (r#"function f(): string {} function main() {}"#, r#"type"#),
        (
            r#"function f(): string { return; } function main() {}"#,
            r#"type"#,
        ),
        (r#"function main() { return "a"; }"#, r#"type"#),
        (
            r#"function main() { const x = "a"; x = "b"; }"#,
            r#"unsupported"#,
        ),
        (
            r#"function f(c: Function<void>): () => int { if (true) { return c; } return f(c); } function main() {}"#,
            r#"type"#,
        ),
        (
            r#"function f(c: Function<void>): () => int { return g(c); } function g(c: Function<void>): () => void { return f(c); } function main() {}"#,
            r#"type"#,
        ),
        (
            r#"function main() { return; console.log("a"); }"#,
            r#"unsupported"#,
        ),
        (r#"function main() { if ("a") {} }"#, r#"type"#),
        (r#"function main() { const x = 1.5n; }"#, r#"unsupported"#),
        (r#"function main() { "oops }"#, r#"syntax"#),
        (r#"function main() { console.log("\uZ000"); }"#, r#"syntax"#),
        (r#"/* no end"#, r#"syntax"#),
        (r#"function main() {"#, r#"syntax"#),
        (r#"function main() { console.log("a") }"#, r#"syntax"#),
        (r#"function other() {}"#, r#"entry"#),
        (r#"function main(value: string) {}"#, r#"entry"#),
        (r#"function main() { main(); }"#, r#"unsupported"#),
        (r#"function clone() {} function main() {}"#, r#"duplicate"#),
        (
            r#"function main() { const console = "a"; }"#,
            r#"duplicate"#,
        ),
        (
            r#"type User = { name: string; };
function f(x: User): string { const name = x.name; return x.name; } function main() {}"#,
            r#"partial-transfer"#,
        ),
        (r#"function main() { clone(true); }"#, r#"unsupported"#),
        (r#"function main() { "a".unknown(); }"#, r#"unsupported"#),
    ];
    for (source, code) in cases {
        reject(source, code);
    }
}

#[test]
fn original_source_corpus() {
    for case in support::corpus::CASES {
        let source =
            std::fs::read_to_string(support::fixture_dir().join(format!("{}.tsr", case.name)))
                .unwrap();
        match &case.expected {
            support::corpus::Expected::Output(stdout) => {
                let rust = execute(&source, stdout, Some(case.contracts));
                if case.name != "clone_independence" {
                    assert!(!rust.contains("Clone::clone("), "{}", case.name);
                }
            }
            support::corpus::Expected::Rejected { source_code, .. } => {
                let diagnostic = reject(&source, source_code);
                assert!(!diagnostic.related.is_empty(), "{}", case.name);
                assert!(tsrust_compiler::line_column(&source, diagnostic.span).0 > 1);
                assert_ne!(diagnostic.span, diagnostic.related[0].0);
            }
        }
    }
}

#[test]
fn strings_escape_rust_syntax() {
    let value = "\"{}\\\n\t\r\u{8}\u{c}\0 café 😀";
    let source = format!(
        "function main() {{ console.log({}); }}",
        tsrust_compiler::json_string(value)
    );
    // Rust output is compared as bytes, preserving carriage returns as well.
    execute(&source, &format!("{value}\n"), None);
}

#[test]
fn check_needs_no_rust_and_rejects_before_backend() {
    for (name, valid) in [("read_reuse", true), ("use_after_call", false)] {
        let path = support::fixture_dir().join(format!("{name}.tsr"));
        let source = std::fs::read_to_string(&path).unwrap();
        let output = support::run(
            support::cli()
                .arg("check")
                .arg(path)
                .arg("--json")
                .env("PATH", ""),
            10,
        );
        assert_eq!(output.status.code(), Some(if valid { 0 } else { 1 }));
        let expected = match tsrust_compiler::check(&source) {
            Ok(analysis) => analysis.to_json(),
            Err(diagnostic) => format!(
                "{{\"ok\":false,\"diagnostics\":[{}]}}",
                diagnostic.to_json(&source)
            ),
        };
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("{expected}\n")
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn cli_build_emit_run_and_source_protection() {
    let directory = support::TempDirectory::new();
    let original = "function main() { console.log(\"hello\"); }";
    let source = directory.source(original);
    for command in ["emit", "build", "run"] {
        let output = directory.0.join(if command == "emit" {
            "generated.rs"
        } else {
            "program"
        });
        let mut cli = support::cli();
        cli.arg(command).arg(&source);
        if command != "run" {
            cli.arg("-o").arg(&output);
        }
        let result = support::run(&mut cli, 60);
        support::success(&result);
        if command == "run" {
            assert_eq!(result.stdout, b"hello\n");
        } else {
            assert!(output.exists());
        }
    }
    let result = support::run(
        support::cli()
            .arg("emit")
            .arg(&source)
            .arg("-o")
            .arg(&source),
        10,
    );
    assert!(!result.status.success());
    assert_eq!(std::fs::read_to_string(&source).unwrap(), original);
}

#[test]
fn native_build_uses_pinned_backend_outside_the_workspace() {
    let directory = support::TempDirectory::new();
    let source = directory.source("function main() { console.log(\"pinned\"); }");
    std::fs::write(
        directory.0.join("rust-toolchain.toml"),
        "[toolchain]\nchannel = \"unavailable-for-test\"\n",
    )
    .unwrap();
    let binary = directory
        .0
        .join(format!("pinned{}", std::env::consts::EXE_SUFFIX));
    let output = support::run(
        support::cli()
            .arg("build")
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .current_dir(&directory.0)
            .env("RUSTUP_TOOLCHAIN", "unavailable-for-test"),
        60,
    );
    support::success(&output);
    let run = support::run(&mut std::process::Command::new(&binary), 10);
    support::success(&run);
    assert_eq!(run.stdout, b"pinned\n");
}

#[test]
fn missing_backend_reports_setup_without_fallback_or_output() {
    let directory = support::TempDirectory::new();
    let source = directory.source("function main() { console.log(\"hello\"); }");
    let binary = directory.0.join("program");
    let output = support::run(
        support::cli()
            .arg("build")
            .arg(source)
            .arg("-o")
            .arg(&binary)
            .env("PATH", "")
            // The backend is also found under the Cargo home; hide that too.
            .env("HOME", &directory.0)
            .env("USERPROFILE", &directory.0)
            .env_remove("CARGO_HOME"),
        10,
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(!binary.exists());
    let message = String::from_utf8(output.stderr).unwrap();
    assert!(message.contains(&format!(
        "rustup toolchain install {} --profile minimal",
        tsrust_compiler::RUST_BACKEND_VERSION
    )));
    assert!(!message.contains("compiler defect"));
}

#[test]
fn cli_release_rejects_unsupported_commands_and_duplicate_options() {
    for (command, options, expected) in [
        (
            "check",
            vec!["--release"],
            "--release applies only to build/run",
        ),
        (
            "emit",
            vec!["--release"],
            "--release applies only to build/run",
        ),
        (
            "build",
            vec!["--release", "--release"],
            "Unknown or repeated option: --release",
        ),
        (
            "run",
            vec!["--release", "--release"],
            "Unknown or repeated option: --release",
        ),
    ] {
        let result = support::run(
            support::cli()
                .arg(command)
                .arg("nonexistent.tsr")
                .args(options)
                .env("PATH", ""),
            10,
        );
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains(expected));
    }
    let help = support::run(support::cli().arg("--help"), 10);
    support::success(&help);
    assert!(String::from_utf8_lossy(&help.stdout).contains("--release                 Optimize build/run/test"));
}

#[cfg(unix)]
#[test]
fn cli_profiles_pass_explicit_checked_flags_to_the_pinned_backend() {
    use std::os::unix::fs::PermissionsExt;
    let directory = support::TempDirectory::new();
    let source = directory.source("function main() { console.log(\"profile\"); }");
    let mock = directory.0.join("rustup");
    std::fs::write(
        &mock,
        format!(
            r#"#!/bin/sh
if [ "$4" = '--version' ]; then
    printf '%s\n' "$@" > "$TSRUST_TEST_VERIFY_ARGS"
    printf 'rustc {} (mock)\n'
    exit 0
fi
printf '%s\n' "$@" > "$TSRUST_TEST_ARGS"
while [ "$#" -gt 0 ]; do
    if [ "$1" = '-o' ]; then
        shift
        printf '#!/bin/sh\nexit 0\n' > "$1"
        /bin/chmod +x "$1"
        exit 0
    fi
    shift
done
exit 1
"#,
            tsrust_compiler::RUST_BACKEND_VERSION
        ),
    )
    .unwrap();
    std::fs::set_permissions(&mock, std::fs::Permissions::from_mode(0o755)).unwrap();
    let verification = directory.0.join("backend-verify-args");
    let binary = directory.0.join("program");
    for command in ["build", "run"] {
        for release in [false, true] {
            let arguments = directory.0.join(if release {
                "backend-args-release"
            } else {
                "backend-args-debug"
            });
            let mut cli = support::cli();
            cli.arg(command)
                .arg(&source)
                .env("PATH", &directory.0)
                .env("TSRUST_TEST_ARGS", &arguments)
                .env("TSRUST_TEST_VERIFY_ARGS", &verification);
            if command == "build" {
                cli.arg("-o").arg(&binary);
            }
            if release {
                cli.arg("--release");
            }
            if verification.exists() {
                std::fs::remove_file(&verification).unwrap();
            }
            support::success(&support::run(&mut cli, 10));
            let verified = std::fs::read_to_string(&verification).unwrap();
            assert_eq!(
                verified.lines().collect::<Vec<_>>(),
                [
                    "run",
                    tsrust_compiler::RUST_BACKEND_VERSION,
                    "rustc",
                    "--version"
                ]
            );
            let captured = std::fs::read_to_string(&arguments).unwrap();
            let args: Vec<_> = captured.lines().collect();
            assert_eq!(args.len(), 11);
            assert_eq!(
                &args[..8],
                &[
                    "run",
                    tsrust_compiler::RUST_BACKEND_VERSION,
                    "rustc",
                    "--edition=2024",
                    "-C",
                    if release {
                        "opt-level=3"
                    } else {
                        "opt-level=0"
                    },
                    "-C",
                    "overflow-checks=on"
                ]
            );
            assert!(std::path::Path::new(args[8]).is_absolute());
            assert_eq!(std::path::Path::new(args[8]).extension().unwrap(), "rs");
            assert_eq!(args[9], "-o");
            assert!(std::path::Path::new(args[10]).is_absolute());
            if command == "build" {
                assert_eq!(std::path::Path::new(args[10]), binary);
            }
        }
    }
}

#[test]
fn cli_debug_and_release_preserve_native_output_and_checked_failures() {
    let directory = support::TempDirectory::new();
    for (source_text, expected, error) in [
        (
            "function main() { const text = \"  ΟΣ İ😀  \"; console.log(text.trim().toLowerCase()); console.log(text.length); console.log(6 * 7); }",
            "ος i\u{307}😀\n15\n42\n",
            None,
        ),
        (
            "function add(value: int): int { return value + 1; } function main() { console.log(\"before\"); console.log(add(9223372036854775807)); }",
            "before\n",
            Some("integer addition overflow"),
        ),
    ] {
        let source = directory.source(source_text);
        for command in ["build", "run"] {
            for release in [false, true] {
                let binary = directory
                    .0
                    .join(format!("program{}", std::env::consts::EXE_SUFFIX));
                let mut cli = support::cli();
                cli.arg(command).arg(&source);
                if command == "build" {
                    cli.arg("-o").arg(&binary);
                }
                if release {
                    cli.arg("--release");
                }
                let output = support::run(&mut cli, 60);
                let result = if command == "build" {
                    support::success(&output);
                    support::run(&mut std::process::Command::new(&binary), 10)
                } else {
                    output
                };
                assert_eq!(result.stdout, expected.as_bytes());
                if let Some(error) = error {
                    assert!(!result.status.success());
                    assert!(String::from_utf8_lossy(&result.stderr).contains(error));
                } else {
                    support::success(&result);
                    assert!(result.stderr.is_empty());
                }
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn missing_linker_reports_environment_setup_without_output(){
 let directory=support::TempDirectory::new();
 let source=directory.source("function main(){console.log(\"link\");}");
 let binary=directory.0.join("program");
 let output=support::run(support::cli().arg("build").arg(&source).arg("-o").arg(&binary).env("PATH", "").env("TSRUST_CACHE_DIR",directory.0.join("cache")),60);
 assert!(!output.status.success());
 let message=String::from_utf8_lossy(&output.stderr);
 assert!(message.contains("Native linker unavailable"),"{message}");
 assert!(message.contains("linker") && message.contains("not found"),"{message}");
 assert!(message.contains("PATH"),"{message}");
 assert!(!message.contains("compiler defect"),"{message}");
 assert!(!binary.exists());
}

#[cfg(unix)]
#[test]
fn rejected_generated_code_keeps_backend_details_and_is_not_missing_linker(){
 use std::os::unix::fs::PermissionsExt;
 let directory=support::TempDirectory::new();
 let source=directory.source("function main(){}");
 let mock=directory.0.join("rustup");
 std::fs::write(&mock,format!(r#"#!/bin/sh
if [ "$4" = '--version' ]; then
 printf 'rustc {} (mock)\n'
 exit 0
fi
printf 'error[E0308]: mismatched types\n' >&2
exit 1
"#,tsrust_compiler::RUST_BACKEND_VERSION)).unwrap();
 std::fs::set_permissions(&mock,std::fs::Permissions::from_mode(0o755)).unwrap();
 let binary=directory.0.join("program");
 let output=support::run(support::cli().arg("build").arg(&source).arg("-o").arg(&binary).env("PATH",&directory.0).env("TSRUST_CACHE_DIR",directory.0.join("cache")),15);
 assert!(!output.status.success());
 let message=String::from_utf8_lossy(&output.stderr);
 assert!(message.contains("compiler defect"),"{message}");
 assert!(message.contains("error[E0308]: mismatched types"),"{message}");
 assert!(!message.contains("Native linker unavailable"),"{message}");
 assert!(!binary.exists());
}

// A reader that closes the pipe early ends the command quietly: no panic
// message is written.
#[cfg(unix)]
#[test]
fn closed_standard_output_does_not_panic() {
    use std::io::Read;
    use std::process::Stdio;
    let directory = support::TempDirectory::new();
    // Many parameters make the contract listing long enough to outlive the reader.
    let mut source = String::new();
    for index in 0..400 {
        source.push_str(&format!("function f{index}(a: string, b: string, c: string): int {{ return a.length + b.length + c.length; }}\n"));
    }
    source.push_str("function main() {}\n");
    let path = directory.source(&source);
    let mut child = support::cli()
        .arg("check")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // Read one byte, then close the pipe while the command is still writing.
    let mut stdout = child.stdout.take().unwrap();
    let mut first = [0u8; 1];
    stdout.read_exact(&mut first).unwrap();
    drop(stdout);
    let mut stderr = String::new();
    child.stderr.take().unwrap().read_to_string(&mut stderr).unwrap();
    child.wait().unwrap();
    assert!(!stderr.contains("panicked"), "{stderr}");
    assert!(!stderr.contains("Broken pipe"), "{stderr}");
}
