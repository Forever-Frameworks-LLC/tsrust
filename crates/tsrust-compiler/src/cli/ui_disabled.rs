//! `tsrust ui` until GPUI Kit support lands (docs/plans/gpui-application-platform.md).
pub(super) const UNAVAILABLE:&str="GPUI Kit UI support is in development; `tsrust ui add` and `tsrust new --ui` are not available yet. Tsrust applications expose HTTP APIs today.";

pub(super) fn run(_args:impl Iterator<Item=String>)->Result<u8,String> {Err(UNAVAILABLE.into())}
