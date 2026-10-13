//! Explicit state boundaries for the checker continuation migration.
mod closure;
mod block;
mod returns;
mod expression;
mod checked_expression;
mod call;
mod driver;
pub(super) use call::CallDispatch;
mod arguments;

mod literals;
pub(super) use literals::StoredValueFrame;
mod records;
mod access;
pub(super) use access::FieldDispatch;
