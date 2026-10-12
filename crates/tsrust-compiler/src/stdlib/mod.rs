//! Bundled standard-library modules: checked Tsrust surfaces and native backends.
pub mod path;
pub(crate) mod bytes;
pub(crate) mod os;
pub(crate) mod testing;
pub(crate) mod fs;
pub(crate) fn source(module: &str) -> Option<&'static str> {
    match module { "std:testing" => Some(testing::SOURCE), _ => None }
}

pub(crate) mod env;
pub(crate) mod time;

pub(crate) mod json;
#[cfg(test)]
mod schema;
