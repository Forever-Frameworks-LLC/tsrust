use crate::native_bindings::{NativeBackend, NativeBinding, NativeType};
pub(crate) const PLATFORM: NativeBinding = NativeBinding {
    module: "std:os", name: "platform", parameters: &[], result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_os_platform" }, features: &["os-identity"],
};
pub(crate) const ARCH: NativeBinding = NativeBinding {
    module: "std:os", name: "arch", parameters: &[], result: NativeType::String,
    backend: NativeBackend::Bundled { symbol: "tsrust_std_os_arch" }, features: &["os-identity"],
};
