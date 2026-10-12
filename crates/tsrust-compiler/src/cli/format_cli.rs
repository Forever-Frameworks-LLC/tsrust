//! Recursive formatting is isolated from compiler and native-build dispatch.
use std::{
    env, fs,
    io::IsTerminal,
    path::PathBuf,
};

pub(super) fn run(mut args: impl Iterator<Item = String>) -> Result<u8, String> {
    let mut inputs = Vec::new();
    let mut check = false;
    let mut write = false;
    let mut color = "auto".to_owned();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" if !check && !write => check = true,
            "--write" if !check && !write => write = true,
            "--color" => {
                color = args
                    .next()
                    .ok_or("--color requires auto, always or never")?
            }
            _ if arg.starts_with("--color=") => color = arg[8..].to_owned(),
            _ if arg.starts_with('-') => {
                return Err(format!("Unknown or repeated format option: {arg}"));
            }
            _ => inputs.push(PathBuf::from(arg)),
        }
    }
    if !matches!(color.as_str(), "auto" | "always" | "never") {
        return Err("--color requires auto, always or never".into());
    }
    let cwd = env::current_dir().map_err(|error| error.to_string())?;
    let batch = inputs.len() != 1 || inputs.first().is_some_and(|path| path.is_dir());
    if inputs.is_empty() {
        inputs.push(super::project_config::default_root()?);
    }
    let files = tsrust_compiler::source_discovery::discover(&inputs)?;
    let color = color == "always"
        || (color == "auto"
            && std::io::stderr().is_terminal()
            && env::var_os("NO_COLOR").is_none()
            && env::var("TERM").as_deref() != Ok("dumb"));
    let mut changed = 0;
    let mut written = 0;
    let mut failed = false;
    for path in &files {
        let display = path
            .strip_prefix(&cwd)
            .unwrap_or(path)
            .display()
            .to_string();
        let source = match fs::read_to_string(path) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{display}: {error}");
                failed = true;
                continue;
            }
        };
        let formatted = match tsrust_compiler::format(&source) {
            Ok(formatted) => formatted,
            Err(error) => {
                eprint!("{}", error.render_terminal(&display, &source, color));
                failed = true;
                continue;
            }
        };
        if formatted != source {
            changed += 1;
            if !check && (write || batch) {
                match fs::write(path, &formatted) {
                    Ok(()) => written += 1,
                    Err(error) => {
                        eprintln!("{display}: {error}");
                        failed = true;
                    }
                }
            }
            if check && batch {
                println!("{display}");
            }
        }
        if !check && !write && !batch {
            print!("{formatted}");
        }
    }
    if batch {
        if check && changed > 0 {
            println!(
                "{changed} {} need formatting.",
                if changed == 1 { "file" } else { "files" }
            );
        } else if !check {
            println!(
                "Formatted {written} {}.",
                if written == 1 { "file" } else { "files" }
            );
        }
    }
    Ok(u8::from(failed || check && changed > 0))
}
