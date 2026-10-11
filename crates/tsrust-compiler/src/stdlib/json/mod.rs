//! Bundled JSON declarations and native runtime.
pub(crate) mod bindings;
pub(crate) mod emit;
#[cfg(test)]
include!("runtime.rs");
#[cfg(test)]
mod lines;
#[cfg(test)]
mod writer;
