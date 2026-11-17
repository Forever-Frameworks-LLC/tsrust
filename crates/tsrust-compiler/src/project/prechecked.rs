//! Checked and emitted programs computed ahead of a build.
//!
//! While `tsrust dev` runs, the language server checks the application entry
//! against the editor's unsaved text after typing pauses and stores the emitted
//! Rust here. When the files are saved, `tsrust dev` loads the module graph,
//! and if a stored result has the same graph identity (every module's id and
//! text) from the same compiler build, it skips checking and emission. The
//! check is a deterministic function of that graph, so a reused result is the
//! one a fresh check would produce. `tsrust dev` stores its own checks here too,
//! so returning to sources it already checked (an edit, then its undo) skips
//! the check as well.
use crate::project::{ProjectGraph, SourceId};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

/// Identity of this compiler's source code, computed by the build script.
pub const COMPILER_IDENTITY: &str = env!("TSRUST_SOURCE_IDENTITY");

/// Stored results kept per project; older ones are removed.
const RETAINED: usize = 4;

/// The emitted program of one checked module graph.
pub struct Prechecked {
    pub rust: String,
    pub features: Vec<String>,
    pub keys: crate::native::stable_names::Keys,
    /// Checked by the language server rather than by `tsrust dev` itself.
    pub editor: bool,
}

/// Every module's id and source text, plus the entry and each resolved import
/// target. Equal identities give equal checked programs.
pub fn graph_identity(graph: &ProjectGraph) -> Vec<u8> {
    let mut identity = Vec::new();
    let mut push = |value: &[u8]| {
        identity.extend_from_slice(&(value.len() as u64).to_le_bytes());
        identity.extend_from_slice(value);
    };
    push(graph.entry.0.as_bytes());
    for (id, module) in &graph.modules {
        push(id.0.as_bytes());
        push(module.source.text.as_bytes());
        for import in &module.imports {
            push(import.target.as_ref().map_or(&[][..], |target| target.0.as_bytes()));
        }
    }
    identity
}

fn directory(root: &Path) -> PathBuf {
    root.join("target").join("tsrust-native").join("prechecked")
}

fn file(root: &Path, identity: &[u8]) -> PathBuf {
    let mut hash = Sha256::new();
    hash.update(COMPILER_IDENTITY.as_bytes());
    hash.update(identity);
    let name = hash.finalize().iter().take(16).map(|byte| format!("{byte:02x}")).collect::<String>();
    directory(root).join(format!("{name}.checked"))
}

/// Whether a result for this graph identity is stored.
pub fn contains(root: &Path, identity: &[u8]) -> bool {
    file(root, identity).is_file()
}

/// Store a result. The file is written beside its final name and renamed, so a
/// reader sees a complete result or none.
pub fn store(root: &Path, identity: &[u8], value: &Prechecked) -> Result<(), String> {
    let directory = directory(root);
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let header = serde_json::json!({ "features": value.features, "keys": value.keys, "identity": identity.len(), "editor": value.editor });
    let mut bytes = serde_json::to_vec(&header).map_err(|e| e.to_string())?;
    bytes.push(b'\n');
    bytes.extend_from_slice(identity);
    bytes.extend_from_slice(value.rust.as_bytes());
    let target = file(root, identity);
    let pending = target.with_extension(format!("pending-{}", std::process::id()));
    std::fs::write(&pending, &bytes).and_then(|()| std::fs::rename(&pending, &target)).map_err(|e| {
        let _ = std::fs::remove_file(&pending);
        e.to_string()
    })?;
    prune(&directory);
    Ok(())
}

/// Load the result for this graph identity, comparing the stored identity in full.
pub fn load(root: &Path, identity: &[u8]) -> Option<Prechecked> {
    let bytes = std::fs::read(file(root, identity)).ok()?;
    let newline = bytes.iter().position(|&byte| byte == b'\n')?;
    let header: serde_json::Value = serde_json::from_slice(&bytes[..newline]).ok()?;
    let length = usize::try_from(header.get("identity")?.as_u64()?).ok()?;
    let rest = &bytes[newline + 1..];
    if rest.len() < length || &rest[..length] != identity {
        return None;
    }
    Some(Prechecked {
        rust: String::from_utf8(rest[length..].to_vec()).ok()?,
        features: serde_json::from_value(header.get("features")?.clone()).ok()?,
        keys: serde_json::from_value(header.get("keys")?.clone()).ok()?,
        editor: header.get("editor").and_then(serde_json::Value::as_bool).unwrap_or(true),
    })
}

fn prune(directory: &Path) {
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    let mut files = entries
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "checked"))
        .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
        .collect::<Vec<_>>();
    files.sort_by(|left, right| right.0.cmp(&left.0));
    for (_, path) in files.into_iter().skip(RETAINED) {
        let _ = std::fs::remove_file(path);
    }
}

/// A check in progress for one graph identity. Dropping it removes the marker,
/// whether the check stored a result, failed or was cancelled.
pub struct Pending(PathBuf);
impl Drop for Pending {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// Mark that this process is checking this graph identity, so `tsrust dev` waits
/// for the result instead of starting the same check.
pub fn begin(root: &Path, identity: &[u8]) -> Option<Pending> {
    let path = file(root, identity).with_extension("pending");
    std::fs::create_dir_all(path.parent()?).ok()?;
    std::fs::write(&path, std::process::id().to_string()).ok()?;
    let _ = std::fs::write(directory(root).join("server"), format!("{}\n{COMPILER_IDENTITY}\n", std::process::id()));
    Some(Pending(path))
}

fn alive(path: &Path) -> bool {
    std::fs::read_to_string(path).ok().and_then(|text| text.lines().next()?.trim().parse().ok()).is_some_and(process_alive)
}

/// Whether a running language server that checks ahead for this project was
/// built from different compiler sources. Its results can never match this
/// build, so it should be restarted. A server file without an identity comes
/// from a build older than this check, which is also a different build.
pub fn editor_on_other_build(root: &Path) -> bool {
    let path = directory(root).join("server");
    let Ok(text) = std::fs::read_to_string(&path) else { return false };
    alive(&path) && text.lines().nth(1).map(str::trim) != Some(COMPILER_IDENTITY)
}

/// The stored result for this graph identity, waiting while another process is
/// still checking it. When a language server that checks for this project is
/// running but has not started on this identity, wait up to `grace` for it to
/// begin (a save reaches the editor and `tsrust dev` at about the same time).
/// Returns `None` when there is nothing to wait for, the check fails, or
/// `cancelled` is set.
pub fn wait(root: &Path, identity: &[u8], grace: std::time::Duration, cancelled: &std::sync::atomic::AtomicBool) -> Option<Prechecked> {
    let pending = file(root, identity).with_extension("pending");
    let server = directory(root).join("server");
    let started = std::time::Instant::now();
    loop {
        if let Some(result) = load(root, identity) {
            return Some(result);
        }
        let checking = alive(&pending);
        // A server from another build can never store this identity; do not wait for it.
        if cancelled.load(std::sync::atomic::Ordering::Relaxed) || !checking && !(started.elapsed() < grace && alive(&server) && !editor_on_other_build(root)) {
            return None;
        }
        std::thread::sleep(std::time::Duration::from_millis(25));
    }
}

fn marker(root: &Path) -> PathBuf {
    root.join("target").join("tsrust-native").join("dev-session")
}

/// Removes the dev-session marker when `tsrust dev` stops.
pub struct DevSession(PathBuf);
impl Drop for DevSession {
    fn drop(&mut self) {
        // Leave a marker that another running session has since written.
        if self.0.parent().and_then(Path::parent).and_then(Path::parent).is_some_and(announced_by_this_process) {
            let _ = std::fs::remove_file(&self.0);
        }
    }
}

/// Whether the dev-session marker of this project names this process.
pub fn announced_by_this_process(root: &Path) -> bool {
    std::fs::read_to_string(marker(root)).ok().and_then(|text| text.lines().next()?.parse::<u32>().ok()) == Some(std::process::id())
}

/// Announce a running `tsrust dev` for this project and its entry, so the
/// language server checks ahead of saves only while someone is building.
pub fn announce_dev_session(root: &Path, entry: &Path) -> Option<DevSession> {
    let marker = marker(root);
    std::fs::create_dir_all(marker.parent()?).ok()?;
    std::fs::write(&marker, format!("{}\n{}\n", std::process::id(), entry.display())).ok()?;
    Some(DevSession(marker))
}

/// The project root and entry of a running `tsrust dev` whose project contains `path`.
pub fn dev_session_for(path: &Path) -> Option<(PathBuf, SourceId)> {
    for root in path.ancestors().skip(1) {
        let Ok(text) = std::fs::read_to_string(marker(root)) else { continue };
        let mut lines = text.lines();
        let pid = lines.next()?.parse::<i32>().ok()?;
        let entry = lines.next()?;
        return process_alive(pid).then(|| (root.to_owned(), SourceId(entry.to_owned())));
    }
    None
}

#[cfg(unix)]
fn process_alive(pid: i32) -> bool {
    unsafe extern "C" {
        fn kill(pid: i32, signal: i32) -> i32;
    }
    // Signal 0 checks existence without delivering a signal.
    pid > 0 && unsafe { kill(pid, 0) } == 0
}

#[cfg(not(unix))]
fn process_alive(_: i32) -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_results_round_trip_and_require_the_full_identity() {
        let root = std::env::temp_dir().join(format!("tsrust-prechecked-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let keys = crate::native::stable_names::Keys { functions: vec!["main".into()], ..Default::default() };
        let value = Prechecked { rust: "fn main(){}\n".into(), features: vec!["ui".into()], keys: keys.clone(), editor: false };
        assert!(load(&root, b"graph").is_none());
        store(&root, b"graph", &value).unwrap();
        assert!(contains(&root, b"graph"));
        let loaded = load(&root, b"graph").unwrap();
        assert_eq!((loaded.rust.as_str(), loaded.features.as_slice(), &loaded.keys, loaded.editor), ("fn main(){}\n", ["ui".to_owned()].as_slice(), &keys, false));
        assert!(load(&root, b"other").is_none());
        for index in 0..RETAINED + 2 {
            store(&root, format!("graph{index}").as_bytes(), &value).unwrap();
        }
        assert_eq!(std::fs::read_dir(directory(&root)).unwrap().count(), RETAINED);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn waiting_follows_a_pending_check_and_stops_without_one() {
        use std::sync::atomic::AtomicBool;
        use std::time::{Duration, Instant};
        let root = std::env::temp_dir().join(format!("tsrust-prechecked-wait-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let value = || Prechecked { rust: "fn main(){}\n".into(), features: Vec::new(), keys: Default::default(), editor: true };
        let idle = AtomicBool::new(false);
        // Nothing pending and no checking server: return at once.
        let started = Instant::now();
        assert!(wait(&root, b"a", Duration::from_secs(5), &idle).is_none());
        assert!(started.elapsed() < Duration::from_secs(1));
        // A check in progress in this process: wait until it stores the result.
        let pending = begin(&root, b"a").unwrap();
        let writer = {
            let root = root.clone();
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(200));
                store(&root, b"a", &Prechecked { rust: "fn main(){}\n".into(), features: Vec::new(), keys: Default::default(), editor: true }).unwrap();
                drop(pending);
            })
        };
        assert_eq!(wait(&root, b"a", Duration::ZERO, &idle).unwrap().rust, value().rust);
        writer.join().unwrap();
        // A pending check that ends without a result: stop waiting.
        let failed = begin(&root, b"b").unwrap();
        let ender = std::thread::spawn(move || { std::thread::sleep(Duration::from_millis(100)); drop(failed); });
        assert!(wait(&root, b"b", Duration::ZERO, &idle).is_none());
        ender.join().unwrap();
        // Cancellation ends a wait for a check still in progress.
        let _still_running = begin(&root, b"c").unwrap();
        assert!(wait(&root, b"c", Duration::ZERO, &AtomicBool::new(true)).is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_language_server_from_another_build_is_reported() {
        let root = std::env::temp_dir().join(format!("tsrust-prechecked-server-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        assert!(!editor_on_other_build(&root));
        let pending = begin(&root, b"a").unwrap();
        drop(pending);
        assert!(!editor_on_other_build(&root), "this build");
        std::fs::write(directory(&root).join("server"), std::process::id().to_string()).unwrap();
        assert!(editor_on_other_build(&root), "no identity: an older build");
        std::fs::write(directory(&root).join("server"), format!("{}\nother\n", std::process::id())).unwrap();
        assert!(editor_on_other_build(&root));
        std::fs::write(directory(&root).join("server"), "0\nother\n").unwrap();
        assert!(!editor_on_other_build(&root), "not running");
        let _ = std::fs::remove_dir_all(&root);
    }
}
