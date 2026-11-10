//! Application commands issued from handlers: notifications, window control,
//! clipboard, file dialogs, URLs, updates and extra windows. They queue on the UI thread
//! and the renderer runs them on its next frame; replies run as handlers do.
#![cfg_attr(not(feature = "gpui"), allow(dead_code))]
use super::{Node, window::Options};
use std::cell::{Cell, RefCell};

pub(crate) type Reply = Box<dyn FnOnce(&str) -> Result<(), String>>;

pub(crate) enum Command {
    Plain {name: String, payload: String},
    Reply {name: String, payload: String, reply: Reply},
    Window {node: Box<Node>, title: String, options: String},
}

const PLAIN: &[&str] = &["notify", "theme", "set-title", "minimize", "zoom", "fullscreen", "close", "quit", "open-url", "write-clipboard", "relaunch", "register-themes", "use-theme", "input"];
const REPLIES: &[&str] = &["read-clipboard", "pick-files", "save-file", "check-update", "install-update", "read-input"];

thread_local! {
    static QUEUE: RefCell<Vec<Command>> = const {RefCell::new(Vec::new())};
    static DARK: Cell<bool> = const {Cell::new(false)};
}

/// Queue a command with a JSON payload; unknown names are rejected.
pub fn command(name: &str, payload: &str) -> Result<(), String> {
    if !PLAIN.contains(&name) {return Err(format!("Unknown UI command: {name}"));}
    QUEUE.with(|queue| queue.borrow_mut().push(Command::Plain {name: name.to_owned(), payload: payload.to_owned()}));
    Ok(())
}

/// Queue a command whose result (text) is delivered to `reply` later.
pub fn command_reply(name: &str, payload: &str, reply: impl FnOnce(&str) -> Result<(), String> + 'static) -> Result<(), String> {
    if !REPLIES.contains(&name) {return Err(format!("Unknown UI command: {name}"));}
    QUEUE.with(|queue| queue.borrow_mut().push(Command::Reply {name: name.to_owned(), payload: payload.to_owned(), reply: Box::new(reply)}));
    Ok(())
}

/// Queue another window; its options are checked now.
pub fn open_window(node: Node, title: String, options: &str) -> Result<(), String> {
    Options::parse(options)?;
    QUEUE.with(|queue| queue.borrow_mut().push(Command::Window {node: Box::new(node), title, options: options.to_owned()}));
    Ok(())
}

/// The system appearance seen by the last rendered frame: "light" or "dark".
pub fn appearance() -> String {(if DARK.with(Cell::get) {"dark"} else {"light"}).to_owned()}

pub(crate) fn set_dark(dark: bool) {DARK.with(|value| value.set(dark));}

pub(crate) fn take() -> Vec<Command> {QUEUE.with(|queue| std::mem::take(&mut *queue.borrow_mut()))}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn commands_queue_in_order_and_reject_unknown_names() {
        command("notify", r#"{"message":"Saved"}"#).unwrap();
        command_reply("read-clipboard", "{}", |_| Ok(())).unwrap();
        assert!(command("format-disk", "{}").is_err());
        assert!(command_reply("notify", "{}", |_| Ok(())).is_err());
        let names = take().into_iter().map(|command| match command {Command::Plain {name, ..} | Command::Reply {name, ..} => name, Command::Window {title, ..} => title}).collect::<Vec<_>>();
        assert_eq!(names, ["notify", "read-clipboard"]);
        assert!(take().is_empty());
        set_dark(true);
        assert_eq!(appearance(), "dark");
    }
}
