//! Shared deterministic project-file discovery for CLI and editor clients.
use std::{fs, path::PathBuf};

pub fn discover(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    Ok(discover_inner(inputs, false, true, &mut || false)?.expect("uninterrupted discovery"))
}

/// Source files and nested project configuration for consistent dev snapshots.
pub fn discover_project_inputs(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, String> {
    Ok(discover_inner(inputs, true, true, &mut || false)?.expect("uninterrupted discovery"))
}

/// Cancellable source discovery. Compiler/diagnostic callers include ignored files;
/// editor background discovery can honor workspace-local .ignore/.gitignore rules.
/// Explicit file inputs always remain selected. None means cancellation, never an
/// empty successful scan that could clear diagnostics. Hidden files remain visible.
pub fn discover_cancellable(
    inputs: &[PathBuf],
    include_ignored: bool,
    mut interrupted: impl FnMut() -> bool,
) -> Result<Option<Vec<PathBuf>>, String> {
    discover_inner(inputs, false, include_ignored, &mut interrupted)
}

fn discover_inner(
    inputs: &[PathBuf],
    include_configs: bool,
    include_ignored: bool,
    interrupted: &mut impl FnMut() -> bool,
) -> Result<Option<Vec<PathBuf>>, String> {
    let mut files = Vec::new();
    for path in inputs {
        if interrupted() { return Ok(None); }
        let metadata = fs::symlink_metadata(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() { continue; }
        if metadata.is_file() {
            files.push(path.clone());
            continue;
        }
        if !metadata.is_dir() { continue; }
        let mut builder = ignore::WalkBuilder::new(path);
        builder.standard_filters(false)
            .ignore(!include_ignored).git_ignore(!include_ignored)
            .require_git(false).follow_links(false)
            .filter_entry(|entry| entry.depth() == 0 || !entry.file_type().is_some_and(|kind|
                kind.is_dir() && matches!(entry.file_name().to_str(), Some(".git" | "node_modules" | "target"))));
        let mut walker = builder.build();
        loop {
            if interrupted() { return Ok(None); }
            let Some(entry) = walker.next() else { break; };
            let entry = entry.map_err(|error| format!("{}: {error}", path.display()))?;
            if let Some(error) = entry.error() {
                return Err(format!("{}: {error}", entry.path().display()));
            }
            if entry.file_type().is_some_and(|kind| kind.is_file())
                && entry.path().extension().is_some_and(|extension|
                    extension == "tsr" || extension == "tsrx"
                    || include_configs && matches!(extension.to_str(), Some("rs" | "json" | "jsonc" | "txt")))
            {
                files.push(entry.into_path());
            }
        }
    }
    let mut canonical = Vec::with_capacity(files.len());
    for path in files {
        if interrupted() { return Ok(None); }
        canonical.push(fs::canonicalize(&path).map_err(|error| format!("{}: {error}", path.display()))?);
    }
    canonical.sort();
    canonical.dedup();
    Ok(Some(canonical))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_preserves_inputs_and_applies_only_requested_workspace_ignores() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("workspace");
        fs::create_dir_all(root.join("nested")).unwrap();
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(temporary.path().join(".gitignore"), "workspace/\n").unwrap();
        for name in ["z.tsr", ".hidden.tsr", "ignored.tsr", "nested/keep.tsrx", "nested/drop.tsr", "target/build.tsr", "tsrust.config.json", "native.rs", "note.md"] {
            fs::write(root.join(name), "").unwrap();
        }
        fs::write(root.join(".gitignore"), "ignored.tsr\nnested/*.tsr*\n").unwrap();
        fs::write(root.join("nested/.gitignore"), "!keep.tsrx\n").unwrap();
        let selected = |names: &[&str]| {
            let mut paths = names.iter().map(|name| fs::canonicalize(root.join(name)).unwrap()).collect::<Vec<_>>();
            paths.sort(); paths
        };
        assert_eq!(discover(&[root.clone(), root.join("z.tsr")]).unwrap(), selected(&["z.tsr", ".hidden.tsr", "ignored.tsr", "nested/keep.tsrx", "nested/drop.tsr"]));
        assert_eq!(discover_cancellable(&[root.clone()], false, || false).unwrap().unwrap(), selected(&["z.tsr", ".hidden.tsr", "nested/keep.tsrx"]));
        assert_eq!(discover_cancellable(&[root.clone(), root.join("ignored.tsr")], false, || false).unwrap().unwrap(), selected(&["z.tsr", ".hidden.tsr", "nested/keep.tsrx", "ignored.tsr"]));
        assert_eq!(discover_project_inputs(&[root.clone()]).unwrap(), selected(&["z.tsr", ".hidden.tsr", "ignored.tsr", "nested/keep.tsrx", "nested/drop.tsr", "tsrust.config.json", "native.rs"]));
        assert_eq!(discover_cancellable(&[root.clone()], true, || true).unwrap(), None);
        let mut polls = 0;
        assert_eq!(discover_cancellable(&[root.clone()], true, || { polls += 1; polls > 3 }).unwrap(), None);
        assert!(discover(&[root.join("missing")]).unwrap_err().contains("missing"));
        fs::write(root.join(".ignore"), "z.tsr\n").unwrap();
        assert!(!discover_cancellable(&[root.clone()], false, || false).unwrap().unwrap().contains(&fs::canonicalize(root.join("z.tsr")).unwrap()));
        fs::remove_file(root.join(".ignore")).unwrap();
        fs::write(root.join(".gitignore"), "z.tsr\n").unwrap();
        assert!(!discover_cancellable(&[root.clone()], false, || false).unwrap().unwrap().contains(&fs::canonicalize(root.join("z.tsr")).unwrap()));
        #[cfg(unix)] {
            std::os::unix::fs::symlink(temporary.path(), root.join("loop")).unwrap();
            std::os::unix::fs::symlink(root.join("z.tsr"), root.join("alias.tsr")).unwrap();
            assert_eq!(discover(&[root]).unwrap().len(), 5);
        }
    }
}
