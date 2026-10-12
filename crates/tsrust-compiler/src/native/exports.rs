//! Shared native module export lookup keeps type and callable identities distinct.
use crate::{native_bindings::{self, NativeBinding}, native_types::{self, Declaration}};

#[derive(Clone, Copy, Debug)]
pub enum Export {
    Type(&'static Declaration),
    Function(&'static NativeBinding),
}

impl Export {
    pub fn module(self) -> &'static str {
        match self { Self::Type(item) => item.module, Self::Function(item) => item.module }
    }
}

/// Resolve a native export without pretending a type declaration is callable.
/// Source-name aliases belong to the importing module, never to this registry.
pub fn lookup(module: &str, name: &str) -> Result<Export, String> {
    if let Some(item) = native_types::lookup(module, name) { return Ok(Export::Type(item)); }
    if native_bindings::BINDINGS.iter().any(|item| item.module == module && item.name == name) {
        return native_bindings::lookup(module, name).map(Export::Function);
    }
    if native_bindings::BINDINGS.iter().any(|item| item.module == module)
        || [crate::native_signatures::NativeTypeId::Bytes, crate::native_signatures::NativeTypeId::ByteError]
            .into_iter().any(|id| native_types::declaration(id).module == module)
    {
        Err(format!("Standard-library module {module:?} does not export {name:?}."))
    } else {
        Err(format!("Unknown standard-library module {module:?}."))
    }
}
