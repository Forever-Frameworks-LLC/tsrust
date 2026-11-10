//! Application commands issued from handlers: notifications, window control,
//! clipboard, file dialogs, URLs, updates and extra windows. They queue on the UI thread
//! and the renderer runs them on its next frame; replies run as handlers do.
#![cfg_attr(not(feature = "gpui"), allow(dead_code))]
use super::{Node, window::Options};
use std::{cell::{Cell, RefCell},rc::{Rc,Weak}};

pub(crate) struct Owner(Rc<()>);
pub(crate) struct Scope(Option<Weak<()>>);
impl Owner {
    pub(crate) fn new()->Self {Self(Rc::new(()))}
    pub(crate) fn context(&self)->impl Fn()->Scope+Clone+'static {
        let owner=Rc::downgrade(&self.0);
        move||Scope(ACTIVE.with(|active|active.replace(Some(owner.clone()))))
    }
    pub(crate) fn enter(&self)->Scope {Scope(ACTIVE.with(|active|active.replace(Some(Rc::downgrade(&self.0)))))}
}
impl Drop for Owner {
    fn drop(&mut self) {
        let owner=Rc::downgrade(&self.0);
        QUEUE.with(|queue|queue.borrow_mut().retain(|pending|pending.owner.as_ref().is_none_or(|target|!Weak::ptr_eq(target,&owner))));
    }
}
impl Drop for Scope {fn drop(&mut self){ACTIVE.with(|active|*active.borrow_mut()=self.0.take());}}
struct Pending {owner:Option<Weak<()>>,command:Command}
fn enqueue(command:Command) {
    let owner=ACTIVE.with(|active|active.borrow().clone());
    QUEUE.with(|queue|queue.borrow_mut().push(Pending{owner,command}));
}

pub(crate) type Reply = Box<dyn FnOnce(&str) -> Result<(), String>>;

pub(crate) enum Command {
    Plain {name: String, payload: String},
    Reply {name: String, payload: String, reply: Reply},
    Window {node: Box<Node>, title: String, options: String},
}

const PLAIN: &[&str] = &["notify", "theme", "set-title", "minimize", "zoom", "fullscreen", "close", "quit", "open-url", "write-clipboard", "relaunch", "register-themes", "use-theme", "input", "dismiss-notification", "clear-notifications"];
const REPLIES: &[&str] = &["read-clipboard", "pick-files", "save-file", "check-update", "install-update", "read-input", "notify-events"];

thread_local! {
    static QUEUE: RefCell<Vec<Pending>> = const {RefCell::new(Vec::new())};
    static ACTIVE: RefCell<Option<Weak<()>>> = const {RefCell::new(None)};
    static DARK: Cell<bool> = const {Cell::new(false)};
}

/// Queue a command with a JSON payload; unknown names are rejected.
pub fn command(name: &str, payload: &str) -> Result<(), String> {
    if !PLAIN.contains(&name) {return Err(format!("Unknown UI command: {name}"));}
    enqueue(Command::Plain {name: name.to_owned(), payload: payload.to_owned()});
    Ok(())
}

/// Queue a command whose result (text) is delivered to `reply` later.
pub fn command_reply(name: &str, payload: &str, reply: impl FnOnce(&str) -> Result<(), String> + 'static) -> Result<(), String> {
    if !REPLIES.contains(&name) {return Err(format!("Unknown UI command: {name}"));}
    enqueue(Command::Reply {name: name.to_owned(), payload: payload.to_owned(), reply: Box::new(reply)});
    Ok(())
}

/// Queue another window; its options are checked now.
pub fn open_window(node: Node, title: String, options: &str) -> Result<(), String> {
    Options::parse(options)?;
    enqueue(Command::Window {node: Box::new(node), title, options: options.to_owned()});
    Ok(())
}

/// The system appearance seen by the last rendered frame: "light" or "dark".
pub fn appearance() -> String {(if DARK.with(Cell::get) {"dark"} else {"light"}).to_owned()}

pub(crate) fn set_dark(dark: bool) {DARK.with(|value| value.set(dark));}

pub(crate) fn take(owner:&Owner) -> Vec<Command> {
    let pending=QUEUE.with(|queue|std::mem::take(&mut *queue.borrow_mut()));
    let mut output=Vec::new();let mut remaining=Vec::new();
    for item in pending {
        match &item.owner {
            None=>output.push(item.command),
            Some(target) if Weak::ptr_eq(target,&Rc::downgrade(&owner.0))=>output.push(item.command),
            Some(target) if target.upgrade().is_some()=>remaining.push(item),
            _=>{},
        }
    }
    QUEUE.with(|queue|queue.borrow_mut().extend(remaining));
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_keep_their_commands_in_order_and_drop_closed_window_replies() {
        let first=Owner::new();let second=Owner::new();
        {let _scope=first.enter();command("set-title","first").unwrap();
            {let _scope=second.enter();command("set-title","second").unwrap();}
            command("close","first").unwrap();}
        let names=|commands:Vec<Command>|commands.into_iter().map(|command|match command {Command::Plain{name,payload}=>format!("{name}:{payload}"),_=>panic!("Unexpected command")}).collect::<Vec<_>>();
        assert_eq!(names(take(&second)),["set-title:second"]);
        assert_eq!(names(take(&first)),["set-title:first","close:first"]);
        let drops=Rc::new(Cell::new(0));
        struct Probe(Rc<Cell<usize>>);impl Drop for Probe{fn drop(&mut self){self.0.set(self.0.get()+1);}}
        {let _scope=first.enter();let probe=Probe(drops.clone());command_reply("read-clipboard","{}",move |_|{let _=&probe;Ok(())}).unwrap();}
        drop(first);assert_eq!(drops.get(),1);assert!(take(&second).is_empty());
        command("notify","startup").unwrap();assert_eq!(names(take(&second)),["notify:startup"]);
    }
    #[test]
    fn captured_context_targets_the_origin_without_retaining_a_closed_window() {
        let first=Owner::new();let second=Owner::new();let context=first.context();
        {let _other=second.enter();{let _origin=context();command("close","origin").unwrap();}command("notify","other").unwrap();}
        assert!(matches!(&take(&first)[0],Command::Plain{name,..} if name=="close"));
        assert!(matches!(&take(&second)[0],Command::Plain{name,..} if name=="notify"));
        drop(first);{let _origin=context();command("close","closed").unwrap();}
        assert!(take(&second).is_empty());
    }
    #[test]
    fn commands_queue_in_order_and_reject_unknown_names() {
        let owner=Owner::new();
        command("notify", r#"{"message":"Saved"}"#).unwrap();
        command_reply("read-clipboard", "{}", |_| Ok(())).unwrap();
        assert!(command("format-disk", "{}").is_err());
        assert!(command_reply("notify", "{}", |_| Ok(())).is_err());
        let names = take(&owner).into_iter().map(|command| match command {Command::Plain {name, ..} | Command::Reply {name, ..} => name, Command::Window {title, ..} => title}).collect::<Vec<_>>();
        assert_eq!(names, ["notify", "read-clipboard"]);
        assert!(take(&owner).is_empty());
        set_dark(true);
        assert_eq!(appearance(), "dark");
    }
}
