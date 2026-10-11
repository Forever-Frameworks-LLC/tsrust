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
    // Source capabilities remain disabled until checked storage integration lands.
    independent_clone: false, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static ORDERED_SET: Declaration = Declaration {
    id: NativeTypeId::OrderedSet, module: "std:collections", name: "OrderedSet",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Source capabilities remain disabled until checked storage integration lands.
    independent_clone: false, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

static HEAP: Declaration = Declaration {
    id: NativeTypeId::Heap, module: "std:collections", name: "Heap",
    type_parameters: &["T"], representation: Representation::Opaque,
    // Source capabilities remain disabled until checked storage integration lands.
    independent_clone: true, collection_key: false, thread_transfer: false,
    fields: &[], capabilities: &[],
};

/// Exhaustive identity lookup prevents missing declarations as IDs are added.
pub fn declaration(id: NativeTypeId) -> &'static Declaration {
    match id {
        NativeTypeId::Bytes => &BYTES,
        NativeTypeId::ByteError => &BYTE_ERROR,
        NativeTypeId::ByteView => &BYTE_VIEW,
        NativeTypeId::ByteViewMut => &BYTE_VIEW_MUT,
        NativeTypeId::File => &FILE,
        NativeTypeId::FsError => &FS_ERROR,
        NativeTypeId::Deque => &DEQUE,
        NativeTypeId::OrderedMap => &ORDERED_MAP,
        NativeTypeId::OrderedSet => &ORDERED_SET,
        NativeTypeId::Heap => &HEAP,
    }
}

/// Exact module/name lookup does not grant aliases a new declaration identity.
/// This registry alone does not publish types through the source module loader.
pub fn lookup(module: &str, name: &str) -> Option<&'static Declaration> {
    [&BYTES, &BYTE_ERROR, &BYTE_VIEW, &BYTE_VIEW_MUT, &FILE, &FS_ERROR, &DEQUE, &HEAP].into_iter().find(|item| item.module == module && item.name == name)
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
