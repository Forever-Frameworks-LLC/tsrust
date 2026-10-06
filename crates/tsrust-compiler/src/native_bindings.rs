//! Compiler-owned standard-library binding metadata. Language import integration is staged separately.
use crate::Mode;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeType {
    String,
    Boolean,
    Int,
    Number,
    Void,
    /// Owned reusable callback with no arguments and a void result.
    OwnedVoidCallback,
    /// Private polymorphic compiler intrinsic; never emitted as a native call.
    SharedCallback,
    /// Private read-only value observation, resolved by compiler checking.
    ObservedValue,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeParameter {
    pub name: &'static str,
    pub ty: NativeType,
    pub access: Mode,
    pub optional: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeBackend {
    /// Bundled generated Rust code; no external package graph or network access.
    Bundled { symbol: &'static str },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeBinding {
    pub module: &'static str,
    pub name: &'static str,
    pub parameters: &'static [NativeParameter],
    pub result: NativeType,
    pub backend: NativeBackend,
    pub features: &'static [&'static str],
}
impl NativeBinding {
    pub fn accepts_arity(&self, count: usize) -> bool {
        let required = self
            .parameters
            .iter()
            .take_while(|parameter| !parameter.optional)
            .count();
        (required..=self.parameters.len()).contains(&count)
    }
}
pub const BINDINGS: &[NativeBinding] = &[
    crate::stdlib::testing::bindings::CONTAINS_VALUE,
    crate::stdlib::testing::bindings::LENGTH_VALUE,
    crate::stdlib::testing::bindings::HAS_LENGTH,
    crate::stdlib::testing::bindings::NUMERIC_VALUE,
    crate::stdlib::testing::bindings::SHARE_CALLBACK,
    crate::stdlib::path::bindings::BASENAME,
    crate::stdlib::testing::bindings::CHECK_ASSERTION,
    crate::stdlib::testing::bindings::REGISTER_TEST,
    crate::stdlib::testing::bindings::RUN_TESTS,
    crate::stdlib::testing::bindings::FINISH_TESTS,
    crate::stdlib::testing::bindings::BEGIN_SUITE,
    crate::stdlib::testing::bindings::END_SUITE,
    crate::stdlib::testing::bindings::BEGIN_FILE,
    crate::stdlib::testing::bindings::REGISTER_HOOK,
];
pub fn lookup(module: &str, name: &str) -> Result<&'static NativeBinding, String> {
    if !BINDINGS.iter().any(|binding| binding.module == module) {
        return Err(format!("Unknown standard-library module {module:?}."));
    }
    BINDINGS
        .iter()
        .find(|binding| binding.module == module && binding.name == name)
        .ok_or_else(|| format!("Standard-library module {module:?} does not export {name:?}."))
}
/// Deterministic native feature selection from resolved imports. Unknown exports are errors.
pub fn selected_features<'a>(
    imports: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<BTreeSet<&'static str>, String> {
    let mut selected = BTreeSet::new();
    for (module, name) in imports {
        selected.extend(lookup(module, name)?.features.iter().copied());
    }
    Ok(selected)
}

/// Internal AST identity cannot be spelled as a source identifier.
pub fn by_symbol(symbol: &str) -> Option<(usize, &'static NativeBinding)> {
    let (module, name) = symbol.strip_prefix("@native:")?.split_once('#')?;
    BINDINGS
        .iter()
        .enumerate()
        .find(|(_, binding)| binding.module == module && binding.name == name)
}
