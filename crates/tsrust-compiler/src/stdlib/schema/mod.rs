//! Internal schema backend. No public source API is accepted yet.
#[cfg(test)]
mod issues;
#[cfg(test)]
mod numeric;
#[cfg(test)]
mod scalar;
#[cfg(test)]
mod array;
pub(crate) mod namespace;
pub(crate) mod emit;
