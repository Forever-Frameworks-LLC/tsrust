//! Counts of answers that would mislead an editor user. Each one is a
//! defect when it is not zero; the typing harness asserts on them.
use std::sync::atomic::{AtomicU64, Ordering};

/// Answers given from an index that did not match the current project text.
pub static STALE_ANSWERS: AtomicU64 = AtomicU64::new(0);
/// Member completions that offered nothing although the receiver type was known.
pub static EMPTY_KNOWN_RECEIVER: AtomicU64 = AtomicU64::new(0);
/// Member completions that would have offered names from the enclosing scope.
pub static NON_MEMBER_AFTER_SEPARATOR: AtomicU64 = AtomicU64::new(0);

pub fn count(counter: &AtomicU64, what: &str) {
    let total = counter.fetch_add(1, Ordering::Relaxed) + 1;
    eprintln!("tsrust-lsp: {what} (total {total})");
}

pub fn snapshot() -> serde_json::Value {
    serde_json::json!({
        "staleAnswers": STALE_ANSWERS.load(Ordering::Relaxed),
        "emptyKnownReceiver": EMPTY_KNOWN_RECEIVER.load(Ordering::Relaxed),
        "nonMemberAfterSeparator": NON_MEMBER_AFTER_SEPARATOR.load(Ordering::Relaxed),
    })
}
