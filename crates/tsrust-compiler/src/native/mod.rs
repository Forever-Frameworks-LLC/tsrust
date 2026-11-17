//! Native binding schemas, dependency selection, and runtime emission.

pub(crate) mod binding_types;
pub mod bindings;
pub mod cache;
pub mod codegen_units;
pub(crate) mod emit;
pub mod exports;
pub mod return_borrows;
pub mod runtime;
pub mod signatures;
pub mod stable_names;
pub mod type_resolution;
pub mod types;

pub mod dependency_patches;
pub mod url_patch;
