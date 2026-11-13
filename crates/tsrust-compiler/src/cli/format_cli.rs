//! Recursive formatting is isolated from compiler and native-build dispatch.
use std::{
    env, fs,
    io::{IsTerminal, Read},
    path::PathBuf,
};

pub(super) fn run(mut args: impl Iterator<Item = String>) -> Result<u8, String> {
    let mut inputs = Vec::new();
    let mut check = false;
    let mut write = false;
    let mut color = "auto".to_owned();
    let mut progress = false;
    let mut text_limit: Option<usize> = None;
    let mut stdin_path = None;
    let mut options = tsrust_compiler::FormatOptions::default();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--progress" if !progress => progress = true,
            "--stdin-filepath" if stdin_path.is_none() => {
                stdin_path = Some(PathBuf::from(args.next().ok_or("--stdin-filepath requires a source path")?));
            }
            "--text-files-up-to" if text_limit.is_none() => {
                let value = args.next().ok_or("--text-files-up-to requires a positive byte limit")?;
                text_limit = Some(value.parse::<usize>().ok().filter(|limit| *limit > 0 && *limit < usize::MAX)
                    .ok_or("--text-files-up-to requires a positive byte limit")?);
            }
            "--options" => {
                let value = args.next().ok_or("--options requires a JSON object")?;
                let value = serde_json::from_str(&value).map_err(|error| format!("Invalid format options JSON: {error}"))?;
                options = tsrust_compiler::FormatOptions::overlay(&options, &value)?;
            }
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
    if let Some(path) = stdin_path {
        if text_limit.is_some() { return Err("--text-files-up-to applies only to disk files".into()); }
        if !inputs.is_empty() || check || write { return Err("--stdin-filepath cannot be combined with paths, --check or --write".into()); }
        let path = if path.is_absolute() { path } else { cwd.join(path) };
        let options = tsrust_compiler::FormatOptions::for_path(&path, &options)?;
        let mut source = String::new();
        std::io::stdin().take(16 * 1024 * 1024 + 1).read_to_string(&mut source).map_err(|error| error.to_string())?;
        if source.len() > 16 * 1024 * 1024 { return Err("Formatter input exceeds 16 MiB.".into()); }
        let result = tsrust_compiler::format_with_options(&source, &options).map_err(|error| error.render_terminal(&path.display().to_string(), &source, false))?;
        print!("{result}");
        return Ok(0);
    }
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
    if progress { eprintln!("{}", super::styled(&format!("Formatting {} source files…", files.len()), "1;36", color)); }
    let mut changed = 0;
    let mut written = 0;
    let mut skipped = 0;
    let mut failed = false;
    for path in &files {
        let display = path
            .strip_prefix(&cwd)
            .unwrap_or(path)
            .display()
            .to_string();
        let source = match read_source(path, text_limit) {
            Ok(Some(source)) => source,
            Ok(None) => {
                skipped += 1;
                continue;
            }
            Err(error) => {
                eprintln!("{display}: {error}");
                failed = true;
                continue;
            }
        };
        let resolved = tsrust_compiler::FormatOptions::for_path(path, &options)?;
        let formatted = match tsrust_compiler::format_with_options(&source, &resolved) {
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
    if skipped > 0 {
        eprintln!("Skipped {skipped} binary or oversized source {}.", if skipped == 1 { "file" } else { "files" });
    }
    if batch || (write && progress) {
        let stdout_color = super::color_enabled(std::io::stdout().is_terminal());
        if check && changed > 0 {
            let message = format!("{changed} {} need formatting.", if changed == 1 { "file" } else { "files" });
            println!("{}", super::styled(&message, "1;33", stdout_color));
        } else if !check {
            let message = format!("Formatted {written} {}.", if written == 1 { "file" } else { "files" });
            println!("{}", super::styled(&message, if failed { "1;31" } else { "1;32" }, stdout_color));
        }
    }
    Ok(u8::from(failed || check && changed > 0))
}

fn read_source(path: &std::path::Path, text_limit: Option<usize>) -> std::io::Result<Option<String>> {
    let Some(limit) = text_limit else { return fs::read_to_string(path).map(Some) };
    let file = fs::File::open(path)?;
    if file.metadata()?.len() > limit as u64 { return Ok(None); }
    let mut bytes = Vec::new();
    file.take(limit as u64 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > limit || bytes.contains(&0) { return Ok(None); }
    Ok(String::from_utf8(bytes).ok())
}
