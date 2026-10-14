//! Explicit Result propagation through the existing typed throw transport.
use crate::native_bindings::{NativeBackend, NativeBinding, NativeParameter, NativeType};
pub(crate) const UNWRAP: NativeBinding = NativeBinding {
    module: "std:result", name: "unwrap", returned_borrow: None,
    parameters: &[NativeParameter { name: "result", ty: NativeType::ObservedValue, access: crate::Mode::Consume, optional: false, rest: false }],
    result: NativeType::ObservedValue,
    backend: NativeBackend::Bundled { symbol: "result_unwrap_intrinsic" }, features: &[],
};
