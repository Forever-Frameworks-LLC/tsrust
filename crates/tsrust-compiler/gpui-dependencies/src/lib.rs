//! Compiler-owned GPUI desktop dependency closure (backend set plus GPUI Kit).
pub use serde;
pub use serde_json;

// This is the real PTY/process boundary, exercised against the managed dependency closure.
#[cfg(all(test, feature = "terminal"))]
#[path = "../../src/stdlib/ui/terminal.rs"]
mod terminal;

#[cfg(all(test, feature = "gpui"))]
#[path = "../../src/stdlib/ui/image_asset.rs"]
mod image_asset;
