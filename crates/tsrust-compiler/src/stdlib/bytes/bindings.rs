use crate::{Mode, native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, native_signatures::{NativeTypeId, Signature}};
pub(crate) const VIEW: NativeBinding = view_binding(false);
pub(crate) const VIEW_MUT: NativeBinding = view_binding(true);
const fn view_binding(mutable: bool) -> NativeBinding {
    use crate::native_return_borrows::{BorrowAccess, ReturnSlot, ReturnedBorrow};
    NativeBinding {
        module: "std:bytes", name: if mutable {"viewMut"} else {"view"},
        parameters: if mutable {&[
            NativeParameter {name:"bytes",ty:NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)),access:Mode::Mutate,optional:false,rest:false},
            NativeParameter {name:"start",ty:NativeType::Int,access:Mode::Copy,optional:false,rest:false},
            NativeParameter {name:"end",ty:NativeType::Int,access:Mode::Copy,optional:false,rest:false},
        ]} else {&[
            NativeParameter {name:"bytes",ty:NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)),access:Mode::Read,optional:false,rest:false},
            NativeParameter {name:"start",ty:NativeType::Int,access:Mode::Copy,optional:false,rest:false},
            NativeParameter {name:"end",ty:NativeType::Int,access:Mode::Copy,optional:false,rest:false},
        ]},
        result: if mutable {NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::ByteViewMut),&Signature::Named(NativeTypeId::ByteError)))} else {NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::ByteView),&Signature::Named(NativeTypeId::ByteError)))},
        returned_borrow:Some(ReturnedBorrow {source_parameter:0,access:if mutable {BorrowAccess::Exclusive} else {BorrowAccess::ReadOnly},payload:if mutable {NativeTypeId::ByteViewMut} else {NativeTypeId::ByteView},slot:ReturnSlot::ResultOk}),
        backend:NativeBackend::Bundled {symbol:if mutable {"tsrust_std_bytes_view_mut"} else {"tsrust_std_bytes_view"}}, features:&["bytes"],
    }
}
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

const CODEC_BYTES: crate::native_signatures::Signature = Signature::Named(NativeTypeId::Bytes);
const CODEC_ERROR: crate::native_signatures::Signature = Signature::Named(NativeTypeId::ByteError);
pub(crate) const BYTE_ORDER: Signature = Signature::Union(&[Signature::StringLiteral("little"), Signature::StringLiteral("big")]);
const CODEC_READ: &[NativeParameter] = &[NativeParameter { name: "bytes", ty: NativeType::Structured(&CODEC_BYTES), access: Mode::Read, optional: false, rest: false }];
const CODEC_TEXT: &[NativeParameter] = &[NativeParameter { name: "text", ty: NativeType::String, access: Mode::Read, optional: false, rest: false }];
const fn codec(name: &'static str, symbol: &'static str, parameters: &'static [NativeParameter], bytes: bool, base64: bool) -> NativeBinding {
    NativeBinding { module: "std:bytes", name, parameters,
        result: NativeType::Structured(if bytes { &Signature::Result(&CODEC_BYTES, &CODEC_ERROR) } else { &Signature::Result(&Signature::String, &CODEC_ERROR) }),
        returned_borrow: None, backend: NativeBackend::Bundled { symbol }, features: if base64 { &["bytes", "byte-codecs"] } else { &["bytes"] },
    }
}
pub(crate) const TO_HEX: NativeBinding = codec("toHex", "tsrust_std_bytes_to_hex", CODEC_READ, false, false);
pub(crate) const FROM_HEX: NativeBinding = codec("fromHex", "tsrust_std_bytes_from_hex", CODEC_TEXT, true, false);
pub(crate) const TO_UTF8_LOSSY: NativeBinding = codec("toUtf8Lossy", "tsrust_std_bytes_to_utf8_lossy", CODEC_READ, false, false);
pub(crate) const TO_BASE64: NativeBinding = codec("toBase64", "tsrust_std_bytes_to_base64", CODEC_READ, false, true);
pub(crate) const FROM_BASE64: NativeBinding = codec("fromBase64", "tsrust_std_bytes_from_base64", CODEC_TEXT, true, true);
pub(crate) const TO_BASE64_URL: NativeBinding = codec("toBase64Url", "tsrust_std_bytes_to_base64_url", CODEC_READ, false, true);
pub(crate) const FROM_BASE64_URL: NativeBinding = codec("fromBase64Url", "tsrust_std_bytes_from_base64_url", CODEC_TEXT, true, true);
pub(crate) const FROM_UTF16: NativeBinding = codec("fromUtf16", "tsrust_std_bytes_from_utf16", &[
    NativeParameter { name: "text", ty: NativeType::String, access: Mode::Read, optional: false, rest: false },
    NativeParameter { name: "byteOrder", ty: NativeType::Structured(&BYTE_ORDER), access: Mode::Read, optional: false, rest: false },
], true, false);
pub(crate) const TO_UTF16: NativeBinding = codec("toUtf16", "tsrust_std_bytes_to_utf16", &[
    NativeParameter { name: "bytes", ty: NativeType::Structured(&CODEC_BYTES), access: Mode::Read, optional: false, rest: false },
    NativeParameter { name: "byteOrder", ty: NativeType::Structured(&BYTE_ORDER), access: Mode::Read, optional: false, rest: false },
], false, false);
