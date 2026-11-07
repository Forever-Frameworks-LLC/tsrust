//! Direct backend witnesses; generated behavior has a separate project-aware target.
#[path="../src/stdlib/fs/runtime/resource.rs"]pub(crate) mod resource;
#[path="../src/stdlib/fs/runtime/fs.rs"]pub(crate) mod fs;
#[path="../src/stdlib/fs/runtime/buffered_fs.rs"]pub(crate) mod buffered_fs;
mod tsrust_native_files{pub(crate) use crate::{fs,buffered_fs};}
#[cfg(feature="logging")] include!("../src/stdlib/logging/runtime.rs");
include!("../src/stdlib/futures/runtime.rs");
include!("../src/stdlib/io/runtime.rs");
