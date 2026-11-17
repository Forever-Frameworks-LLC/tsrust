//! Background checks of a running `tsrust dev` application ahead of saves.
//!
//! When typing pauses, the latest workspace text (open buffers over disk) is
//! checked from the dev session's entry and the emitted Rust is stored with
//! `tsrust_compiler::project_prechecked`. `tsrust dev` reuses it when the saved
//! files have the same module graph. A newer edit cancels a check in progress.
//! Nothing runs unless a `tsrust dev` session is active for the edited project.
use crate::workspace::WorkspaceSnapshot;
use crossbeam_channel::{Receiver, Sender, unbounded};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use tsrust_compiler::{project_cli::CheckedInput, project_prechecked as prechecked, project_session::ProjectSession};

struct Request {
    workspace: WorkspaceSnapshot,
    path: PathBuf,
    cancelled: Arc<AtomicBool>,
}

pub struct Prechecker {
    requests: Sender<Request>,
    current: Arc<Mutex<Option<Arc<AtomicBool>>>>,
}

impl Prechecker {
    pub fn new() -> Result<Self, String> {
        let (requests, receiver) = unbounded();
        std::thread::Builder::new()
            .name("tsrust-precheck".into())
            .stack_size(tsrust_compiler::COMPILER_STACK_BYTES)
            .spawn(move || run(receiver))
            .map_err(|error| error.to_string())?;
        Ok(Self { requests, current: Default::default() })
    }

    /// Check the dev entry that contains `path`, if any, against `workspace`,
    /// cancelling an earlier check that has not finished.
    pub fn request(&self, workspace: WorkspaceSnapshot, path: PathBuf) {
        let cancelled = Arc::new(AtomicBool::new(false));
        if let Ok(mut current) = self.current.lock()
            && let Some(previous) = current.replace(cancelled.clone())
        {
            previous.store(true, Ordering::Relaxed);
        }
        let _ = self.requests.send(Request { workspace, path, cancelled });
    }
}

fn run(receiver: Receiver<Request>) {
    let mut session = ProjectSession::default();
    while let Ok(mut request) = receiver.recv() {
        while let Ok(newer) = receiver.try_recv() {
            request = newer;
        }
        if request.cancelled.load(Ordering::Relaxed) {
            continue;
        }
        let trace = std::env::var_os("TSRUST_LSP_TRACE_TIMINGS").is_some();
        let started = std::time::Instant::now();
        let stored = precheck(&mut session, &request);
        if trace {
            eprintln!("[precheck] {} {:.1}ms", stored.unwrap_or("skipped"), started.elapsed().as_secs_f64() * 1000.0);
        }
    }
}

fn precheck(session: &mut ProjectSession, request: &Request) -> Option<&'static str> {
    let (root, entry) = prechecked::dev_session_for(&request.path)?;
    let identity = session.graph_identity(&request.workspace, &entry.0).ok()?;
    if prechecked::contains(&root, &identity) {
        return Some("already stored");
    }
    let analysis = tsrust_compiler::project_session::with_cancellation(request.cancelled.clone(), || {
        session.check(&request.workspace, &entry.0).map_err(|error| error.message)
    })
    .ok()?;
    let identity = prechecked::graph_identity(&analysis.graph);
    let checked = CheckedInput::Project(analysis);
    let (rust, features) = checked.emit_rust_with_native_features();
    let value = prechecked::Prechecked { rust, features: features.into_iter().map(str::to_owned).collect(), keys: checked.name_keys() };
    prechecked::store(&root, &identity, &value).ok()?;
    Some("stored")
}
