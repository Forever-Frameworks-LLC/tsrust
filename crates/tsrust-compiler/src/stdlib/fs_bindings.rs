use crate::{Mode, native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, native_signatures::{NativeTypeId, Signature}};

pub const OPEN: NativeBinding = NativeBinding {
    module: "std:fs", name: "open", returned_borrow: None,
    parameters: &[NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::File), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_open" }, features: &["fs"],
};
pub const READ: NativeBinding = NativeBinding {
    module: "std:fs", name: "read", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "length", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_read" }, features: &["fs", "bytes"],
};
pub const CLOSE: NativeBinding = NativeBinding {
    module: "std:fs", name: "close", returned_borrow: None,
    parameters: &[NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Consume, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_close" }, features: &["fs"],
};

pub const CREATE: NativeBinding = NativeBinding {
    name: "create", backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_create" }, ..OPEN
};
pub const WRITE: NativeBinding = NativeBinding {
    module: "std:fs", name: "write", returned_borrow: None,
    parameters: &[
        NativeParameter { name: "file", ty: NativeType::Structured(&Signature::Named(NativeTypeId::File)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Int, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_write" }, features: &["fs", "bytes"],
};
pub const WRITE_ALL: NativeBinding = NativeBinding {
    name: "writeAll", result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::FsError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_write_all" }, ..WRITE
};

pub const READ_FILE: NativeBinding = NativeBinding {
    name: "readFile", parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "maxBytes", ty: NativeType::Int, access: Mode::Copy, optional: true, rest: false },
    ], backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_read_file" }, ..READ
};
pub const WRITE_FILE: NativeBinding = NativeBinding {
    name: "writeFile", parameters: &[
        NativeParameter { name: "path", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false },
    ], backend: NativeBackend::Bundled { symbol: "tsrust_std_fs_write_file" }, ..WRITE_ALL
};
