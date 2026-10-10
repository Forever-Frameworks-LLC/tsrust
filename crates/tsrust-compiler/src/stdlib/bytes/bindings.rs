use crate::{Mode, native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType}, native_signatures::{NativeTypeId, Signature}};
pub(crate) const FROM_ARRAY: NativeBinding = NativeBinding {
    module: "std:bytes", name: "fromArray",
    parameters: &[NativeParameter { name: "values", ty: NativeType::Structured(&Signature::Array(&Signature::Int)), access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Structured(&Signature::Result(&Signature::Named(NativeTypeId::Bytes), &Signature::Named(NativeTypeId::ByteError))),
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_from_array" }, features: &["bytes"],
};
pub(crate) const LENGTH: NativeBinding = NativeBinding {
    module: "std:bytes", name: "length",
    parameters: &[NativeParameter { name: "bytes", ty: NativeType::Structured(&Signature::Named(NativeTypeId::Bytes)), access: Mode::Read, optional: false, rest: false }],
    result: NativeType::Int,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_bytes_length" }, features: &["bytes"],
};
