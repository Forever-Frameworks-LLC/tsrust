//! Public schema integration with compiler plans and embedded validation runtime.
#[cfg(test)]
#[path = "runtime/issues.rs"]
mod issues;
#[cfg(test)]
#[path = "runtime/numeric.rs"]
mod numeric;
#[cfg(test)]
#[path = "runtime/scalar.rs"]
mod scalar;
#[cfg(test)]
#[path = "runtime/string.rs"]
mod string;
#[cfg(test)]
#[path = "runtime/array.rs"]
mod array;
#[cfg(test)]
#[path = "runtime/presence.rs"]
mod presence;
#[path = "compiler/namespace.rs"]
pub(crate) mod namespace;
#[path = "compiler/emit.rs"]
pub(crate) mod emit;

#[cfg(test)]
#[path = "runtime/plan.rs"]
mod runtime;

#[path = "compiler/builders.rs"]
pub(crate) mod builders;

#[cfg(test)]
#[path = "runtime/literal.rs"]
mod literal;

#[cfg(test)]
#[path = "runtime/object.rs"]
mod object;

#[path = "compiler/defaults.rs"]
pub(crate) mod defaults;

#[cfg(test)]
#[path = "runtime/union.rs"]
mod union;
#[cfg(test)]
#[path = "runtime/trial.rs"]
mod trial;

#[path = "compiler/plans.rs"]
pub(crate) mod plans;
#[path = "compiler/pipelines.rs"]
pub(crate) mod pipelines;

#[path = "compiler/recursive.rs"]
mod recursive;

#[path = "compiler/type_contract.rs"]
pub(crate) mod type_contract;
