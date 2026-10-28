//! Filesystem binding metadata, application bridges and bundled runtime.
pub(crate) mod bindings;
pub(crate) mod emit;
// The consuming-close backend explicitly supports macOS/Linux. Compile its
// descriptor-based unit harness only on those hosts, matching runtime/fs.rs.
#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
pub(crate) mod runtime;
