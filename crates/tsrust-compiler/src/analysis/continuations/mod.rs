//! Explicit state boundaries for the checker continuation migration.
mod closure;
mod block;
mod returns;
mod expression;
mod checked_expression;
mod call;
mod driver;
pub(super) use call::CallDispatch;
