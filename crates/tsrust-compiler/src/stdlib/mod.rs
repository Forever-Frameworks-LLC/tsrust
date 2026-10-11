//! Bundled standard-library modules: checked Tsrust surfaces and native backends.
pub mod path;
pub(crate) mod bytes;
pub(crate) mod os;
pub(crate) mod testing;
#[cfg(test)]
mod resource;
#[cfg(test)]
mod fs;
#[cfg(test)]
mod buffered_fs;
pub(crate) mod fs_bindings;
pub(crate) mod fs_emit;
pub(crate) fn source(module: &str) -> Option<&'static str> {
    match module { "std:testing" => Some(testing::SOURCE), _ => None }
}

#[cfg(test)]
mod env;
