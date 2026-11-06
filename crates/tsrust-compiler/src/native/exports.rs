//! Shared native module export lookup keeps type and callable identities distinct.
use crate::{native_bindings::{self, NativeBinding}, native_types::{self, Declaration}};

#[derive(Clone, Copy, Debug)]
pub enum Export {
    Type(&'static Declaration),
    Alias(&'static Alias),
    Function(&'static NativeBinding),
    Namespace(Namespace),
}

/// A transparent source type alias, expanded by the ordinary alias machinery.
#[derive(Clone, Copy, Debug)]
pub struct Alias {
    pub module: &'static str,
    pub name: &'static str,
    pub signature: crate::native_signatures::Signature,
}
static MISSED_TICK_BEHAVIOR: Alias = Alias {module:"std:futures",name:"MissedTickBehavior",signature:crate::native_signatures::Signature::Union(&[crate::native_signatures::Signature::StringLiteral("burst"),crate::native_signatures::Signature::StringLiteral("delay"),crate::native_signatures::Signature::StringLiteral("skip")])};
static SEEK_ORIGIN: Alias = Alias { module: "std:fs", name: "SeekOrigin", signature: crate::stdlib::fs::bindings::SEEK_ORIGIN_SIGNATURE };

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
        match self { Self::Type(item) => item.module, Self::Alias(item) => item.module, Self::Function(item) => item.module, Self::Namespace(item) => item.module() }
    }
}

/// Resolve a native export without pretending a type declaration is callable.
/// Source-name aliases belong to the importing module, never to this registry.
pub fn lookup(module: &str, name: &str) -> Result<Export, String> {
    if module==MISSED_TICK_BEHAVIOR.module && name==MISSED_TICK_BEHAVIOR.name{return Ok(Export::Alias(&MISSED_TICK_BEHAVIOR));}
    if module == SEEK_ORIGIN.module && name == SEEK_ORIGIN.name { return Ok(Export::Alias(&SEEK_ORIGIN)); }
    if module=="std:futures" && matches!(name,"SendError"|"ChannelError") {return Ok(Export::Type(native_types::declaration(if name=="SendError"{crate::native_signatures::NativeTypeId::SendError}else{crate::native_signatures::NativeTypeId::ChannelError})));}
    if module=="std:url" && name=="StringView" {return Ok(Export::Type(native_types::declaration(crate::native_signatures::NativeTypeId::StringView)));}
    if matches!(module, "std:process" | "std:io" | "std:locale" | "std:regex" | "std:url" | "std:schema" | "std:threads" | "std:futures" | "std:cache" | "std:http" | "std:sql" | "std:sqlite") && name.starts_with("__") { return Err(format!("Standard-library module {module:?} does not export {name:?}.")); }
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
