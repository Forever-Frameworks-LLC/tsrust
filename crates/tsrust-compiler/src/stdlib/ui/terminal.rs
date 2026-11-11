//! PTY ownership and VT state for the native Terminal component.
use alacritty_terminal::{
    event::{Event, EventListener, WindowSize},
    event_loop::{EventLoop, EventLoopSender, Msg},
    grid::Dimensions,
    sync::FairMutex,
    term::{Config, Osc52, Term},
    tty,
};
use std::{borrow::Cow, io, path::Path, sync::{Arc, Mutex, OnceLock}, thread::JoinHandle};

#[derive(Clone, Copy)]
pub(super) struct Size(pub WindowSize);
impl Dimensions for Size {
    fn total_lines(&self) -> usize {self.screen_lines()}
    fn screen_lines(&self) -> usize {usize::from(self.0.num_lines)}
    fn columns(&self) -> usize {usize::from(self.0.num_cols)}
}

#[derive(Default, Clone)]
pub(super) struct Status {
    pub title: String,
    pub exited: bool,
    pub code: Option<i32>,
}

#[derive(Clone)]
pub(super) struct Proxy {
    wake: async_channel::Sender<()>,
    status: Arc<Mutex<Status>>,
    sender: Arc<OnceLock<EventLoopSender>>,
    size: Arc<Mutex<Size>>,
}
impl EventListener for Proxy {
    fn send_event(&self, event: Event) {
        match event {
            Event::Title(title) => {
                let mut status = self.status.lock().unwrap();
                status.title = title.chars().take(4096).collect();
            }
            Event::ResetTitle => self.status.lock().unwrap().title.clear(),
            Event::ChildExit(status) => {
                let mut state = self.status.lock().unwrap();
                state.exited = true;
                state.code = status.code();
            }
            Event::Exit => self.status.lock().unwrap().exited = true,
            Event::PtyWrite(text) => self.write(text.into_bytes()),
            Event::TextAreaSizeRequest(format) => {
                let size = self.size.lock().unwrap().0;
                self.write(format(size).into_bytes());
            }
            // Terminal output cannot read/write the clipboard or launch a URL.
            Event::ClipboardStore(..) | Event::ClipboardLoad(..) | Event::ColorRequest(..) => return,
            _ => {}
        }
        // Grid/status are authoritative; only one pending redraw notification is needed.
        let _ = self.wake.try_send(());
    }
}
impl Proxy {
    fn write(&self, bytes: Vec<u8>) {
        if let Some(sender) = self.sender.get() {let _ = sender.send(Msg::Input(Cow::Owned(bytes)));}
    }
}

pub(super) struct Session {
    pub terminal: Arc<FairMutex<Term<Proxy>>>,
    pub wake: async_channel::Receiver<()>,
    proxy: Proxy,
    sender: EventLoopSender,
    thread: Option<JoinHandle<()>>,
    #[cfg(unix)]
    controller: std::fs::File,
    #[cfg(unix)]
    pid: nix::unistd::Pid,
}

impl Session {
    pub fn start(directory: &Path, shell: Option<tty::Shell>, size: Size, history: usize) -> io::Result<Self> {
        if size.columns() < 2 || size.screen_lines() == 0 {return Err(io::Error::other("Terminal dimensions must be at least 2 columns and 1 row"));}
        if !directory.is_dir() {return Err(io::Error::other("Terminal working directory does not exist"));}
        let (wake, receiver) = async_channel::bounded(1);
        let proxy = Proxy {wake, status: Arc::default(), sender: Arc::default(), size: Arc::new(Mutex::new(size))};
        let terminal = Arc::new(FairMutex::new(Term::new(Config {scrolling_history: history, osc52: Osc52::Disabled, ..Config::default()}, &size, proxy.clone())));
        let mut options = tty::Options {shell, working_directory: Some(directory.to_owned()), drain_on_exit: true, ..tty::Options::default()};
        options.env.insert("TERM".into(), "xterm-256color".into());
        options.env.insert("COLORTERM".into(), "truecolor".into());
        let pty = tty::new(&options, size.0, 0)?;
        #[cfg(unix)] let controller = pty.file().try_clone()?;
        #[cfg(unix)] let pid = nix::unistd::Pid::from_raw(pty.child().id() as i32);
        let event_loop = EventLoop::new(terminal.clone(), proxy.clone(), pty, true, false)?;
        let sender = event_loop.channel();
        let _ = proxy.sender.set(sender.clone());
        let worker = event_loop.spawn();
        // Consume the returned event loop so its PTY is dropped/reaped on natural exit.
        let finished = proxy.clone();
        let thread = std::thread::spawn(move || {
            drop(worker.join());
            finished.status.lock().unwrap().exited = true;
            let _ = finished.wake.try_send(());
        });
        Ok(Self {terminal, wake: receiver, proxy, sender, thread: Some(thread), #[cfg(unix)] controller, #[cfg(unix)] pid})
    }

    pub fn status(&self) -> Status {self.proxy.status.lock().unwrap().clone()}

    pub fn write(&self, bytes: Vec<u8>) -> Result<(), String> {
        if self.status().exited {return Err("Terminal session has exited".into());}
        self.sender.send(Msg::Input(Cow::Owned(bytes))).map_err(|error| error.to_string())
    }

    pub fn resize(&self, size: Size) -> Result<(), String> {
        if size.columns() < 2 || size.screen_lines() == 0 {return Err("Terminal dimensions must be at least 2 columns and 1 row".into());}
        {
            let mut previous = self.proxy.size.lock().unwrap();
            if previous.0.num_cols == size.0.num_cols && previous.0.num_lines == size.0.num_lines && previous.0.cell_width == size.0.cell_width && previous.0.cell_height == size.0.cell_height {return Ok(());}
            *previous = size;
        }
        // The parser can request dimensions while holding the terminal lock.
        self.terminal.lock().resize(size);
        self.sender.send(Msg::Resize(size.0)).map_err(|error| error.to_string())?;
        Ok(())
    }

    /// Force-close owns the PTY's foreground job and shell, never an arbitrary PID.
    /// Ordinary hide/show leaves the session mounted and does not call this.
    pub fn close(&mut self) {
        if self.thread.is_none() {return;}
        #[cfg(unix)] if !self.status().exited {
            use nix::{sys::signal::{killpg, Signal}, unistd::tcgetpgrp};
            // Query the still-open controlling terminal, avoiding a cached foreground job PID.
            if let Ok(group) = tcgetpgrp(&self.controller) && group.as_raw() > 0 {let _ = killpg(group, Signal::SIGKILL);}
            if !self.status().exited {let _ = killpg(self.pid, Signal::SIGKILL);}
        }
        let _ = self.sender.send(Msg::Shutdown);
        if let Some(thread) = self.thread.take() {
            // UI teardown must not block on the kernel or child reaper.
            std::thread::spawn(move || {let _ = thread.join();});
        }
    }
}

impl Drop for Session {fn drop(&mut self) {self.close();}}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    fn size(columns: u16, lines: u16) -> Size {Size(WindowSize {num_cols: columns, num_lines: lines, cell_width: 8, cell_height: 16})}
    fn wait(session: &Session, predicate: impl Fn(&Session) -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !predicate(session) {assert!(Instant::now() < deadline, "PTY did not reach the expected state"); std::thread::sleep(Duration::from_millis(5));}
    }
    fn visible(session: &Session) -> String {session.terminal.lock().grid().display_iter().map(|cell| cell.c).collect()}

    #[test]
    fn real_pty_parses_unicode_color_resize_input_exit_and_coalesces_wakes() {
        let shell = tty::Shell::new("/bin/sh".into(), vec!["-c".into(), "test -t 0 && printf '\\033[31mPTY-é\\033[0m'; read answer; stty size; printf 'answer:%s' \"$answer\"; exit 7".into()]);
        let session = Session::start(Path::new("/tmp"), Some(shell), size(80, 24), 100).unwrap();
        wait(&session, |session| visible(session).contains("PTY-é"));
        {
            use alacritty_terminal::vte::ansi::{Color, NamedColor};
            let term = session.terminal.lock();
            assert!(term.grid().display_iter().any(|cell| cell.c == 'P' && cell.fg == Color::Named(NamedColor::Red)));
        }
        session.resize(size(100, 30)).unwrap();
        session.write(b"hello\n".to_vec()).unwrap();
        wait(&session, |session| session.status().exited);
        assert_eq!(session.status().code, Some(7));
        assert!(visible(&session).contains("30 100"));
        assert!(visible(&session).contains("answer:hello"));
        assert!(session.wake.len() <= 1);
        assert!(session.write(vec![b'x']).is_err());
        assert!(session.resize(size(0, 0)).is_err());
    }

    #[test]
    fn close_reaps_a_shell_which_ignores_hangup() {
        let shell = tty::Shell::new("/bin/sh".into(), vec!["-c".into(), "trap '' HUP; printf ready; while :; do sleep 1; done".into()]);
        let mut session = Session::start(Path::new("/tmp"), Some(shell), size(80, 24), 100).unwrap();
        wait(&session, |session| visible(session).contains("ready"));
        let pid = session.pid;
        session.close();
        wait(&session, |session| session.status().exited);
        let deadline = Instant::now() + Duration::from_secs(5);
        while nix::sys::signal::kill(pid, None).is_ok() {assert!(Instant::now() < deadline, "Terminal shell was not reaped"); std::thread::sleep(Duration::from_millis(5));}
    }
}
