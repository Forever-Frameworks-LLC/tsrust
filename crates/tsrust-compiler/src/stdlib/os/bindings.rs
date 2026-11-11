use crate::native_bindings::{NativeBackend, NativeBinding, NativeType};
use crate::native_signatures::{NativeTypeId,Signature};
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
pub(crate) const AVAILABLE_PARALLELISM:NativeBinding=NativeBinding{module:"std:os",name:"availableParallelism",parameters:&[],result:NativeType::Structured(&Signature::Result(&Signature::Int,&Signature::Named(NativeTypeId::OsError))),returned_borrow:None,backend:NativeBackend::Bundled{symbol:"tsrust_std_os_available_parallelism"},features:&["os-observations"]};
pub(crate) const HOSTNAME:NativeBinding=NativeBinding{name:"hostname",result:NativeType::Structured(&Signature::Result(&Signature::String,&Signature::Named(NativeTypeId::OsError))),backend:NativeBackend::Bundled{symbol:"tsrust_std_os_hostname"},features:&["hostname"],..AVAILABLE_PARALLELISM};
