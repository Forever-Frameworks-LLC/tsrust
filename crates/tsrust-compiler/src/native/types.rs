//! Native declarations describe public identity separately from callable exports.
//! Opaque declarations expose no source construction or implementation fields.
use crate::native_signatures::{NativeTypeId, Signature};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Representation {
    Opaque,
    Record,
    /// A scoped native value whose storage is borrowed, never independently owned.
    Borrowed(crate::native_return_borrows::BorrowAccess),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Capability { Inspection, StringConversion, PropertyReflection, ObjectSpread, Equality }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub signature: Signature,
    pub optional: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub id: NativeTypeId,
    pub module: &'static str,
    pub name: &'static str,
    pub type_parameters: &'static [&'static str],
    pub representation: Representation,
    /// Explicit clone produces independent storage. This never permits implicit copies.
    pub independent_clone: bool,
    /// Eligibility for current scalar/enum collection keys, not object identity.
    pub collection_key: bool,
    /// The owned backend representation may cross a native thread boundary.
    /// This metadata does not itself expose a source thread API.
    pub thread_transfer: bool,
    pub fields: &'static [Field],
    pub capabilities: &'static [Capability],
}

impl Declaration {
    pub fn borrow_access(&self) -> Option<crate::native_return_borrows::BorrowAccess> {
        match self.representation {
            Representation::Borrowed(access) => Some(access),
            _ => None,
        }
    }
    pub fn source_constructible(&self) -> bool {
        self.representation == Representation::Record
    }
}

static SCHEMA: Declaration = Declaration {
    id: NativeTypeId::Schema, module: "std:schema", name: "Schema",
    type_parameters: &["T"], representation: Representation::Opaque,
    independent_clone: false, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static BYTES: Declaration = Declaration {
    id: NativeTypeId::Bytes,
    module: "std:bytes",
    name: "Bytes",
    type_parameters: &[],
    representation: Representation::Opaque,
    independent_clone: true,
    collection_key: false,
    thread_transfer: true,
    fields: &[],
    capabilities: &[],
};
static FILE: Declaration = Declaration {
    id: NativeTypeId::File, module: "std:fs", name: "File",
    type_parameters: &[],
    representation: Representation::Opaque, independent_clone: false,
    collection_key: false, thread_transfer: false, fields: &[], capabilities: &[],
};
static BUFFERED_READER: Declaration = Declaration { id: NativeTypeId::BufferedReader, name: "BufferedReader", ..FILE };
static BUFFERED_WRITER: Declaration = Declaration { id: NativeTypeId::BufferedWriter, name: "BufferedWriter", ..FILE };
static FS_ERROR: Declaration = Declaration {
    id: NativeTypeId::FsError, module: "std:fs", name: "FsError",
    type_parameters: &[],
    representation: Representation::Record, independent_clone: true,
    collection_key: false, thread_transfer: true,
    capabilities: &[Capability::Inspection, Capability::StringConversion, Capability::PropertyReflection, Capability::ObjectSpread, Capability::Equality],
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "operation", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
        Field { name: "osCode", signature: Signature::Int, optional: true },
    ],
};
static ENV_ERROR: Declaration = Declaration {
    id: NativeTypeId::EnvError, module: "std:env", name: "EnvError",
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
    ], ..FS_ERROR
};
static JSON_VALUE: Declaration = Declaration {id:NativeTypeId::JsonValue,module:"std:json",name:"JsonValue",..BYTES};
static STRING_VIEW: Declaration = Declaration {id:NativeTypeId::StringView,module:"std:json",name:"StringView",capabilities:&[Capability::Inspection,Capability::StringConversion],..BYTE_VIEW};
static JSON_RAW: Declaration = Declaration {id:NativeTypeId::JsonRaw,module:"std:json",name:"JsonRaw",..BYTE_VIEW};
static JSON_VIEW: Declaration = Declaration {id:NativeTypeId::JsonView,module:"std:json",name:"JsonView",..BYTE_VIEW};
static JSON_ERROR: Declaration = Declaration {id:NativeTypeId::JsonError,module:"std:json",name:"JsonError",fields:&[
    Field{name:"code",signature:Signature::String,optional:false},
    Field{name:"message",signature:Signature::String,optional:false},
    Field{name:"offset",signature:Signature::Int,optional:false},
],..FS_ERROR};
static JSONL_WRITER_VIEW: Declaration = Declaration {id:NativeTypeId::JsonlWriterView,module:"std:json",name:"JsonlWriterView",..BYTE_VIEW_MUT};
static JSONL_WRITER: Declaration = Declaration {id:NativeTypeId::JsonlWriter,module:"std:json",name:"JsonlWriter",..FILE};
static JSONL_READER_VIEW: Declaration = Declaration {id:NativeTypeId::JsonlReaderView,module:"std:json",name:"JsonlReaderView",..BYTE_VIEW_MUT};
static JSONL_READER: Declaration = Declaration {id:NativeTypeId::JsonlReader,module:"std:json",name:"JsonlReader",..FILE};
static JSONL_ERROR: Declaration = Declaration {id:NativeTypeId::JsonlError,module:"std:json",name:"JsonlError",fields:&[
    Field{name:"code",signature:Signature::String,optional:false},
    Field{name:"message",signature:Signature::String,optional:false},
    Field{name:"operation",signature:Signature::String,optional:false},
    Field{name:"line",signature:Signature::Int,optional:false},
    Field{name:"offset",signature:Signature::Int,optional:false},
    Field{name:"osCode",signature:Signature::Int,optional:true},
],..FS_ERROR};
static JSON_BUILD_LIMITS: Declaration = Declaration {id:NativeTypeId::JsonBuildLimits,module:"std:json",name:"JsonBuildLimits",fields:&[
 Field{name:"maxDepth",signature:Signature::Int,optional:false},
 Field{name:"maxEntries",signature:Signature::Int,optional:false},
 Field{name:"maxNodes",signature:Signature::Int,optional:false},
],..FS_ERROR};
static JSON_ENTRY: Declaration = Declaration {id:NativeTypeId::JsonEntry,module:"std:json",name:"JsonEntry",fields:&[
 Field{name:"key",signature:Signature::String,optional:false},
 Field{name:"value",signature:Signature::Named(NativeTypeId::JsonValue),optional:false},
],..FS_ERROR};
static JSON_LIMITS: Declaration = Declaration {id:NativeTypeId::JsonLimits,module:"std:json",name:"JsonLimits",fields:&[
    Field{name:"maxBytes",signature:Signature::Int,optional:false},
    Field{name:"maxDepth",signature:Signature::Int,optional:false},
    Field{name:"maxEntries",signature:Signature::Int,optional:false},
    Field{name:"maxNodes",signature:Signature::Int,optional:false},
],..FS_ERROR};
static BYTE_ERROR: Declaration = Declaration {
    id: NativeTypeId::ByteError,
    module: "std:bytes",
    name: "ByteError",
    type_parameters: &[],
    representation: Representation::Record,
    independent_clone: true,
    collection_key: false,
    thread_transfer: true,
    capabilities: &[Capability::Inspection, Capability::StringConversion, Capability::PropertyReflection, Capability::ObjectSpread, Capability::Equality],
    fields: &[
        Field { name: "code", signature: Signature::String, optional: false },
        Field { name: "message", signature: Signature::String, optional: false },
        Field { name: "offset", signature: Signature::Int, optional: true },
    ],
};

static BYTE_VIEW: Declaration = Declaration {
    id: NativeTypeId::ByteView,
    module: "std:bytes",
    name: "ByteView",
    type_parameters: &[],
    representation: Representation::Borrowed(crate::native_return_borrows::BorrowAccess::ReadOnly),
    independent_clone: false,
    collection_key: false,
    thread_transfer: false,
    fields: &[],
    capabilities: &[],
};
static BYTE_VIEW_MUT: Declaration = Declaration {
    id: NativeTypeId::ByteViewMut,
    module: "std:bytes",
    name: "ByteViewMut",
    type_parameters: &[],
    representation: Representation::Borrowed(crate::native_return_borrows::BorrowAccess::Exclusive),
    independent_clone: false,
    collection_key: false,
    thread_transfer: false,
    fields: &[],
    capabilities: &[],
};

// Opaque collection identity; operations validate the specialized element type.
static DEQUE: Declaration = Declaration {
    id: NativeTypeId::Deque, module: "std:collections", name: "Deque",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Cloneability also requires every specialized element to support cloning.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static ORDERED_MAP: Declaration = Declaration {
    id: NativeTypeId::OrderedMap, module: "std:collections", name: "OrderedMap",
    type_parameters: &["K", "V"], representation: Representation::Opaque,
    // Payload cloneability is checked transitively through the map definition.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static ORDERED_SET: Declaration = Declaration {
    id: NativeTypeId::OrderedSet, module: "std:collections", name: "OrderedSet",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Scalar keys support independent cloning; thread transfer remains pending.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static HEAP: Declaration = Declaration {
    id: NativeTypeId::Heap, module: "std:collections", name: "Heap",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Scalar keys support independent cloning; thread transfer remains pending.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

/// Exhaustive identity lookup prevents missing declarations as IDs are added.
pub fn declaration(id: NativeTypeId) -> &'static Declaration {
    match id {
        NativeTypeId::Schema => &SCHEMA,
        NativeTypeId::Bytes => &BYTES,
        NativeTypeId::ByteError => &BYTE_ERROR,
        NativeTypeId::ByteView => &BYTE_VIEW,
        NativeTypeId::ByteViewMut => &BYTE_VIEW_MUT,
        NativeTypeId::File => &FILE,
        NativeTypeId::BufferedReader => &BUFFERED_READER,
        NativeTypeId::BufferedWriter => &BUFFERED_WRITER,
        NativeTypeId::FsError => &FS_ERROR,
        NativeTypeId::EnvError => &ENV_ERROR,
        NativeTypeId::JsonValue => &JSON_VALUE,
        NativeTypeId::JsonView => &JSON_VIEW,
        NativeTypeId::JsonRaw => &JSON_RAW,
        NativeTypeId::StringView => &STRING_VIEW,
        NativeTypeId::JsonError => &JSON_ERROR,
        NativeTypeId::JsonLimits => &JSON_LIMITS,
        NativeTypeId::JsonBuildLimits => &JSON_BUILD_LIMITS,
        NativeTypeId::JsonEntry => &JSON_ENTRY,
        NativeTypeId::JsonlReader => &JSONL_READER,
        NativeTypeId::JsonlReaderView => &JSONL_READER_VIEW,
        NativeTypeId::JsonlWriter => &JSONL_WRITER,
        NativeTypeId::JsonlWriterView => &JSONL_WRITER_VIEW,
        NativeTypeId::JsonlError => &JSONL_ERROR,
        NativeTypeId::Deque => &DEQUE,
        NativeTypeId::OrderedMap => &ORDERED_MAP,
        NativeTypeId::OrderedSet => &ORDERED_SET,
        NativeTypeId::Heap => &HEAP,
    }
}

/// Exact module/name lookup does not grant aliases a new declaration identity.
/// This registry alone does not publish types through the source module loader.
pub fn lookup(module: &str, name: &str) -> Option<&'static Declaration> {
    [&SCHEMA, &JSON_RAW, &JSONL_WRITER_VIEW, &JSONL_READER_VIEW, &JSONL_WRITER, &JSON_BUILD_LIMITS, &JSON_ENTRY, &STRING_VIEW, &JSONL_READER, &JSONL_ERROR, &JSON_VALUE, &JSON_VIEW, &JSON_ERROR, &JSON_LIMITS, &ENV_ERROR, &BYTES, &BYTE_ERROR, &BYTE_VIEW, &BYTE_VIEW_MUT, &FILE, &BUFFERED_READER, &BUFFERED_WRITER, &FS_ERROR, &DEQUE, &HEAP, &ORDERED_SET, &ORDERED_MAP].into_iter().find(|item| item.module == module && item.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::native_return_borrows::BorrowAccess;

    #[test]
    fn byte_view_identities_are_nonconstructible_borrowed_storage() {
        for (id, access) in [
            (NativeTypeId::ByteView, BorrowAccess::ReadOnly),
            (NativeTypeId::ByteViewMut, BorrowAccess::Exclusive),
        ] {
            let declaration = declaration(id);
            assert_eq!(declaration.borrow_access(), Some(access));
            assert!(!declaration.source_constructible());
            assert!(!declaration.independent_clone);
            assert!(!declaration.collection_key);
            assert!(!declaration.thread_transfer);
            assert!(declaration.fields.is_empty());
        }
        assert_eq!(declaration(NativeTypeId::Bytes).borrow_access(), None);
        assert_eq!(declaration(NativeTypeId::ByteError).borrow_access(), None);
    }
}
