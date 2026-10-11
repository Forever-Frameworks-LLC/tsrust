//! Compiler-owned standard-library binding metadata. Language import integration is staged separately.
use crate::Mode;
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeType {
    Structured(&'static crate::native_signatures::Signature),
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
impl NativeType {
    /// Shared semantic signature projection for native checking and editor help.
    pub fn signature(self) -> crate::native_signatures::Signature {
        use crate::native_signatures::{Intrinsic, Signature};
        match self {
            Self::Structured(signature) => *signature,
            Self::String => Signature::String,
            Self::Boolean => Signature::Boolean,
            Self::Int => Signature::Int,
            Self::Number => Signature::Number,
            Self::Void => Signature::Void,
            Self::OwnedVoidCallback => Signature::Intrinsic(Intrinsic::OwnedVoidCallback),
            Self::SharedCallback => Signature::Intrinsic(Intrinsic::SharedCallback),
            Self::ObservedValue => Signature::Intrinsic(Intrinsic::ObservedValue),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeParameter {
    pub name: &'static str,
    pub ty: NativeType,
    pub access: Mode,
    pub optional: bool,
    /// Final homogeneous rest parameter; ty describes each argument, not a public array argument.
    pub rest: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeBackend {
    /// Bundled generated Rust code and compiler-owned dependencies; no user package graph or build-time network access.
    Bundled { symbol: &'static str },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeBinding {
    pub module: &'static str,
    pub name: &'static str,
    pub parameters: &'static [NativeParameter],
    pub result: NativeType,
    /// Source loan carried by a native return payload, when present.
    pub returned_borrow: Option<crate::native_return_borrows::ReturnedBorrow>,
    pub backend: NativeBackend,
    pub features: &'static [&'static str],
}
impl NativeBinding {
    /// Further native types that a parameter accepts besides its declared type.
    /// A byte observer reads any readable byte storage; a byte mutator writes any
    /// writable one. A stored function value keeps the declared parameter type.
    pub fn argument_alternatives(&self, position: usize) -> &'static [crate::native_signatures::NativeTypeId] {
        use crate::native_signatures::NativeTypeId::{ByteView, ByteViewMut, BufferedReader, BufferedWriter};
        if self.module == "std:fs" && position == 0 {
            match self.name {
                "read" => return &[BufferedReader],
                "write" | "writeAll" => return &[BufferedWriter],
                "close" => return &[BufferedReader, BufferedWriter],
                _ => {},
            }
        }
        if self.module == "std:fs" && position == 1 && matches!(self.name, "write" | "writeAll" | "writeFile") {
            return &[ByteView, ByteViewMut];
        }
        if self.module != "std:bytes" || position != 0 {
            return &[];
        }
        match self.name {
            // A view of a view borrows the parent view for as long as it lives.
            "length" | "at" | "copy" | "toUtf8" | "view" => &[ByteView, ByteViewMut],
            "set" | "viewMut" => &[ByteViewMut],
            _ => &[],
        }
    }
    pub fn accepts_arity(&self, count: usize) -> bool {
        let required = self
            .parameters
            .iter()
            .take_while(|parameter| !parameter.optional && !parameter.rest)
            .count();
        count >= required && (self.parameters.last().is_some_and(|parameter| parameter.rest) || count <= self.parameters.len())
    }
}
pub const BINDINGS: &[NativeBinding] = &[
    crate::stdlib::json::bindings::PARSE,
    crate::stdlib::json::bindings::PARSE_JSONC,
    crate::stdlib::json::bindings::OPEN_JSONL,
    crate::stdlib::json::bindings::READ_JSONL,
    crate::stdlib::json::bindings::CLOSE_JSONL,
    crate::stdlib::json::bindings::STRINGIFY,
    crate::stdlib::json::bindings::KIND,
    crate::stdlib::env::bindings::GET,
    crate::stdlib::time::MONOTONIC_NOW,
    crate::stdlib::fs_bindings::BUFFER_READER,
    crate::stdlib::fs_bindings::BUFFER_WRITER,
    crate::stdlib::fs_bindings::FLUSH,
    crate::stdlib::fs_bindings::OPEN,
    crate::stdlib::fs_bindings::CREATE,
    crate::stdlib::fs_bindings::READ_FILE,
    crate::stdlib::fs_bindings::WRITE_FILE,
    crate::stdlib::fs_bindings::WRITE,
    crate::stdlib::fs_bindings::WRITE_ALL,
    crate::stdlib::fs_bindings::READ,
    crate::stdlib::fs_bindings::CLOSE,
    crate::stdlib::bytes::bindings::FROM_ARRAY,
    crate::stdlib::bytes::bindings::LENGTH,
    crate::stdlib::bytes::bindings::AT,
    crate::stdlib::bytes::bindings::SET,
    crate::stdlib::bytes::bindings::ALLOC,
    crate::stdlib::bytes::bindings::RESIZE,
    crate::stdlib::bytes::bindings::APPEND,
    crate::stdlib::bytes::bindings::COPY,
    crate::stdlib::bytes::bindings::FROM_UTF8,
    crate::stdlib::bytes::bindings::TO_UTF8,
    crate::stdlib::testing::bindings::CONTAINS_VALUE,
    crate::stdlib::testing::bindings::LENGTH_VALUE,
    crate::stdlib::testing::bindings::HAS_LENGTH,
    crate::stdlib::testing::bindings::NUMERIC_VALUE,
    crate::stdlib::testing::bindings::ASSERTION_DETAILS,
    crate::stdlib::testing::bindings::SHARE_CALLBACK,
    crate::stdlib::path::bindings::BASENAME,
    crate::stdlib::path::bindings::DIRNAME,
    crate::stdlib::path::bindings::EXTNAME,
    crate::stdlib::path::bindings::IS_ABSOLUTE,
    crate::stdlib::path::bindings::NORMALIZE,
    crate::stdlib::testing::bindings::CHECK_ASSERTION,
    crate::stdlib::testing::bindings::REGISTER_TEST,
    crate::stdlib::testing::bindings::RUN_TESTS,
    crate::stdlib::testing::bindings::FINISH_TESTS,
    crate::stdlib::testing::bindings::BEGIN_SUITE,
    crate::stdlib::testing::bindings::END_SUITE,
    crate::stdlib::testing::bindings::BEGIN_FILE,
    crate::stdlib::testing::bindings::REGISTER_HOOK,
    crate::stdlib::os::bindings::PLATFORM,
    crate::stdlib::os::bindings::ARCH,
    crate::stdlib::os::bindings::ENDIANNESS,
    crate::stdlib::path::bindings::JOIN,
    crate::stdlib::path::bindings::RELATIVE,
    crate::stdlib::bytes::bindings::VIEW,
    crate::stdlib::bytes::bindings::VIEW_MUT,
];
pub fn lookup(module: &str, name: &str) -> Result<&'static NativeBinding, String> {
    if !BINDINGS.iter().any(|binding| binding.module == module) {
        return Err(format!("Unknown standard-library module {module:?}."));
    }
    BINDINGS
        .iter()
        .find(|binding| binding.module == module && binding.name == name)
        .ok_or_else(|| format!("Standard-library module {module:?} does not export {name:?}."))
        .and_then(|binding| {
            binding.validate_returned_borrow().map_err(|error| format!("Invalid native returned-borrow contract for {module}::{name}: {error:?}."))?;
            Ok(binding)
        })
}
/// Deterministic native feature selection from resolved imports. Unknown exports are errors.
pub fn selected_features<'a>(
    imports: impl IntoIterator<Item = (&'a str, &'a str)>,
) -> Result<BTreeSet<&'static str>, String> {
    let mut selected = BTreeSet::new();
    for (module, name) in imports {
        if let crate::native_exports::Export::Function(binding) = crate::native_exports::lookup(module, name)? {
            selected.extend(binding.features.iter().copied());
        }
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
