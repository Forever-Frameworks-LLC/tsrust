//! Bundled JSON declarations and native runtime.
pub(crate) mod bindings;
pub(crate) mod namespace;
pub(crate) mod emit;
include!("runtime/parser.rs");
#[cfg(test)]
include!("runtime/operations.rs");
#[cfg(test)]
mod lines;
#[cfg(test)]
mod writer;
pub(crate) mod lines_emit;

pub(crate) mod views_emit;

#[cfg(test)]
pub(crate) mod typed;

pub(crate) mod writer_emit;

#[cfg(test)]
pub(crate) mod raw;

pub(crate) mod raw_emit;

#[cfg(test)]
pub(crate) mod source;
