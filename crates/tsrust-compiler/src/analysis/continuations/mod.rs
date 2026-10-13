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
mod defaults;

mod literals;
pub(super) use literals::StoredValueFrame;
mod records;
mod access;
pub(super) use access::FieldDispatch;

mod operators;

mod coalesce;
mod logical;
mod assignment;
pub(super) use assignment::Kind as AssignmentKind;
mod sequence;
mod store;

mod branches;
mod declarations;
mod loops;
mod exceptions;
#[cfg(test)]
pub(super) use exceptions::RegionMode;
