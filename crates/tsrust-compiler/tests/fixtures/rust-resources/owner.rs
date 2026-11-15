use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

// Test-only lifetime observation, not a resource registry.
static DROPS: AtomicI64 = AtomicI64::new(0);
static READY: AtomicBool = AtomicBool::new(false);

/// A private native owner; use create, read and append rather than its fields.
#[tsrust_resource]
pub struct Session {
    text: String,
}

impl Drop for Session {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}

/// Create an independent native owner from moved text.
pub fn create(text: String) -> Session {
    Session { text }
}

/// Read an owned copy without transferring the session.
pub fn read(session: &Session) -> String {
    session.text.clone()
}

/// Borrow the owner exclusively and append text.
pub fn append(session: &mut Session, text: &str) {
    session.text.push_str(text);
}

/// Transfer the same owner without copying or reconstructing it.
pub fn transfer(session: Session) -> Session {
    session
}

pub fn optional(text: String) -> Option<Session> {
    Some(create(text))
}

pub fn checked(text: String) -> Result<Session, String> {
    if text.is_empty() { Err("empty".into()) } else { Ok(create(text)) }
}

pub fn drops() -> i64 {
    DROPS.load(Ordering::SeqCst)
}

pub fn wait(task: &TaskContext) {
    READY.store(true, Ordering::SeqCst);
    while !task.cancellation_requested() { std::thread::yield_now(); }
}

pub fn ready() -> bool { READY.load(Ordering::SeqCst) }

pub fn many() -> Vec<Session> { vec![create("one".into()), create("two".into())] }
pub fn roundtrip(values: Vec<Session>) -> Vec<Session> { values }
