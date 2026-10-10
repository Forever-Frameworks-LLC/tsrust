use crate::native_bindings::{NativeBackend, NativeBinding, NativeType};
pub(crate) const PLATFORM: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:os", name: "platform", parameters: &[], result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_os_platform" }, features: &["os-identity"],
};
pub(crate) const ARCH: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:os", name: "arch", parameters: &[], result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_os_arch" }, features: &["os-identity"],
};

pub(crate) const ENDIANNESS: NativeBinding = NativeBinding { returned_borrow: None,
    module: "std:os", name: "endianness", parameters: &[], result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_os_endianness" }, features: &["os-identity"],
};
