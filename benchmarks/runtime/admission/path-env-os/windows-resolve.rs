// Admission-only composition over the existing Windows root parser/resolver.
// This file is included inside existing_resolution, where those helpers live.
pub fn windows_many(
    paths: &[&str],
    mut cwd: impl FnMut() -> Result<String, String>,
    mut drive_cwd: impl FnMut(&str) -> Result<Option<String>, String>,
) -> Result<String, String> {
    let mut device = String::new();
    let mut absolute = false;
    let mut tails = Vec::new();
    for path in paths.iter().rev().copied().filter(|path| !path.is_empty()) {
        let root = windows_root(path);
        if !root.device.is_empty() {
            if !device.is_empty() && !device.eq_ignore_ascii_case(&root.device) {
                continue;
            }
            if device.is_empty() {
                device = root.device;
            }
        }
        if !absolute {
            if !path[root.tail..].is_empty() {
                tails.push(&path[root.tail..]);
            }
            absolute = root.absolute;
        }
        // A rooted tail can still need a device from an earlier operand.
        if absolute && !device.is_empty() {
            break;
        }
    }
    let tail = tails.into_iter().rev().collect::<Vec<_>>().join("\\");
    let composed = if absolute { format!("{device}\\{tail}") } else { format!("{device}{tail}") };
    let root = windows_root(&composed);
    // Query only an unresolved drive. A failed Unicode conversion must not
    // become an absent drive context and trigger an unrelated cwd fallback.
    let drive = if !root.device.is_empty() && !root.absolute {
        drive_cwd(&root.device)?
    } else {
        None
    };
    resolve_path_checked(&composed, true, &mut cwd, &mut |_| Ok(drive.clone()))
}
