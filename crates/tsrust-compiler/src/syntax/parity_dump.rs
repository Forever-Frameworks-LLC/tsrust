//! Opt-in parser parity dump. It records every parser-facing result for a
//! corpus directory so two source revisions can be compared byte for byte.
//! No compiler execution path depends on this module.
use super::*;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn fingerprint(text: &str) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn program_text(result: Result<Program, Diagnostic>) -> String {
    match result {
        Ok(program) => {
            let mut aliases = program.type_aliases.iter().collect::<Vec<_>>();
            aliases.sort_by(|left, right| left.0.cmp(right.0));
            format!(
                "Ok records={:?} functions={:?} module={:?} calls={:?} checks={:?} bases={:?} local={:?} aliases={:?}",
                program.records,
                program.functions,
                program.module_bindings,
                program.call_type_arguments,
                (&program.type_argument_checks,&program.interface_argument_checks),
                program.unresolved_bases,
                program.unresolved_local_function,
                aliases
            )
        }
        Err(error) => format!("Err {error:?}"),
    }
}

struct ExpressionState {
    value: Result<Expr, Diagnostic>,
    at: usize,
    depth: usize,
    pending: Vec<Stmt>,
    patterns: Vec<Stmt>,
    calls: Vec<(Span, Vec<crate::type_syntax::ValueAnnotation>)>,
}

fn expression_state(source: &str) -> Result<ExpressionState, Diagnostic> {
    let mut parser = Parser::new(source, Default::default(), Default::default())?;
    let value = parser.expression();
    Ok(ExpressionState {
        value,
        at: parser.i,
        depth: parser.depth,
        pending: std::mem::take(&mut parser.pending),
        patterns: std::mem::take(&mut parser.parameter_patterns),
        calls: std::mem::take(&mut parser.call_type_arguments),
    })
}

fn expression_text(state: Result<ExpressionState, Diagnostic>) -> String {
    match state {
        Ok(state) => format!(
            "{:?} at={} depth={} pending={:?} patterns={:?} calls={:?}",
            state.value, state.at, state.depth, state.pending, state.patterns, state.calls
        ),
        Err(error) => format!("Lex {error:?}"),
    }
}

const LABELS: [&str; 9] = [
    "raw",
    "raw-test",
    "entry",
    "diagnostics",
    "diagnostics-test",
    "format",
    "type",
    "match",
    "expression",
];

/// Parse on an ordinary 2 MiB thread, exactly like a test worker. Only the
/// derived Debug rendering of the finished result uses the caller's larger
/// stack; that renderer is test tooling, not a compiler path.
fn observe(source: &str, label: &'static str) -> String {
    fn ordinary<T: Send>(work: impl FnOnce() -> T + Send) -> T {
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .stack_size(2 * 1024 * 1024)
                .spawn_scoped(scope, work)
                .unwrap()
                .join()
                .unwrap()
        })
    }
    match label {
        "raw" => program_text(ordinary(|| parse_raw(source, false, Default::default(), Default::default()))),
        "raw-test" => program_text(ordinary(|| parse_raw(source, true, Default::default(), Default::default()))),
        "entry" => program_text(ordinary(|| parse(source))),
        "diagnostics" => format!("{:?}", ordinary(|| module_diagnostics(source, false))),
        "diagnostics-test" => format!("{:?}", ordinary(|| module_diagnostics(source, true))),
        "format" => format!("{:?}", ordinary(|| crate::format(source))),
        "type" => format!("{:?}", ordinary(|| crate::type_syntax::parse(source))),
        "match" => format!("{:?}", ordinary(|| match_syntax::parse(source))),
        "expression" => expression_text(ordinary(|| expression_state(source))),
        _ => unreachable!("parity label"),
    }
}

fn files(root: &Path, out: &mut Vec<PathBuf>) {
    let mut entries = std::fs::read_dir(root)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<Vec<_>>();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            files(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Resumable: completed files are skipped, and an observation named in the
/// skip list is recorded as ABORT. The marker names the observation running
/// when a process dies, so the driver can add it to the skip list.
fn dump(corpus: &Path, output: &Path, marker: &Path, skip: &std::collections::HashSet<String>, full: Option<&Path>) {
    use std::io::Write as _;
    let done = std::fs::read_to_string(output).unwrap_or_default();
    let done = done
        .lines()
        .filter_map(|line| line.split('\t').next())
        .collect::<std::collections::HashSet<_>>();
    let mut paths = Vec::new();
    files(corpus, &mut paths);
    let mut report = std::fs::OpenOptions::new().create(true).append(true).open(output).unwrap();
    for path in paths {
        let name = path.strip_prefix(corpus).unwrap().to_string_lossy().into_owned();
        if done.contains(name.as_str()) {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let mut line = name.clone();
        let mut text = String::new();
        for label in LABELS {
            let key = format!("{name}\t{label}");
            if skip.contains(&key) {
                write!(line, "\t{label}=ABORT").unwrap();
                continue;
            }
            std::fs::write(marker, &key).unwrap();
            let observed = observe(&source, label);
            write!(line, "\t{label}={:016x}", fingerprint(&observed)).unwrap();
            if full.is_some() {
                writeln!(text, "## {label}\n{observed}").unwrap();
            }
        }
        line.push('\n');
        report.write_all(line.as_bytes()).unwrap();
        if let Some(directory) = full {
            let target = directory.join(format!("{}.txt", name.replace('/', "__")));
            std::fs::write(target, text).unwrap();
        }
    }
    report.flush().unwrap();
    let _ = std::fs::remove_file(marker);
}

#[test]
#[ignore = "Opt-in parity dump; requires TSRUST_PARITY_CORPUS and TSRUST_PARITY_OUT"]
fn dump_parser_parity() {
    let corpus = PathBuf::from(std::env::var("TSRUST_PARITY_CORPUS").expect("corpus directory"));
    let output = PathBuf::from(std::env::var("TSRUST_PARITY_OUT").expect("output path"));
    let marker = output.with_extension("running");
    let full = std::env::var("TSRUST_PARITY_FULL").ok().map(PathBuf::from);
    let skip = std::env::var("TSRUST_PARITY_SKIP")
        .ok()
        .map(|path| std::fs::read_to_string(path).unwrap_or_default())
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect::<std::collections::HashSet<_>>();
    std::thread::Builder::new()
        .stack_size(1 << 30)
        .spawn(move || dump(&corpus, &output, &marker, &skip, full.as_deref()))
        .unwrap()
        .join()
        .unwrap();
}
