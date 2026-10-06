//! Native test file discovery. Explicit files may use any .tsr name.
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
pub(crate) fn select(paths: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    let defaults = [PathBuf::from(".")];
    let paths = if paths.is_empty() { &defaults[..] } else { paths };
    let mut selected = Vec::new();
    let mut seen = BTreeSet::new();
    for path in paths {
        if path.is_dir() {
            let mut discovered = Vec::new();
            visit(path, &mut discovered)?;
            discovered.sort();
            for file in discovered { add(&file, &mut seen, &mut selected)?; }
        } else {
            if path.extension().and_then(|value| value.to_str()) != Some("tsr") {
                return Err(format!("Test input must be a .tsr file or directory: {}", path.display()));
            }
            add(path, &mut seen, &mut selected)?;
        }
    }
    if selected.is_empty() { return Err("No test files found (.test.tsr or .spec.tsr).".into()); }
    Ok(selected)
}
fn add(path: &Path, seen: &mut BTreeSet<PathBuf>, selected: &mut Vec<PathBuf>) -> Result<(), String> {
    let canonical = std::fs::canonicalize(path).map_err(|error| format!("{}: {error}", path.display()))?;
    if seen.insert(canonical.clone()) { selected.push(canonical); }
    Ok(())
}
fn visit(path: &Path, selected: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in std::fs::read_dir(path).map_err(|error| format!("{}: {error}", path.display()))? {
        let entry = entry.map_err(|error| error.to_string())?;
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if kind.is_dir() {
            if name.starts_with('.') || matches!(name.as_ref(), "target" | "node_modules" | "archive") { continue; }
            visit(&entry.path(), selected)?;
        } else if kind.is_file() && crate::project::is_test_entry(&name) {
            selected.push(entry.path());
        }
    }
    Ok(())
}
