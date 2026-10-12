//! Native binding schemas, dependency selection, and runtime emission.

pub(crate) mod binding_types;
pub mod bindings;
pub mod cache;
pub(crate) mod emit;
pub mod exports;
pub mod return_borrows;
pub mod runtime;
pub mod signatures;
pub mod type_resolution;
pub mod types;
