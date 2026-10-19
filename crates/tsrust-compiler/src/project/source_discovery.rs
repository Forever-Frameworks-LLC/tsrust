//! Shared deterministic project-file discovery for CLI and editor clients.
use std::{fs, path::PathBuf};

pub fn discover(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    discover_inner(inputs,false)
}

/// Source files and nested project configuration for consistent dev snapshots.
pub fn discover_project_inputs(inputs:&[PathBuf])->Result<Vec<PathBuf>,String> {discover_inner(inputs,true)}

fn discover_inner(inputs:&[PathBuf],include_configs:bool)->Result<Vec<PathBuf>,String> {
    let mut pending = inputs.to_vec();
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        let metadata =
            fs::symlink_metadata(&path).map_err(|error| format!("{}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            for entry in
                fs::read_dir(&path).map_err(|error| format!("{}: {error}", path.display()))?
            {
                let entry = entry.map_err(|error| error.to_string())?;
                let kind = entry.file_type().map_err(|error| error.to_string())?;
                if kind.is_symlink() {
                    continue;
                }
                if kind.is_dir()
                    && !matches!(
                        entry.file_name().to_str(),
                        Some(".git" | "node_modules" | "target")
                    )
                {
                    pending.push(entry.path());
                } else if kind.is_file()
                    && (include_configs && entry.file_name()=="tsrust.config.json" || entry
                        .path()
                        .extension()
                        .is_some_and(|extension| (extension == "tsr" || extension == "tsrx") || include_configs && matches!(extension.to_str(),Some("rs"|"json"|"jsonc"))))
                {
                    files.push(entry.path());
                }
            }
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    let mut canonical = files
        .into_iter()
        .map(|path| fs::canonicalize(&path).map_err(|error| format!("{}: {error}", path.display())))
        .collect::<Result<Vec<_>, _>>()?;
    canonical.sort();
    canonical.dedup();
    Ok(canonical)
}

