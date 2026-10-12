//! Shared native module export lookup keeps type and callable identities distinct.
use crate::{native_bindings::{self, NativeBinding}, native_types::{self, Declaration}};

#[derive(Clone, Copy, Debug)]
pub enum Export {
    Type(&'static Declaration),
    Function(&'static NativeBinding),
    Namespace(Namespace),
}

/// Imported namespace identities survive aliases and ordinary lexical linking.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Namespace { Schema }
impl Namespace {
    pub fn module(self) -> &'static str { match self { Self::Schema => "std:schema" } }
    pub fn symbol(self) -> &'static str { match self { Self::Schema => "@namespace:std:schema#s" } }
}

/// Resolve known modules even before any callable bindings are registered.
pub fn known_module(name: &str) -> Option<&'static str> {
    if name == "std:schema" { return Some("std:schema"); }
    native_bindings::BINDINGS.iter().find(|binding| binding.module == name).map(|binding| binding.module)
}

impl Export {
    pub fn module(self) -> &'static str {
        match self { Self::Type(item) => item.module, Self::Function(item) => item.module, Self::Namespace(item) => item.module() }
    }
}

/// Resolve a native export without pretending a type declaration is callable.
/// Source-name aliases belong to the importing module, never to this registry.
pub fn lookup(module: &str, name: &str) -> Result<Export, String> {
    if module == "std:schema" && name == "s" { return Ok(Export::Namespace(Namespace::Schema)); }
    if let Some(item) = native_types::lookup(module, name) { return Ok(Export::Type(item)); }
    if native_bindings::BINDINGS.iter().any(|item| item.module == module && item.name == name) {
        return native_bindings::lookup(module, name).map(Export::Function);
    }
    if known_module(module).is_some()
        || [crate::native_signatures::NativeTypeId::Bytes, crate::native_signatures::NativeTypeId::ByteError]
            .into_iter().any(|id| native_types::declaration(id).module == module)
    {
        Err(format!("Standard-library module {module:?} does not export {name:?}."))
    } else {
        Err(format!("Unknown standard-library module {module:?}."))
    }
}
