//! Shared bounded, serial filesystem traversal for compiler and checked runtime APIs.
use std::path::Path;

pub fn walk(
    root: &Path,
    include_ignored: bool,
    interrupted: &mut impl FnMut() -> bool,
    mut visit: impl FnMut(ignore::DirEntry) -> Result<(), String>,
) -> Result<bool, String> {
        let mut builder = ignore::WalkBuilder::new(root);
        builder.standard_filters(false)
            .ignore(!include_ignored).git_ignore(!include_ignored)
            .require_git(false).follow_links(false)
            .filter_entry(|entry| entry.depth() == 0 || !entry.file_type().is_some_and(|kind|
                kind.is_dir() && matches!(entry.file_name().to_str(), Some(".git" | "node_modules" | "target"))));
        let mut walker = builder.build();
        loop {
            if interrupted() { return Ok(false); }
            let Some(entry) = walker.next() else { break; };
            let entry = entry.map_err(|error| format!("{}: {error}", root.display()))?;
            if let Some(error) = entry.error() {
                return Err(format!("{}: {error}", entry.path().display()));
            }
            if entry.depth() > 0 && entry.file_type().is_some_and(|kind| kind.is_file() || kind.is_dir()) {
                visit(entry)?;
            }
        }
    Ok(true)
}
