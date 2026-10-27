// Application-owned lifetime guard for the desktop sidecar, not an RPC transport.
use std::io::{IsTerminal, Read};

/// Arm a parent-owned stdin pipe. EOF/error terminates this process with code 74.
/// Graceful shutdown must keep that pipe open until the HTTP server has drained.
pub fn watch_parent_pipe() -> Result<bool, String> {
    if std::io::stdin().is_terminal() {
        return Err("Desktop backend requires a parent-owned stdin pipe".into());
    }
    std::thread::Builder::new().name("desktop-parent-pipe".into()).spawn(|| {
        let mut input = std::io::stdin().lock();
        let mut buffer = [0u8; 256];
        loop {
            match input.read(&mut buffer) {
                Ok(0) | Err(_) => std::process::exit(74),
                Ok(_) => {}
            }
        }
    }).map(|_| true).map_err(|_| "Cannot start desktop parent lifetime guard".into())
}
