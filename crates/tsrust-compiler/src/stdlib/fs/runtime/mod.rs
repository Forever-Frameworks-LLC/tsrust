//! Compile the same backend sources embedded into generated file modules.
mod resource;
pub(crate) mod fs;
pub(crate) mod buffered_fs;

// Lifecycle tests inspect release without exposing backend fields outside this module.
pub(crate) fn raw_fd(file: &fs::FileHandle) -> std::os::fd::RawFd {
    use std::os::fd::AsRawFd;
    file.get().0.as_raw_fd()
}
