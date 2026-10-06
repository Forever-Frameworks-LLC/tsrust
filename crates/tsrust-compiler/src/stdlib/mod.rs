//! Bundled standard-library modules: checked Tsrust surfaces and native backends.
pub mod path;
pub(crate) mod testing;
pub(crate) fn source(module: &str) -> Option<&'static str> {
    match module { "std:testing" => Some(testing::SOURCE), _ => None }
}
