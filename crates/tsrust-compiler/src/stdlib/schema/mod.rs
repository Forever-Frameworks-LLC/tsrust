//! Public scalar schema integration and test-only composite backend groundwork.
#[cfg(test)]
mod issues;
#[cfg(test)]
mod numeric;
#[cfg(test)]
mod scalar;
#[cfg(test)]
mod string;
#[cfg(test)]
mod array;
#[cfg(test)]
mod presence;
pub(crate) mod namespace;
pub(crate) mod emit;

#[cfg(test)]
mod runtime;

pub(crate) mod builders;

#[cfg(test)]
mod literal;
