use crate::{Mode, native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, native_signatures::{NativeTypeId, Signature}};
pub(crate) const FROM_ARRAY: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "fromArray",
    parameters: &[NativeParameter { name: "values", ty: NativeType::Structured(&Signature::Array(&Signature::Int)), access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_from_array" }, features: &["bytes"],
};
pub(crate) const LENGTH: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "length",
    parameters: &[NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Int,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_length" }, features: &["bytes"],
};
pub(crate) const AT: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "at",
    parameters: &[
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "index", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Int, &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_at" }, features: &["bytes"],
};
pub(crate) const SET: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "set",
    parameters: &[
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "index", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
        NativeParameter { name: "value", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_set" }, features: &["bytes"],
};
pub(crate) const ALLOC: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "alloc",
    parameters: &[
        NativeParameter { name: "length", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
        NativeParameter { name: "fill", ty: NativeType::Int, access: Mode::Copy, optional: true, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_alloc" }, features: &["bytes"],
};
pub(crate) const RESIZE: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "resize",
    parameters: &[
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "length", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
        NativeParameter { name: "fill", ty: NativeType::Int, access: Mode::Copy, optional: true, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_resize" }, features: &["bytes"],
};
pub(crate) const APPEND: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "append",
    parameters: &[
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Mutate, optional: false, rest: false },
        NativeParameter { name: "values", ty: NativeType::Structured(&Signature::Array(&Signature::Int)), access: Mode::Read, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Void, &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_append" }, features: &["bytes"],
};
pub(crate) const COPY: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "copy",
    parameters: &[
        NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false },
        NativeParameter { name: "start", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
        NativeParameter { name: "end", ty: NativeType::Int, access: Mode::Copy, optional: false, rest: false },
    ],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_copy" }, features: &["bytes"],
};
pub(crate) const FROM_UTF8: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "fromUtf8",
    parameters: &[NativeParameter { name: "text", ty: NativeType::String, access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_from_utf8" }, features: &["bytes"],
};
pub(crate) const TO_UTF8: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:bytes", name: "toUtf8",
    parameters: &[NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::String, &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_to_utf8" }, features: &["bytes"],
};
