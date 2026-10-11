//! Bundled JSON declarations and native runtime.
pub(crate) mod bindings;
pub(crate) mod emit;
include!("runtime.rs");
#[cfg(test)]
mod lines;
#[cfg(test)]
mod writer;
pub(crate) mod lines_emit;
