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
